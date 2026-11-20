// One Android WebView, and the clients that hand its callbacks to PageTracker.
//
// Its rules live in PageTracker, which is tested on the JVM; this file itself runs only on a
// device, where the adapter suite (src/androidTest) drives it against the real WebView.
package com.tigercodes.webtiger.webview

import android.annotation.SuppressLint
import android.content.Context
import android.content.MutableContextWrapper
import android.graphics.Bitmap
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import android.net.Uri
import android.net.http.SslError
import android.os.Handler
import android.os.Looper
import android.os.Message
import android.view.View
import android.view.ViewGroup
import android.webkit.GeolocationPermissions
import android.webkit.HttpAuthHandler
import android.webkit.JsPromptResult
import android.webkit.JsResult
import android.webkit.PermissionRequest
import android.webkit.RenderProcessGoneDetail
import android.webkit.SslErrorHandler
import android.webkit.URLUtil
import android.webkit.ValueCallback
import android.webkit.WebChromeClient
import android.webkit.WebResourceError
import android.webkit.WebResourceRequest
import android.webkit.WebResourceResponse
import android.webkit.WebView
import android.webkit.WebViewClient
import androidx.webkit.WebViewCompat
import androidx.webkit.WebViewFeature
import com.tigercodes.webtiger.core.NavigationVerdict
import com.tigercodes.webtiger.core.RequestVerdict
import com.tigercodes.webtiger.core.ResourceType
import com.tigercodes.webtiger.core.v1.CloseRequested
import com.tigercodes.webtiger.core.v1.DialogKind
import com.tigercodes.webtiger.core.v1.EngineEvent
import com.tigercodes.webtiger.core.v1.NavigationCommand
import com.tigercodes.webtiger.core.v1.PermissionKind
import java.io.ByteArrayInputStream

/** Makes pages over this device's WebView. */
internal class WebViewPages(private val context: Context) : PageFactory {
    /**
     * A login set is a WebView profile, which needs a WebView that has them. The answer is the
     * installed WebView's, so it is asked each time: the WebView is updated behind the app.
     */
    override val keepsLoginSetsApart: Boolean
        get() =
            try {
                WebViewFeature.isFeatureSupported(WebViewFeature.MULTI_PROFILE)
            } catch (_: RuntimeException) {
                false
            }

    override fun create(webViewId: Long, loginSet: String, host: PageHost): Page = WebViewPage(context, webViewId, loginSet, host, ::isOnline)

    /** Whether the device has a network that reaches the internet. */
    private fun isOnline(): Boolean {
        val connectivity = context.getSystemService(ConnectivityManager::class.java) ?: return true
        val capabilities = connectivity.getNetworkCapabilities(connectivity.activeNetwork) ?: return false
        return capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)
    }
}

/**
 * A web view in a login set's profile. It outlives the activity that shows it (000 §6.5: a
 * fold or a rotation reloads nothing), so it is made with a context whose base is the
 * application; the window that shows it lends it the activity while it is on screen
 * (ui/WebViewHost.kt).
 */
internal class WebViewPage(
    appContext: Context,
    private val id: Long,
    private val loginSet: String,
    private val host: PageHost,
    private val online: () -> Boolean,
) : Page {
    private val context = MutableContextWrapper(appContext)
    private val tracker = PageTracker(id, host::report)

    /**
     * The zoom the page was last given, in per cent: a web view made anew is given it again.
     * Before the web view, which is given it as it is made: a field below that one is still
     * nothing then, and a text zoom of nothing is a page with no text in it.
     */
    private var zoomPercent = ACTUAL_SIZE

    /** Null once the renderer has died, until the page is loaded again. */
    private var webView: WebView? = create()

    /** The page's address, for Guard's check, which the WebView makes on its own threads. */
    @Volatile private var topUrl = ""

    override val view: View? get() = webView

    override fun load(url: String): Boolean {
        val view = webView ?: recreate() ?: return false
        tracker.willLoad(url)
        view.loadUrl(url)
        return true
    }

    override fun command(command: NavigationCommand) {
        when (command) {
            NavigationCommand.NAVIGATION_COMMAND_BACK ->
                webView?.takeIf { it.canGoBack() }?.let { view ->
                    tracker.willTraverse()
                    view.goBack()
                }
            NavigationCommand.NAVIGATION_COMMAND_FORWARD ->
                webView?.takeIf { it.canGoForward() }?.let { view ->
                    tracker.willTraverse()
                    view.goForward()
                }
            NavigationCommand.NAVIGATION_COMMAND_RELOAD -> {
                // After a failure, or with a new web view in a dead one's place, the engine
                // has nothing of its own to reload: the address is loaded again.
                val again = tracker.reloadTarget
                if (again != null) {
                    load(again)
                } else {
                    webView?.let { view ->
                        tracker.willReload()
                        view.reload()
                    }
                }
            }
            NavigationCommand.NAVIGATION_COMMAND_STOP ->
                webView?.let { view ->
                    tracker.willStop()
                    view.stopLoading()
                }
            NavigationCommand.NAVIGATION_COMMAND_UNSPECIFIED -> Unit
        }
    }

    /** The text the web view is now finding: the same text again goes to another match. */
    private var finding = ""

    /**
     * The System WebView has no public way to zoom a whole page; what it has is the size of
     * the text, which is what a person who zooms a page is most often after (012 §6). A level
     * the core does not have is held to between half and five times.
     */
    override fun zoom(percent: Int) {
        zoomPercent = percent.coerceIn(SMALLEST_ZOOM, LARGEST_ZOOM)
        webView?.settings?.textZoom = zoomPercent
    }

    override fun find(text: String, backwards: Boolean) {
        val view = webView ?: return
        when {
            text.isEmpty() -> {
                finding = ""
                view.clearMatches()
            }
            // The same text again: the next match, or the one before. The WebView goes round
            // from the last to the first by itself.
            text == finding -> view.findNext(!backwards)
            else -> {
                finding = text
                view.findAllAsync(text)
            }
        }
    }

    override fun close() {
        webView?.let { view ->
            view.stopLoading()
            destroy(view)
        }
        webView = null
    }

    /** Out of whatever window shows it, then destroyed: in that order, as the WebView asks. */
    private fun destroy(view: WebView) {
        (view.parent as? ViewGroup)?.removeView(view)
        view.destroy()
    }

    /**
     * A long press landed on the page. True when it was on a link, in words or in a picture,
     * or on a picture of the web: the press is then nobody else's, and whoever shows the menu
     * is told what it was on (018, 022). Of a link the WebView is asked where it leads, and
     * says so in a message of its own. A picture's address it names at once, and for a
     * picture that is a link that is all it names at once.
     */
    private fun pressed(view: WebView): Boolean {
        val under = view.hitTestResult
        val picture = under.extra.orEmpty()
        return when (under.type) {
            WebView.HitTestResult.IMAGE_TYPE -> {
                // A picture a page made itself can be neither opened in a tab nor fetched:
                // the press stays the WebView's.
                if (isOfTheWeb(picture)) host.pressed(id, Pressed(picture = picture))
                isOfTheWeb(picture)
            }
            WebView.HitTestResult.SRC_ANCHOR_TYPE, WebView.HitTestResult.SRC_IMAGE_ANCHOR_TYPE -> {
                val shown = if (under.type == WebView.HitTestResult.SRC_IMAGE_ANCHOR_TYPE) picture else ""
                val answer =
                    Handler(Looper.getMainLooper()) { said ->
                        if (view === webView) host.pressed(id, Pressed(link = said.data.getString("url").orEmpty(), picture = shown))
                        true
                    }
                view.requestFocusNodeHref(answer.obtainMessage())
                true
            }
            else -> false
        }
    }

    /**
     * A file the person chose in a menu (022). The WebView does not fetch it: it is offered
     * like a download a page asked for, as this web view's requests say who they are, with no
     * name suggested and its kind not known: nobody has asked the site yet.
     */
    override fun save(url: String) {
        val view = webView ?: return
        host.download(id, DownloadOffer(url = url, userAgent = view.settings.userAgentString.orEmpty(), suggestedName = "", mimeType = "", length = 0, chosen = true))
    }

    private fun recreate(): WebView? =
        try {
            create().also { webView = it }
        } catch (_: RuntimeException) {
            null
        }

    private fun create(): WebView {
        // Asked here as well as by the engine: no web view is ever made that would have to run
        // in the default profile, whatever became of the installed WebView in between.
        if (WebViewFeature.isFeatureSupported(WebViewFeature.MULTI_PROFILE)) {
            // A search the WebView's own bar for selected text asks the system for is the
            // person's wish to search, and the core's to answer (023): the context hears it,
            // and no other app is started.
            val view = WebView(SearchingContext(context) { words -> host.searchFor(id, words) })
            try {
                // Before anything else is done with the web view, as the API requires: its pages
                // run in the login set's profile (made here if it is new).
                WebViewCompat.setProfile(view, loginSet)
            } catch (refused: RuntimeException) {
                view.destroy()
                throw refused
            }
            configure(view)
            return view
        }
        throw IllegalStateException("engine.no_profiles")
    }

    @SuppressLint("SetJavaScriptEnabled") // A browser runs the pages' scripts.
    private fun configure(view: WebView) {
        view.settings.apply {
            javaScriptEnabled = true
            domStorageEnabled = true
            // Pages are laid out for their own viewport, and zoom with two fingers.
            useWideViewPort = true
            loadWithOverviewMode = true
            builtInZoomControls = true
            displayZoomControls = false
            // Web content reads nothing of the device: no files, no content providers.
            allowFileAccess = false
            allowContentAccess = false
            // A page that asks for a new window is heard (`onCreateWindow`); without this the
            // WebView loads the address in the web view that asked. A page gets a window when
            // somebody tapped for one, and not by asking as it loads: the WebView's own blocker.
            setSupportMultipleWindows(true)
            javaScriptCanOpenWindowsAutomatically = false
        }
        view.settings.textZoom = zoomPercent
        view.webViewClient = Client()
        view.webChromeClient = Chrome()
        // The WebView does not download: it says one was asked for (013). What it was loading
        // then comes to nothing, and the page stays. The name is what the site suggests, read
        // as the platform reads it; the core makes the name the file is saved under.
        view.setDownloadListener { url, userAgent, contentDisposition, mimeType, contentLength ->
            if (view === webView && url != null) {
                tracker.becameDownload(url)
                host.download(
                    id,
                    DownloadOffer(
                        url = url,
                        userAgent = userAgent.orEmpty(),
                        suggestedName = URLUtil.guessFileName(url, contentDisposition, mimeType),
                        mimeType = mimeType.orEmpty(),
                        length = contentLength,
                    ),
                )
            }
        }
        // A long press on a link or on a picture is for its menu (018, 022); anywhere else it
        // is the WebView's own, which selects the words under the finger.
        view.setOnLongClickListener { held -> held === webView && pressed(view) }
        // What a find found, once the WebView has finished counting (010). A web view made
        // anew has found nothing.
        finding = ""
        view.setFindListener { activeMatchOrdinal, numberOfMatches, isDoneCounting ->
            // Only of a find that is on: what the WebView reports with none is nobody's answer.
            if (view === webView && isDoneCounting && finding.isNotEmpty()) tracker.findResult(activeMatchOrdinal, numberOfMatches)
        }
    }

    /** Navigation, as the engine reports it. Every callback but one is on the main thread. */
    private inner class Client : WebViewClient() {
        /**
         * The navigation policy. The engine asks about navigations a page starts and about
         * every redirect; not about a load the adapter itself asked for, a form's POST, or a
         * step back or forward. In a frame it asks only about addresses that are not http(s).
         */
        override fun shouldOverrideUrlLoading(view: WebView, request: WebResourceRequest): Boolean {
            if (view !== webView) return true
            val url = request.url.toString()
            val mainFrame = request.isForMainFrame
            val verdict = host.navigationVerdict(url, id, mainFrame, request.hasGesture())
            if (verdict == NavigationVerdict.ALLOW) {
                if (mainFrame) tracker.navigationAllowed(url)
                return false
            }
            // An address for the system is refused here like any other, and reported first, for
            // a frame as for the page: the adapter opens nothing (007).
            if (verdict == NavigationVerdict.EXTERNAL) tracker.outsideLink(url, request.hasGesture())
            if (mainFrame) tracker.navigationRefused(url, request.isRedirect)
            return true
        }

        override fun onPageStarted(view: WebView, url: String, favicon: Bitmap?) {
            if (view === webView) tracker.pageStarted(url)
        }

        override fun doUpdateVisitedHistory(view: WebView, url: String, isReload: Boolean) {
            if (view !== webView) return
            topUrl = url
            val newPage = tracker.committed(url, view.canGoBack(), view.canGoForward())
            if (newPage && finding.isNotEmpty()) {
                // A new page ends the find (010 §3). Left alone, the WebView goes on with it:
                // it looks for the old text in the new page, unasked, and marks what it finds.
                finding = ""
                view.clearMatches()
            }
        }

        override fun onPageFinished(view: WebView, url: String) {
            if (view === webView) tracker.pageFinished(url)
        }

        override fun onReceivedError(view: WebView, request: WebResourceRequest, error: WebResourceError) {
            // The page's own load only. A resource of the page that failed is the page's business.
            if (view === webView && request.isForMainFrame) {
                tracker.failed(netErrorCode(error.errorCode, online()), request.url.toString())
            }
        }

        /**
         * A server asks who is there (017). Left alone the WebView gives it no answer. It is
         * reported, and the load waits for the core's word. The WebView names the server's host
         * and nothing more of it; it says the stored answer is not to be used again once the
         * server has refused one for this request.
         */
        override fun onReceivedHttpAuthRequest(view: WebView, handler: HttpAuthHandler, host: String?, realm: String?) {
            if (view !== webView) {
                handler.cancel()
                return
            }
            this@WebViewPage.host.signIn(id, host.orEmpty(), realm.orEmpty(), tracker.headingFor, again = !handler.useHttpAuthUsernamePassword()) { typed ->
                if (typed != null) handler.proceed(typed.name, typed.password) else handler.cancel()
            }
        }

        override fun onReceivedSslError(view: WebView, handler: SslErrorHandler, error: SslError) {
            // A certificate the system does not trust is never accepted on the user's behalf.
            handler.cancel()
            // And said to have failed for the kind of trouble the certificate was (021).
            if (view === webView) tracker.certificateRefused(error.url, certificateErrorCode(error.primaryError))
        }

        /**
         * Guard's check, for every request the page makes: called on the WebView's own
         * threads, so nothing of the web view is touched here. The WebView does not say what
         * a request is for beyond whether it is the page itself.
         */
        override fun shouldInterceptRequest(view: WebView, request: WebResourceRequest): WebResourceResponse? {
            val url = request.url.toString()
            val mainFrame = request.isForMainFrame
            val verdict = host.requestVerdict(url, if (mainFrame) url else topUrl, id, if (mainFrame) ResourceType.DOCUMENT else ResourceType.OTHER)
            // Until Guard exists (W4) nothing is blocked, and how a blocked request should look
            // to its page is settled then. For now: an empty answer that says "forbidden".
            return if (verdict == RequestVerdict.BLOCK) {
                WebResourceResponse("text/plain", "utf-8", 403, "Blocked", emptyMap(), ByteArrayInputStream(ByteArray(0)))
            } else {
                null
            }
        }

        /**
         * The process drawing the page died. A WebView whose renderer is gone cannot be used
         * again, so it is destroyed; the next load makes another in the same login set.
         * Returning true keeps the app alive.
         */
        override fun onRenderProcessGone(view: WebView, detail: RenderProcessGoneDetail): Boolean {
            if (view === webView) {
                webView = null
                destroy(view)
                tracker.rendererGone()
            }
            return true
        }
    }

    private inner class Chrome : WebChromeClient() {
        override fun onProgressChanged(view: WebView, newProgress: Int) {
            if (view === webView) tracker.progress(newProgress)
        }

        override fun onReceivedTitle(view: WebView, title: String?) {
            if (view === webView) tracker.title(title.orEmpty())
        }

        /**
         * The page asked for a new window: a link with a target, or `window.open`. The engine
         * makes the web view and reports it; the WebView is handed it here, and loads the
         * address in it. False, and the page gets no window, when none could be made.
         */
        override fun onCreateWindow(view: WebView, isDialog: Boolean, isUserGesture: Boolean, resultMsg: Message): Boolean {
            if (view !== webView) return false
            val transport = resultMsg.obj as? WebView.WebViewTransport ?: return false
            val window = host.newWindow(id, isUserGesture)?.view as? WebView ?: return false
            transport.webView = window
            resultMsg.sendToTarget()
            return true
        }

        /**
         * The page asked for its own window to be closed (`window.close()`). Whether it may is
         * the core's answer: it closes a tab a page opened, and no other.
         */
        override fun onCloseWindow(window: WebView) {
            if (window === webView) host.report(EngineEvent(web_view_id = id, close_requested = CloseRequested()))
        }

        /**
         * A page says something (016). The WebView would show a dialog of its own: it is told
         * the app sees to it, and the page's script waits for the core's word.
         */
        override fun onJsAlert(view: WebView, url: String?, message: String?, result: JsResult): Boolean {
            says(DialogKind.DIALOG_KIND_ALERT, url, message, "") { accepted, _ -> if (accepted) result.confirm() else result.cancel() }
            return true
        }

        override fun onJsConfirm(view: WebView, url: String?, message: String?, result: JsResult): Boolean {
            says(DialogKind.DIALOG_KIND_CONFIRM, url, message, "") { accepted, _ -> if (accepted) result.confirm() else result.cancel() }
            return true
        }

        override fun onJsPrompt(view: WebView, url: String?, message: String?, defaultValue: String?, result: JsPromptResult): Boolean {
            says(DialogKind.DIALOG_KIND_PROMPT, url, message, defaultValue.orEmpty()) { accepted, text -> if (accepted) result.confirm(text) else result.cancel() }
            return true
        }

        /**
         * A page asks to keep the person when they leave ("Leave site?"). It is never asked
         * about (016 §2): the person leaves.
         */
        override fun onJsBeforeUnload(view: WebView, url: String?, message: String?, result: JsResult): Boolean {
            result.confirm()
            return true
        }

        private fun says(kind: DialogKind, url: String?, message: String?, defaultText: String, answer: (Boolean, String) -> Unit) {
            host.dialog(id, url.orEmpty(), kind, message.orEmpty(), defaultText) { accepted, text ->
                // A page that has gone meanwhile has nobody to tell.
                try {
                    answer(accepted, text)
                } catch (_: RuntimeException) {
                }
            }
        }

        /**
         * A page asks for the camera, the microphone or both (014). It is reported, with who
         * asks, and waits for the core's word. Whatever else a page asks for this way
         * (protected media, MIDI) is not offered: a request with any of it is refused whole.
         */
        /**
         * A page's file input was pressed (019). Left alone the WebView does nothing. The
         * engine's chooser is asked, and the WebView given what was picked, or told that
         * nothing was. A source to capture from (the camera) is not offered: the chooser is.
         */
        override fun onShowFileChooser(view: WebView, callback: ValueCallback<Array<Uri>>, params: FileChooserParams): Boolean {
            if (view !== webView) return false
            val asked = FilesAsked(several = params.mode == FileChooserParams.MODE_OPEN_MULTIPLE, kinds = params.acceptTypes.orEmpty().filter { it.isNotBlank() })
            host.files(id, asked) { picked -> callback.onReceiveValue(picked?.map(Uri::parse)?.toTypedArray()) }
            return true
        }

        override fun onPermissionRequest(request: PermissionRequest) {
            val kinds =
                request.resources.orEmpty().map { resource ->
                    when (resource) {
                        PermissionRequest.RESOURCE_VIDEO_CAPTURE -> PermissionKind.PERMISSION_KIND_CAMERA
                        PermissionRequest.RESOURCE_AUDIO_CAPTURE -> PermissionKind.PERMISSION_KIND_MICROPHONE
                        else -> null
                    }
                }
            if (kinds.isEmpty() || kinds.any { it == null }) {
                request.deny()
                return
            }
            asking[request] =
                host.permissionAsked(id, request.origin?.toString().orEmpty(), kinds.filterNotNull()) { granted ->
                    asking.remove(request)
                    // A request whose page has gone meanwhile has nobody to tell.
                    try {
                        if (granted) request.grant(request.resources) else request.deny()
                    } catch (_: RuntimeException) {
                    }
                }
        }

        override fun onPermissionRequestCanceled(request: PermissionRequest) {
            asking.remove(request)?.let(host::permissionWithdrawn)
        }

        /**
         * A page asks where the device is (014). The WebView is never told to keep the answer:
         * what a site was allowed is the core's to remember, per login set.
         */
        override fun onGeolocationPermissionsShowPrompt(origin: String?, callback: GeolocationPermissions.Callback?) {
            if (origin == null || callback == null) return
            locating =
                host.permissionAsked(id, origin, listOf(PermissionKind.PERMISSION_KIND_LOCATION)) { granted ->
                    locating = null
                    callback.invoke(origin, granted, false)
                }
        }

        override fun onGeolocationPermissionsHidePrompt() {
            locating?.let(host::permissionWithdrawn)
            locating = null
        }
    }

    /** The page's requests for the camera or the microphone that wait for an answer, each by the adapter's name for it. */
    private val asking = HashMap<PermissionRequest, Long>()

    /** The page's request for the location that waits for an answer. */
    private var locating: Long? = null

    private companion object {
        const val ACTUAL_SIZE = 100
        const val SMALLEST_ZOOM = 50
        const val LARGEST_ZOOM = 500
    }
}

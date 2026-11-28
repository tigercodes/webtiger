// One Android WebView, and the clients that hand its callbacks to PageTracker.
//
// Its rules live in PageTracker, which is tested on the JVM; this file itself runs only on a
// device, where the adapter suite (src/androidTest) drives it against the real WebView.
package com.tigercodes.webtiger.webview

import androidx.webkit.ProfileStore
import androidx.webkit.WebStorageCompat
import java.io.File
import java.io.IOException
import android.annotation.SuppressLint
import android.content.Context
import android.content.MutableContextWrapper
import android.graphics.Bitmap
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import android.net.Uri
import android.net.http.SslError
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.os.Message
import android.view.View
import android.view.ViewGroup
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputConnection
import android.webkit.CookieManager
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
import com.tigercodes.webtiger.core.StorageVerdict
import com.tigercodes.webtiger.core.v1.CloseRequested
import com.tigercodes.webtiger.core.v1.DialogKind
import com.tigercodes.webtiger.core.v1.EngineEvent
import com.tigercodes.webtiger.core.v1.IconReceived
import com.tigercodes.webtiger.core.v1.NavigationCommand
import com.tigercodes.webtiger.core.v1.PermissionKind
import java.io.ByteArrayInputStream
import okio.ByteString.Companion.toByteString

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

    override fun create(webViewId: Long, loginSet: String, forgetful: Boolean, privacySignal: Boolean, host: PageHost): Page =
        WebViewPage(context, webViewId, loginSet, forgetful, privacySignal, host, ::isOnline)

    /** The private login sets whose profiles are to be deleted (037): one name to a line, in a file that is never backed up. */
    private val thrownAway = File(context.noBackupFilesDir, "private-login-sets")

    override fun willThrowAway(loginSet: String): Boolean =
        synchronized(PrivateProfiles) {
            try {
                val listed = PrivateProfiles.read(thrownAway)
                if (loginSet !in listed) PrivateProfiles.write(thrownAway, listed + loginSet)
                true
            } catch (_: IOException) {
                false
            }
        }

    override fun clear(loginSet: String) {
        val profile =
            try {
                ProfileStore.getInstance().getProfile(loginSet)
            } catch (_: RuntimeException) {
                null
            } ?: return
        profile.cookieManager.removeAllCookies(null)
        // A current WebView clears everything a site stored in one call; an older one clears
        // its stored values and databases, and leaves caches and workers until the profile is
        // deleted (003 §5.2).
        if (WebViewFeature.isFeatureSupported(WebViewFeature.DELETE_BROWSING_DATA)) {
            WebStorageCompat.deleteBrowsingData(profile.webStorage) {}
        } else {
            profile.webStorage.deleteAllData()
        }
    }

    override fun sweep() {
        synchronized(PrivateProfiles) {
            // Once in a run: a profile this run has used cannot be deleted by it.
            if (PrivateProfiles.swept) return
            PrivateProfiles.swept = true
            val left =
                try {
                    PrivateProfiles.read(thrownAway)
                } catch (_: IOException) {
                    return
                }
            PrivateProfiles.leftAtStart = left
            if (left.isEmpty() || !keepsLoginSetsApart) return
            val store =
                try {
                    ProfileStore.getInstance()
                } catch (_: RuntimeException) {
                    return
                }
            // One that will not go is kept on the list, for the next start.
            val kept =
                left.filter { name ->
                    try {
                        store.deleteProfile(name)
                        false
                    } catch (_: RuntimeException) {
                        true
                    }
                }
            PrivateProfiles.deletedAtStart = left - kept.toSet()
            try {
                PrivateProfiles.write(thrownAway, kept)
            } catch (_: IOException) {
                // Still listed: tried again at the next start, when there is nothing to delete.
            }
        }
    }

    /** Whether the device has a network that reaches the internet. */
    private fun isOnline(): Boolean {
        val connectivity = context.getSystemService(ConnectivityManager::class.java) ?: return true
        val capabilities = connectivity.getNetworkCapabilities(connectivity.activeNetwork) ?: return false
        return capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)
    }
}

/**
 * A web view in a login set's profile. It outlives the activity that shows it (000 §6.5: a
 * fold or a rotation reloads nothing), so its context is one whose base can be changed: the
 * application's while it is not on the screen, and the activity's, lent by the window that
 * shows it, while it is (ui/WebViewHost.kt). What a page puts over itself in a window of the
 * WebView's own, a list of choices for one, needs that activity (032).
 */
internal class WebViewPage(
    appContext: Context,
    private val id: Long,
    private val loginSet: String,
    /** The login set is a private window's: the keyboard is asked not to learn (037 §5.5). */
    private val forgetful: Boolean,
    /** The web view's pages are told Global Privacy Control (069). */
    private val privacySignal: Boolean,
    private val host: PageHost,
    private val online: () -> Boolean,
) : Page {
    private val context = MutableContextWrapper(appContext)
    private val tracker = PageTracker(id, host::report)

    /** What the engine and the adapter said of this page last, by kind and with no address: for a test's message. */
    fun heard(): List<String> = tracker.heardLately()

    /**
     * The zoom the page was last given, in per cent: a web view made anew is given it again.
     * Before the web view, which is given it as it is made: a field below that one is still
     * nothing then, and a text zoom of nothing is a page with no text in it.
     */
    private var zoomPercent = ACTUAL_SIZE

    /**
     * What the page is doing that would keep its tab awake (052), as far as is known. Before
     * the web view, which is given its ear for it as it is made.
     */
    private val doing = PageDoing()

    /** Null once the renderer has died, until the page is loaded again. */
    private var webView: WebView? = create()

    /** The page's address, for Guard's check and the storage question, which the WebView makes on its own threads. */
    @Volatile private var topUrl = ""

    override val view: View? get() = webView

    override fun load(url: String): Boolean {
        val view = webView ?: recreate() ?: return false
        tracker.willLoad(url)
        // A load the shell begins carries the signal's header (069), where the web view's
        // pages are told it. What a page fetches of its own accord the WebView gives a shell
        // no way to add to.
        if (privacySignal) view.loadUrl(url, PrivacySignal.HEADERS) else view.loadUrl(url)
        return true
    }

    /**
     * The WebView's own state (051): what `saveState` writes, which is one array of bytes under
     * one name (the adapter suite holds it to that). Null when the WebView writes none (it has
     * been nowhere), or writes anything else: nothing is handed on that could not be handed
     * back as it was.
     */
    override fun state(): ByteArray? {
        val view = webView ?: return null
        val saved = Bundle()
        if (view.saveState(saved) == null) return null
        if (saved.keySet() != setOf(STATE_NAME)) return null
        return saved.getByteArray(STATE_NAME)
    }

    /**
     * The WebView drawn into a picture (059). It draws itself when it is in no window, as a
     * tab's is that is behind, and when it never was in one and has a size (the adapter suite
     * holds it to both, on a device).
     */
    override fun picture(mostWide: Int, mostBytes: Int): Pictured? = webView?.let { Pictures.of(it, mostWide, mostBytes) }

    /**
     * Hands the WebView a state, which it takes or does not, and says by the list it gives
     * back. One it took is loading the page the state is at: when that is `url`, the page the
     * tab is at and the core's policy was asked about, that is the load. The WebView asks the
     * app about no load that goes through its own list, so a state that is at any other page
     * is not followed: the web view that took it is let go, and another loads `url`, as one
     * does when the state was not taken at all.
     */
    override fun restore(state: ByteArray, url: String): Boolean {
        val view = webView ?: recreate() ?: return false
        val list =
            try {
                view.restoreState(Bundle().apply { putByteArray(STATE_NAME, state) })
            } catch (_: RuntimeException) {
                null
            }
        if (list != null && list.size > 0) {
            if (list.currentItem?.url == url) {
                tracker.willLoad(url)
                return true
            }
            view.stopLoading()
            destroy(view)
            webView = null
        }
        return load(url)
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
            val searching = SearchingContext(context) { words -> host.searchFor(id, words) }
            val view = if (forgetful) UnlearnedWebView(searching) else WebView(searching)
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

    /**
     * Puts the adapter's script, and its ear, in every page and frame of a web view (052),
     * where the WebView can be given them. One that cannot leaves the adapter deaf, which
     * [PageDoing] takes the careful way. What a frame says is taken up in its turn, like what
     * a page asks: the WebView says it at once, and a commit it has not yet said may be before it.
     */
    private fun listenForDoings(view: WebView) {
        val can = WebViewFeature.isFeatureSupported(WebViewFeature.DOCUMENT_START_SCRIPT) && WebViewFeature.isFeatureSupported(WebViewFeature.WEB_MESSAGE_LISTENER)
        doing.deaf = !can
        if (!can) return
        WebViewCompat.addWebMessageListener(view, PageDoing.NAME, setOf("*")) { _, message, _, _, _ ->
            PageDoing.parse(message.data.orEmpty())?.let { said ->
                inTurn {
                    if (view === webView) {
                        doing.heard(said)
                        reportDoing()
                    }
                }
            }
        }
        WebViewCompat.addDocumentStartJavaScript(view, PageDoing.SCRIPT, setOf("*"))
    }

    /** Tells the core what the page is doing, when it is not what the core was last told. */
    private fun reportDoing() {
        doing.news()?.let { news -> host.report(EngineEvent(web_view_id = id, page_activity_changed = news)) }
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
        // A site in another site's page is not sent its cookies, and what it sets there is
        // not kept (070). It is the WebView's own habit for an app built for a recent
        // Android: said outright all the same, so that it does not hang on a habit.
        CookieManager.getInstance().setAcceptThirdPartyCookies(view, false)
        view.settings.textZoom = zoomPercent
        view.webViewClient = Client()
        view.webChromeClient = Chrome()
        listenForDoings(view)
        if (privacySignal) PrivacySignal.tell(view)
        // A frame of another site than the page's has no storage of its site's own (071): the
        // WebView does not keep it apart, so a script of the adapter's does, in the frames the
        // core says. The page is the one whose commit the adapter has heard of.
        ThirdPartyStorage.keepApart(view) { origin, topOrigin -> host.storageVerdict(origin, topOrigin, topUrl, id) != StorageVerdict.OWN }
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
            val newPage = tracker.committed(url, view.canGoBack(), view.canGoForward(), view.title)
            if (newPage) {
                // The page before's doings went with it (052).
                doing.committed()
                reportDoing()
            }
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
            // Whose certificate it is, and which: the one presented, by a hash of itself.
            val presented =
                certificateOrigin(error.url)?.let { origin ->
                    error.certificate.x509Certificate?.encoded?.let { Refused(origin, certificateFingerprint(it)) }
                }
            // A certificate the system does not trust is let through only where a person went
            // on past it (026): that one, at that host and port, in this web view's login set.
            if (presented != null && host.certificates.trusts(loginSet, presented)) {
                handler.proceed()
                return
            }
            // Otherwise it is never accepted on the person's behalf.
            handler.cancel()
            if (view !== webView) return
            // The load is said to have failed for the kind of trouble the certificate was
            // (021). When it is the page's own, the certificate is held: the core may say to
            // let it through.
            val code = certificateErrorCode(error.primaryError)
            val passable = presented != null && code != "net.tls"
            if (tracker.certificateRefused(error.url, code, held = passable) && passable) {
                host.certificates.refused(id, checkNotNull(presented))
            }
        }

        /**
         * Guard's check, for every request the page makes: called on the WebView's own
         * threads, so nothing of the web view is touched here. The WebView does not say what
         * a request is for: it is told, as far as it can be, from what the request says of
         * itself ([RequestKind], 065).
         */
        override fun shouldInterceptRequest(view: WebView, request: WebResourceRequest): WebResourceResponse? {
            val url = request.url.toString()
            val mainFrame = request.isForMainFrame
            val kind = RequestKind.of(mainFrame, request.requestHeaders.orEmpty())
            val verdict = host.requestVerdict(url, if (mainFrame) url else topUrl, id, kind)
            // A request Guard refuses (064) is answered here, with nothing, as forbidden: it
            // never leaves the device.
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
         * The WebView has the icon of the page it shows (046): it fetched the picture itself,
         * as part of loading the page. It is handed to the core as a small PNG, with the page
         * it is the icon of. Whether it is kept is the core's.
         */
        override fun onReceivedIcon(view: WebView, icon: Bitmap?) {
            if (view !== webView || icon == null) return
            val png = iconPng(icon) ?: return
            host.report(EngineEvent(web_view_id = id, icon_received = IconReceived(png = png.toByteString(), page_url = view.url.orEmpty())))
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
            inTurn {
                host.dialog(id, url.orEmpty(), kind, message.orEmpty(), defaultText) { accepted, text ->
                    // A page that has gone meanwhile has nobody to tell.
                    try {
                        answer(accepted, text)
                    } catch (_: RuntimeException) {
                    }
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
            inTurn {
                asking[request] =
                    host.permissionAsked(id, request.origin?.toString().orEmpty(), kinds.filterNotNull()) { granted ->
                        asking.remove(request)
                        // A request whose page has gone meanwhile has nobody to tell.
                        try {
                            if (granted) request.grant(request.resources) else request.deny()
                        } catch (_: RuntimeException) {
                        }
                        // A page that was given the camera or the microphone holds it (052).
                        if (granted) {
                            doing.granted()
                            reportDoing()
                        }
                    }
            }
        }

        /** In its turn too: after the request it takes back, however soon it follows it. */
        override fun onPermissionRequestCanceled(request: PermissionRequest) {
            inTurn { asking.remove(request)?.let(host::permissionWithdrawn) }
        }

        /**
         * A page asks where the device is (014). The WebView is never told to keep the answer:
         * what a site was allowed is the core's to remember, per login set.
         */
        override fun onGeolocationPermissionsShowPrompt(origin: String?, callback: GeolocationPermissions.Callback?) {
            if (origin == null || callback == null) return
            inTurn {
                locating =
                    host.permissionAsked(id, origin, listOf(PermissionKind.PERMISSION_KIND_LOCATION)) { granted ->
                        locating = null
                        callback.invoke(origin, granted, false)
                    }
            }
        }

        override fun onGeolocationPermissionsHidePrompt() {
            inTurn {
                locating?.let(host::permissionWithdrawn)
                locating = null
            }
        }
    }

    /**
     * Takes up what a page asks of the person behind whatever already waits for this thread.
     *
     * That a page has arrived, the WebView says by a message to this thread
     * (`doUpdateVisitedHistory`, like its other words about a load); what a page asks, it says
     * at once. A page that asks as it loads can so be heard before the message that says it
     * arrived has been read. The core would then ask on behalf of the page that is going, and
     * end the question, unanswered, when the commit came after all: the page was told no, and
     * nobody had been asked. (Seen on an emulator: a page that asked for the camera as it
     * loaded, heard in the same millisecond as its commit, and before it.) A question taken up
     * in its turn reaches the core after the page it is of.
     *
     * A page that is closed before its turn comes has nobody to ask for.
     */
    private fun inTurn(ask: () -> Unit) {
        turns.post { if (webView != null) ask() }
    }

    private val turns = Handler(Looper.getMainLooper())

    /** The page's requests for the camera or the microphone that wait for an answer, each by the adapter's name for it. */
    private val asking = HashMap<PermissionRequest, Long>()

    /** The page's request for the location that waits for an answer. */
    private var locating: Long? = null

    private companion object {
        /** The name the WebView writes its state under, in the bundle `saveState` fills. */
        const val STATE_NAME = "WEBVIEW_CHROMIUM_STATE"
        const val ACTUAL_SIZE = 100
        const val SMALLEST_ZOOM = 50
        const val LARGEST_ZOOM = 500
    }
}

/**
 * A private window's web view (037 §5.5). A web view tells the keyboard about a page's field
 * itself; this one adds, to whatever it tells, that nothing typed there is to be learned from.
 * It is asked of the keyboard, which is the person's own: what it does about it is its own.
 */
private class UnlearnedWebView(context: Context) : WebView(context) {
    override fun onCreateInputConnection(outAttrs: EditorInfo): InputConnection? =
        super.onCreateInputConnection(outAttrs).also {
            outAttrs.imeOptions = outAttrs.imeOptions or EditorInfo.IME_FLAG_NO_PERSONALIZED_LEARNING
        }
}

/**
 * The list of private login sets whose profiles are to be deleted (037), and what was done
 * about it as this run started. One for the process: the WebView's profiles are the process's.
 */
internal object PrivateProfiles {
    /** The list has been gone through in this run. */
    var swept = false

    /** The private login sets an earlier run left, as the list had them when this run first looked. Null until then. */
    var leftAtStart: List<String>? = null

    /** Those of them whose profiles were deleted. */
    var deletedAtStart: List<String> = emptyList()

    fun read(list: File): List<String> = if (list.exists()) list.readLines().filter { SystemWebViewEngine.isUuid(it) } else emptyList()

    /** Written whole, to a file beside it that then takes its place: a list half written is never read. */
    fun write(list: File, names: List<String>) {
        val next = File(list.parentFile, list.name + ".next")
        next.writeText(names.joinToString(separator = "") { it + "\n" })
        if (!next.renameTo(list)) throw IOException("the list could not be put in place")
    }
}

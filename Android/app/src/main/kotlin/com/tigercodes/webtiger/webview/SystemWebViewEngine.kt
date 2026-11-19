// The System WebView engine adapter (000 §4.4, 003 §5): Android's WebView behind the WebViews
// host service.
//
// The adapter creates, loads and destroys web views when the core asks, and reports what each
// one does. It decides nothing: whether a navigation may happen is the core's answer
// (`navigationVerdict`), and what the address field shows is the core's snapshot.
//
// One web view is made unasked: the one a page asks for as a new window, which the WebView
// must be handed at once (006). The adapter makes it, names it and reports it; what becomes
// of it is the core's to say.
package com.tigercodes.webtiger.webview

import android.content.Context
import android.view.View
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.setValue
import androidx.webkit.WebViewCompat
import com.tigercodes.webtiger.core.CoreClient
import com.tigercodes.webtiger.core.HostServices
import com.tigercodes.webtiger.core.NavigationVerdict
import com.tigercodes.webtiger.core.RequestVerdict
import com.tigercodes.webtiger.core.ResourceType
import com.tigercodes.webtiger.core.v1.Ack
import com.tigercodes.webtiger.core.v1.EngineEvent
import com.tigercodes.webtiger.core.v1.EngineInfo
import com.tigercodes.webtiger.core.v1.EngineKind
import com.tigercodes.webtiger.core.v1.Error
import com.tigercodes.webtiger.core.v1.HostPong
import com.tigercodes.webtiger.core.v1.HostRequest
import com.tigercodes.webtiger.core.v1.HostResponse
import com.tigercodes.webtiger.core.v1.NavigationCommand
import com.tigercodes.webtiger.core.v1.NewWindowRequested
import com.tigercodes.webtiger.core.v1.WebViewCommand
import com.tigercodes.webtiger.core.v1.WebViewCreate
import com.tigercodes.webtiger.core.v1.WebViewLoad

/**
 * Answers the WebViews host service with the System WebView. Used on the main thread only, as
 * a WebView must be; the core's fast path is the one thing asked from other threads.
 */
class SystemWebViewEngine internal constructor(private val factory: PageFactory) : HostServices {
    /** An engine over this device's WebView. */
    constructor(context: Context) : this(WebViewPages(context.applicationContext))

    private val pages = HashMap<Long, Page>()

    /** The login set each web view's pages run in. A window a page asks for is in the asking page's. */
    private val loginSets = HashMap<Long, String>()

    /**
     * How many web views this engine has made unasked, for pages that asked for a window. Each
     * is named by the next id of the shell's own, and no id is used twice.
     */
    private var ownWebViews = 0L

    @Volatile private var core: CoreClient? = null

    /**
     * Counts every change to the set of web views. A window reads it while it draws, so it
     * draws again when the web view it shows arrives, is replaced or goes.
     */
    var generation by mutableIntStateOf(0)
        private set

    /** Gives the engine the core its reports go to. Called once, after the core has started. */
    fun attach(core: CoreClient) {
        this.core = core
    }

    /** The view of the web view the core named `id`, for a window to show. */
    fun webView(id: Long): View? = pages[id]?.view

    /** How many web views exist. A tab that is asleep has none. */
    val webViewCount: Int get() = pages.size

    override suspend fun answer(request: HostRequest): HostResponse {
        val ping = request.ping
        val create = request.web_view_create
        val load = request.web_view_load
        val command = request.web_view_command
        val destroy = request.web_view_destroy
        val response =
            when {
                ping != null -> HostResponse(pong = HostPong(payload = ping.payload))
                create != null -> create(create)
                load != null -> load(load)
                command != null -> command(command)
                destroy != null -> {
                    pages.remove(destroy.web_view_id)?.close()
                    loginSets.remove(destroy.web_view_id)
                    done
                }
                // A request from a core newer than this shell.
                else -> failure("host.unsupported")
            }
        generation += 1
        return response
    }

    private fun create(create: WebViewCreate): HostResponse {
        if (create.engine != EngineKind.ENGINE_KIND_CHROMIUM) return failure("engine.unavailable")
        if (pages.containsKey(create.web_view_id)) return failure("webview.exists")
        val loginSet = create.login_set
        // A private window's login set is kept in memory and never written. A WebView profile
        // is always written to disk, so there is none to give it: refused, not approximated.
        if (loginSet?.ephemeral == true) return failure("engine.no_private")
        val name = loginSet?.id.orEmpty()
        if (!isUuid(name)) return failure("webview.bad_login_set")
        // Without profiles every web view would share the one default data store, and a login
        // set's cookies would be every other's. No web view is better than that.
        if (!factory.keepsLoginSetsApart) return failure("engine.no_profiles")
        val page =
            try {
                factory.create(create.web_view_id, name, Reports())
            } catch (_: RuntimeException) {
                // The device has no WebView, or it is being updated. The exception's words
                // stay here.
                return failure("engine.unavailable")
            }
        pages[create.web_view_id] = page
        loginSets[create.web_view_id] = name
        return done
    }

    /**
     * A page asked for a new window, and the WebView must be handed the web view at once: that
     * is what lets the page that asked keep its hold on the one it opened. The web view is made
     * in the asking page's login set, named with an id of the shell's own, shown nowhere, and
     * reported. The core adopts it into a tab or has it destroyed (006).
     */
    private fun newWindow(askingId: Long, userGesture: Boolean): Page? {
        val loginSet = loginSets[askingId] ?: return null
        if (!factory.keepsLoginSetsApart) return null
        val id = OWN_ID or (ownWebViews + 1)
        val page =
            try {
                factory.create(id, loginSet, Reports())
            } catch (_: RuntimeException) {
                return null
            }
        ownWebViews += 1
        pages[id] = page
        loginSets[id] = loginSet
        generation += 1
        // The WebView does not say where the page wants the window to go.
        core?.report(EngineEvent(web_view_id = askingId, new_window_requested = NewWindowRequested(new_web_view_id = id, user_gesture = userGesture)))
        return page
    }

    private fun load(load: WebViewLoad): HostResponse {
        val page = pages[load.web_view_id] ?: return failure("webview.unknown")
        // The core sends only addresses its own policy allows. It is asked once more here
        // because of what WebView.loadUrl does with the others: a `javascript:` address is run
        // as script in the page that is showing.
        val allowed =
            load.url.isNotEmpty() &&
                core?.navigationVerdict(load.url, load.web_view_id, mainFrame = true, userGesture = true) == NavigationVerdict.ALLOW
        if (!allowed) return failure("webview.bad_address")
        return if (page.load(load.url)) done else failure("engine.unavailable")
    }

    private fun command(command: WebViewCommand): HostResponse {
        val page = pages[command.web_view_id] ?: return failure("webview.unknown")
        when (command.command) {
            NavigationCommand.NAVIGATION_COMMAND_BACK,
            NavigationCommand.NAVIGATION_COMMAND_FORWARD,
            NavigationCommand.NAVIGATION_COMMAND_RELOAD,
            NavigationCommand.NAVIGATION_COMMAND_STOP,
            -> page.command(command.command)
            NavigationCommand.NAVIGATION_COMMAND_UNSPECIFIED -> return failure("webview.bad_command")
        }
        return done
    }

    private val done = HostResponse(ack = Ack())

    private fun failure(code: String) = HostResponse(error = Error(code = code))

    /** What the pages are given: their reports and questions go to the core that is attached. */
    private inner class Reports : PageHost {
        override fun report(event: EngineEvent) {
            core?.report(event)
        }

        override fun navigationVerdict(url: String, webViewId: Long, mainFrame: Boolean, userGesture: Boolean): NavigationVerdict =
            core?.navigationVerdict(url, webViewId, mainFrame, userGesture) ?: NavigationVerdict.CANCEL

        override fun requestVerdict(url: String, sourceUrl: String, webViewId: Long, resource: ResourceType): RequestVerdict =
            core?.requestVerdict(url, sourceUrl, webViewId, resource) ?: RequestVerdict.ALLOW

        override fun newWindow(askingId: Long, userGesture: Boolean): Page? = this@SystemWebViewEngine.newWindow(askingId, userGesture)
    }

    companion object {
        /**
         * The bit that marks a web view's id as the shell's own: the core's ids never have it
         * (`NewWindowRequested`, engine.proto). It is the sign bit of a Long, so these ids are
         * the negative ones.
         */
        const val OWN_ID = Long.MIN_VALUE

        /** What this engine is, for `CoreConfig.engines`. */
        fun info(context: Context): EngineInfo =
            EngineInfo(
                kind = EngineKind.ENGINE_KIND_CHROMIUM,
                version = WebViewCompat.getCurrentWebViewPackage(context)?.versionName.orEmpty(),
            )

        /** Whether `text` is a UUID as the contract writes one: lower case, with hyphens. */
        internal fun isUuid(text: String): Boolean =
            text.length == 36 &&
                text.indices.all { index ->
                    val character = text[index]
                    if (index == 8 || index == 13 || index == 18 || index == 23) character == '-' else character in '0'..'9' || character in 'a'..'f'
                }
    }
}

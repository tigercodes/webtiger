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
import android.os.Handler
import android.os.Looper
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
import com.tigercodes.webtiger.core.StorageVerdict
import com.tigercodes.webtiger.core.v1.Ack
import com.tigercodes.webtiger.core.v1.DialogKind
import com.tigercodes.webtiger.core.v1.EngineEvent
import com.tigercodes.webtiger.core.v1.EngineInfo
import com.tigercodes.webtiger.core.v1.EngineKind
import com.tigercodes.webtiger.core.v1.Error
import com.tigercodes.webtiger.core.v1.HostPong
import com.tigercodes.webtiger.core.v1.HostRequest
import com.tigercodes.webtiger.core.v1.HostResponse
import com.tigercodes.webtiger.core.v1.LinkOpenRequested
import com.tigercodes.webtiger.core.v1.NavigationCommand
import com.tigercodes.webtiger.core.v1.NewWindowRequested
import com.tigercodes.webtiger.core.v1.PermissionKind
import com.tigercodes.webtiger.core.v1.SearchRequested
import com.tigercodes.webtiger.core.v1.WebViewCommand
import com.tigercodes.webtiger.core.v1.WebViewCreate
import com.tigercodes.webtiger.core.v1.WebViewLoad
import com.tigercodes.webtiger.core.v1.WebViewPicture
import com.tigercodes.webtiger.core.v1.WebViewSnapshot
import com.tigercodes.webtiger.core.v1.WebViewState
import okio.ByteString
import okio.ByteString.Companion.toByteString

/**
 * Answers the WebViews host service with the System WebView. Used on the main thread only, as
 * a WebView must be; the core's fast path is the one thing asked from other threads.
 */
class SystemWebViewEngine internal constructor(
    private val factory: PageFactory,
    fetcher: Fetcher = NoFetcher,
    /** What the system lets the app itself use (014): a page is let have no more. */
    system: SystemPermissions = NoSystemPermissions,
    /** How a page's file input is given files (019): nothing is picked unless the app itself gave the way to the system's picker. */
    chooser: FileChooser = NoFiles,
    /** Asks where a file the person chose is, and what its site says of it, before it is fetched (022). */
    asker: Asker = NoAsker,
    /** Runs something a moment later, on the main thread: how the downloads' progress is asked after. */
    private val later: (() -> Unit) -> Unit = {},
) : HostServices {
    /**
     * An engine over this device's WebView. Its downloads go to the app's own folder unless it
     * is told the device's: only the app itself, started for real, saves where the person's own
     * files are (013).
     */
    constructor(
        context: Context,
        downloads: DownloadPlace = DownloadPlace.APP,
        system: SystemPermissions = AppPermissions(context),
        files: FileChooser = NoFiles,
    ) : this(
        WebViewPages(context.applicationContext),
        AppFetcher(context.applicationContext, downloads),
        system,
        files,
        SystemAsker(),
        Handler(Looper.getMainLooper()).let { main -> { tick: () -> Unit -> main.postDelayed(tick, DOWNLOAD_PACE_MS) } },
    )

    /** What pages ask to use of the device's (014), while the core has not said. */
    private val permissions = Permissions(system) { event -> core?.report(event) }

    /** Downloads: what is being fetched for a page, and what waits for the core's word. */
    private val downloads = Downloads(fetcher, asker) { event -> core?.report(event) }

    /** What pages say or ask (016), while the core has not said how each is answered. */
    private val dialogs = Dialogs { event -> core?.report(event) }

    /** The web views whose pages have a file chooser open (019). */
    private val files = Files(chooser)

    /** The servers that asked who is there (017), while the core has not said how each is answered. */
    private val signIns = SignIns { event -> core?.report(event) }

    /**
     * Told when the person presses and holds a link or a picture of a page (018, 022): the web
     * view, and what was pressed. The window shows the menu; with nobody to tell, nothing
     * comes of the press.
     */
    var onPressed: ((Long, Pressed) -> Unit)? = null

    /**
     * The person chose, in a menu, to save a file of a web view's page (022): what a link
     * leads to, or a picture. It is offered as a download the person asked for: whether it
     * starts, and under what name, is the core's to say. Only an address of the web is
     * fetched.
     */
    fun saveFile(webViewId: Long, url: String) {
        if (isOfTheWeb(url)) pages[webViewId]?.save(url)
    }

    /**
     * The person asked, by a link's menu, for the link in a new tab (018). The core is told:
     * whether a tab opens is its to say.
     */
    fun openLink(webViewId: Long, url: String, background: Boolean) {
        core?.report(EngineEvent(web_view_id = webViewId, link_open_requested = LinkOpenRequested(url = url, background = background, user_gesture = true)))
    }

    /** A tick is on its way: asking after the downloads is not asked for twice. */
    private var ticking = false

    private val pages = HashMap<Long, Page>()

    /** The login set each web view's pages run in. A window a page asks for is in the asking page's. */
    private val loginSets = HashMap<Long, String>()

    /** The web views whose pages are told Global Privacy Control (069). */
    private val privacySignalled = HashSet<Long>()

    /** The login sets of this run that are private windows' (037): cleared when their last page is gone. */
    private val thrownAway = HashSet<String>()

    /** The profiles an earlier run's private windows left have been looked for. */
    private var swept = false

    /** The certificates a person went on past (026). Until the process ends, and kept nowhere else. */
    internal val certificates = Certificates()

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

    /**
     * What the engine and the adapter said last of the web view the core named `id`, by kind
     * and with no address: for a test's failure message.
     */
    internal fun heard(id: Long): List<String> = (pages[id] as? WebViewPage)?.heard().orEmpty()

    /** How many web views exist. A tab that is asleep has none. */
    val webViewCount: Int get() = pages.size

    override suspend fun answer(request: HostRequest): HostResponse {
        val ping = request.ping
        val create = request.web_view_create
        val load = request.web_view_load
        val command = request.web_view_command
        val destroy = request.web_view_destroy
        val find = request.web_view_find
        val zoom = request.web_view_set_zoom
        val start = request.download_start
        val cancel = request.download_cancel
        val permission = request.permission_answer
        val dialog = request.dialog_answer
        val signIn = request.sign_in_answer
        val trust = request.web_view_trust_certificate
        val saveState = request.web_view_save_state
        val restore = request.web_view_restore
        val snapshot = request.web_view_snapshot
        val response =
            when {
                // The core's word on a download this adapter reported (013).
                start != null ->
                    if (downloads.start(start.download_id, start.file_name)) {
                        watchDownloads()
                        done
                    } else {
                        failure("download.unknown")
                    }
                cancel != null -> {
                    downloads.cancel(cancel.download_id)
                    done
                }
                // The core's word on what a page asked to use (014). Done, also for a request
                // that no longer waits: its page went away.
                permission != null -> {
                    permissions.answer(permission.request_id, permission.granted)
                    done
                }
                // The core's word on what a page said or asked (016). Done, also for one
                // that no longer waits.
                dialog != null -> {
                    dialogs.answer(dialog.request_id, dialog.accepted, dialog.text)
                    done
                }
                // The core's word on a server that asked who is there (017). Done, also for
                // one that no longer waits.
                signIn != null -> {
                    signIns.answer(signIn.request_id, signIn.accepted, signIn.name, signIn.password)
                    done
                }
                zoom != null -> {
                    val page = pages[zoom.web_view_id]
                    if (page == null) {
                        failure("webview.unknown")
                    } else {
                        page.zoom(zoom.percent)
                        done
                    }
                }
                find != null -> {
                    val page = pages[find.web_view_id]
                    if (page == null) {
                        failure("webview.unknown")
                    } else {
                        page.find(find.text, find.backwards)
                        done
                    }
                }
                // The person went on past a certificate (026): what this web view last refused
                // is let through from now, in its login set.
                trust != null -> {
                    val loginSet = loginSets[trust.web_view_id]
                    when {
                        loginSet == null -> failure("webview.unknown")
                        certificates.trust(trust.web_view_id, loginSet) -> done
                        else -> failure("certificate.unknown")
                    }
                }
                ping != null -> HostResponse(pong = HostPong(payload = ping.payload))
                create != null -> create(create)
                load != null -> load(load)
                saveState != null -> state(saveState.web_view_id)
                snapshot != null -> picture(snapshot)
                restore != null -> go(restore.web_view_id, restore.url) { page -> page.restore(restore.state.toByteArray(), restore.url) }
                command != null -> command(command)
                destroy != null -> {
                    // What its page was asking goes with it: refused, while there is a page to tell.
                    permissions.dropped(destroy.web_view_id)
                    dialogs.dropped(destroy.web_view_id)
                    signIns.dropped(destroy.web_view_id)
                    files.dropped(destroy.web_view_id)
                    pages.remove(destroy.web_view_id)?.close()
                    val loginSet = loginSets.remove(destroy.web_view_id)
                    privacySignalled.remove(destroy.web_view_id)
                    certificates.dropped(destroy.web_view_id)
                    // A private window's login set, with its last page gone (037): what its
                    // pages stored is removed now, and its profile when the app next starts.
                    if (loginSet != null && loginSet in thrownAway && loginSet !in loginSets.values) factory.clear(loginSet)
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
        val name = loginSet?.id.orEmpty()
        if (!isUuid(name)) return failure("webview.bad_login_set")
        // Without profiles every web view would share the one default data store, and a login
        // set's cookies would be every other's. No web view is better than that.
        if (!factory.keepsLoginSetsApart) return failure("engine.no_profiles")
        // The profiles of the private windows of an earlier run are deleted before any web
        // view is made (037): the only time the WebView lets a profile be deleted.
        if (!swept) {
            swept = true
            factory.sweep()
        }
        // A private window's login set (037). The WebView has no login set in memory: a
        // profile is always on disk. So it is a profile to throw away: written down as one
        // before it is made, cleared when its last page is gone, and deleted when the app
        // next starts. One that cannot be written down is not made.
        if (loginSet?.ephemeral == true) {
            if (!factory.willThrowAway(name)) return failure("engine.no_private")
            thrownAway.add(name)
        }
        val page =
            try {
                factory.create(create.web_view_id, name, forgetful = name in thrownAway, privacySignal = create.global_privacy_control, Reports())
            } catch (_: RuntimeException) {
                // The device has no WebView, or it is being updated. The exception's words
                // stay here.
                return failure("engine.unavailable")
            }
        pages[create.web_view_id] = page
        loginSets[create.web_view_id] = name
        if (create.global_privacy_control) privacySignalled.add(create.web_view_id)
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
                // A window a page opened is told the signal as the page that opened it is (069).
                factory.create(id, loginSet, forgetful = loginSet in thrownAway, privacySignal = askingId in privacySignalled, Reports())
            } catch (_: RuntimeException) {
                return null
            }
        ownWebViews += 1
        pages[id] = page
        loginSets[id] = loginSet
        if (askingId in privacySignalled) privacySignalled.add(id)
        generation += 1
        // The WebView does not say where the page wants the window to go.
        core?.report(EngineEvent(web_view_id = askingId, new_window_requested = NewWindowRequested(new_web_view_id = id, user_gesture = userGesture)))
        return page
    }

    private fun load(load: WebViewLoad): HostResponse = go(load.web_view_id, load.url) { page -> page.load(load.url) }

    /**
     * Sends a web view to an address: by a load, or by a state that is at it (051), as `how`
     * does it.
     */
    private fun go(webViewId: Long, url: String, how: (Page) -> Boolean): HostResponse {
        val page = pages[webViewId] ?: return failure("webview.unknown")
        // The core sends only addresses its own policy allows. It is asked once more here
        // because of what WebView.loadUrl does with the others: a `javascript:` address is run
        // as script in the page that is showing.
        val allowed = url.isNotEmpty() && core?.navigationVerdict(url, webViewId, mainFrame = true, userGesture = true) == NavigationVerdict.ALLOW
        if (!allowed) return failure("webview.bad_address")
        return if (how(page)) done else failure("engine.unavailable")
    }

    /** A web view's own state (051), or none where its engine has none to give. */
    private fun state(webViewId: Long): HostResponse {
        val page = pages[webViewId] ?: return failure("webview.unknown")
        return HostResponse(web_view_state = WebViewState(state = page.state()?.toByteString() ?: ByteString.EMPTY))
    }

    /**
     * A picture of a web view as it is (059), for where its page will be when its tab wakes:
     * with no bytes when there is none to give, or none as small as was asked for.
     */
    private fun picture(asked: WebViewSnapshot): HostResponse {
        val page = pages[asked.web_view_id] ?: return failure("webview.unknown")
        val taken = page.picture(asked.max_width, asked.max_bytes) ?: return HostResponse(web_view_picture = WebViewPicture())
        return HostResponse(web_view_picture = WebViewPicture(jpeg = taken.jpeg.toByteString(), width = taken.width, height = taken.height))
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
        override val certificates: Certificates get() = this@SystemWebViewEngine.certificates

        override fun report(event: EngineEvent) {
            core?.report(event)
        }

        override fun navigationVerdict(url: String, webViewId: Long, mainFrame: Boolean, userGesture: Boolean): NavigationVerdict =
            core?.navigationVerdict(url, webViewId, mainFrame, userGesture) ?: NavigationVerdict.CANCEL

        override fun requestVerdict(url: String, sourceUrl: String, webViewId: Long, resource: ResourceType): RequestVerdict =
            core?.requestVerdict(url, sourceUrl, webViewId, resource) ?: RequestVerdict.ALLOW

        // With no core to ask, a frame is kept apart: a third party is let at its storage on
        // the core's word alone (071).
        override fun storageVerdict(origin: String, topOrigin: String, pageUrl: String, webViewId: Long): StorageVerdict =
            core?.storageVerdict(origin, topOrigin, pageUrl, webViewId) ?: StorageVerdict.APART

        override fun newWindow(askingId: Long, userGesture: Boolean): Page? = this@SystemWebViewEngine.newWindow(askingId, userGesture)

        override fun permissionAsked(webViewId: Long, origin: String, kinds: List<PermissionKind>, answer: (Boolean) -> Unit): Long =
            permissions.asked(webViewId, origin, kinds, answer)

        override fun permissionWithdrawn(request: Long) = permissions.withdrawn(request)

        override fun dialog(webViewId: Long, url: String, kind: DialogKind, message: String, defaultText: String, answer: (Boolean, String) -> Unit) =
            dialogs.asked(webViewId, url, kind, message, defaultText, answer)

        override fun signIn(webViewId: Long, host: String, realm: String, heading: String, again: Boolean, answer: (Typed?) -> Unit) =
            signIns.asked(webViewId, host, realm, heading, again, answer)

        override fun files(webViewId: Long, asked: FilesAsked, give: (List<String>?) -> Unit) = files.choose(webViewId, asked, give)

        override fun searchFor(webViewId: Long, words: String) {
            // The core is told, as the person's doing: where the search goes, and whether a
            // tab opens for it, are its to say.
            core?.report(EngineEvent(web_view_id = webViewId, search_requested = SearchRequested(text = words, user_gesture = true)))
        }

        override fun pressed(webViewId: Long, pressed: Pressed) {
            // A picture that is not of the web can be neither opened in a tab nor fetched; and
            // a link the WebView could say nothing of leads nowhere.
            val told = pressed.copy(picture = pressed.picture.takeIf(::isOfTheWeb).orEmpty())
            if (told.link.isNotEmpty() || told.picture.isNotEmpty()) onPressed?.invoke(webViewId, told)
        }

        override fun download(webViewId: Long, offer: DownloadOffer) {
            // A download is its login set's: its cookies are what the fetch is given.
            val loginSet = loginSets[webViewId] ?: return
            downloads.offered(webViewId, loginSet, offer)
        }
    }

    /**
     * Asks how the downloads that are coming stand, and reports what changed. The engine asks
     * by itself, a couple of times a second, for as long as one is coming.
     */
    fun pollDownloads() {
        downloads.poll()
    }

    private fun watchDownloads() {
        if (ticking || !downloads.isBusy) return
        ticking = true
        later {
            ticking = false
            pollDownloads()
            watchDownloads()
        }
    }

    companion object {
        /** How long between two looks at the downloads that are coming. */
        private const val DOWNLOAD_PACE_MS = 400L

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

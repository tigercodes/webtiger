// One core and one engine for the whole app. Windows ask it for their models.
package com.tigercodes.webtiger.ui

import android.content.Context
import com.tigercodes.webtiger.core.CoreClient
import com.tigercodes.webtiger.core.CoreError
import com.tigercodes.webtiger.core.NativeCore
import com.tigercodes.webtiger.core.perform
import com.tigercodes.webtiger.core.v1.CloseWindow
import com.tigercodes.webtiger.core.v1.CoreConfig
import com.tigercodes.webtiger.core.v1.EngineInfo
import com.tigercodes.webtiger.core.v1.Event
import com.tigercodes.webtiger.core.v1.FakeOptions
import com.tigercodes.webtiger.core.v1.MemoryLevel
import com.tigercodes.webtiger.core.v1.MemoryShort
import com.tigercodes.webtiger.core.v1.OpenWindow
import com.tigercodes.webtiger.core.v1.Platform
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.Subscribe
import com.tigercodes.webtiger.core.v1.TestMode
import com.tigercodes.webtiger.core.v1.WindowSnapshot
import com.tigercodes.webtiger.core.v1.WindowTopic
import com.tigercodes.webtiger.webview.AppFiles
import com.tigercodes.webtiger.webview.AppPermissions
import com.tigercodes.webtiger.webview.DownloadPlace
import com.tigercodes.webtiger.webview.NoFiles
import com.tigercodes.webtiger.webview.SystemWebViewEngine
import java.io.File
import java.util.Locale
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.MainScope
import kotlinx.coroutines.launch

/**
 * How the app was asked to start. Fake mode, for UI suites and screenshots, keeps the store in
 * memory and makes ids deterministic; [fakeStart] names an address every new window opens.
 * [catalogue] asks for the design catalogue in place of a browser (004).
 */
data class LaunchOptions(
    val fake: Boolean = false,
    val fakeStart: String = "",
    val catalogue: Boolean = false,
    /** Fake mode: the run is to be a first run, in which nobody has chosen where searches go (020). */
    val firstRun: Boolean = false,
    /**
     * Fake mode: how far the core's clock moves each time it is read, in seconds, for a suite
     * that must see a day pass (049). 0: a second.
     */
    val clockStep: Int = 0,
    /**
     * Fake mode: how long a tab is out of sight before the core puts it to sleep, in seconds,
     * for a suite that must see one sleep (053). 0: the usual half hour.
     */
    val sleepAfter: Int = 0,
    /**
     * Fake mode: the most tabs that are alive at once, for a suite that must see the limit met
     * (054). 0: none, whatever the device.
     */
    val liveTabs: Int = 0,
    /**
     * Fake mode: how often the core acts as though the system had said it is short of memory,
     * in seconds, for a suite that must see what follows (055). 0: never.
     */
    val memoryShortEvery: Int = 0,
)

/**
 * The app's connection to the core: started once, shared by every window. Used on the main
 * thread, where [scope] runs.
 */
class BrowserSession(
    val core: CoreClient,
    val engine: SystemWebViewEngine,
    internal val scope: CoroutineScope,
    /**
     * The app's own permissions (014), for a window to give the way to ask the system by. Null
     * where nothing may bring the system's dialog up: fake mode, and every session a test makes.
     */
    val systemPermissions: AppPermissions? = null,
    /**
     * Where "Copy Link" puts an address (018). The session's own, unless it is given the
     * system's: fake mode, and every session a test makes, leave the person's clipboard alone.
     */
    val clipboard: LinkClipboard = MemoryClipboard(),
    /**
     * The app's file chooser (019), for a window to give the way to the system's picker by.
     * Null where nothing may start it: fake mode, and every session a test makes.
     */
    val files: AppFiles? = null,
) {
    private val windows = HashMap<Long, (WindowSnapshot) -> Unit>()

    /**
     * What the system says of its memory, passed on to the core (055). Null in a session that
     * does not listen to the system: fake mode's, and every session a test makes.
     */
    var memory: MemoryWatch? = null
        private set

    /** Windows being opened, and the snapshots that came for subscriptions not yet known to be theirs. */
    private var opening = 0
    private val unclaimed = HashMap<Long, WindowSnapshot>()

    init {
        engine.attach(core)
        scope.launch { core.events.collect(::receive) }
    }

    /** Opens a window in the core and delivers its snapshots to `update` until it is closed. */
    internal suspend fun openWindow(private: Boolean = false, update: (WindowSnapshot) -> Unit): OpenedWindow {
        opening += 1
        try {
            val opened =
                core.request(Request(open_window = OpenWindow(private_window = private))).window_opened
                    ?: throw CoreError.Refused(code = "response.unexpected", field = "")
            val subscribed =
                core.request(Request(subscribe = Subscribe(window = WindowTopic(window_id = opened.window_id)))).subscribed
                    ?: throw CoreError.Refused(code = "response.unexpected", field = "")
            windows[subscribed.subscription_id] = update
            // The first snapshot follows the answer at once, and may have been taken off the
            // events before this coroutine was told its subscription's id.
            unclaimed.remove(subscribed.subscription_id)?.let(update)
            return OpenedWindow(opened.window_id, opened.tab_id, subscribed.subscription_id)
        } finally {
            opening -= 1
            if (opening == 0) unclaimed.clear()
        }
    }

    internal fun closeWindow(window: OpenedWindow) {
        windows.remove(window.subscriptionId)
        send(Request(close_window = CloseWindow(window_id = window.windowId)))
    }

    /**
     * Sends a request whose answer nobody waits for. A refusal shows in the next snapshot (a
     * failed load) or changes nothing (a command for a tab with no page); there is nothing
     * more to tell the user here.
     *
     * What was done first reaches the core first: each request is sent from a coroutine of its
     * own, but the main thread starts them one after the other, in the order they were
     * launched, and a request is handed to the core before its coroutine first waits. A scope
     * on a pool of threads would not keep that order.
     */
    internal fun send(request: Request) {
        scope.launch {
            try {
                core.perform(request)
            } catch (_: CoreError) {
            }
        }
    }

    /** Says to the core that the system is short of memory, so short (055). Nothing is waited for. */
    internal fun memoryShort(level: MemoryLevel) {
        send(Request(memory_short = MemoryShort(level = level)))
    }

    private fun receive(event: Event) {
        val snapshot = event.snapshot ?: return
        val window = snapshot.window ?: return
        val update = windows[snapshot.subscription_id]
        if (update != null) {
            update(window)
        } else if (opening > 0) {
            unclaimed[snapshot.subscription_id] = window
        }
    }

    companion object {
        /** Starts the Rust core for this process. Throws [CoreError] when it would not start. */
        fun live(context: Context, launch: LaunchOptions): BrowserSession {
            val application = context.applicationContext
            val permissions = AppPermissions(application)
            // A page's file input opens the system's picker (019). Not in fake mode: its engine
            // picks nothing, and starts nothing of the system's.
            val files = AppFiles().takeIf { !launch.fake }
            val engine = SystemWebViewEngine(application, downloadPlace(launch), permissions, files ?: NoFiles)
            val config =
                coreConfig(
                    launch = launch,
                    locale = Locale.getDefault().toLanguageTag(),
                    appVersion = application.packageManager.getPackageInfo(application.packageName, 0).versionName.orEmpty(),
                    engine = SystemWebViewEngine.info(application),
                    // The app's own files, which no other app can read. The core makes the folder.
                    dataDir = File(application.filesDir, "core").path,
                    device = Device.of(application),
                )
            // In fake mode nothing leaves the app: a link for the system opens no other app (007).
            val opener = if (launch.fake) OutsideOpener.nothing else SystemOpener(application)
            // Host requests are answered on the main thread, where a WebView must be used.
            // A file is opened from where the real app saves: fake mode saves in the app's own
            // folder, and opens nothing.
            val opensFiles = if (launch.fake) FileOpener.nothing else SystemFileOpener(application)
            val core = NativeCore.start(config, ShellHost(engine, opener, opensFiles), Dispatchers.Main)
            val clipboard = if (launch.fake) MemoryClipboard() else SystemClipboard(application)
            val session = BrowserSession(core, engine, MainScope(), permissions.takeIf { mayAskTheSystem(launch) }, clipboard, files)
            if (hearsTheSystemsMemory(launch)) {
                session.memory = MemoryWatch.of(application, session.scope, session::memoryShort).also(application::registerComponentCallbacks)
            }
            return session
        }

        /**
         * Whether the system's word that it is short of memory is passed on to the core (055).
         * Not in fake mode: a suite's run must not go by how busy its device is, and it has a
         * knob for the core to be told by.
         */
        internal fun hearsTheSystemsMemory(launch: LaunchOptions): Boolean = !launch.fake

        /**
         * What the core is started with. In fake mode the store is in memory, so no folder is
         * named, and every new window opens [LaunchOptions.fakeStart].
         */
        /**
         * Where the app's downloads go (013): the device's Downloads. In fake mode, the app's
         * own folder: a UI suite writes nothing where a person's files are.
         */
        internal fun downloadPlace(launch: LaunchOptions): DownloadPlace = if (launch.fake) DownloadPlace.APP else DownloadPlace.DEVICE

        /**
         * Whether a window may have the system ask the person for the app's own permissions
         * (014): the camera, the microphone, the location. Not in fake mode: a UI suite puts no
         * dialog of the system's on the screen, and a page is let have what the app holds.
         */
        internal fun mayAskTheSystem(launch: LaunchOptions): Boolean = !launch.fake

        internal fun coreConfig(launch: LaunchOptions, locale: String, appVersion: String, engine: EngineInfo, dataDir: String, device: Device): CoreConfig =
            CoreConfig(
                platform = Platform.PLATFORM_ANDROID,
                locale = locale,
                app_version = appVersion,
                engines = listOf(engine),
                test_mode = if (launch.fake) TestMode.TEST_MODE_FAKE else TestMode.TEST_MODE_LIVE,
                fake = if (launch.fake) fakeOptions(launch) else null,
                data_dir = if (launch.fake) "" else dataDir,
                // What the app runs on (054): the core works out the limit of live tabs from it.
                device_kind = device.kind,
                memory_megabytes = device.megabytes,
            )

        /** Fake mode's knobs, as the launch asked for them. */
        private fun fakeOptions(launch: LaunchOptions): FakeOptions =
            FakeOptions(
                start_address = launch.fakeStart,
                first_run = launch.firstRun,
                seconds_a_read = launch.clockStep,
                sleep_after_seconds = launch.sleepAfter,
                live_tabs = launch.liveTabs,
                memory_short_every_seconds = launch.memoryShortEvery,
            )
    }
}

internal data class OpenedWindow(val windowId: Long, val tabId: Long, val subscriptionId: Long)

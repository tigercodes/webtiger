// The Rust core, through Kotlin: the same contract the Rust tests drive, crossing the real JNI
// glue into the real library (built for this machine by :core-engine:cargoBuildHost).
package com.tigercodes.webtiger.core

import com.tigercodes.webtiger.core.v1.Ack
import com.tigercodes.webtiger.core.v1.AddressView
import com.tigercodes.webtiger.core.v1.CloseTab
import com.tigercodes.webtiger.core.v1.CoreConfig
import com.tigercodes.webtiger.core.v1.CoreStatus
import com.tigercodes.webtiger.core.v1.EngineEvent
import com.tigercodes.webtiger.core.v1.EngineInfo
import com.tigercodes.webtiger.core.v1.EngineKind
import com.tigercodes.webtiger.core.v1.Event
import com.tigercodes.webtiger.core.v1.FakeOptions
import com.tigercodes.webtiger.core.v1.ForgetHistory
import com.tigercodes.webtiger.core.v1.HostPong
import com.tigercodes.webtiger.core.v1.HostRequest
import com.tigercodes.webtiger.core.v1.HostResponse
import com.tigercodes.webtiger.core.v1.LoadState
import com.tigercodes.webtiger.core.v1.NavigationCommitted
import com.tigercodes.webtiger.core.v1.NavigationFinished
import com.tigercodes.webtiger.core.v1.OpenTab
import com.tigercodes.webtiger.core.v1.OpenWindow
import com.tigercodes.webtiger.core.v1.Ping
import com.tigercodes.webtiger.core.v1.Platform
import com.tigercodes.webtiger.core.v1.QueryHistory
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.SubmitAddress
import com.tigercodes.webtiger.core.v1.SuggestAddresses
import com.tigercodes.webtiger.core.v1.Subscribe
import com.tigercodes.webtiger.core.v1.TabView
import com.tigercodes.webtiger.core.v1.TestMode
import com.tigercodes.webtiger.core.v1.TitleChanged
import com.tigercodes.webtiger.core.v1.WindowOpened
import com.tigercodes.webtiger.core.v1.WindowTopic
import java.io.File
import java.nio.file.Files
import java.util.UUID
import java.util.concurrent.CopyOnWriteArrayList
import java.util.concurrent.Executors
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExecutorCoroutineDispatcher
import kotlinx.coroutines.asCoroutineDispatcher
import kotlinx.coroutines.awaitCancellation
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import kotlinx.coroutines.withTimeoutOrNull
import okio.ByteString.Companion.encodeUtf8
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Test

/** A host that records what the core asks and answers as a shell with a working engine would. */
private class RecordingHost : HostServices {
    val requests = CopyOnWriteArrayList<HostRequest>()
    val threads = CopyOnWriteArrayList<Thread>()

    override suspend fun answer(request: HostRequest): HostResponse {
        requests.add(request)
        threads.add(Thread.currentThread())
        val ping = request.ping
        return if (ping != null) HostResponse(pong = HostPong(payload = ping.payload)) else HostResponse(ack = Ack())
    }
}

/** How long a test waits for the core to say something. */
private const val PATIENCE_MS = 10_000L

class NativeCoreTest {
    /** Stands in for the main thread: the one thread host requests are answered on. */
    private val hostThread: ExecutorCoroutineDispatcher = Executors.newSingleThreadExecutor().asCoroutineDispatcher()
    private val collecting = CoroutineScope(Dispatchers.Default)
    private val cores = ArrayList<NativeCore>()

    @After
    fun stopEverything() {
        cores.forEach { it.stop() }
        collecting.cancel()
        hostThread.close()
    }

    private fun fakeConfig(start: String = "") =
        CoreConfig(
            platform = Platform.PLATFORM_ANDROID,
            locale = "en-GB",
            test_mode = TestMode.TEST_MODE_FAKE,
            fake = FakeOptions(start_address = start),
            app_version = "0.0.0-test",
            engines = listOf(EngineInfo(kind = EngineKind.ENGINE_KIND_CHROMIUM, version = "test")),
        )

    private fun start(config: CoreConfig = fakeConfig(), host: HostServices = RecordingHost()): NativeCore =
        NativeCore.start(config, host, hostThread).also { cores.add(it) }

    /** Keeps everything a core says unasked. */
    private fun eventsOf(core: NativeCore): CopyOnWriteArrayList<Event> {
        val seen = CopyOnWriteArrayList<Event>()
        collecting.launch { core.events.collect { seen.add(it) } }
        return seen
    }

    /** The first tab snapshot that matches, waiting for it. */
    private suspend fun firstTab(events: List<Event>, matches: (TabView) -> Boolean): TabView? =
        withTimeoutOrNull(PATIENCE_MS) {
            var found: TabView? = null
            while (found == null) {
                found = events.firstNotNullOfOrNull { event -> event.snapshot?.window?.tab?.takeIf(matches) }
                if (found == null) delay(5)
            }
            found
        }

    private suspend fun waitUntil(done: () -> Boolean) {
        withTimeout(PATIENCE_MS) { while (!done()) delay(5) }
    }

    private suspend fun openWindow(core: NativeCore): WindowOpened {
        val opened = core.request(Request(open_window = OpenWindow())).window_opened
        assertNotNull("expected WindowOpened", opened)
        val subscribed = core.request(Request(subscribe = Subscribe(window = WindowTopic(window_id = opened!!.window_id)))).subscribed
        assertNotNull("expected Subscribed", subscribed)
        assertTrue("a subscription's id is never 0", subscribed!!.subscription_id != 0L)
        return opened
    }

    @Test
    fun theLibrarySpeaksThisShellsContract() {
        assertEquals(NativeCore.CONTRACT_MAJOR, CoreLibrary.version() ushr 16)
        assertEquals(NativeCore.CONTRACT_MINOR, CoreLibrary.version() and 0xFFFF)
        start()
    }

    @Test
    fun aStartedCoreSaysItIsReady() =
        runBlocking {
            val events = eventsOf(start())
            waitUntil { events.isNotEmpty() }
            assertEquals(CoreStatus.State.STATE_READY, events.first().status?.state)
        }

    @Test
    fun pingGoesToTheCoreAndRoundThroughTheHost() =
        runBlocking {
            val host = RecordingHost()
            val core = start(host = host)

            val direct = core.request(Request(ping = Ping(payload = "hello".encodeUtf8()))).pong
            assertEquals("hello".encodeUtf8(), direct?.payload)
            assertEquals(false, direct?.via_host)
            assertEquals(NativeCore.CONTRACT_MAJOR, direct?.contract_major)
            assertEquals(NativeCore.CONTRACT_MINOR, direct?.contract_minor)

            val round = core.request(Request(ping = Ping(payload = "hello".encodeUtf8(), via_host = true))).pong
            assertEquals(true, round?.via_host)
            assertEquals("hello".encodeUtf8(), round?.payload)
            assertEquals(1, host.requests.size)
            assertNotNull(host.requests[0].ping)
        }

    @Test
    fun aPageLoadsThroughTheContract() =
        runBlocking {
            val host = RecordingHost()
            val core = start(host = host)
            val events = eventsOf(core)
            val opened = openWindow(core)

            val empty = firstTab(events) { it.tab_id == opened.tab_id }
            assertEquals("a new tab has no web view", 0L, empty?.web_view_id)
            assertEquals(LoadState.LOAD_STATE_IDLE, empty?.load_state)

            core.perform(Request(submit_address = SubmitAddress(tab_id = opened.tab_id, text = "example.com")))

            // The core asked for a web view in the Space's login set, then for the load: in that
            // order, and on the one thread that answers the host's requests.
            assertEquals(2, host.requests.size)
            val create = host.requests[0].web_view_create
            val load = host.requests[1].web_view_load
            assertNotNull("the first host request creates the web view", create)
            assertNotNull("the second host request loads the address", load)
            assertEquals(EngineKind.ENGINE_KIND_CHROMIUM, create!!.engine)
            assertTrue("a web view's id is never 0", create.web_view_id != 0L)
            val loginSet = create.login_set?.id.orEmpty()
            assertEquals("a login set is named by a UUID, lower case", UUID.fromString(loginSet).toString(), loginSet)
            assertEquals(false, create.login_set?.ephemeral)
            assertEquals(create.web_view_id, load!!.web_view_id)
            assertEquals("https://example.com/", load.url)
            assertEquals(1, host.threads.distinct().size)

            // Until the page commits, the field shows where the user sent the tab.
            val loading = firstTab(events) { it.load_state == LoadState.LOAD_STATE_LOADING }
            assertEquals(create.web_view_id, loading?.web_view_id)
            assertEquals("example.com", loading?.address?.display)

            // The engine reports the page arriving; the core's snapshot says what to show.
            val id = create.web_view_id
            core.report(EngineEvent(web_view_id = id, navigation_committed = NavigationCommitted(url = "https://example.com/")))
            core.report(EngineEvent(web_view_id = id, title_changed = TitleChanged(title = "Example Domain")))
            core.report(EngineEvent(web_view_id = id, navigation_finished = NavigationFinished()))

            val tab = firstTab(events) { it.load_state == LoadState.LOAD_STATE_IDLE && it.title.isNotEmpty() }
            assertEquals("Example Domain", tab?.title)
            assertEquals(id, tab?.web_view_id)
            assertEquals("https://example.com/", tab?.address?.url)
            assertEquals("example.com", tab?.address?.display)
            assertEquals("example.com", tab?.address?.registrable_domain)
            assertEquals(AddressView.Security.SECURITY_SECURE, tab?.address?.security)
            assertEquals(EngineKind.ENGINE_KIND_CHROMIUM, tab?.engine)
        }

    /**
     * Every kind of answer the core gives comes back as what it is. (The client named the kinds
     * it knew, and took any other for no answer at all: a new tab's, and a page of history.)
     */
    @Test
    fun everyKindOfAnswerComesBackAsWhatItIs() =
        runBlocking {
            val core = start()
            val opened = openWindow(core)

            assertNotNull(core.request(Request(ping = Ping())).pong)
            val tab = core.request(Request(open_tab = OpenTab(window_id = opened.window_id))).tab_opened
            assertNotNull("a new tab's answer names the tab", tab)
            assertTrue(tab!!.tab_id != 0L && tab.tab_id != opened.tab_id)
            val history = core.request(Request(query_history = QueryHistory(window_id = opened.window_id))).history_page
            assertNotNull("a page of history is a page, also when nothing was visited", history)
            assertTrue(history!!.entries.isEmpty())
            val offered = core.request(Request(suggest_addresses = SuggestAddresses(window_id = opened.window_id, text = "example"))).address_suggestions
            assertNotNull("what is offered is an answer, also when nothing is", offered)
            assertTrue(offered!!.suggestions.isEmpty())
            assertNotNull(core.request(Request(close_tab = CloseTab(tab_id = tab.tab_id))).ack)
            assertNotNull(core.request(Request(forget_history = ForgetHistory(window_id = opened.window_id, everything = true))).ack)
        }

    /** History through the contract (008): a page that committed is a visit, found by a word. */
    @Test
    fun aVisitIsWrittenAndFoundThroughTheContract() =
        runBlocking {
            val host = RecordingHost()
            val core = start(host = host)
            val opened = openWindow(core)
            core.perform(Request(submit_address = SubmitAddress(tab_id = opened.tab_id, text = "example.com")))
            val id = host.requests[0].web_view_create!!.web_view_id
            core.report(EngineEvent(web_view_id = id, navigation_committed = NavigationCommitted(url = "https://example.com/")))
            core.report(EngineEvent(web_view_id = id, title_changed = TitleChanged(title = "Example Domain")))
            core.report(EngineEvent(web_view_id = id, navigation_finished = NavigationFinished()))

            val found = core.request(Request(query_history = QueryHistory(window_id = opened.window_id, text = "exam"))).history_page

            assertEquals(listOf("https://example.com/"), found?.entries?.map { it.url })
            assertEquals("Example Domain", found?.entries?.first()?.title)
            assertEquals("example.com", found?.entries?.first()?.display)
        }

    /**
     * The tabs come back (009), through Kotlin and the real core: a core is stopped, and another
     * started on the same folder is given the first one's tabs, in order, with the one that was
     * in front loaded and the other asleep.
     */
    @Test
    fun theTabsComeBackWhenTheCoreStartsAgain() =
        runBlocking {
            val folder = java.nio.file.Files.createTempDirectory("webtiger-test").toFile()
            try {
                val config = fakeConfig().copy(test_mode = TestMode.TEST_MODE_LIVE, data_dir = folder.path, fake = null)

                // The first run: a page in the first tab, and a second tab in front of it.
                val first = start(config)
                val opened = openWindow(first)
                first.perform(Request(submit_address = SubmitAddress(tab_id = opened.tab_id, text = "example.com")))
                first.request(Request(open_tab = OpenTab(window_id = opened.window_id, text = "example.org")))
                // The snapshots are published, and with them the session is written, before the
                // next request is answered.
                first.request(Request(ping = Ping()))
                first.stop()

                // The second run, on the same folder.
                val host = RecordingHost()
                val second = start(config, host)
                val events = eventsOf(second)
                val again = openWindow(second)
                waitUntil { events.any { it.snapshot?.window != null } }
                val window = events.firstNotNullOf { it.snapshot?.window }

                assertEquals("the tabs, in their order", listOf("example.com", "example.org"), window.tabs.map { it.host })
                assertEquals("the one that was in front is in front", window.tabs.last().tab_id, window.tab?.tab_id)
                assertEquals(again.tab_id, window.tab?.tab_id)
                assertEquals("https://example.org/", window.tab?.address?.url)
                // Only the tab in front was loaded: one web view, one load.
                assertEquals(listOf("https://example.org/"), host.requests.mapNotNull { it.web_view_load?.url })
                assertEquals("a web view and its load, and nothing for the tab behind", 2, host.requests.size)
            } finally {
                folder.deleteRecursively()
            }
        }

    @Test
    fun fakeModeOpensItsStartAddressInEveryNewWindow() =
        runBlocking {
            val host = RecordingHost()
            val core = start(fakeConfig(start = "http://127.0.0.1:8080/start"), host)
            openWindow(core)
            assertEquals(2, host.requests.size)
            assertNotNull(host.requests[0].web_view_create)
            assertEquals("http://127.0.0.1:8080/start", host.requests[1].web_view_load?.url)
        }

    @Test
    fun aRefusalIsThrownWithItsCode() {
        val core = start()
        val refusal =
            assertThrows(CoreError.Refused::class.java) {
                runBlocking { core.perform(Request(submit_address = SubmitAddress(tab_id = 9999, text = "example.com"))) }
            }
        assertEquals(CoreError.Refused(code = "tab.unknown", field = "tab_id"), refusal)
    }

    @Test
    fun aRefusedStartSaysWhy() {
        val live = fakeConfig().copy(test_mode = TestMode.TEST_MODE_LIVE) // and no data folder
        assertEquals(
            CoreError.StartRefused("start.data_dir"),
            assertThrows(CoreError.StartRefused::class.java) { NativeCore.start(live, RecordingHost(), hostThread) },
        )
        val none = fakeConfig().copy(engines = emptyList())
        assertEquals(
            CoreError.StartRefused("start.no_engine"),
            assertThrows(CoreError.StartRefused::class.java) { NativeCore.start(none, RecordingHost(), hostThread) },
        )
    }

    @Test
    fun liveModeKeepsItsStoreInTheDataFolder() {
        val folder = Files.createTempDirectory("webtiger-test-").toFile()
        try {
            val config = fakeConfig().copy(test_mode = TestMode.TEST_MODE_LIVE, data_dir = folder.path)
            start(config).stop()
            assertTrue(File(folder, "webtiger.sqlite").isFile)
        } finally {
            folder.deleteRecursively()
        }
    }

    @Test
    fun theNavigationPolicyAnswersSynchronously() {
        val core = start()

        fun verdict(url: String, mainFrame: Boolean = true) = core.navigationVerdict(url, webViewId = 1, mainFrame = mainFrame, userGesture = false)
        assertEquals(NavigationVerdict.ALLOW, verdict("https://example.com/"))
        assertEquals(NavigationVerdict.ALLOW, verdict("about:blank"))
        assertEquals(NavigationVerdict.CANCEL, verdict("javascript:alert(1)"))
        assertEquals(NavigationVerdict.CANCEL, verdict("file:///etc/passwd"))
        assertEquals(NavigationVerdict.CANCEL, verdict("data:text/html,x"))
        assertEquals(NavigationVerdict.ALLOW, verdict("data:text/html,x", mainFrame = false))
        assertEquals(NavigationVerdict.EXTERNAL, verdict("mailto:ada@example.com"))
        assertEquals(NavigationVerdict.CANCEL, verdict(""))
        // An address outside the Basic Multilingual Plane crosses JNI as surrogates and arrives whole.
        assertEquals(NavigationVerdict.ALLOW, verdict("https://example.com/🐯"))
        assertEquals(NavigationVerdict.EXTERNAL, verdict("mailto:🐯@example.com"))
    }

    @Test
    fun guardAllowsEveryRequestUntilItExists() {
        val core = start()
        assertEquals(
            RequestVerdict.ALLOW,
            core.requestVerdict("https://tracker.example.com/pixel.gif", "https://example.com/", webViewId = 1, resource = ResourceType.IMAGE),
        )
    }

    @Test
    fun aHostThatThrowsIsAnsweredWithACodeAndTheNextRequestIsStillAnswered() =
        runBlocking {
            var calls = 0
            val host =
                object : HostServices {
                    override suspend fun answer(request: HostRequest): HostResponse {
                        calls += 1
                        if (calls == 1) error("https://secret.example.com/ must not travel")
                        return HostResponse(pong = HostPong(payload = request.ping!!.payload))
                    }
                }
            val core = start(host = host)
            val via = Request(ping = Ping(payload = "x".encodeUtf8(), via_host = true))
            try {
                core.request(via)
                fail("expected the host's failure")
            } catch (refused: CoreError.Refused) {
                assertEquals("host.failed", refused.code)
            }
            assertEquals(true, core.request(via).pong?.via_host)
        }

    @Test
    fun stoppingFailsTheRequestsStillWaiting() =
        runBlocking {
            val asked = CompletableDeferred<Unit>()
            val host =
                object : HostServices {
                    override suspend fun answer(request: HostRequest): HostResponse {
                        asked.complete(Unit)
                        awaitCancellation()
                    }
                }
            val core = start(host = host)
            val waiting = CompletableDeferred<Throwable?>()
            launch(Dispatchers.Default) {
                waiting.complete(runCatching { core.request(Request(ping = Ping(via_host = true))) }.exceptionOrNull())
            }
            withTimeout(PATIENCE_MS) { asked.await() }
            core.stop()
            assertEquals(CoreError.Stopped(), withTimeout(PATIENCE_MS) { waiting.await() })
        }

    @Test
    fun aStoppedCoreRefusesEverything() {
        val core = start()
        core.stop()
        core.stop() // twice is harmless
        assertThrows(CoreError.Stopped::class.java) { runBlocking { core.perform(Request(ping = Ping())) } }
        assertEquals(NavigationVerdict.CANCEL, core.navigationVerdict("https://example.com/", webViewId = 1, mainFrame = true, userGesture = false))
        assertEquals(RequestVerdict.ALLOW, core.requestVerdict("https://example.com/", "https://example.com/", webViewId = 1, resource = ResourceType.DOCUMENT))
        core.report(EngineEvent(web_view_id = 1, navigation_finished = NavigationFinished())) // nobody to tell: harmless
    }

    @Test
    fun theEventsEndWhenTheCoreStops() =
        runBlocking {
            val core = start()
            core.stop()
            val seen = withTimeout(PATIENCE_MS) { ArrayList<Event>().also { all -> core.events.collect { all.add(it) } } }
            assertEquals(listOf(CoreStatus.State.STATE_READY), seen.map { it.status?.state })
        }

    @Test
    fun stoppingFromTheCoresOwnThreadStillStops() =
        runBlocking {
            val core = start()
            val stoppedOn = CompletableDeferred<Thread>()
            // Unconfined: the collector runs wherever the event was delivered, which for a
            // snapshot is the core's own thread, inside its listener.
            val collector =
                launch(Dispatchers.Unconfined) {
                    core.events.collect { event ->
                        if (event.snapshot != null && !stoppedOn.isCompleted) {
                            core.stop()
                            stoppedOn.complete(Thread.currentThread())
                        }
                    }
                }
            val here = Thread.currentThread()
            openWindow(core)
            val thread = withTimeout(PATIENCE_MS) { stoppedOn.await() }
            assertFalse("the snapshot was delivered on the core's thread", thread === here)
            withTimeout(PATIENCE_MS) { collector.join() } // the events ended: the core has stopped
            assertThrows(CoreError.Stopped::class.java) { runBlocking { core.perform(Request(ping = Ping())) } }
            assertEquals(NavigationVerdict.CANCEL, core.navigationVerdict("https://example.com/", webViewId = 1, mainFrame = true, userGesture = false))
        }
}

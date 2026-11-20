// The whole Android shell above the WebView, over the real Rust core: the window's model, the
// session, the engine adapter and the page tracker, with the core deciding and the shell
// drawing. The one thing that is not real is the WebView: its callbacks are made by the test,
// in the order a WebView makes them. (On a device, AppWindowTest does this with a real WebView:
// 003 §8.)
package com.tigercodes.webtiger

import com.tigercodes.webtiger.core.NativeCore
import com.tigercodes.webtiger.core.v1.AddressView
import com.tigercodes.webtiger.core.v1.CoreConfig
import com.tigercodes.webtiger.core.v1.EngineInfo
import com.tigercodes.webtiger.core.v1.EngineKind
import com.tigercodes.webtiger.core.v1.FakeOptions
import com.tigercodes.webtiger.core.v1.LoadState
import com.tigercodes.webtiger.core.v1.NavigationCommand
import com.tigercodes.webtiger.core.v1.Platform
import com.tigercodes.webtiger.core.v1.TestMode
import com.tigercodes.webtiger.ui.BrowserModel
import com.tigercodes.webtiger.ui.BrowserSession
import com.tigercodes.webtiger.ui.LaunchOptions
import com.tigercodes.webtiger.webview.DownloadPlace
import com.tigercodes.webtiger.webview.FakePage
import com.tigercodes.webtiger.webview.FakePages
import com.tigercodes.webtiger.webview.SystemWebViewEngine
import java.util.concurrent.Executors
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.asCoroutineDispatcher
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeout
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class ThroughTheCoreTest {
    /** Stands in for the main thread: host requests are answered and the window is drawn on it. */
    private val main = Executors.newSingleThreadExecutor().asCoroutineDispatcher()
    private val scope = CoroutineScope(main)
    private val pages = FakePages()
    private val engine = SystemWebViewEngine(pages)
    private var core: NativeCore? = null

    @After
    fun stop() {
        core?.stop()
        scope.cancel()
        main.close()
    }

    private fun config(launch: LaunchOptions): CoreConfig =
        BrowserSession.coreConfig(
            launch = launch,
            locale = "en-GB",
            appVersion = "0.0.0-test",
            engine = EngineInfo(kind = EngineKind.ENGINE_KIND_CHROMIUM, version = "test"),
            dataDir = "/nowhere",
        )

    private fun session(launch: LaunchOptions = LaunchOptions(fake = true)): BrowserSession {
        val started = NativeCore.start(config(launch), engine, main).also { core = it }
        return BrowserSession(started, engine, scope)
    }

    private suspend fun <T> onMain(block: suspend () -> T): T = withContext(main) { block() }

    private suspend fun await(what: String, done: () -> Boolean) {
        try {
            withTimeout(10_000) { while (!onMain { done() }) delay(5) }
        } catch (_: kotlinx.coroutines.TimeoutCancellationException) {
            throw AssertionError("never happened: $what")
        }
    }

    /** The callbacks a current WebView makes for a page that loads. */
    private suspend fun arrive(page: FakePage, url: String, title: String, canGoBack: Boolean = false) =
        onMain {
            page.tracker.progress(10)
            page.tracker.pageStarted(url)
            page.tracker.committed(url, canGoBack, canGoForward = false)
            page.tracker.title(title)
            page.tracker.progress(100)
            page.tracker.pageFinished(url)
        }

    @Test
    fun fakeModeIsWhatTheLaunchAskedFor() {
        val fake = config(LaunchOptions(fake = true, fakeStart = "http://127.0.0.1:8080/start"))
        assertEquals(TestMode.TEST_MODE_FAKE, fake.test_mode)
        assertEquals(FakeOptions(start_address = "http://127.0.0.1:8080/start"), fake.fake)
        assertEquals("fake mode names no folder: nothing on the device is read or written", "", fake.data_dir)
        assertEquals(Platform.PLATFORM_ANDROID, fake.platform)
        assertEquals(listOf(EngineKind.ENGINE_KIND_CHROMIUM), fake.engines.map { it.kind })

        val live = config(LaunchOptions())
        assertEquals(TestMode.TEST_MODE_LIVE, live.test_mode)
        assertNull(live.fake)
        assertEquals("/nowhere", live.data_dir)
    }

    @Test
    fun onlyTheAppStartedForRealMayHaveTheSystemAskThePerson() {
        assertEquals(true, BrowserSession.mayAskTheSystem(LaunchOptions()))
        assertEquals("fake mode brings no dialog of the system's up", false, BrowserSession.mayAskTheSystem(LaunchOptions(fake = true)))
        // And a session nobody gave the app's permissions has no way to ask.
        assertNull(session().systemPermissions)
    }

    @Test
    fun onlyTheAppStartedForRealSavesWhereThePersonsFilesAre() {
        assertEquals(DownloadPlace.DEVICE, BrowserSession.downloadPlace(LaunchOptions()))
        assertEquals("fake mode saves in the app's own folder", DownloadPlace.APP, BrowserSession.downloadPlace(LaunchOptions(fake = true)))
    }

    @Test
    fun aPageLoadsThroughTheCoreInTheSpacesLoginSet() =
        runBlocking {
            val model = BrowserModel(session())
            onMain { model.open() }
            await("the window's first snapshot") { model.tab.tab_id != 0L }
            assertFalse("a new tab has no page", onMain { model.hasPage })

            onMain {
                model.isEditingAddress = true
                model.addressText = "example.com"
                model.submit()
            }
            // The core decided what was typed, asked for a web view, then for the load.
            await("the load reaching the web view") { pages.made.size == 1 && pages.made[0].loaded.isNotEmpty() }
            val page = pages.made[0]
            assertEquals(listOf("https://example.com/"), page.loaded)
            assertTrue("a login set is a UUID: ${page.loginSet.length} characters", SystemWebViewEngine.isUuid(page.loginSet))
            await("the tab loading, showing where the user sent it") {
                model.isLoading && model.hasPage && model.addressText == "example.com" && model.tab.web_view_id == page.id
            }

            arrive(page, "https://example.com/", "Example Domain")
            await("the page shown as the core says it is") { model.tab.load_state == LoadState.LOAD_STATE_IDLE && model.tab.title == "Example Domain" }
            onMain {
                assertEquals("example.com", model.addressText)
                assertEquals("https://example.com/", model.address.url)
                assertEquals(AddressView.Security.SECURITY_SECURE, model.address.security)
                assertEquals(1.0, model.tab.progress, 0.0)
                assertFalse(model.tab.can_go_back)
            }
        }

    @Test
    fun aPageCannotPutAnAddressInTheFieldItHasNotBecome() =
        runBlocking {
            val model = BrowserModel(session())
            onMain {
                model.open()
                model.addressText = "example.com"
                model.submit()
            }
            await("the load") { pages.made.isNotEmpty() && pages.made[0].loaded.isNotEmpty() }
            val page = pages.made[0]
            arrive(page, "https://example.com/", "Example Domain")
            await("the page") { model.tab.title == "Example Domain" }

            // The page starts a load of its own: the policy allows it, and it is on its way.
            onMain {
                page.tracker.navigationAllowed("https://bank.example.com/login")
                page.tracker.progress(10)
            }
            await("the tab loading again") { model.isLoading }
            assertEquals("the field still shows the page on screen", "example.com", onMain { model.addressText })

            // Only when it commits is it the page's address.
            onMain {
                page.tracker.pageStarted("https://bank.example.com/login")
                page.tracker.committed("https://bank.example.com/login", canGoBack = true, canGoForward = false)
                page.tracker.pageFinished("https://bank.example.com/login")
            }
            await("the new page's address") { model.addressText == "bank.example.com/login" && !model.isLoading }
            assertTrue(onMain { model.tab.can_go_back })
        }

    @Test
    fun aFailedLoadIsShownByItsCodeAndTriedAgain() =
        runBlocking {
            val model = BrowserModel(session())
            onMain {
                model.open()
                model.addressText = "nowhere.example.com"
                model.submit()
            }
            await("the load") { pages.made.isNotEmpty() && pages.made[0].loaded.isNotEmpty() }
            val page = pages.made[0]
            val url = "https://nowhere.example.com/"
            // What a WebView reports for a host that cannot be found, its own error page included.
            onMain {
                page.tracker.progress(10)
                page.tracker.pageStarted(url)
                page.tracker.failed("net.dns", url)
                page.tracker.pageFinished(url)
                page.tracker.committed(url, canGoBack = false, canGoForward = false)
                page.tracker.title("Web page not available")
                page.tracker.progress(100)
            }
            await("the failure") { model.hasFailed }
            onMain {
                assertEquals("net.dns", model.tab.error_code)
                assertEquals("nowhere.example.com", model.addressText)
                assertEquals("the engine's own error page lends the tab no title", "", model.tab.title)
            }
            // Nothing the engine said afterwards turned the failure into a finished page.
            delay(50)
            assertTrue(onMain { model.hasFailed })

            onMain { model.perform(NavigationCommand.NAVIGATION_COMMAND_RELOAD) }
            await("the reload reaching the web view") { page.commands == listOf(NavigationCommand.NAVIGATION_COMMAND_RELOAD) }
        }

    @Test
    fun fakeModeOpensItsStartAddressAndClosingTheWindowDestroysItsWebView() =
        runBlocking {
            val model = BrowserModel(session(LaunchOptions(fake = true, fakeStart = "http://127.0.0.1:8080/start")))
            onMain { model.open() }
            await("the start address loading") { pages.made.size == 1 && pages.made[0].loaded == listOf("http://127.0.0.1:8080/start") }
            await("the window showing it") { model.hasPage && model.addressText == "http://127.0.0.1:8080/start" }
            assertEquals(1, onMain { engine.webViewCount })

            onMain { model.close() }
            await("the web view destroyed") { pages.made[0].closed && engine.webViewCount == 0 }
        }

    @Test
    fun withoutProfilesTheTabFailsRatherThanShareItsData() =
        runBlocking {
            pages.keepsLoginSetsApart = false
            val model = BrowserModel(session())
            onMain {
                model.open()
                model.addressText = "example.com"
                model.submit()
            }
            await("the refusal shown") { model.hasFailed }
            onMain {
                assertEquals("engine.no_profiles", model.tab.error_code)
                assertFalse("the tab has no web view after all", model.hasPage)
                assertTrue(pages.made.isEmpty())
            }
        }
}

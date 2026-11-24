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
import com.tigercodes.webtiger.core.v1.QuerySpaces
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.design.Tokens
import com.tigercodes.webtiger.core.v1.TestMode
import com.tigercodes.webtiger.ui.ArchiveModel
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

    /**
     * A Space's icon (050), through the real core: the window's model gives the Space the first
     * icon the core lists, and the window says it; one the core did not list is refused by its
     * code, and the Space keeps the one it has. The shell has no list of its own to check by.
     */
    @Test
    fun aSpaceIsGivenAnIconTheCoreListsAndNoOther() =
        runBlocking {
            val session = session()
            val model = BrowserModel(session)
            onMain { model.open() }
            await("the window's first snapshot") { model.tab.tab_id != 0L && model.spaceId != 0L }
            val listed = session.core.request(Request(query_spaces = QuerySpaces())).spaces_page!!
            assertEquals(24, listed.icons.size)
            assertEquals("each once", 24, listed.icons.toSet().size)
            assertEquals("a Space is made with none", "", listed.spaces.first().icon)
            assertEquals("", onMain { model.spaceIcon })

            onMain { model.editSpace(null, null, null, listed.icons.first()) }
            await("the window saying its Space's icon") { model.spaceIcon == listed.icons.first() }

            onMain { model.openSpaces() }
            await("the list") { model.spaces != null && model.spaceIcons == listed.icons }
            onMain { model.editSpace(null, null, null, "\uD83D\uDE00") }
            await("the refusal") { model.spaceRefusal == "space.icon_unknown" }
            assertEquals("the Space keeps the icon it has", listed.icons.first(), onMain { model.spaceIcon })
        }

    /**
     * The colours the real core lists (025) are the ones this shell has a paint for, by the same
     * names and in the same order. (Their words are the strings': `SpacesTest`.)
     */
    @Test
    fun theShellHasAPaintForEveryColourTheCoreLists() =
        runBlocking {
            val listed = session().core.request(Request(query_spaces = QuerySpaces())).spaces_page
            assertEquals(Tokens.SpaceColour.named.map { it.first }, listed?.colours)
            assertTrue(listed!!.colours.isNotEmpty())
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

    /**
     * Words another app handed over (028), through the real core: the window's model hands
     * them on, and the core opens a new tab in front and asks its web view for the chosen
     * engine's search for them. Words that look like an address are searched for all the same.
     * Nothing is fetched: the web view here is the test's.
     */
    @Test
    fun wordsFromAnotherAppAreSearchedForInANewTabAndNeverTakenForAnAddress() =
        runBlocking {
            val model = BrowserModel(session())
            onMain { model.open() }
            await("the window's first snapshot") { model.tab.tab_id != 0L }
            val first = onMain { model.tab.tab_id }

            onMain { model.searchFor("  tigers in\nthe wild ") }

            await("the search reaching a web view") { pages.made.size == 1 && pages.made[0].loaded.isNotEmpty() }
            assertEquals(listOf("https://duckduckgo.com/?q=tigers+in+the+wild"), pages.made[0].loaded)
            await("the new tab in front, saying where the search went") {
                model.tabs.size == 2 && model.tab.tab_id != first && model.address.url == "https://duckduckgo.com/?q=tigers+in+the+wild"
            }
            assertEquals("after the tab that was in front", first, onMain { model.tabs.first().tab_id })

            onMain { model.searchFor("example.com") }

            await("the second search reaching its web view") { pages.made.size == 2 && pages.made[1].loaded.isNotEmpty() }
            assertEquals("searched for, not gone to", listOf("https://duckduckgo.com/?q=example.com"), pages.made[1].loaded)
            assertEquals(3, onMain { model.tabs.size })
        }

    /**
     * A link pressed in another app (031), through the real core: the window's model hands the
     * address on, and the core opens a new tab in front and asks its web view for that
     * address, as it was given. What is no address of the web opens nothing: it is not mended,
     * and not searched for. Nothing is fetched: the web view here is the test's.
     */
    @Test
    fun anAddressFromAnotherAppOpensInANewTabAndWhatIsNoAddressOfTheWebOpensNothing() =
        runBlocking {
            val model = BrowserModel(session())
            onMain { model.open() }
            await("the window's first snapshot") { model.tab.tab_id != 0L }
            val first = onMain { model.tab.tab_id }

            onMain { model.openFromOutside("https://example.com/story?id=7") }

            await("the address reaching a web view") { pages.made.size == 1 && pages.made[0].loaded.isNotEmpty() }
            assertEquals(listOf("https://example.com/story?id=7"), pages.made[0].loaded)
            await("the new tab in front, saying where it was sent") {
                model.tabs.size == 2 && model.tab.tab_id != first && model.address.url == "https://example.com/story?id=7"
            }
            assertEquals("after the tab that was in front", first, onMain { model.tabs.first().tab_id })

            // What typed in the address field would be mended, or searched for, or run if
            // anything ran typed text: refused, each of them.
            for (other in listOf("example.org", "tigers in the wild", "javascript:alert(1)", "file:///sdcard/page.html", "intent://example.org/#Intent;scheme=https;end")) {
                onMain { model.openFromOutside(other) }
            }
            // A second address of the web is asked after them, and is answered after them.
            onMain { model.openFromOutside("http://example.org/") }

            await("the second address reaching its web view") { pages.made.size == 2 && pages.made[1].loaded.isNotEmpty() }
            assertEquals("as it was given: not made https", listOf("http://example.org/"), pages.made[1].loaded)
            assertEquals("the others opened nothing", 3, onMain { model.tabs.size })
            assertEquals(2, pages.made.size)
        }

    /**
     * Tidying (049), through the real core, with its clock at a day a read: the window's model
     * gives the Space twelve hours; at the core's next look (a window that opens is one) the
     * tab that was not in front is tidied, and its web view destroyed; the Archive's model
     * lists it by its name and its address as the address field shows one; brought back, it is
     * a tab again, in front, loading its address. The shell timed nothing.
     */
    @Test
    fun aTabOutOfSightIsTidiedByTheCoreAndBroughtBackFromItsArchive() =
        runBlocking {
            val session = session(LaunchOptions(fake = true, clockStep = 86_400))
            val model = BrowserModel(session)
            onMain { model.open() }
            await("the window's first snapshot") { model.tab.tab_id != 0L && model.spaceId != 0L }
            // The times there are, as the real core lists them, never first; and a Space is
            // made with none. (Their words are the strings': `SpaceTidyingTest`.)
            val listed = session.core.request(Request(query_spaces = QuerySpaces())).spaces_page!!
            assertEquals(listOf(0, 43_200, 86_400, 604_800, 2_592_000), listed.tidy_times)
            assertEquals(0, listed.spaces.first().tidy_after_seconds)

            // Two tabs with pages: the first, and one a link from another app opened, in front.
            onMain {
                model.isEditingAddress = true
                model.addressText = "example.com"
                model.submit()
            }
            await("the first load") { pages.made.size == 1 && pages.made[0].loaded.isNotEmpty() }
            arrive(pages.made[0], "https://example.com/", "Example Domain")
            await("the first page named") { model.tab.title == "Example Domain" }
            onMain { model.openFromOutside("https://example.org/story") }
            await("the second load") { pages.made.size == 2 && pages.made[1].loaded.isNotEmpty() }
            arrive(pages.made[1], "https://example.org/story", "A Story")
            await("two tabs, the second in front") { model.tabs.size == 2 && model.tab.title == "A Story" }

            // Twelve hours, for the Space the window shows: nothing goes when it is given.
            onMain { model.editSpace(null, null, 43_200) }
            withTimeout(10_000) {
                while (session.core.request(Request(query_spaces = QuerySpaces())).spaces_page?.spaces?.first()?.tidy_after_seconds != 43_200) delay(5)
            }
            assertEquals(2, onMain { model.tabs.size })

            // A look: the core looks when a window opens (and once a minute, which this test
            // does not wait for). By its clock days have gone.
            val other = BrowserModel(session)
            onMain { other.open() }
            await("the tab out of sight tidied") { model.tabs.size == 1 }
            assertEquals("the one in front stays", "A Story", onMain { model.tab.title })
            await("its web view destroyed") { pages.made[0].closed }
            assertFalse(pages.made[1].closed)

            val archive = ArchiveModel(session, onMain { model.windowId!! })
            onMain { archive.reload() }
            await("the Archive listed") { archive.loaded }
            onMain {
                assertEquals(listOf("Example Domain"), archive.entries.map { it.title })
                assertEquals(listOf("https://example.com/"), archive.entries.map { it.url })
                assertEquals("as the address field shows it", listOf("example.com"), archive.entries.map { it.display })
            }

            var back = 0
            onMain { archive.bringBack(archive.entries.first()) { back += 1 } }
            await("a tab again, the last of the row, in front") {
                back == 1 && model.tabs.size == 2 && model.tab.tab_id == model.tabs.last().tab_id && model.address.url == "https://example.com/"
            }
            await("its address asked of a new web view") { pages.made.size == 3 && pages.made[2].loaded == listOf("https://example.com/") }
            assertEquals("and the Archive has it no longer", 0, onMain { archive.entries.size })
            onMain { archive.reload() }
            await("asked again") { archive.loaded && archive.entries.isEmpty() }
        }

    /** A launch that says nothing of the clock leaves it as fake mode has it; one that does is handed on. */
    @Test
    fun fakeModesClockMovesAsTheLaunchAskedFor() {
        assertEquals(0, config(LaunchOptions(fake = true)).fake?.seconds_a_read)
        assertEquals(86_400, config(LaunchOptions(fake = true, clockStep = 86_400)).fake?.seconds_a_read)
        assertNull("and outside fake mode there is no such thing", config(LaunchOptions(clockStep = 86_400)).fake)
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

// A window's model against a scripted core: what it sends, and what it shows.
package com.tigercodes.webtiger.ui

import com.tigercodes.webtiger.core.CoreError
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.Ack
import com.tigercodes.webtiger.core.v1.AddressView
import com.tigercodes.webtiger.core.v1.FindView
import com.tigercodes.webtiger.core.v1.LoadState
import com.tigercodes.webtiger.core.v1.NavigationCommand
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.Response
import com.tigercodes.webtiger.core.v1.Subscribed
import com.tigercodes.webtiger.core.v1.TabSummary
import com.tigercodes.webtiger.core.v1.TabView
import com.tigercodes.webtiger.core.v1.WindowOpened
import com.tigercodes.webtiger.core.v1.WindowSnapshot
import com.tigercodes.webtiger.webview.FakePages
import com.tigercodes.webtiger.webview.SystemWebViewEngine
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test

/** A snapshot of window 1, tab 2, as the core would publish it. */
internal fun snapshot(
    url: String = "",
    display: String = "",
    title: String = "",
    state: LoadState = LoadState.LOAD_STATE_IDLE,
    webView: Long = 0,
    change: (TabView) -> TabView = { it },
): WindowSnapshot =
    WindowSnapshot(
        window_id = 1,
        tab = change(TabView(tab_id = 2, web_view_id = webView, title = title, load_state = state, address = AddressView(url = url, display = display))),
    )

/** A core that opens window 1 with tab 2 under subscription 3, and says Ack to the rest. */
internal fun windowScript(request: Request): Response =
    when {
        request.open_window != null -> Response(window_opened = WindowOpened(window_id = 1, tab_id = 2))
        request.subscribe != null -> Response(subscribed = Subscribed(subscription_id = 3))
        else -> Response(ack = Ack())
    }

/**
 * A session over `core` that ends with the test, and whose coroutines run at once, where they
 * are started.
 */
@OptIn(ExperimentalCoroutinesApi::class)
internal fun TestScope.unconfinedSession(core: ScriptedCore): BrowserSession =
    BrowserSession(core, SystemWebViewEngine(FakePages()), CoroutineScope(backgroundScope.coroutineContext + UnconfinedTestDispatcher(testScheduler)))

class BrowserModelTest {
    /**
     * A session whose coroutines run at once, where they are started: what the model sends has
     * been sent, and what the core emits has been drawn, by the next line of the test.
     */
    private fun TestScope.session(core: ScriptedCore): BrowserSession = unconfinedSession(core)

    @Test
    fun openingAsksTheCoreForAWindowAndSubscribes() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = BrowserModel(session(core))
            model.open()
            model.open() // once only
            assertEquals(3, core.requests.size)
            assertNotNull(core.requests[0].open_window)
            assertEquals(1L, core.requests[1].subscribe?.window?.window_id)
            // And, the window open, where searches go (020).
            assertNotNull(core.requests[2].query_search)
        }

    @Test
    fun submittingSendsWhatWasTypedAsTyped() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = BrowserModel(session(core))
            model.open()
            model.isEditingAddress = true
            model.addressText = "  rust borrow checker "
            model.submit()
            val submit = core.requests.last().submit_address
            assertEquals(2L, submit?.tab_id)
            assertEquals("the core decides what typed text is; the shell does not tidy it", "  rust borrow checker ", submit?.text)
            assertFalse(model.isEditingAddress)
        }

    @Test
    fun theFieldFollowsTheCoreUnlessTheUserIsTyping() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = BrowserModel(session(core))
            model.open()

            core.emit(snapshot(url = "https://example.com/", display = "example.com", title = "Example"), subscription = 3)
            assertEquals("Example", model.tab.title)
            assertEquals("example.com", model.addressText)

            model.isEditingAddress = true
            assertEquals("editing starts from the whole address", "https://example.com/", model.addressText)
            model.addressText = "exam"
            core.emit(snapshot(url = "https://example.com/inbox", display = "example.com/inbox", title = "Inbox"), subscription = 3)
            assertEquals("Inbox", model.tab.title)
            assertEquals("typing is never overwritten", "exam", model.addressText)

            model.isEditingAddress = false
            assertEquals("leaving the field shows the page again", "example.com/inbox", model.addressText)
        }

    @Test
    fun aFieldWithTheKeyboardFollowsThePageUntilSomethingIsTyped() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = BrowserModel(session(core))
            model.open()

            // The field has the keyboard before any page is there, and nothing is typed: as when
            // it takes the focus by itself at launch, on a device with a keyboard.
            model.isEditingAddress = true
            assertEquals("", model.addressText)
            assertFalse(model.hasTypedAddress)
            core.emit(snapshot(url = "https://example.com/", display = "example.com", title = "Example"), subscription = 3)
            assertEquals("an untouched field shows the page that arrived, whole", "https://example.com/", model.addressText)

            core.emit(snapshot(url = "https://example.com/inbox", display = "example.com/inbox", title = "Inbox"), subscription = 3)
            assertEquals("and goes on following it", "https://example.com/inbox", model.addressText)

            model.addressText = "exam"
            assertTrue(model.hasTypedAddress)
            core.emit(snapshot(url = "https://example.com/sent", display = "example.com/sent", title = "Sent"), subscription = 3)
            assertEquals("until something is typed", "exam", model.addressText)
        }

    // Tabs (005)

    /** A snapshot whose tab in front is `front`, with `others` beside it in the strip. */
    private fun tabs(front: Long, url: String, display: String, others: List<Long> = emptyList()): WindowSnapshot =
        snapshot(url = url, display = display) { it.copy(tab_id = front) }
            .copy(tabs = (listOf(front) + others).map { TabSummary(tab_id = it) })

    @Test
    fun theWindowsControlsActOnTheTabInFront() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = BrowserModel(session(core))
            model.open()
            // Before any snapshot, the tab the window opened with.
            model.perform(NavigationCommand.NAVIGATION_COMMAND_STOP)
            assertEquals(2L, core.requests.last().tab_command?.tab_id)

            // Another tab is in front now: the controls are its.
            core.emit(tabs(front = 9, url = "https://example.org/", display = "example.org", others = listOf(2)), subscription = 3)
            assertEquals("the strip lists the window's tabs in their order", listOf(9L, 2L), model.tabs.map { it.tab_id })
            model.perform(NavigationCommand.NAVIGATION_COMMAND_RELOAD)
            assertEquals(9L, core.requests.last().tab_command?.tab_id)
            model.isEditingAddress = true
            model.addressText = "example.com"
            model.submit()
            assertEquals(9L, core.requests.last().submit_address?.tab_id)
        }

    @Test
    fun openingClosingAndActivatingTabsAskTheCore() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = BrowserModel(session(core))
            model.open()
            core.emit(tabs(front = 2, url = "", display = "", others = listOf(5)), subscription = 3)

            model.openTab()
            val open = core.requests.last().open_tab
            assertEquals(1L, open?.window_id)
            assertEquals("a new tab is empty", "", open?.text)
            assertEquals("and in front", false, open?.background)
            model.activate(5)
            assertEquals(5L, core.requests.last().activate_tab?.tab_id)
            model.closeTab(5)
            assertEquals(5L, core.requests.last().close_tab?.tab_id)

            // The tab in front is in front already: nothing is asked.
            val asked = core.requests.size
            model.activate(2)
            assertEquals(asked, core.requests.size)
        }

    @Test
    fun whatWasTypedIsDroppedWhenAnotherTabComesInFront() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = BrowserModel(session(core))
            model.open()
            core.emit(tabs(front = 2, url = "https://example.com/", display = "example.com"), subscription = 3)
            model.isEditingAddress = true
            model.addressText = "half an addr"

            // The same tab, changed: what is being typed stays.
            core.emit(tabs(front = 2, url = "https://example.com/next", display = "example.com/next"), subscription = 3)
            assertEquals("half an addr", model.addressText)

            // Another tab in front: it was typed for the tab that was there.
            core.emit(tabs(front = 9, url = "https://example.org/", display = "example.org", others = listOf(2)), subscription = 3)
            assertEquals("the field shows the tab that is in front", "https://example.org/", model.addressText)
            assertFalse(model.hasTypedAddress)
        }

    // Find on the page (010)

    /** A window whose tab in front has a page, in which things can be found. */
    private suspend fun TestScope.finding(core: ScriptedCore): BrowserModel {
        val model = BrowserModel(session(core))
        model.open()
        core.emit(snapshot(url = "https://example.com/", display = "example.com", webView = 4), subscription = 3)
        return model
    }

    private fun finds(core: ScriptedCore): List<String> =
        core.requests.mapNotNull { it.find_in_page }.map { find -> "${find.tab_id} ${find.text.ifEmpty { "(stop)" }}${if (find.backwards) " back" else ""}" }

    @Test
    fun whatIsTypedInTheFindBarIsLookedForAtOnce() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = finding(core)
            assertFalse(model.isFinding)
            // Nothing is looked for while the bar is shut.
            model.find("stray")
            model.find("")

            model.openFind()
            model.find("n")
            model.find("ne")
            model.findNext()
            model.findPrevious()

            assertTrue(model.isFinding)
            assertEquals("each letter, then the next match, then the one before", listOf("2 n", "2 ne", "2 ne", "2 ne back"), finds(core))
        }

    @Test
    fun closingTheFindBarStopsTheFind() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = finding(core)
            model.openFind()
            model.find("needle")

            model.closeFind()

            assertFalse(model.isFinding)
            assertEquals("the bar opens empty the next time", "", model.findText)
            assertEquals(listOf("2 needle", "2 (stop)"), finds(core))
            // Closed already: nothing more is sent.
            model.closeFind()
            assertEquals(2, finds(core).size)
        }

    @Test
    fun emptyingTheFieldStopsTheFindAndTheBarStaysOpen() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = finding(core)
            model.openFind()
            model.find("needle")

            model.find("")

            assertTrue(model.isFinding)
            assertEquals(listOf("2 needle", "2 (stop)"), finds(core))
            // There is no find left to stop when the bar is closed.
            model.closeFind()
            assertEquals(2, finds(core).size)
        }

    @Test
    fun anotherTabInFrontStopsTheFindInTheTabThatHadIt() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = finding(core)
            model.openFind()
            model.find("needle")

            core.emit(tabs(front = 9, url = "https://example.org/", display = "example.org", others = listOf(2)), subscription = 3)

            assertFalse("the bar closes", model.isFinding)
            assertEquals("and the find stops in the tab it was in, not the one in front", listOf("2 needle", "2 (stop)"), finds(core))
        }

    @Test
    fun thereIsNothingToFindInATabWithNoPage() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = BrowserModel(session(core))
            model.open()

            model.openFind()
            model.find("needle")

            assertFalse(model.isFinding)
            assertEquals(emptyList<String>(), finds(core))
        }

    @Test
    fun theFindBarSaysWhatTheCoreSaysWasFound() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = finding(core)
            model.openFind()
            model.find("needle")
            assertEquals("nothing is known yet: that is not \"not found\"", BrowserModel.FindOutcome.Nothing, model.findOutcome)

            fun publish(find: FindView) =
                core.emit(snapshot(url = "https://example.com/", display = "example.com", webView = 4) { it.copy(find = find) }, subscription = 3)

            publish(FindView(text = "needle", answered = true, found = true, counted = true, matches = 12, current = 3))
            assertEquals(BrowserModel.FindOutcome.Match(3, of = 12), model.findOutcome)
            // An engine that does not count: found, and nothing more to say.
            publish(FindView(text = "needle", answered = true, found = true))
            assertEquals(BrowserModel.FindOutcome.Nothing, model.findOutcome)
            publish(FindView(text = "needle", answered = true))
            assertEquals(BrowserModel.FindOutcome.NotFound, model.findOutcome)
            // Another letter typed: what the core said was of the text before.
            model.find("needles")
            assertEquals(BrowserModel.FindOutcome.Nothing, model.findOutcome)
        }

    @Test
    fun editingAnEmptyTabStartsFromNothing() =
        runTest {
            val model = BrowserModel(session(ScriptedCore(::windowScript)))
            model.open()
            model.isEditingAddress = true
            assertEquals("", model.addressText)
        }

    @Test
    fun snapshotsForAnotherSubscriptionAreIgnored() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = BrowserModel(session(core))
            model.open()
            core.emit(snapshot(url = "https://other.example.org/", display = "other.example.org", title = "Other"), subscription = 99)
            assertEquals("", model.tab.title)
            core.emit(snapshot(url = "https://example.com/", display = "example.com", title = "Mine"), subscription = 3)
            assertEquals("Mine", model.tab.title)
        }

    @Test
    fun theFirstSnapshotIsDrawnEvenWhenItComesBeforeItsSubscriptionIsKnown() =
        runTest {
            // The core publishes a subscription's first snapshot right behind its answer. Here it
            // has been taken off the events before the model heard the subscription's id.
            lateinit var core: ScriptedCore
            core =
                ScriptedCore { request ->
                    if (request.subscribe != null) {
                        core.emit(snapshot(url = "https://example.com/", display = "example.com", title = "First"), subscription = 3)
                    }
                    windowScript(request)
                }
            val model = BrowserModel(session(core))
            model.open()
            assertEquals("First", model.tab.title)
            assertEquals("example.com", model.addressText)
        }

    @Test
    fun commandsAndClosingReachTheCore() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = BrowserModel(session(core))
            model.perform(NavigationCommand.NAVIGATION_COMMAND_RELOAD) // no window yet: nothing to ask
            model.submit()
            assertTrue(core.requests.isEmpty())

            model.open()
            model.perform(NavigationCommand.NAVIGATION_COMMAND_RELOAD)
            val command = core.requests.last().tab_command
            assertEquals(2L, command?.tab_id)
            assertEquals(NavigationCommand.NAVIGATION_COMMAND_RELOAD, command?.command)

            model.close()
            assertEquals(1L, core.requests.last().close_window?.window_id)
            model.close() // once only
            // The window, its subscription, where searches go, the reload, and the closing.
            assertEquals(5, core.requests.size)
            // A closed window draws nothing more.
            core.emit(snapshot(title = "Late"), subscription = 3)
            assertEquals("", model.tab.title)
        }

    @Test
    fun stateFlagsFollowTheSnapshot() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = BrowserModel(session(core))
            model.open()
            assertFalse(model.hasPage)
            assertFalse(model.isLoading)

            core.emit(snapshot(url = "https://example.com/", display = "example.com", state = LoadState.LOAD_STATE_LOADING, webView = 4), subscription = 3)
            assertTrue(model.isLoading)
            assertTrue(model.hasPage)
            assertFalse(model.hasFailed)

            core.emit(
                snapshot(url = "https://example.com/", display = "example.com", state = LoadState.LOAD_STATE_FAILED, webView = 4) { it.copy(error_code = "net.dns") },
                subscription = 3,
            )
            assertTrue(model.hasFailed)
            assertFalse(model.isLoading)
            assertEquals("net.dns", model.tab.error_code)
        }

    @Test
    fun aRefusalIsNotACrash() =
        runTest {
            val core =
                ScriptedCore { request ->
                    if (request.submit_address != null || request.tab_command != null) throw CoreError.Refused(code = "tab.no_page", field = "tab_id")
                    windowScript(request)
                }
            val model = BrowserModel(session(core))
            model.open()
            model.addressText = "example.com"
            model.submit()
            model.perform(NavigationCommand.NAVIGATION_COMMAND_BACK)
            // The window, its subscription, where searches go, and the two that were refused.
            assertEquals(5, core.requests.size)
        }

    @Test
    fun aCoreThatWillNotOpenAWindowLeavesAnEmptyTab() =
        runTest {
            val core = ScriptedCore { throw CoreError.Refused(code = "window.limit", field = "") }
            val model = BrowserModel(session(core))
            model.open()
            assertFalse(model.hasPage)
            model.submit() // nothing to send it to
            assertEquals(1, core.requests.size)
        }
}

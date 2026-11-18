// A window's model against a scripted core: what it sends, and what it shows.
package com.tigercodes.webtiger.ui

import com.tigercodes.webtiger.core.CoreError
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.Ack
import com.tigercodes.webtiger.core.v1.AddressView
import com.tigercodes.webtiger.core.v1.LoadState
import com.tigercodes.webtiger.core.v1.NavigationCommand
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.Response
import com.tigercodes.webtiger.core.v1.Subscribed
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
            assertEquals(2, core.requests.size)
            assertNotNull(core.requests[0].open_window)
            assertEquals(1L, core.requests[1].subscribe?.window?.window_id)
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
            assertEquals(4, core.requests.size)
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
            assertEquals(4, core.requests.size)
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

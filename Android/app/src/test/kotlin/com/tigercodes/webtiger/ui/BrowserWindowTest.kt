// The window itself, drawn by Compose on Robolectric's Android, over a scripted core: what it
// shows for each state the core publishes, and what the user's gestures send. No WebView is in
// these tests, and no device: the page's place is drawn, the page is not.
package com.tigercodes.webtiger.ui

import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.assertIsNotSelected
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.onAllNodesWithContentDescription
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.assertIsNotFocused
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performImeAction
import androidx.compose.ui.test.performTextReplacement
import androidx.compose.ui.text.TextRange
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.AddressView
import com.tigercodes.webtiger.core.v1.LoadState
import com.tigercodes.webtiger.core.v1.FindView
import com.tigercodes.webtiger.core.v1.NavigationCommand
import com.tigercodes.webtiger.core.v1.TabSummary
import com.tigercodes.webtiger.design.WebTigerTheme
import com.tigercodes.webtiger.webview.FakePages
import com.tigercodes.webtiger.webview.SystemWebViewEngine
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancel
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class BrowserWindowTest {
    @get:Rule val compose = createComposeRule()

    private val core = ScriptedCore(::windowScript)
    private val scope = CoroutineScope(Dispatchers.Unconfined)
    private val session = BrowserSession(core, SystemWebViewEngine(FakePages()), scope)
    private val model = BrowserModel(session)

    @After
    fun end() {
        scope.cancel()
    }

    private fun show() {
        compose.setContent { WebTigerTheme { BrowserWindow(session, model) } }
        compose.waitForIdle()
    }

    /** The core publishes; the window has drawn it when this returns. */
    private fun publish(
        url: String = "",
        display: String = "",
        state: LoadState = LoadState.LOAD_STATE_IDLE,
        webView: Long = 0,
        security: AddressView.Security = AddressView.Security.SECURITY_UNSPECIFIED,
        error: String = "",
        canGoBack: Boolean = false,
        canGoForward: Boolean = false,
        progress: Double = 0.0,
    ) {
        compose.runOnIdle {
            core.emit(
                snapshot(url = url, display = display, state = state, webView = webView) {
                    it.copy(
                        address = it.address?.copy(security = security),
                        error_code = error,
                        can_go_back = canGoBack,
                        can_go_forward = canGoForward,
                        progress = progress,
                    )
                },
                subscription = 3,
            )
        }
        compose.waitForIdle()
    }

    @Test
    fun aNewWindowOpensInTheCoreAndShowsAnEmptyTab() {
        show()
        assertTrue(core.requests[0].open_window != null)
        assertEquals(1L, core.requests[1].subscribe?.window?.window_id)
        compose.onNodeWithTag("empty-tab").assertIsDisplayed()
        compose.onNodeWithText("Where to?").assertIsDisplayed()
        compose.onNodeWithText("Search or enter address").assertIsDisplayed()
        // Nowhere to go back or forward to, and nothing to reload.
        compose.onNodeWithTag("back").assertIsNotEnabled()
        compose.onNodeWithTag("forward").assertIsNotEnabled()
        compose.onNodeWithTag("reload").assertIsNotEnabled()
        compose.onNodeWithContentDescription("Back").assertExists()
        compose.onNodeWithContentDescription("Address and search").assertExists()
    }

    @Test
    fun typingAnAddressSendsItAsTyped() {
        show()
        compose.onNodeWithTag("address-field").performClick()
        compose.onNodeWithTag("address-field").performTextReplacement("example.com/a b")
        compose.onNodeWithTag("address-field").performImeAction()
        compose.waitForIdle()
        val submit = core.requests.last().submit_address
        assertEquals(2L, submit?.tab_id)
        assertEquals("example.com/a b", submit?.text)
        // The keyboard has left the field: what it shows is the core's again.
        compose.onNodeWithTag("address-field").assertIsNotFocused()
        assertTrue(!model.isEditingAddress)
    }

    @Test
    fun aPageShowsWhatTheCoreSaysItIs() {
        show()
        publish(url = "https://example.com/", display = "example.com", webView = 4, security = AddressView.Security.SECURITY_SECURE, canGoBack = true)
        compose.onNodeWithTag("address-field").assertTextEquals("example.com")
        compose.onNodeWithTag("security-secure").assertExists()
        compose.onNodeWithContentDescription("Secure connection").assertExists()
        compose.onNodeWithTag("page").assertExists()
        compose.onNodeWithTag("empty-tab").assertDoesNotExist()
        compose.onNodeWithTag("back").assertIsEnabled()
        compose.onNodeWithTag("forward").assertIsNotEnabled()
        compose.onNodeWithTag("reload").assertIsEnabled()

        publish(url = "http://example.com/", display = "http://example.com", webView = 4, security = AddressView.Security.SECURITY_INSECURE)
        compose.onNodeWithTag("security-insecure").assertExists()
        compose.onNodeWithContentDescription("Not secure").assertExists()
        compose.onNodeWithTag("security-secure").assertDoesNotExist()
    }

    @Test
    fun theToolbarSendsItsCommands() {
        show()
        publish(url = "https://example.com/", display = "example.com", webView = 4, canGoBack = true, canGoForward = true)
        compose.onNodeWithTag("back").performClick()
        compose.onNodeWithTag("forward").performClick()
        compose.onNodeWithTag("reload").performClick()
        compose.waitForIdle()
        assertEquals(
            listOf(NavigationCommand.NAVIGATION_COMMAND_BACK, NavigationCommand.NAVIGATION_COMMAND_FORWARD, NavigationCommand.NAVIGATION_COMMAND_RELOAD),
            core.requests.mapNotNull { it.tab_command?.command },
        )
    }

    @Test
    fun whileLoadingReloadIsStop() {
        show()
        publish(url = "https://example.com/", display = "example.com", state = LoadState.LOAD_STATE_LOADING, webView = 4, progress = 0.4)
        compose.onNodeWithTag("reload").assertDoesNotExist()
        compose.onNodeWithTag("stop").assertIsEnabled().performClick()
        compose.waitForIdle()
        assertEquals(NavigationCommand.NAVIGATION_COMMAND_STOP, core.requests.last().tab_command?.command)
        compose.onNodeWithContentDescription("Stop").assertExists()
    }

    @Test
    fun aFailureIsShownInWordsAndCanBeTriedAgain() {
        show()
        publish(url = "https://nowhere.example.com/", display = "nowhere.example.com", state = LoadState.LOAD_STATE_FAILED, webView = 4, error = "net.dns")
        compose.onNodeWithTag("failed-page").assertIsDisplayed()
        compose.onNodeWithText("This page didn’t load").assertIsDisplayed()
        compose.onNodeWithTag("failed-words").assertTextEquals("The site’s address couldn’t be found.")
        // The failure covers the web view: whatever the engine drew for it is not shown.
        compose.onNodeWithTag("page").assertDoesNotExist()
        compose.onNodeWithText("net.dns").assertDoesNotExist()

        compose.onNodeWithTag("retry").performClick()
        compose.waitForIdle()
        assertEquals(NavigationCommand.NAVIGATION_COMMAND_RELOAD, core.requests.last().tab_command?.command)

        publish(url = "https://nowhere.example.com/", display = "nowhere.example.com", state = LoadState.LOAD_STATE_FAILED, error = "net.something_new")
        compose.onNodeWithTag("failed-words").assertTextEquals("Something went wrong while loading the page.")
    }

    @Test
    fun theFieldHoldsTheWholeAddressSelectedWhileItHasTheKeyboard() {
        show()
        publish(url = "https://example.com/inbox", display = "example.com/inbox", webView = 4)
        compose.onNodeWithTag("address-field").performClick()
        compose.waitForIdle()
        assertTrue(model.isEditingAddress)
        compose.onNodeWithTag("address-field").assertTextEquals("https://example.com/inbox")
        val selection = compose.onNodeWithTag("address-field").fetchSemanticsNode().config.getOrNull(SemanticsProperties.TextSelectionRange)
        assertEquals("typing replaces the address", TextRange(0, "https://example.com/inbox".length), selection)

        // What the core publishes meanwhile does not overwrite what is being typed.
        compose.onNodeWithTag("address-field").performTextReplacement("exam")
        publish(url = "https://example.com/sent", display = "example.com/sent", webView = 4)
        compose.onNodeWithTag("address-field").assertTextEquals("exam")
    }

    @Test
    fun atRestTheFieldShowsTheStartOfTheAddress() {
        show()
        publish(url = "https://example.com/a/path/longer/than/the/field/is/wide", display = "example.com/a/path/longer/than/the/field/is/wide", webView = 4)
        // A field scrolls to its caret: at the start, the site is what stays in view.
        val selection = compose.onNodeWithTag("address-field").fetchSemanticsNode().config.getOrNull(SemanticsProperties.TextSelectionRange)
        assertEquals("an address too long for the field loses its tail, never its site", TextRange.Zero, selection)
    }

    @Test
    fun theStripNamesTheWindowsTabsAndSendsWhatIsDoneToThem() {
        show()
        compose.runOnIdle {
            core.emit(
                snapshot(url = "https://example.com/", display = "example.com", title = "Example Domain", webView = 4).copy(
                    tabs =
                        listOf(
                            TabSummary(tab_id = 2, title = "Example Domain", host = "example.com"),
                            TabSummary(tab_id = 5, host = "example.org", loading = true),
                            TabSummary(tab_id = 6),
                        ),
                ),
                subscription = 3,
            )
        }
        compose.waitForIdle()
        // Each tab by its title, or its site, or the words for a tab with no page.
        compose.onAllNodesWithTag("tab").assertCountEquals(3)
        compose.onAllNodesWithTag("tab")[0].assertTextEquals("Example Domain").assertIsSelected()
        compose.onAllNodesWithTag("tab")[1].assertTextEquals("example.org").assertIsNotSelected()
        compose.onAllNodesWithTag("tab")[2].assertTextEquals("New Tab")
        compose.onNodeWithContentDescription("Tabs").assertExists()

        compose.onAllNodesWithTag("tab")[1].performClick()
        compose.waitForIdle()
        assertEquals(5L, core.requests.last().activate_tab?.tab_id)
        // With three tabs on a screen this narrow, and three buttons beside them since zoom's
        // (012), the third tab's own button is past the strip's end, and so is the button
        // after the tabs: the strip is scrolled to each.
        compose.onAllNodesWithTag("close-tab")[2].performScrollTo().performClick()
        compose.waitForIdle()
        assertEquals(6L, core.requests.last().close_tab?.tab_id)
        compose.onNodeWithTag("new-tab").performScrollTo().performClick()
        compose.waitForIdle()
        assertEquals(1L, core.requests.last().open_tab?.window_id)
        compose.onAllNodesWithContentDescription("Close Tab").assertCountEquals(3)
        compose.onNodeWithContentDescription("New Tab").assertExists()
    }

    // Find on the page (010)

    private fun aPage(find: FindView? = null) {
        compose.runOnIdle {
            core.emit(snapshot(url = "https://example.com/", display = "example.com", webView = 4) { it.copy(find = find) }, subscription = 3)
        }
        compose.waitForIdle()
    }

    @Test
    fun theFindBarIsShutUntilAskedForAndThereIsNothingToFindWithNoPage() {
        show()
        compose.onNodeWithTag("find-field").assertDoesNotExist()
        compose.onNodeWithTag("find").assertIsNotEnabled()

        aPage()
        compose.onNodeWithTag("find").assertIsEnabled().performClick()
        compose.waitForIdle()

        compose.onNodeWithTag("find-field").assertExists()
        compose.onNodeWithContentDescription("Find on page").assertExists()
        compose.onNodeWithTag("find-outcome").assertDoesNotExist()
        compose.onNodeWithTag("find-next").assertIsNotEnabled()
    }

    @Test
    fun theFindBarSendsWhatIsTypedAndWhatIsPressed() {
        show()
        aPage()
        compose.onNodeWithTag("find").performClick()
        compose.waitForIdle()

        compose.onNodeWithTag("find-field").performTextInput("needle")
        compose.waitForIdle()
        compose.onNodeWithTag("find-next").assertIsEnabled().performClick()
        compose.onNodeWithTag("find-previous").performClick()
        compose.onNodeWithTag("find-field").performImeAction()
        compose.waitForIdle()

        val finds = core.requests.mapNotNull { it.find_in_page }
        assertEquals(listOf("needle", "needle", "needle", "needle"), finds.map { it.text })
        assertEquals("typed, next, previous, Return", listOf(false, false, true, false), finds.map { it.backwards })
        assertEquals(setOf(2L), finds.map { it.tab_id }.toSet())

        compose.onNodeWithTag("find-done").performClick()
        compose.waitForIdle()
        compose.onNodeWithTag("find-field").assertDoesNotExist()
        assertEquals("closing stops the find", "", core.requests.last().find_in_page?.text)
    }

    @Test
    fun theFindBarSaysWhatWasFound() {
        show()
        aPage()
        compose.onNodeWithTag("find").performClick()
        compose.onNodeWithTag("find-field").performTextInput("needle")
        compose.waitForIdle()
        compose.onNodeWithTag("find-outcome").assertDoesNotExist()

        aPage(FindView(text = "needle", answered = true, found = true, counted = true, matches = 12, current = 3))
        compose.onNodeWithTag("find-outcome").assertTextEquals("3 of 12")

        aPage(FindView(text = "needle", answered = true))
        compose.onNodeWithTag("find-outcome").assertTextEquals("Not found")
    }

    @Test
    fun aStartThatFailedSaysItsCode() {
        compose.setContent { WebTigerTheme { StartFailed("store.open") } }
        compose.onNodeWithTag("start-failed").assertIsDisplayed()
        compose.onNodeWithText("WebTiger couldn’t start").assertIsDisplayed()
        compose.onNodeWithText("Error code: store.open").assertIsDisplayed()
    }
}

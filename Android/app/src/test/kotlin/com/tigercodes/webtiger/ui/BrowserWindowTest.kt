// The window itself, drawn by Compose on Robolectric's Android, over a scripted core: what it
// shows for each state the core publishes, and what the user's gestures send. No WebView is in
// these tests, and no device: the page's place is drawn, the page is not.
package com.tigercodes.webtiger.ui

import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.assertIsNotFocused
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performImeAction
import androidx.compose.ui.test.performTextReplacement
import androidx.compose.ui.text.TextRange
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.AddressView
import com.tigercodes.webtiger.core.v1.LoadState
import com.tigercodes.webtiger.core.v1.NavigationCommand
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
    fun aStartThatFailedSaysItsCode() {
        compose.setContent { WebTigerTheme { StartFailed("store.open") } }
        compose.onNodeWithTag("start-failed").assertIsDisplayed()
        compose.onNodeWithText("WebTiger couldn’t start").assertIsDisplayed()
        compose.onNodeWithText("Error code: store.open").assertIsDisplayed()
    }
}

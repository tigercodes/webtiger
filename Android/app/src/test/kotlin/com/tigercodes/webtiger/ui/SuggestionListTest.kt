// The list of places offered under the address field (011), drawn by Compose on Robolectric's
// Android over a scripted core: what it shows, and what choosing sends.
package com.tigercodes.webtiger.ui

import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertTextContains
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextReplacement
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.AddressView
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
class SuggestionListTest {
    @get:Rule val compose = createComposeRule()

    private val core = ScriptedCore(::offeringScript)
    private val scope = CoroutineScope(Dispatchers.Unconfined)
    private val session = BrowserSession(core, SystemWebViewEngine(FakePages()), scope)
    private val model = BrowserModel(session)

    @After
    fun end() {
        scope.cancel()
    }

    /** The window over a secure page. */
    private fun showAPage() {
        compose.setContent { WebTigerTheme { BrowserWindow(session, model) } }
        compose.waitForIdle()
        compose.runOnIdle {
            core.emit(
                snapshot(url = "https://example.com/", display = "example.com", webView = 4) {
                    it.copy(address = it.address?.copy(security = AddressView.Security.SECURITY_SECURE))
                },
                subscription = 3,
            )
        }
        compose.waitForIdle()
    }

    @Test
    fun nothingIsListedUntilSomethingIsTyped() {
        showAPage()
        compose.onAllNodesWithTag("suggestion").assertCountEquals(0)
        compose.onNodeWithTag("security-secure").assertExists()

        // The keyboard alone: the field holds the page's address, and nothing is offered for it.
        compose.onNodeWithTag("address-field").performClick()
        compose.waitForIdle()
        compose.onAllNodesWithTag("suggestion").assertCountEquals(0)
        compose.onNodeWithTag("security-secure").assertExists()
        assertEquals(0, core.requests.count { it.suggest_addresses != null })
    }

    @Test
    fun whatIsTypedIsAskedAboutAndThePlacesOfferedAreListed() {
        showAPage()
        compose.onNodeWithTag("address-field").performClick()
        compose.onNodeWithTag("address-field").performTextReplacement("rus")
        compose.waitForIdle()

        assertEquals("rus", core.requests.last().suggest_addresses?.text)
        compose.onAllNodesWithTag("suggestion").assertCountEquals(3)
        // Each by its title and its address.
        compose.onAllNodesWithTag("suggestion")[0].assertTextContains("rus 1")
        compose.onAllNodesWithTag("suggestion")[0].assertTextContains("example.com/rus/1")
        // No padlock vouches for what was typed.
        compose.onNodeWithTag("security-secure").assertDoesNotExist()
    }

    @Test
    fun choosingAPlaceGoesThereAndTheListIsGone() {
        showAPage()
        compose.onNodeWithTag("address-field").performClick()
        compose.onNodeWithTag("address-field").performTextReplacement("rus")
        compose.waitForIdle()

        compose.onAllNodesWithTag("suggestion")[1].performClick()
        compose.waitForIdle()

        val submit = core.requests.last().submit_address
        assertEquals("https://example.com/rus/2", submit?.text)
        assertEquals(2L, submit?.tab_id)
        compose.onAllNodesWithTag("suggestion").assertCountEquals(0)
        // The field shows the page's address again, and its mark, until the core says otherwise.
        compose.onNodeWithTag("security-secure").assertExists()
    }

    /** A tab that is open is a row that says so (038), and choosing it brings the tab in front. */
    @Test
    fun aTabThatIsOpenIsARowThatSaysSoAndChoosingItBringsItInFront() {
        showAPage()
        compose.onNodeWithTag("address-field").performClick()
        compose.onNodeWithTag("address-field").performTextReplacement("tabs")
        compose.waitForIdle()

        // Three rows; the first is the tab, and only it says so, with its title and address.
        compose.onAllNodesWithTag("suggestion").assertCountEquals(3)
        compose.onAllNodesWithTag("suggestion-switch", useUnmergedTree = true).assertCountEquals(1)
        compose.onAllNodesWithTag("suggestion")[0].assertTextContains("Switch to Tab")
        compose.onAllNodesWithTag("suggestion")[0].assertTextContains("tabs 1")
        compose.onAllNodesWithTag("suggestion")[0].assertTextContains("example.com/tabs/1")

        compose.onAllNodesWithTag("suggestion")[0].performClick()
        compose.waitForIdle()

        assertEquals(7L, core.requests.last().activate_tab?.tab_id)
        assertTrue("nothing is loaded", core.requests.none { it.submit_address != null })
        compose.onAllNodesWithTag("suggestion").assertCountEquals(0)
        compose.onNodeWithTag("security-secure").assertExists()
    }
}

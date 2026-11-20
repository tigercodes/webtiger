// The History list drawn (008), over a scripted core: what it shows, and what a person's choices
// send. And the window's way into it.
package com.tigercodes.webtiger.ui

import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertTextContains
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.Ack
import com.tigercodes.webtiger.core.v1.HistoryEntry
import com.tigercodes.webtiger.core.v1.HistoryPage
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.Response
import com.tigercodes.webtiger.design.WebTigerTheme
import com.tigercodes.webtiger.webview.FakePages
import com.tigercodes.webtiger.webview.SystemWebViewEngine
import java.time.ZoneOffset
import java.util.Locale
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancel
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class HistoryListTest {
    @get:Rule val compose = createComposeRule()

    private val scope = CoroutineScope(Dispatchers.Unconfined)

    @After
    fun end() {
        scope.cancel()
    }

    private fun session(core: ScriptedCore) = BrowserSession(core, SystemWebViewEngine(FakePages()), scope)

    /** A core that opens a window, and has two visits: one with a title, one with none. */
    private val core =
        ScriptedCore { request ->
            val query = request.query_history
            when {
                query == null -> windowScript(request)
                query.text.isNotEmpty() -> Response(history_page = HistoryPage())
                else ->
                    Response(
                        history_page =
                            HistoryPage(
                                entries =
                                    listOf(
                                        HistoryEntry(visit_id = 2, title = "Example Domain", url = "https://example.com/", display = "example.com", visited_at_ms = 1_767_268_800_000),
                                        HistoryEntry(visit_id = 1, url = "https://example.net/plain.txt", display = "example.net/plain.txt", visited_at_ms = 1_767_268_000_000),
                                    ),
                                more = true,
                            ),
                    )
            }
        }

    private val opened = ArrayList<HistoryEntry>()
    private var done = 0

    private fun show() {
        val model = HistoryModel(session(core), windowId = 1)
        compose.setContent { WebTigerTheme { HistoryList(model, onOpen = { opened.add(it) }, onDone = { done += 1 }, zone = ZoneOffset.UTC, locale = Locale.UK) } }
        compose.waitForIdle()
    }

    @Test
    fun aVisitIsShownByItsTitleItsAddressAndWhen() {
        show()
        compose.onAllNodesWithTag("history-entry").assertCountEquals(2)
        val first = compose.onAllNodesWithTag("history-entry")[0]
        first.assertTextContains("Example Domain")
        first.assertTextContains("example.com")
        first.assertTextContains("1 Jan 2026, 12:00")
        // A page with no title is named by its address.
        compose.onAllNodesWithTag("history-entry")[1].assertTextContains("example.net/plain.txt")
        compose.onNodeWithText("History").assertExists()
        compose.onNodeWithTag("history-older").assertExists()
    }

    @Test
    fun choosingAVisitOpensItAndDoneIsDone() {
        show()
        compose.onAllNodesWithTag("history-entry")[0].performClick()
        assertEquals(listOf("https://example.com/"), opened.map { it.url })
        compose.onNodeWithTag("history-done").performClick()
        assertEquals(1, done)
    }

    @Test
    fun wordsTypedAreAskedForAndNothingFoundIsSaid() {
        show()
        compose.onNodeWithContentDescription("Search history").assertExists()
        compose.onNodeWithTag("history-search").performTextInput("zebra")
        compose.waitForIdle()
        assertEquals("zebra", core.requests.last().query_history?.text)
        compose.onNodeWithTag("history-nothing").assertTextContains("No visits match.")
    }

    @Test
    fun aVisitIsForgottenByItsButton() {
        show()
        compose.onAllNodesWithTag("history-forget").assertCountEquals(2)
        compose.onAllNodesWithTag("history-forget")[0].performClick()
        compose.waitForIdle()
        assertEquals(2L, core.requests.last().forget_history?.visit_id)
        compose.onAllNodesWithTag("history-entry").assertCountEquals(1)
    }

    @Test
    fun clearAsksWhichVisitsAndEverythingForgetsThemAll() {
        show()
        compose.onNodeWithTag("history-clear").performClick()
        compose.onNodeWithText("Forget which visits?").assertExists()
        compose.onNodeWithText("The Last Hour").assertExists()
        compose.onNodeWithText("Today").assertExists()
        // Nothing has been forgotten by asking.
        assertEquals(0, core.requests.count { it.forget_history != null })

        compose.onNodeWithTag("history-clear-everything").performClick()
        compose.waitForIdle()

        val forgets = core.requests.mapNotNull { it.forget_history }
        assertEquals(listOf(true), forgets.map { it.everything })
        assertNotNull("and what is left is asked for again", core.requests.last().query_history)
        compose.onNodeWithText("Forget which visits?").assertDoesNotExist()
    }

    @Test
    fun cancelForgetsNothing() {
        show()
        compose.onNodeWithTag("history-clear").performClick()
        compose.onNodeWithText("Cancel").performClick()
        compose.waitForIdle()
        assertEquals(0, core.requests.count { it.forget_history != null })
        compose.onAllNodesWithTag("history-entry").assertCountEquals(2)
    }

    // The window's way into the list

    @Test
    fun theWindowsButtonOpensTheListAndAVisitChosenLoadsInTheTabInFront() {
        val session = session(core)
        val model = BrowserModel(session)
        compose.setContent { WebTigerTheme { BrowserWindow(session, model) } }
        compose.waitForIdle()
        compose.onAllNodesWithTag("history-entry").assertCountEquals(0)

        // The button is the way to both lists since 013, and is named for both.
        compose.onNodeWithContentDescription("History and Downloads").performClick()
        compose.waitForIdle()
        assertEquals("the window's own history is asked for", 1L, core.requests.last().query_history?.window_id)
        compose.onAllNodesWithTag("history-entry")[0].performClick()
        compose.waitForIdle()

        val submit = core.requests.last().submit_address
        assertEquals("https://example.com/", submit?.text)
        assertEquals(2L, submit?.tab_id)
        compose.onAllNodesWithTag("history-entry").assertCountEquals(0)
    }
}

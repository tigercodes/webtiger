// Where searches go (020), drawn against a scripted core: the first run's question, what it
// lists, what a press sends, and the empty tab's line with the way to change it.
package com.tigercodes.webtiger.ui

import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.Ack
import com.tigercodes.webtiger.core.v1.Error
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.Response
import com.tigercodes.webtiger.core.v1.SearchEngine
import com.tigercodes.webtiger.core.v1.SearchSettings
import com.tigercodes.webtiger.design.WebTigerTheme
import com.tigercodes.webtiger.webview.FakePages
import com.tigercodes.webtiger.webview.SystemWebViewEngine
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancel
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

private val ENGINES =
    listOf(
        SearchEngine(id = "duckduckgo", name = "DuckDuckGo", host = "duckduckgo.com"),
        SearchEngine(id = "ecosia", name = "Ecosia", host = "www.ecosia.org"),
        SearchEngine(id = "qwant", name = "Qwant", host = "www.qwant.com"),
    )

@RunWith(RobolectricTestRunner::class)
class SearchEnginesTest {
    @get:Rule val compose = createComposeRule()

    private val scope = CoroutineScope(Dispatchers.Unconfined)

    /** What the scripted core says of where searches go, and whether it takes a choice. */
    private var inUse = "duckduckgo"
    private var chosen = false
    private var takes = true
    private val core = ScriptedCore(::script)

    private fun script(request: Request): Response =
        when {
            request.query_search != null -> Response(search_settings = SearchSettings(engines = ENGINES, chosen_id = inUse, chosen = chosen))
            request.choose_search_engine != null ->
                if (takes) {
                    inUse = request.choose_search_engine!!.id
                    chosen = true
                    Response(ack = Ack())
                } else {
                    Response(error = Error(code = "search.unknown_engine"))
                }
            else -> windowScript(request)
        }

    @After
    fun end() {
        scope.cancel()
    }

    private fun aWindow() {
        val session = BrowserSession(core, SystemWebViewEngine(FakePages()), scope)
        compose.setContent { WebTigerTheme { BrowserWindow(session, BrowserModel(session)) } }
        compose.waitForIdle()
    }

    private fun lists(): Int = compose.onAllNodesWithTag("search-engines").fetchSemanticsNodes().size

    /** The engines chosen, as the core was told, in order. */
    private fun choices(): List<String> = core.requests.mapNotNull { it.choose_search_engine?.id }

    @Test
    fun atAFirstRunTheQuestionIsOverTheWindowAndOnePressAnswersIt() {
        aWindow()

        // Nobody has chosen: the list is there, with each engine and where it is, none of
        // them marked, and no way to put it off.
        assertEquals(1, lists())
        compose.onNodeWithTag("search-engine-ecosia").assertTextEquals("Ecosia", "www.ecosia.org")
        compose.onAllNodesWithTag("search-in-use", useUnmergedTree = true).assertCountEquals(0)
        compose.onAllNodesWithTag("search-keep").assertCountEquals(0)

        compose.onNodeWithTag("search-engine-ecosia").performClick()
        compose.waitForIdle()

        assertEquals(listOf("ecosia"), choices())
        assertEquals("the question is answered, and gone", 0, lists())
        compose.onNodeWithTag("search-goes-to").assertTextEquals("Searches go to Ecosia.")
    }

    @Test
    fun aChoiceTheCoreRefusesLeavesTheQuestionWhereItIs() {
        takes = false
        aWindow()

        compose.onNodeWithTag("search-engine-qwant").performClick()
        compose.waitForIdle()

        assertEquals(listOf("qwant"), choices())
        assertEquals(1, lists())
    }

    @Test
    fun aChangeTheCoreRefusesLeavesTheListOpen() {
        chosen = true
        takes = false
        aWindow()
        compose.onNodeWithTag("search-change").performClick()
        compose.waitForIdle()

        compose.onNodeWithTag("search-engine-qwant").performClick()
        compose.waitForIdle()

        assertEquals(listOf("qwant"), choices())
        assertEquals("nothing changed, and the person can see that", 1, lists())
        compose.onNodeWithTag("search-engine-duckduckgo").assertTextEquals("DuckDuckGo", "duckduckgo.com", "In use")
    }

    @Test
    fun whenSomebodyHasChosenNothingIsAskedAndAnEmptyTabSaysWhereSearchesGo() {
        chosen = true
        inUse = "qwant"
        aWindow()

        assertEquals(0, lists())
        compose.onNodeWithTag("search-goes-to").assertTextEquals("Searches go to Qwant.")
        assertEquals(emptyList<String>(), choices())
    }

    @Test
    fun changeOpensTheListWithTheEngineInUseMarkedAndItCanBeKeptOrChanged() {
        chosen = true
        inUse = "qwant"
        aWindow()

        compose.onNodeWithTag("search-change").performClick()
        compose.waitForIdle()

        assertEquals(1, lists())
        // One engine is marked, and only one.
        compose.onAllNodesWithTag("search-in-use", useUnmergedTree = true).assertCountEquals(1)
        compose.onNodeWithTag("search-engine-qwant").assertTextEquals("Qwant", "www.qwant.com", "In use")
        // Kept: the list goes, and nothing was chosen.
        compose.onNodeWithTag("search-keep").performClick()
        compose.waitForIdle()
        assertEquals(0, lists())
        assertEquals(emptyList<String>(), choices())

        // Changed: one press on another.
        compose.onNodeWithTag("search-change").performClick()
        compose.onNodeWithTag("search-engine-duckduckgo").performClick()
        compose.waitForIdle()
        assertEquals(listOf("duckduckgo"), choices())
        assertEquals(0, lists())
        compose.onNodeWithTag("search-goes-to").assertTextEquals("Searches go to DuckDuckGo.")
    }

    @Test
    fun untilTheCoreHasSaidNothingIsAskedAndNothingIsSaid() {
        // A core that knows nothing of search engines: an older one.
        val old = ScriptedCore(::windowScript)
        val session = BrowserSession(old, SystemWebViewEngine(FakePages()), scope)
        compose.setContent { WebTigerTheme { BrowserWindow(session, BrowserModel(session)) } }
        compose.waitForIdle()

        assertEquals(0, lists())
        compose.onAllNodesWithTag("search-goes-to").assertCountEquals(0)
        compose.onAllNodesWithTag("search-change").assertCountEquals(0)
    }
}

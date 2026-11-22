// Sites (039): a tab that is kept. The window's model against a scripted core, and the strip
// drawn by Compose on Robolectric's Android: the one command with two names, the mark, what
// is said of a tab, the button at a Site's end, and a tab's own menu.
package com.tigercodes.webtiger.ui

import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.longClick
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTouchInput
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.AddressView
import com.tigercodes.webtiger.core.v1.TabSummary
import com.tigercodes.webtiger.core.v1.TabView
import com.tigercodes.webtiger.core.v1.WindowSnapshot
import com.tigercodes.webtiger.design.WebTigerTheme
import com.tigercodes.webtiger.webview.FakePages
import com.tigercodes.webtiger.webview.SystemWebViewEngine
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancel
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class SitesTest {
    @get:Rule val compose = createComposeRule()

    private val core = ScriptedCore(::windowScript)
    private val scope = CoroutineScope(Dispatchers.Unconfined)
    private val session = BrowserSession(core, SystemWebViewEngine(FakePages()), scope)
    private val model = BrowserModel(session)

    @After
    fun end() {
        scope.cancel()
    }

    /** The window as the core says it is: tab 2 in front, with a page or without, a Site or not; and the tabs of its strip. */
    private fun said(page: Boolean, kept: Boolean, others: List<TabSummary> = emptyList()): WindowSnapshot =
        WindowSnapshot(
            window_id = 1,
            tab = TabView(tab_id = 2, web_view_id = if (page) 4 else 0, title = "Mail", kept = kept, address = AddressView(url = "https://mail.example.com/", display = "mail.example.com")),
            tabs = listOf(TabSummary(tab_id = 2, title = "Mail", host = "mail.example.com", kept = kept)) + others,
        )

    /** What a core was asked about Sites, in order: "keep 2", "let go 5", "home 2". */
    private fun asked(core: ScriptedCore = this.core): List<String> =
        core.requests.mapNotNull { request ->
            request.keep_tab?.let { "keep ${it.tab_id}" }
                ?: request.let_go_site?.let { "let go ${it.tab_id}" }
                ?: request.send_site_home?.let { "home ${it.tab_id}" }
        }

    @Test
    fun theCommandKeepsATabWithAPageAndLetsASiteGo() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = BrowserModel(unconfinedSession(core))
            // Before the window is open, and for a tab with no page: nothing to keep.
            assertFalse(model.canKeepOrLetGo)
            model.keepOrLetGoTheTabInFront()
            model.open()
            core.emit(said(page = false, kept = false), subscription = 3)
            assertFalse(model.canKeepOrLetGo)
            model.keepOrLetGoTheTabInFront()
            assertEquals("an empty tab is not asked to be kept", emptyList<String>(), asked(core))

            // A page: the tab can be kept, and the core is asked to keep it.
            core.emit(said(page = true, kept = false), subscription = 3)
            assertTrue(model.canKeepOrLetGo)
            model.keepOrLetGoTheTabInFront()
            assertEquals(listOf("keep 2"), asked(core))

            // The core says it is a Site: the same command lets it go; asleep, too.
            core.emit(said(page = true, kept = true), subscription = 3)
            model.keepOrLetGoTheTabInFront()
            core.emit(said(page = false, kept = true), subscription = 3)
            assertTrue(model.canKeepOrLetGo)
            model.keepOrLetGoTheTabInFront()
            assertEquals(listOf("keep 2", "let go 2", "let go 2"), asked(core))

            // A tab of the strip, by what the core says of it.
            core.emit(
                said(page = true, kept = false, others = listOf(TabSummary(tab_id = 5, kept = true, asleep = true), TabSummary(tab_id = 6))),
                subscription = 3,
            )
            model.keepOrLetGo(5)
            model.keepOrLetGo(6)
            assertEquals(listOf("let go 5", "keep 6"), asked(core).takeLast(2))
        }

    /** A Site that has wandered is sent back to its home (039 §7b): only when the core says it is away. */
    @Test
    fun aSiteThatIsAwayIsSentHome() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = BrowserModel(unconfinedSession(core))
            model.open()
            core.emit(said(page = true, kept = true), subscription = 3)
            model.sendHomeTheTabInFront()
            assertEquals("a Site at its home is not asked to go there", emptyList<String>(), asked(core))

            val wandered = said(page = true, kept = true)
            core.emit(wandered.copy(tab = wandered.tab?.copy(away = true, site_name = "Mail", title = "Re: lunch")), subscription = 3)
            model.sendHomeTheTabInFront()
            assertEquals(listOf("home 2"), asked(core))
            // From a tab's own menu the core is asked for that tab.
            model.sendHome(5)
            assertEquals(listOf("home 2", "home 5"), asked(core))
        }

    /** A window whose strip has a Site awake in front, a Site asleep, a tab with a page, and an empty tab. */
    private fun showFourTabs() {
        compose.setContent { WebTigerTheme { BrowserWindow(session, model) } }
        compose.waitForIdle()
        compose.runOnIdle {
            core.emit(
                said(
                    page = true,
                    kept = true,
                    others =
                        listOf(
                            TabSummary(tab_id = 5, title = "Board", host = "board.example.com", kept = true, asleep = true),
                            TabSummary(tab_id = 6, title = "News", host = "news.example.com"),
                            TabSummary(tab_id = 7),
                        ),
                ),
                subscription = 3,
            )
        }
        compose.waitForIdle()
    }

    /** What is said of each tab of the strip beside its name. */
    private fun saidOfTabs(): List<String> =
        compose.onAllNodesWithTag("tab").fetchSemanticsNodes().map { it.config.getOrNull(SemanticsProperties.StateDescription).orEmpty() }

    @Test
    fun theStripMarksASiteAndSaysWhatItIsAndASleepingSiteHasNoButtonAtItsEnd() {
        showFourTabs()
        compose.onAllNodesWithTag("tab").assertCountEquals(4)
        // Two Sites are marked; each tab says what it is, and an ordinary one says nothing.
        compose.onAllNodesWithTag("site-mark", useUnmergedTree = true).assertCountEquals(2)
        compose.onAllNodesWithTag("site-mark-away", useUnmergedTree = true).assertCountEquals(0)
        assertEquals(listOf("Site", "Site, Asleep", "", ""), saidOfTabs())
        // The button at each tab's end: a Site's puts it to sleep, and says so; a sleeping
        // Site has none; the others close.
        val ends =
            compose.onAllNodesWithTag("close-tab", useUnmergedTree = true).fetchSemanticsNodes().map { node ->
                node.config.getOrNull(SemanticsProperties.ContentDescription).orEmpty().ifEmpty {
                    node.children.flatMap { it.config.getOrNull(SemanticsProperties.ContentDescription).orEmpty() }
                }.joinToString()
            }
        assertEquals(listOf("Put to Sleep", "Close Tab", "Close Tab"), ends)

        // The Site's button asks the core to close it, as any tab's does: what closing a Site
        // means is the core's.
        compose.onAllNodesWithTag("close-tab")[0].performClick()
        assertEquals(2L, core.requests.last().close_tab?.tab_id)
    }

    @Test
    fun aTabsOwnMenuKeepsItOrLetsItGoAndAnEmptyTabHasNone() {
        showFourTabs()
        // (The strip is narrower than its four tabs: each is scrolled to, as a finger would.)
        // Held, a Site offers to stop being kept.
        compose.onAllNodesWithTag("tab")[1].performScrollTo().performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onNodeWithTag("keep-or-let-go").assertExists()
        compose.onNodeWithTag("keep-or-let-go").performClick()
        compose.waitForIdle()
        assertEquals(listOf("let go 5"), asked())
        compose.onAllNodesWithTag("keep-or-let-go").assertCountEquals(0)

        // A tab with a page offers to be kept.
        compose.onAllNodesWithTag("tab")[2].performScrollTo().performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onNodeWithTag("keep-or-let-go").performClick()
        compose.waitForIdle()
        assertEquals(listOf("let go 5", "keep 6"), asked())

        // An empty tab has nothing to keep: held, it offers nothing, and nothing is asked.
        compose.onAllNodesWithTag("tab")[3].performScrollTo().performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onAllNodesWithTag("keep-or-let-go").assertCountEquals(0)
        assertEquals(2, asked().size)
        // A press on a tab is still a press: it brings the tab in front.
        compose.onAllNodesWithTag("tab")[2].performScrollTo().performClick()
        assertEquals(6L, core.requests.last().activate_tab?.tab_id)
    }

    @Test
    fun aSiteThatIsAwayHasAnOutlineForItsMarkSaysSoAndItsMenuSendsItHome() {
        compose.setContent { WebTigerTheme { BrowserWindow(session, model) } }
        compose.waitForIdle()
        compose.runOnIdle {
            core.emit(
                said(
                    page = true,
                    kept = false,
                    others = listOf(TabSummary(tab_id = 5, title = "Re: lunch", host = "mail.example.com", kept = true, away = true, site_name = "Mail")),
                ),
                subscription = 3,
            )
        }
        compose.waitForIdle()
        // The mark is the outline, and the tab says it is away.
        compose.onAllNodesWithTag("site-mark-away", useUnmergedTree = true).assertCountEquals(1)
        compose.onAllNodesWithTag("site-mark", useUnmergedTree = true).assertCountEquals(0)
        assertEquals(listOf("", "Site, Away"), saidOfTabs())

        // Its menu has the way home first, by the name it was kept with, and then Stop Keeping.
        compose.onAllNodesWithTag("tab")[1].performScrollTo().performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onNodeWithTag("site-home").assertTextEquals("Return to Mail")
        compose.onNodeWithTag("keep-or-let-go").assertExists()
        compose.onNodeWithTag("site-home").performClick()
        compose.waitForIdle()
        assertEquals(listOf("home 5"), asked())
        compose.onAllNodesWithTag("site-home").assertCountEquals(0)

        // A tab that is no Site, held, has no way home to offer.
        compose.onAllNodesWithTag("tab")[0].performScrollTo().performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onNodeWithTag("keep-or-let-go").assertExists()
        compose.onAllNodesWithTag("site-home").assertCountEquals(0)
    }
}

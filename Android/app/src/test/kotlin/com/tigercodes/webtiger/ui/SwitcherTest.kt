// The phone's switcher (047): the whole row over a window that has no room for a sidebar. The
// window's model against a scripted core, and the window drawn by Compose on Robolectric's
// Android: what the button says, where there is no switcher, and that every row does there
// what it does in the sidebar, and a tab brought in front puts the switcher away.
package com.tigercodes.webtiger.ui

import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.hasAnyAncestor
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.longClick
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTouchInput
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.AddressView
import com.tigercodes.webtiger.core.v1.GroupSummary
import com.tigercodes.webtiger.core.v1.TabSummary
import com.tigercodes.webtiger.core.v1.TabView
import com.tigercodes.webtiger.core.v1.WindowSnapshot
import com.tigercodes.webtiger.design.WebTigerTheme
import com.tigercodes.webtiger.webview.FakePages
import com.tigercodes.webtiger.webview.SystemWebViewEngine
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancel
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
class SwitcherTest {
    @get:Rule val compose = createComposeRule()

    private val core = ScriptedCore(::windowScript)
    private val scope = CoroutineScope(Dispatchers.Unconfined)
    private val session = BrowserSession(core, SystemWebViewEngine(FakePages()), scope)
    private val model = BrowserModel(session)

    @After
    fun end() {
        scope.cancel()
    }

    private val work = GroupSummary(group_id = 30, name = "Work", sites = 1)
    private val house = GroupSummary(group_id = 31, name = "House", shut = true, sites = 3)

    /**
     * The row as a core lists it: a Favourite; a Site in no Group; the open Group's Site; none
     * of the shut Group's three; and two tabs, the first of them in front. Nine tabs in all.
     */
    private val row =
        listOf(
            TabSummary(tab_id = 8, title = "Mail", host = "mail.example.com", kept = true, favourite = true, site_name = "Mail"),
            TabSummary(tab_id = 7, title = "Wiki", host = "wiki.example.com", kept = true, site_name = "Wiki"),
            TabSummary(tab_id = 5, title = "Docs", host = "docs.example.com", kept = true, site_name = "Docs", group_id = 30),
            TabSummary(tab_id = 2, title = "News", host = "news.example.com"),
            TabSummary(tab_id = 6, title = "Story", host = "news.example.com"),
        )

    private fun said(): WindowSnapshot =
        WindowSnapshot(
            window_id = 1,
            tab = TabView(tab_id = 2, web_view_id = 4, title = "News", address = AddressView(url = "https://news.example.com/", display = "news.example.com")),
            tabs = row,
            groups = listOf(work, house),
            row_tabs = 8,
        )

    /** What the core was asked about tabs and Groups, in order: "front 7", "close 6", "new", "shut 30". */
    private fun asked(): List<String> =
        core.requests.mapNotNull { request ->
            request.activate_tab?.let { "front ${it.tab_id}" }
                ?: request.close_tab?.let { "close ${it.tab_id}" }
                ?: request.open_tab?.let { "new" }
                ?: request.shut_group?.let { "${if (it.shut) "shut" else "open"} ${it.group_id}" }
        }

    private fun show() {
        compose.setContent { WebTigerTheme { BrowserWindow(session, model) } }
        compose.waitForIdle()
        compose.runOnIdle { core.emit(said(), subscription = 3) }
        compose.waitForIdle()
    }

    /** A thing of the switcher, by its name: the strip under it has things of the same names. */
    private fun listed(tag: String): SemanticsMatcher = hasTestTag(tag) and hasAnyAncestor(hasTestTag("switcher-sheet"))

    private fun open() {
        compose.onNodeWithTag("switcher").performClick()
        compose.waitForIdle()
        compose.onAllNodesWithTag("switcher-sheet").assertCountEquals(1)
    }

    private fun isOpen(): Boolean = compose.onAllNodesWithTag("switcher-sheet").fetchSemanticsNodes().isNotEmpty()

    @Test
    fun theButtonSaysHowManyTabsTheRowHasByTheCoresCount() {
        show()
        // The core's count, with a shut Group's among them: not the five tabs the window was
        // told to show.
        assertEquals(8, model.rowTabs)
        assertEquals(
            listOf("Tabs: 8"),
            compose.onNodeWithTag("switcher").fetchSemanticsNode().config.getOrNull(SemanticsProperties.ContentDescription),
        )
        // The square has the number while it has room for it.
        assertEquals("8", switcherCount(8))
        assertEquals("99", switcherCount(99))
        assertEquals("99+", switcherCount(100))
        assertFalse("shut until it is asked for", isOpen())
    }

    @Test
    @Config(qualifiers = "w720dp-h900dp")
    fun aWindowWithRoomForASidebarHasNoSwitcher() {
        show()
        compose.onAllNodesWithTag("switcher").assertCountEquals(0)
        compose.runOnIdle { model.openSwitcher() }
        compose.waitForIdle()
        assertFalse(model.showingSwitcher)
        assertFalse(isOpen())
    }

    @Test
    fun theSwitcherHasTheWholeRowAndATabBroughtInFrontPutsItAway() {
        show()
        open()
        // Every tab the window was told of, and both Groups' chips, the shut one saying how
        // many it holds; the Space at its head.
        compose.onAllNodes(listed("tab")).assertCountEquals(5)
        compose.onAllNodes(listed("group")).assertCountEquals(2)
        compose.onAllNodes(listed("space-chip")).assertCountEquals(1)
        // The window it covers is nothing to a screen reader while it does: every tab that
        // can be found is a row of the switcher, and the toolbar is not there to be found.
        compose.onAllNodesWithTag("tab").assertCountEquals(5)
        compose.onAllNodesWithTag("address-field").assertCountEquals(0)
        compose.onAllNodesWithTag("switcher").assertCountEquals(0)

        // A row pressed: its tab is asked to come in front, and the switcher goes.
        compose.onAllNodes(listed("tab"))[1].performScrollTo().performClick()
        compose.waitForIdle()
        assertEquals(listOf("front 7"), asked())
        assertFalse("a tab brought in front puts the switcher away", isOpen())
        // And the window is there again.
        compose.onAllNodesWithTag("address-field").assertCountEquals(1)
        compose.onAllNodesWithTag("switcher").assertCountEquals(1)

        // The row of the tab that is in front already: nothing is asked, and the switcher goes.
        open()
        compose.onAllNodes(listed("tab"))[3].performScrollTo().performClick()
        compose.waitForIdle()
        assertEquals(listOf("front 7"), asked())
        assertFalse(isOpen())

        // A Favourite's tile comes in front as a row does.
        open()
        compose.onAllNodes(listed("tab"))[0].performClick()
        compose.waitForIdle()
        assertEquals(listOf("front 7", "front 8"), asked())
        assertFalse(isOpen())
    }

    @Test
    fun whatIsDoneToTheRowInTheSwitcherLeavesItOpen() {
        show()
        open()
        // A tab closed by its row's button, and a Group shut by its chip: asked of the core as
        // in the sidebar, and the switcher stays for the next.
        compose.onAllNodes(listed("close-tab"))[0].performScrollTo().performClick()
        compose.waitForIdle()
        compose.onAllNodes(listed("group"))[0].performScrollTo().performClick()
        compose.waitForIdle()
        assertEquals(listOf("close 7", "shut 30"), asked())
        assertTrue("it stays while the row is put in order", isOpen())

        // "New Tab" opens one, and the switcher goes: the new tab is what is wanted.
        compose.onNode(listed("new-tab")).performScrollTo().performClick()
        compose.waitForIdle()
        assertEquals(listOf("close 7", "shut 30", "new"), asked())
        assertFalse(isOpen())

        // Done puts it away, and asks nothing.
        open()
        compose.onNodeWithTag("switcher-done").performClick()
        compose.waitForIdle()
        assertFalse(isOpen())
        assertEquals(3, asked().size)
    }

    @Test
    fun aRowsOwnMenuWorksInTheSwitcherAndThePageOfAGroupIsOverIt() {
        show()
        open()
        // A row held: its own menu, as in the sidebar. "New Group…" opens the page of a
        // Group over the switcher, which is there again when the page is put away.
        compose.onAllNodes(listed("tab"))[4].performScrollTo().performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onNodeWithTag("group-new").performClick()
        compose.waitForIdle()
        compose.onAllNodesWithTag("group-page").assertCountEquals(1)
        assertTrue("the switcher is under the page", model.showingSwitcher)
        compose.onNodeWithTag("group-cancel").performClick()
        compose.waitForIdle()
        compose.onAllNodesWithTag("group-page").assertCountEquals(0)
        assertTrue("and is there again", isOpen())
        compose.onAllNodes(listed("tab")).assertCountEquals(5)
    }
}

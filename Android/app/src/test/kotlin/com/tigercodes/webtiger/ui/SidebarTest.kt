// The sidebar (043): the window's tabs down its side. The window's model against a scripted
// core, and the window drawn by Compose on Robolectric's Android at a phone's width and at a
// tablet's: where a sidebar is drawn, what is in it, and the switch.
package com.tigercodes.webtiger.ui

import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.AddressView
import com.tigercodes.webtiger.core.v1.GroupSummary
import com.tigercodes.webtiger.core.v1.Layout
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
import org.robolectric.annotation.Config

/** A window as wide as a tablet's: wide enough for a sidebar beside a page, and tall enough for every row of these tests to be in view. */
private const val WIDE = "w720dp-h900dp"

/** A window as narrow as a phone's. */
private const val NARROW = "w411dp-h820dp"

@RunWith(RobolectricTestRunner::class)
class SidebarTest {
    @get:Rule val compose = createComposeRule()

    private val core = ScriptedCore(::windowScript)
    private val scope = CoroutineScope(Dispatchers.Unconfined)
    private val session = BrowserSession(core, SystemWebViewEngine(FakePages()), scope)
    private val model = BrowserModel(session)

    @After
    fun end() {
        scope.cancel()
    }

    private fun tab(id: Long, title: String, kept: Boolean = false, favourite: Boolean = false, group: Long = 0): TabSummary =
        TabSummary(tab_id = id, title = title, host = "example.com", kept = kept || favourite, favourite = favourite, group_id = group, site_name = if (kept || favourite) title else "")

    private val work = GroupSummary(group_id = 30, name = "Work", sites = 2)
    private val house = GroupSummary(group_id = 31, name = "House", shut = true, sites = 3)

    /** A row of every kind of thing, as a core lists it: two Favourites, a Site, Work's two, none of House's, two tabs. */
    private val row =
        listOf(
            tab(1, "Mail", favourite = true),
            tab(3, "Calendar", favourite = true),
            tab(4, "Docs", kept = true),
            tab(2, "Wiki", kept = true, group = 30),
            tab(5, "Board", kept = true, group = 30),
            tab(6, "News"),
            tab(7, "Story"),
        )

    /** The window as the core says it is: tab 2 in front with a page, the row, and its layout. */
    private fun said(layout: Layout): WindowSnapshot =
        WindowSnapshot(
            window_id = 1,
            tab = TabView(tab_id = 2, web_view_id = 4, title = "Wiki", kept = true, address = AddressView(url = "https://example.com/", display = "example.com")),
            tabs = row,
            groups = listOf(work, house),
            layout = layout,
        )

    /** What a core was asked about the layout, in order: "sidebar", "strip". */
    private fun asked(core: ScriptedCore = this.core): List<String> =
        core.requests.mapNotNull { request ->
            request.set_layout?.let { "${if (it.layout == Layout.LAYOUT_SIDEBAR) "sidebar" else if (it.layout == Layout.LAYOUT_STRIP) "strip" else "another"} in ${it.window_id}" }
        }

    private fun show(layout: Layout) {
        compose.setContent { WebTigerTheme { BrowserWindow(session, model) } }
        compose.waitForIdle()
        compose.runOnIdle { core.emit(said(layout), subscription = 3) }
        compose.waitForIdle()
    }

    @Test
    fun theSwitchAsksTheCoreForTheOtherLayoutInAWindowWithRoomForASidebar() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = BrowserModel(unconfinedSession(core))
            // A window has no room for a sidebar until it says it has: nothing is asked, and
            // the strip is drawn whatever the core says.
            model.open()
            core.emit(said(Layout.LAYOUT_SIDEBAR), subscription = 3)
            assertFalse(model.hasRoomForSidebar)
            assertFalse(model.showsSidebar)
            model.switchLayout()
            assertEquals(emptyList<String>(), asked(core))

            // Wide: it has what the core says, and the switch asks for the other.
            model.hasRoomForSidebar = true
            assertTrue(model.showsSidebar)
            model.switchLayout()
            assertEquals(listOf("strip in 1"), asked(core))
            core.emit(said(Layout.LAYOUT_STRIP), subscription = 3)
            assertFalse(model.showsSidebar)
            model.switchLayout()
            assertEquals(listOf("strip in 1", "sidebar in 1"), asked(core))
            // A core from before layouts names none: the strip.
            core.emit(said(Layout.LAYOUT_UNSPECIFIED), subscription = 3)
            assertFalse(model.showsSidebar)
            model.switchLayout()
            assertEquals("sidebar in 1", asked(core).last())
        }

    /** Ctrl+Shift+S (036): the key's command is the switch, and does nothing where there is no room for a sidebar. */
    @Test
    fun theKeysCommandSwitchesTheLayout() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = BrowserModel(unconfinedSession(core))
            model.open()
            core.emit(said(Layout.LAYOUT_STRIP), subscription = 3)
            model.hasRoomForSidebar = true
            model.perform(KeyCommand.SwitchLayout, openLocation = {}, showHistory = {})
            assertEquals(listOf("sidebar in 1"), asked(core))
            model.hasRoomForSidebar = false
            model.perform(KeyCommand.SwitchLayout, openLocation = {}, showHistory = {})
            assertEquals(listOf("sidebar in 1"), asked(core))
        }

    @Test
    fun theSidebarTakesTheRowApartAndKeepsTheCoresOrderInEachPart() {
        val parts = sidebarParts(row, listOf(work, house))
        assertEquals(listOf(1L, 3L), parts.favourites.map { it.tab_id })
        assertEquals(listOf(6L, 7L), parts.others.map { it.tab_id })
        assertEquals(
            listOf("4", "[Work]", "2", "5", "[House]"),
            parts.tree.map { item ->
                when (item) {
                    is StripItem.Tab -> "${item.tab.tab_id}"
                    is StripItem.Group -> "[${item.group.name}]"
                }
            },
        )
        // Tabs alone are under the line, and the tree is empty.
        val plain = sidebarParts(listOf(tab(6, "News"), tab(7, "Story")), emptyList())
        assertEquals(listOf(6L, 7L), plain.others.map { it.tab_id })
        assertTrue(plain.tree.isEmpty() && plain.favourites.isEmpty())
    }

    @Test
    @Config(qualifiers = WIDE)
    fun aWideWindowDrawsTheSidebarWhenTheCoreSaysSo() {
        show(Layout.LAYOUT_SIDEBAR)
        compose.onAllNodesWithTag("sidebar").assertCountEquals(1)
        compose.onAllNodesWithTag("tab-strip").assertCountEquals(0)
        // Every tab the core lists, the Groups' chips, the Favourites' tiles; the address
        // field and the button for a new tab are in the sidebar.
        compose.onAllNodesWithTag("tab").assertCountEquals(7)
        compose.onAllNodesWithTag("group").assertCountEquals(2)
        compose.onAllNodesWithTag("favourite-letter", useUnmergedTree = true).assertCountEquals(2)
        compose.onAllNodesWithTag("address-field").assertCountEquals(1)
        compose.onAllNodesWithTag("new-tab").assertCountEquals(1)
        // Down the side: the Favourites above the tree, the tree above the other tabs; and
        // a Group's Site set in under its Group.
        val tabs = compose.onAllNodesWithTag("tab").fetchSemanticsNodes()
        val groups = compose.onAllNodesWithTag("group").fetchSemanticsNodes()
        val (mail, calendar, docs, wiki, board) = tabs.take(5).map { it.positionInRoot }
        val (news, story) = tabs.drop(5).map { it.positionInRoot }
        val (workAt, houseAt) = groups.map { it.positionInRoot }
        assertEquals("the Favourites are in a row", mail.y, calendar.y)
        assertTrue("Favourites, Docs, [Work], Wiki, Board, [House], News, Story", mail.y < docs.y && docs.y < workAt.y && workAt.y < wiki.y && wiki.y < board.y && board.y < houseAt.y && houseAt.y < news.y && news.y < story.y)
        assertTrue("a Group's Site is set in", wiki.x > docs.x && board.x == wiki.x)

        // A row does what its chip does: a press brings its tab in front; its button closes it.
        compose.onAllNodesWithTag("tab")[5].performScrollTo().performClick()
        assertEquals(6L, core.requests.last().activate_tab?.tab_id)
        compose.onAllNodesWithTag("close-tab")[0].performScrollTo().performClick()
        assertEquals(4L, core.requests.last().close_tab?.tab_id)
        // The window's button names the way back, and asks the core for the strip.
        assertEquals("Tabs at the Top", compose.onNodeWithTag("layout", useUnmergedTree = true).fetchSemanticsNode().let { node -> (node.config.getOrNull(SemanticsProperties.ContentDescription).orEmpty() + node.children.flatMap { it.config.getOrNull(SemanticsProperties.ContentDescription).orEmpty() }).joinToString() })
        compose.onNodeWithTag("layout").performClick()
        assertEquals(listOf("strip in 1"), asked())
    }

    @Test
    @Config(qualifiers = WIDE)
    fun aWideWindowDrawsTheStripWhenTheCoreSaysSoAndOffersTheSwitch() {
        show(Layout.LAYOUT_STRIP)
        compose.onAllNodesWithTag("tab-strip").assertCountEquals(1)
        compose.onAllNodesWithTag("sidebar").assertCountEquals(0)
        compose.onAllNodesWithTag("tab").assertCountEquals(7)
        compose.onNodeWithTag("layout").performClick()
        assertEquals(listOf("sidebar in 1"), asked())
        // The core says the sidebar: the window draws it, with the same tabs.
        compose.runOnIdle { core.emit(said(Layout.LAYOUT_SIDEBAR), subscription = 3) }
        compose.waitForIdle()
        compose.onAllNodesWithTag("sidebar").assertCountEquals(1)
        compose.onAllNodesWithTag("tab-strip").assertCountEquals(0)
        compose.onAllNodesWithTag("tab").assertCountEquals(7)
    }

    /** A row of thousands (000 §7): the sidebar makes the rows that are in view, and not the row. */
    @Test
    @Config(qualifiers = WIDE)
    fun aSidebarOfFiveThousandRowsMakesOnlyTheRowsInView() {
        compose.setContent { WebTigerTheme { BrowserWindow(session, model) } }
        compose.waitForIdle()
        val many = (1L..5000L).map { id -> tab(id, "Page $id", kept = id % 6 != 5L, group = if (id % 6 != 5L && id % 2 == 0L) 30 else 0) }
        // As a core lists a row: the Sites in no Group, the Group's, the tabs.
        val listed = many.filter { it.kept && it.group_id == 0L } + many.filter { it.group_id == 30L } + many.filter { !it.kept }
        compose.runOnIdle {
            core.emit(
                WindowSnapshot(
                    window_id = 1,
                    tab = TabView(tab_id = 5000, web_view_id = 4, title = "Page 5000", address = AddressView(url = "https://example.com/", display = "example.com")),
                    tabs = listed,
                    groups = listOf(work),
                    layout = Layout.LAYOUT_SIDEBAR,
                ),
                subscription = 3,
            )
        }
        compose.waitForIdle()
        compose.onAllNodesWithTag("sidebar").assertCountEquals(1)
        val made = compose.onAllNodesWithTag("tab").fetchSemanticsNodes().size
        assertTrue("some rows are made: $made", made > 0)
        assertTrue("and not the five thousand: $made", made < 100)
        // The tab in front is far down the row, a Site of the Group: it was brought into view,
        // and is among those made.
        val inFront = { compose.onAllNodesWithTag("tab").fetchSemanticsNodes().filter { it.config.getOrNull(SemanticsProperties.Selected) == true }.map { node -> node.config.getOrNull(SemanticsProperties.Text).orEmpty().joinToString { it.text } } }
        assertEquals(listOf("Page 5000"), inFront())
        // Another comes in front, a tab that is no Site, near the end of those under the line:
        // it is brought into view too.
        compose.runOnIdle {
            core.emit(
                WindowSnapshot(
                    window_id = 1,
                    tab = TabView(tab_id = 4997, web_view_id = 4, title = "Page 4997", address = AddressView(url = "https://example.com/", display = "example.com")),
                    tabs = listed,
                    groups = listOf(work),
                    layout = Layout.LAYOUT_SIDEBAR,
                ),
                subscription = 3,
            )
        }
        compose.waitForIdle()
        assertEquals(listOf("Page 4997"), inFront())
        assertTrue(compose.onAllNodesWithTag("tab").fetchSemanticsNodes().size < 100)
    }

    @Test
    @Config(qualifiers = NARROW)
    fun aNarrowWindowDrawsTheStripWhateverTheCoreSaysAndHasNoSwitch() {
        show(Layout.LAYOUT_SIDEBAR)
        compose.onAllNodesWithTag("tab-strip").assertCountEquals(1)
        compose.onAllNodesWithTag("sidebar").assertCountEquals(0)
        compose.onAllNodesWithTag("tab").assertCountEquals(7)
        compose.onAllNodesWithTag("layout").assertCountEquals(0)
        assertFalse(model.hasRoomForSidebar)
        assertEquals(emptyList<String>(), asked())
    }
}

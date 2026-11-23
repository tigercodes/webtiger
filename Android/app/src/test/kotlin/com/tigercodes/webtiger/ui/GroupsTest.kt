// Groups (041): Sites in folders. How the strip's row is made of the core's two lists; the
// window's model against a scripted core; and the strip, a tab's menu, a Group's menu and the
// Group page drawn by Compose on Robolectric's Android.
package com.tigercodes.webtiger.ui

import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.longClick
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.performTextReplacement
import androidx.compose.ui.test.performTouchInput
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.Ack
import com.tigercodes.webtiger.core.v1.AddressView
import com.tigercodes.webtiger.core.v1.Error
import com.tigercodes.webtiger.core.v1.GroupMade
import com.tigercodes.webtiger.core.v1.GroupSummary
import com.tigercodes.webtiger.core.v1.Layout
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.Response
import com.tigercodes.webtiger.core.v1.SpacesPage
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
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class GroupsTest {
    @get:Rule val compose = createComposeRule()

    /** What the scripted core refuses, by what is asked: "make", "rename", "colour"; each with its code. */
    private val refused = HashMap<String, String>()

    /** A core that opens window 1 with tab 2, lists two colours, makes Group 30, and says Ack to the rest. */
    private fun script(request: Request): Response {
        fun unless(what: String, answer: () -> Response): Response = refused[what]?.let { Response(error = Error(code = it)) } ?: answer()
        return when {
            request.query_spaces != null -> Response(spaces_page = SpacesPage(colours = listOf("red", "blue")))
            request.make_group != null -> unless("make") { Response(group_made = GroupMade(group_id = 30)) }
            request.rename_group != null -> unless("rename") { Response(ack = Ack()) }
            request.colour_group != null -> unless("colour") { Response(ack = Ack()) }
            else -> windowScript(request)
        }
    }

    private val core = ScriptedCore(::script)
    private val scope = CoroutineScope(Dispatchers.Unconfined)
    private val session = BrowserSession(core, SystemWebViewEngine(FakePages()), scope)
    private val model = BrowserModel(session)

    @After
    fun end() {
        scope.cancel()
    }

    private fun tab(id: Long, kept: Boolean = false, group: Long = 0, title: String = "Tab $id", host: String = "example.com"): TabSummary =
        TabSummary(tab_id = id, title = title, host = host, kept = kept, group_id = group)

    private val work = GroupSummary(group_id = 30, name = "Work", colour = "red", sites = 2)
    private val house = GroupSummary(group_id = 31, name = "House", shut = true, sites = 3)

    /** The window as the core says it is: tab 2 in front with a page, these tabs and Groups. */
    private fun said(tabs: List<TabSummary>, groups: List<GroupSummary> = emptyList(), private: Boolean = false): WindowSnapshot =
        WindowSnapshot(
            window_id = 1,
            tab = TabView(tab_id = 2, web_view_id = 4, title = "Tab 2", address = AddressView(url = "https://example.com/", display = "example.com")),
            tabs = tabs,
            groups = groups,
            private_window = private,
        )

    /** What a core was asked about Groups, in order. */
    private fun asked(core: ScriptedCore = this.core): List<String> =
        core.requests.mapNotNull { request ->
            request.make_group?.let { "make '${it.name}' in ${it.window_id}" }
                ?: request.rename_group?.let { "name ${it.group_id} '${it.name}'" }
                ?: request.colour_group?.let { "colour ${it.group_id} '${it.colour}'" }
                ?: request.shut_group?.let { "${if (it.shut) "shut" else "open"} ${it.group_id}" }
                ?: request.remove_group?.let { "remove ${it.group_id}" }
                ?: request.move_to_group?.let { "move ${it.tab_id} to ${it.group_id}" }
        }

    /** The row as a line of words: a tab by its id, a Group by its name in brackets. */
    private fun row(tabs: List<TabSummary>, groups: List<GroupSummary>): List<String> =
        stripRow(tabs, groups).map { item ->
            when (item) {
                is StripItem.Tab -> "${item.tab.tab_id}"
                is StripItem.Group -> "[${item.group.name}]"
            }
        }

    @Test
    fun theRowIsTheSitesInNoGroupThenEachGroupWithItsSitesThenTheOtherTabs() {
        val later = GroupSummary(group_id = 32, name = "Later")
        // As the core lists them: a Site in no Group, Work's two, the one of House that is in
        // front (the others are left out, House being shut), and two tabs.
        val tabs = listOf(tab(1, kept = true), tab(2, kept = true, group = 30), tab(3, kept = true, group = 30), tab(4, kept = true, group = 31), tab(5), tab(6))
        assertEquals(listOf("1", "[Work]", "2", "3", "[House]", "4", "[Later]", "5", "6"), row(tabs, listOf(work, house, later)))
        // No Groups: the tabs as they are listed. No tabs but one: the Groups before it.
        assertEquals(listOf("1", "2", "3", "4", "5", "6"), row(tabs, emptyList()))
        assertEquals(listOf("[Work]", "[House]", "5"), row(listOf(tab(5)), listOf(work, house)))
        // A Group that is shut and has nothing shown is drawn all the same, where it stands.
        assertEquals(listOf("1", "[House]", "5"), row(listOf(tab(1, kept = true), tab(5)), listOf(house)))
        // A tab that names a Group the list does not have is drawn as one in none: never lost.
        assertEquals(listOf("7", "[Work]", "2", "5"), row(listOf(tab(7, kept = true, group = 99), tab(2, kept = true, group = 30), tab(5)), listOf(work)))
        // Sites and Groups only: a Group with nothing to show is after the Sites in no Group.
        assertEquals(listOf("1", "[House]"), row(listOf(tab(1, kept = true)), listOf(house)))
        // The tabs are never put in another order than the core's, whatever it is: a shell
        // places the chips, and nothing else.
        assertEquals(listOf("5", "1"), row(listOf(tab(5), tab(1, kept = true)), emptyList()))
        assertEquals(listOf("[Work]", "[House]", "5", "2", "1"), row(listOf(tab(5), tab(2, kept = true, group = 30), tab(1, kept = true)), listOf(work, house)))
        assertEquals(listOf("[Work]", "[House]", "4", "2"), row(listOf(tab(4, kept = true, group = 31), tab(2, kept = true, group = 30)), listOf(work, house)))
    }

    @Test
    fun aGroupIsShutOpenedAndRemovedAndATabIsMovedByAskingTheCore() =
        runTest {
            val core = ScriptedCore(::script)
            val model = BrowserModel(unconfinedSession(core))
            model.open()
            core.emit(said(listOf(tab(2, kept = true, group = 30), tab(5), tab(6, host = "")), listOf(work, house)), subscription = 3)
            assertEquals(listOf("Work", "House"), model.groups.map { it.name })

            // A press on a chip asks for the other of what the core last said.
            model.shutOrOpen(work)
            model.shutOrOpen(house)
            assertEquals(listOf("shut 30", "open 31"), asked(core))
            // A tab is moved, and taken out, by asking; the model only spares the asking for a
            // tab that has nothing to keep.
            assertTrue(model.canBeGrouped(model.tabs[0]))
            assertTrue(model.canBeGrouped(model.tabs[1]))
            assertFalse("an empty tab has nothing to keep", model.canBeGrouped(model.tabs[2]))
            model.moveToGroup(5, 31)
            model.moveToGroup(2, 0)
            model.removeGroup(31)
            assertEquals(listOf("shut 30", "open 31", "move 5 to 31", "move 2 to 0", "remove 31"), asked(core))

            // A private window has no Groups (037): nothing of it can be put in one.
            core.emit(said(listOf(tab(2)), private = true), subscription = 3)
            assertFalse(model.canBeGrouped(model.tabs[0]))
        }

    @Test
    fun aNewGroupIsMadeColouredAndGivenTheTabAndARefusalLeavesThePageOpen() =
        runTest {
            val core = ScriptedCore(::script)
            val model = BrowserModel(unconfinedSession(core))
            model.open()
            core.emit(said(listOf(tab(2), tab(5))), subscription = 3)

            // The page opens for a tab, and the colours to choose among are asked for.
            model.askForANewGroup(5)
            assertEquals(GroupAsk.New(5), model.groupAsked)
            assertEquals(listOf("red", "blue"), model.spaceColours)
            // Its button: the Group is made by the name as it was typed, coloured, and the tab
            // put in it; and the page closes.
            model.finishGroupPage(" Work ", "blue")
            assertEquals(listOf("make ' Work ' in 1", "colour 30 'blue'", "move 5 to 30"), asked(core))
            assertNull(model.groupAsked)
            assertNull(model.groupRefusal)
            // With no colour chosen none is asked for.
            model.askForANewGroup(2)
            model.finishGroupPage("House", "")
            assertEquals(listOf("make 'House' in 1", "move 2 to 30"), asked(core).drop(3))

            // A Group the core will not make: the page stays, with the core's reason, and
            // nothing is coloured or moved. Opened again, it says nothing of it.
            refused["make"] = "group.too_many"
            model.askForANewGroup(2)
            model.finishGroupPage("Play", "red")
            assertEquals("group.too_many", model.groupRefusal)
            assertEquals(GroupAsk.New(2), model.groupAsked)
            assertEquals(listOf("make 'Play' in 1"), asked(core).drop(5))
            model.closeGroupPage()
            model.askForANewGroup(2)
            assertNull(model.groupRefusal)
        }

    @Test
    fun aGroupIsEditedByWhatChangedAndARefusedNameLeavesTheColour() =
        runTest {
            val core = ScriptedCore(::script)
            val model = BrowserModel(unconfinedSession(core))
            model.open()
            core.emit(said(listOf(tab(2, kept = true, group = 30)), listOf(work)), subscription = 3)

            // Nothing changed: nothing is asked, and the page closes.
            model.askToEdit(work)
            assertEquals(GroupAsk.Edit(work), model.groupAsked)
            model.finishGroupPage("Work", "red")
            assertEquals(emptyList<String>(), asked(core))
            assertNull(model.groupAsked)

            // The name alone; the colour alone, also to none; both, the name first.
            model.askToEdit(work)
            model.finishGroupPage("Office", "red")
            model.askToEdit(work)
            model.finishGroupPage("Work", "")
            model.askToEdit(work)
            model.finishGroupPage("Office", "blue")
            assertEquals(listOf("name 30 'Office'", "colour 30 ''", "name 30 'Office'", "colour 30 'blue'"), asked(core))

            // A name the core will not take: the colour is not asked for, and the page stays.
            refused["rename"] = "group.name_empty"
            model.askToEdit(work)
            model.finishGroupPage("  ", "blue")
            assertEquals("name 30 '  '", asked(core).last())
            assertEquals(5, asked(core).size)
            assertEquals("group.name_empty", model.groupRefusal)
            assertEquals(GroupAsk.Edit(work), model.groupAsked)
        }

    /** A window whose strip has a Site in no Group, an open Group with the tab in front and another Site, a shut Group, and a tab. */
    private fun showGroups() {
        compose.setContent { WebTigerTheme { BrowserWindow(session, model) } }
        compose.waitForIdle()
        compose.runOnIdle {
            core.emit(
                said(
                    listOf(
                        tab(1, kept = true, title = "Mail"),
                        tab(2, kept = true, group = 30, title = "Wiki"),
                        tab(3, kept = true, group = 30, title = "Board"),
                        tab(5, title = "News"),
                    ),
                    listOf(work, house),
                ),
                subscription = 3,
            )
        }
        compose.waitForIdle()
    }

    private fun textOf(tag: String): List<String> =
        compose.onAllNodesWithTag(tag).fetchSemanticsNodes().map { node -> node.config.getOrNull(SemanticsProperties.Text).orEmpty().joinToString { it.text } }

    @Test
    fun theStripDrawsAChipForEachGroupWhichSaysWhatItIsAndAPressShutsOrOpensIt() {
        showGroups()
        compose.onAllNodesWithTag("tab").assertCountEquals(4)
        // A chip for each Group, by its name; shut, with how many Sites it holds.
        assertEquals(listOf("Work", "House (3)"), textOf("group"))
        assertEquals(
            listOf("Group, open", "Group, shut, Sites: 3"),
            compose.onAllNodesWithTag("group").fetchSemanticsNodes().map { it.config.getOrNull(SemanticsProperties.StateDescription).orEmpty() },
        )
        // The mark: an outline for the open one, filled for the shut one.
        compose.onAllNodesWithTag("group-mark", useUnmergedTree = true).assertCountEquals(1)
        compose.onAllNodesWithTag("group-mark-shut", useUnmergedTree = true).assertCountEquals(1)
        // Where each stands: the Site in no Group, Work and its two, House, the tab.
        val places = compose.onAllNodesWithTag("tab").fetchSemanticsNodes().map { it.positionInRoot.x } + compose.onAllNodesWithTag("group").fetchSemanticsNodes().map { it.positionInRoot.x }
        val (mail, wiki, board, news) = places.take(4)
        val (workAt, houseAt) = places.drop(4)
        assertTrue("Mail, [Work], Wiki, Board, [House], News: $places", mail < workAt && workAt < wiki && wiki < board && board < houseAt && houseAt < news)

        // A press asks the core for the other of what it is.
        compose.onAllNodesWithTag("group")[0].performScrollTo().performClick()
        compose.onAllNodesWithTag("group")[1].performScrollTo().performClick()
        assertEquals(listOf("shut 30", "open 31"), asked())
    }

    @Test
    fun aGroupsOwnMenuEditsItOrRemovesIt() {
        showGroups()
        compose.onAllNodesWithTag("group")[0].performScrollTo().performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onNodeWithTag("group-remove").assertExists()
        compose.onNodeWithTag("group-edit").performClick()
        compose.waitForIdle()
        // The page that edits it: its name in the field, and nothing asked yet.
        compose.onNodeWithTag("group-title").assertTextEquals("Edit Group")
        compose.onNodeWithTag("group-name").assertTextEquals("Name", "Work")
        compose.onNodeWithTag("group-name").performTextReplacement("Office")
        compose.onNodeWithTag("group-done").performClick()
        compose.waitForIdle()
        assertEquals(listOf("name 30 'Office'"), asked())
        compose.onAllNodesWithTag("group-page").assertCountEquals(0)

        // Removed: the core is asked; what becomes of its Sites is the core's.
        compose.onAllNodesWithTag("group")[1].performScrollTo().performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onNodeWithTag("group-remove").performClick()
        compose.waitForIdle()
        assertEquals(listOf("name 30 'Office'", "remove 31"), asked())
    }

    @Test
    fun aTabsOwnMenuMovesItToAGroupOrToNoneOrToANewOne() {
        showGroups()
        // A Site of Work, held: the Groups by name and "No Group", Work marked as its own.
        compose.onAllNodesWithTag("tab")[2].performScrollTo().performTouchInput { longClick() }
        compose.waitForIdle()
        assertEquals(listOf("Work", "House"), textOf("move-to-group"))
        assertEquals(
            listOf(true, false),
            compose.onAllNodesWithTag("move-to-group").fetchSemanticsNodes().map { it.config.getOrNull(SemanticsProperties.Selected) },
        )
        compose.onAllNodesWithTag("move-to-group")[1].performClick()
        compose.waitForIdle()
        assertEquals(listOf("move 3 to 31"), asked())
        compose.onAllNodesWithTag("move-to-group").assertCountEquals(0)

        // Out of its Group.
        compose.onAllNodesWithTag("tab")[2].performScrollTo().performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onNodeWithTag("group-none").performClick()
        compose.waitForIdle()
        assertEquals(listOf("move 3 to 31", "move 3 to 0"), asked())

        // A tab that is no Site: "No Group" is its own; "New Group…" opens the page, which
        // makes nothing of no name, and then makes the Group and puts the tab in it.
        compose.onAllNodesWithTag("tab")[3].performScrollTo().performTouchInput { longClick() }
        compose.waitForIdle()
        assertEquals(true, compose.onNodeWithTag("group-none").fetchSemanticsNode().config.getOrNull(SemanticsProperties.Selected))
        compose.onNodeWithTag("group-new").performClick()
        compose.waitForIdle()
        compose.onNodeWithTag("group-title").assertTextEquals("New Group")
        compose.onNodeWithTag("group-done").assertIsNotEnabled()
        compose.onNodeWithTag("group-name").performTextInput("Reading")
        compose.onNodeWithTag("group-done").assertIsEnabled()
        // The colours are the core's: the two it listed, and none.
        compose.onNodeWithTag("space-colour-blue").performScrollTo().performClick()
        compose.onNodeWithTag("group-done").performScrollTo().performClick()
        compose.waitForIdle()
        assertEquals(listOf("make 'Reading' in 1", "colour 30 'blue'", "move 5 to 30"), asked().drop(2))
        compose.onAllNodesWithTag("group-page").assertCountEquals(0)
    }

    @Test
    fun whatTheCoreRefusesIsSaidOnThePageInWordsAndCancelLeavesIt() {
        showGroups()
        refused["make"] = "group.too_many"
        compose.onAllNodesWithTag("tab")[3].performScrollTo().performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onNodeWithTag("group-new").performClick()
        compose.waitForIdle()
        compose.onAllNodesWithTag("group-refusal").assertCountEquals(0)
        compose.onNodeWithTag("group-name").performTextInput("Play")
        compose.onNodeWithTag("group-done").performScrollTo().performClick()
        compose.waitForIdle()
        compose.onNodeWithTag("group-refusal").assertTextEquals("There are as many Groups as there can be.")
        // A code this build has no words for is said as what is said when no more is known.
        refused["make"] = "group.some_new_code"
        compose.onNodeWithTag("group-done").performScrollTo().performClick()
        compose.waitForIdle()
        compose.onNodeWithTag("group-refusal").assertTextEquals("The Group couldn’t be made.")
        // Cancel: back to the window, with nothing made.
        compose.onNodeWithTag("group-cancel").performScrollTo().performClick()
        compose.waitForIdle()
        compose.onAllNodesWithTag("group-page").assertCountEquals(0)
        assertEquals(listOf("make 'Play' in 1", "make 'Play' in 1"), asked())
    }

    /** A window whose core names the sidebar (043) has its strip all the same: this shell has no sidebar to draw yet. */
    @Test
    fun theStripIsDrawnWhateverLayoutTheCoreNames() {
        compose.setContent { WebTigerTheme { BrowserWindow(session, model) } }
        compose.waitForIdle()
        compose.runOnIdle {
            core.emit(said(listOf(tab(1, kept = true, title = "Mail"), tab(2, title = "News")), listOf(work)).copy(layout = Layout.LAYOUT_SIDEBAR), subscription = 3)
        }
        compose.waitForIdle()
        compose.onAllNodesWithTag("tab-strip").assertCountEquals(1)
        compose.onAllNodesWithTag("tab").assertCountEquals(2)
        compose.onAllNodesWithTag("group").assertCountEquals(1)
        // Nothing is asked of the core about the layout: there is no switch here.
        assertEquals(0, core.requests.count { it.set_layout != null })
    }

    @Test
    fun aTabWithNothingToKeepAndAPrivateWindowsTabHaveNoMoveToGroup() {
        compose.setContent { WebTigerTheme { BrowserWindow(session, model) } }
        compose.waitForIdle()
        // An empty tab has no menu at all; a private window's tab has its menu without Groups.
        compose.runOnIdle { core.emit(said(listOf(tab(2), tab(7, title = "", host = "")), listOf(work)), subscription = 3) }
        compose.waitForIdle()
        compose.onAllNodesWithTag("tab")[1].performScrollTo().performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onAllNodesWithTag("group-new").assertCountEquals(0)
        compose.runOnIdle { core.emit(said(listOf(tab(2)), private = true), subscription = 3) }
        compose.waitForIdle()
        compose.onAllNodesWithTag("tab")[0].performScrollTo().performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onNodeWithTag("keep-or-let-go").assertExists()
        compose.onAllNodesWithTag("group-new").assertCountEquals(0)
        compose.onAllNodesWithTag("move-to-group").assertCountEquals(0)
        assertEquals(emptyList<String>(), asked())
    }
}

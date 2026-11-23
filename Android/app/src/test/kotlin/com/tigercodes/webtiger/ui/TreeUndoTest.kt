// Undo for what is done to the tree (044): the command that undoes the newest thing done to
// arrange the row. The window's model against a scripted core, and the strip drawn by Compose
// on Robolectric's Android: the command is there when the core says there is something to
// undo, says what that was, is at the top of a tab's menu and of a Group's, and asks the core.
package com.tigercodes.webtiger.ui

import android.content.Context
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.longClick
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTouchInput
import androidx.test.core.app.ApplicationProvider
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
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
class TreeUndoTest {
    @get:Rule val compose = createComposeRule()

    private val core = ScriptedCore(::windowScript)
    private val scope = CoroutineScope(Dispatchers.Unconfined)
    private val session = BrowserSession(core, SystemWebViewEngine(FakePages()), scope)
    private val model = BrowserModel(session)

    @After
    fun end() {
        scope.cancel()
    }

    /** The window as the core says it is: a Group's chip, an empty tab and a tab with a page in front; and what an undo of its row's arrangement would undo. */
    private fun said(undoes: String): WindowSnapshot =
        WindowSnapshot(
            window_id = 1,
            tab = TabView(tab_id = 2, web_view_id = 4, title = "News", address = AddressView(url = "https://example.com/", display = "example.com")),
            tabs = listOf(TabSummary(tab_id = 6), TabSummary(tab_id = 2, title = "News", host = "example.com")),
            groups = listOf(GroupSummary(group_id = 30, name = "Work")),
            undo_arrangement = undoes,
        )

    /** The windows whose arrangement a core was asked to undo, in order. */
    private fun asked(core: ScriptedCore = this.core): List<Long> = core.requests.mapNotNull { it.undo_arrangement?.window_id }

    private fun show(undoes: String) {
        compose.setContent { WebTigerTheme { BrowserWindow(session, model) } }
        compose.waitForIdle()
        compose.runOnIdle { core.emit(said(undoes), subscription = 3) }
        compose.waitForIdle()
    }

    private fun words(kind: String): String = ApplicationProvider.getApplicationContext<Context>().getString(undoWords(kind))

    @Test
    fun theCommandIsThereWhenTheCoreSaysSoAndAsksTheCoreToUndo() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = BrowserModel(unconfinedSession(core))
            // Before the window is open, and while the core says there is nothing: no
            // command, and nothing is asked.
            assertFalse(model.canUndoArrangement)
            assertNull(model.treeUndo)
            model.undoArrangement()
            model.open()
            core.emit(said(undoes = ""), subscription = 3)
            assertFalse(model.canUndoArrangement)
            model.undoArrangement()
            assertEquals(emptyList<Long>(), asked(core))

            // The core says a Group was removed: the command says so, and asks the core to
            // undo the arrangement of this window's row.
            core.emit(said(undoes = "group.removed"), subscription = 3)
            assertTrue(model.canUndoArrangement)
            assertEquals(com.tigercodes.webtiger.R.string.undo_group_removed, model.treeUndo?.words)
            model.treeUndo?.action?.invoke()
            assertEquals(listOf(1L), asked(core))

            // The core says there is nothing left: the command is gone.
            core.emit(said(undoes = ""), subscription = 3)
            assertNull(model.treeUndo)
            model.undoArrangement()
            assertEquals(1, asked(core).size)
        }

    @Test
    fun theCommandSaysWhatItUndoes() {
        val said =
            mapOf(
                "site.kept" to "Undo Keep as Site",
                "site.let_go" to "Undo Stop Keeping",
                "group.made" to "Undo New Group",
                "group.removed" to "Undo Remove Group",
                "group.renamed" to "Undo Rename Group",
                "group.coloured" to "Undo Group Colour",
                "group.moved" to "Undo Move to Group",
                "favourite.added" to "Undo Add to Favourites",
                "favourite.removed" to "Undo Remove from Favourites",
                "tab.moved" to "Undo Move Tab",
                "group.reordered" to "Undo Move Group",
            )
        for ((kind, expected) in said) assertEquals(kind, expected, words(kind))
        // A kind this shell has no words for, as a later core might name: it is undone all
        // the same, and the command says only that.
        assertEquals("Undo", words("tree.something_new"))
    }

    @Test
    fun itIsAtTheTopOfATabsMenuAndOfAGroupsAndAsksTheCore() {
        show(undoes = "group.removed")
        // A tab's own menu: the command first, saying what it undoes; chosen, the core is
        // asked and the menu is put away.
        compose.onAllNodesWithTag("tab")[1].performScrollTo().performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onNodeWithTag("undo-arrangement").assertTextEquals("Undo Remove Group")
        val undo = compose.onNodeWithTag("undo-arrangement").fetchSemanticsNode().boundsInRoot
        val keep = compose.onNodeWithTag("keep-or-let-go").fetchSemanticsNode().boundsInRoot
        assertTrue("it is above the rest of the menu", undo.bottom <= keep.top)
        compose.onNodeWithTag("undo-arrangement").performClick()
        compose.waitForIdle()
        assertEquals(listOf(1L), asked())
        compose.onAllNodesWithTag("undo-arrangement").assertCountEquals(0)

        // A Group's own menu has it too, above "Edit Group…".
        compose.onNodeWithTag("group").performScrollTo().performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onNodeWithTag("undo-arrangement").assertTextEquals("Undo Remove Group")
        val edit = compose.onNodeWithTag("group-edit").fetchSemanticsNode().boundsInRoot
        assertTrue(compose.onNodeWithTag("undo-arrangement").fetchSemanticsNode().boundsInRoot.bottom <= edit.top)
        compose.onNodeWithTag("undo-arrangement").performClick()
        compose.waitForIdle()
        assertEquals(listOf(1L, 1L), asked())

        // An empty tab has nothing to keep and nowhere to go: its menu is the command alone.
        compose.onAllNodesWithTag("tab")[0].performScrollTo().performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onAllNodesWithTag("undo-arrangement").assertCountEquals(1)
        compose.onAllNodesWithTag("keep-or-let-go").assertCountEquals(0)
    }

    @Test
    fun withNothingToUndoNoMenuHasIt() {
        show(undoes = "")
        compose.onAllNodesWithTag("tab")[1].performScrollTo().performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onAllNodesWithTag("keep-or-let-go").assertCountEquals(1)
        compose.onAllNodesWithTag("undo-arrangement").assertCountEquals(0)
        compose.onNodeWithTag("keep-or-let-go").performClick()
        compose.waitForIdle()
        compose.onNodeWithTag("group").performScrollTo().performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onAllNodesWithTag("group-edit").assertCountEquals(1)
        compose.onAllNodesWithTag("undo-arrangement").assertCountEquals(0)
        assertEquals(emptyList<Long>(), asked())
    }

    /** Down the side the rows' menus, and the Groups', have it as the strip's have. */
    @Test
    @Config(qualifiers = "w720dp-h900dp")
    fun downTheSideARowsMenuAndAGroupsHaveItToo() {
        compose.setContent { WebTigerTheme { BrowserWindow(session, model) } }
        compose.waitForIdle()
        compose.runOnIdle { core.emit(said(undoes = "tab.moved").copy(layout = Layout.LAYOUT_SIDEBAR), subscription = 3) }
        compose.waitForIdle()
        compose.onAllNodesWithTag("sidebar").assertCountEquals(1)
        compose.onAllNodesWithTag("tab")[1].performScrollTo().performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onNodeWithTag("undo-arrangement").assertTextEquals("Undo Move Tab")
        compose.onNodeWithTag("undo-arrangement").performClick()
        compose.waitForIdle()
        compose.onNodeWithTag("group").performScrollTo().performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onNodeWithTag("undo-arrangement").performClick()
        compose.waitForIdle()
        assertEquals(listOf(1L, 1L), asked())
    }
}

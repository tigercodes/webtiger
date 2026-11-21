// Undo (033): a tab that was closed comes back. The window's model against a scripted core, and
// the button for a new tab, held, drawn by Compose on Robolectric's Android.
package com.tigercodes.webtiger.ui

import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.longClick
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTouchInput
import com.tigercodes.webtiger.core.ScriptedCore
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
class UndoTest {
    @get:Rule val compose = createComposeRule()

    private val core = ScriptedCore(::windowScript)
    private val scope = CoroutineScope(Dispatchers.Unconfined)
    private val session = BrowserSession(core, SystemWebViewEngine(FakePages()), scope)
    private val model = BrowserModel(session)

    @After
    fun end() {
        scope.cancel()
    }

    /** The window as the core says it is, with what an undo would undo. */
    private fun said(undo: String): WindowSnapshot = WindowSnapshot(window_id = 1, tab = TabView(tab_id = 2), undo = undo)

    /** The windows the core was asked to undo in, in order. */
    private fun undone(): List<Long> = core.requests.mapNotNull { it.undo?.window_id }

    @Test
    fun aClosedTabIsOfferedBackOnlyWhileTheCoreSaysThereIsOne() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = BrowserModel(unconfinedSession(core))

            // Before the window is open, and before the core has said anything: nothing is
            // offered, and asking does nothing.
            assertFalse(model.canReopenClosedTab)
            model.reopenClosedTab()
            model.open()
            assertFalse(model.canReopenClosedTab)
            model.reopenClosedTab()
            assertTrue("nothing is asked while there is nothing to bring back", core.requests.none { it.undo != null })

            // The core says a closed tab can be brought back: asked for with the window, in
            // one request.
            core.emit(said("tab.closed"), subscription = 3)
            assertTrue(model.canReopenClosedTab)
            val before = core.requests.size
            model.reopenClosedTab()
            assertEquals(listOf(1L), core.requests.mapNotNull { it.undo?.window_id })
            assertEquals(before + 1, core.requests.size)

            // A kind of undo this shell does not know is not offered as a closed tab.
            core.emit(said("space.renamed"), subscription = 3)
            assertEquals("space.renamed", model.undoKind)
            assertFalse(model.canReopenClosedTab)
            // And when the core says there is nothing: nothing.
            core.emit(said(""), subscription = 3)
            assertFalse(model.canReopenClosedTab)
            model.reopenClosedTab()
            assertEquals(1, core.requests.count { it.undo != null })
        }

    @Test
    fun theButtonForANewTabHeldOffersTheClosedTabBack() {
        compose.setContent { WebTigerTheme { BrowserWindow(session, model) } }
        compose.waitForIdle()
        compose.runOnIdle { core.emit(said("tab.closed"), subscription = 3) }
        compose.waitForIdle()
        val button = compose.onNodeWithTag("new-tab")
        // A screen reader is told of what holding it does, by its name.
        assertEquals("Reopen Closed Tab", button.fetchSemanticsNode().config.getOrNull(SemanticsActions.OnLongClick)?.label)
        compose.onAllNodesWithTag("reopen-closed-tab").assertCountEquals(0)

        // Held: the menu, with the one thing in it. Nothing is asked of the core yet, and no
        // tab was opened by the hold.
        button.performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onAllNodesWithTag("reopen-closed-tab").assertCountEquals(1)
        assertEquals(emptyList<Long>(), undone())
        assertTrue(core.requests.none { it.open_tab != null })

        // Chosen: the core is asked to undo, with the window, and the menu is gone.
        compose.onNodeWithTag("reopen-closed-tab").performClick()
        compose.waitForIdle()
        assertEquals(listOf(1L), undone())
        compose.onAllNodesWithTag("reopen-closed-tab").assertCountEquals(0)

        // A press, not held, is still a new tab, and no undo.
        button.performClick()
        compose.waitForIdle()
        assertEquals(1, core.requests.count { it.open_tab != null })
        assertEquals(listOf(1L), undone())
    }

    @Test
    fun withNothingToBringBackTheButtonHeldIsOnlyAButton() {
        compose.setContent { WebTigerTheme { BrowserWindow(session, model) } }
        compose.waitForIdle()
        val button = compose.onNodeWithTag("new-tab")
        // Nothing was closed: a screen reader is told of nothing to hold it for, and holding
        // it offers nothing. An item that does nothing is not shown.
        assertNull(button.fetchSemanticsNode().config.getOrNull(SemanticsActions.OnLongClick))
        button.performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onAllNodesWithTag("reopen-closed-tab").assertCountEquals(0)
        assertEquals(emptyList<Long>(), undone())

        // The menu is open when the closed tab is brought back some other way (the core says
        // there is nothing more): the menu goes.
        compose.runOnIdle { core.emit(said("tab.closed"), subscription = 3) }
        compose.waitForIdle()
        button.performTouchInput { longClick() }
        compose.waitForIdle()
        compose.onAllNodesWithTag("reopen-closed-tab").assertCountEquals(1)
        compose.runOnIdle { core.emit(said(""), subscription = 3) }
        compose.waitForIdle()
        compose.onAllNodesWithTag("reopen-closed-tab").assertCountEquals(0)
    }
}

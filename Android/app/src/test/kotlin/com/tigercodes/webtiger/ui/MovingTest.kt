// Moving a tab in its row (045): the commands that move a tab a place. The window's model
// against a scripted core, and the row drawn by Compose on Robolectric's Android: what the
// core is asked, that a command is grey the way the core says the tab cannot go, and the
// commands' words, which go by how the row is drawn.
package com.tigercodes.webtiger.ui

import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.platform.LocalLayoutDirection
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
import androidx.compose.ui.test.performTouchInput
import androidx.compose.ui.unit.LayoutDirection
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.AddressView
import com.tigercodes.webtiger.core.v1.Layout
import com.tigercodes.webtiger.core.v1.RowStep
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

/** A window wide enough for a sidebar beside a page. */
private const val WIDE = "w720dp-h900dp"

@RunWith(RobolectricTestRunner::class)
class MovingTest {
    @get:Rule val compose = createComposeRule()

    private val core = ScriptedCore(::windowScript)
    private val scope = CoroutineScope(Dispatchers.Unconfined)
    private val session = BrowserSession(core, SystemWebViewEngine(FakePages()), scope)
    private val model = BrowserModel(session)

    @After
    fun end() {
        scope.cancel()
    }

    /** A tab as the core says it is: which way a step would move it. */
    private fun tab(
        id: Long,
        back: Boolean = false,
        forward: Boolean = false,
        title: String = "Tab $id",
        host: String = "example.com",
        kept: Boolean = false,
        favourite: Boolean = false,
    ): TabSummary = TabSummary(tab_id = id, title = title, host = host, kept = kept || favourite, favourite = favourite, steps_back = back, steps_forward = forward)

    /** The window as the core says it is: tab 2 in front with a page, and these tabs. */
    private fun said(tabs: List<TabSummary>, layout: Layout = Layout.LAYOUT_STRIP): WindowSnapshot =
        WindowSnapshot(
            window_id = 1,
            tab = TabView(tab_id = 2, web_view_id = 4, title = "Tab 2", address = AddressView(url = "https://example.com/", display = "example.com")),
            tabs = tabs,
            layout = layout,
        )

    /** What a core was asked to move, in order: "2 back", "5 forward". */
    private fun asked(core: ScriptedCore = this.core): List<String> =
        core.requests.mapNotNull { request ->
            request.move_tab?.let { "${it.tab_id} ${if (it.by_ == RowStep.ROW_STEP_BACK) "back" else if (it.by_ == RowStep.ROW_STEP_FORWARD) "forward" else "by no step"}" }
        }

    /**
     * A row of two Favourites, a Site alone among the Sites, two tabs (the second in front) and
     * an empty tab: by the core's word each of a pair can go towards the other, the empty tab
     * back, and the Site nowhere.
     */
    private val row =
        listOf(
            tab(8, forward = true, favourite = true, title = "Mail"),
            tab(9, back = true, favourite = true, title = "Board"),
            tab(7, kept = true, title = "Wiki"),
            tab(5, forward = true, title = "News"),
            tab(2, back = true, forward = true, title = "Story"),
            tab(6, back = true, title = "", host = ""),
        )

    private fun show(layout: Layout = Layout.LAYOUT_STRIP, direction: LayoutDirection = LayoutDirection.Ltr) {
        compose.setContent {
            CompositionLocalProvider(LocalLayoutDirection provides direction) {
                WebTigerTheme { BrowserWindow(session, model) }
            }
        }
        compose.waitForIdle()
        compose.runOnIdle { core.emit(said(row, layout), subscription = 3) }
        compose.waitForIdle()
    }

    /**
     * Opens the menu of the tab at a place in the row, by a long press; brought into view
     * first where the row scrolls (the sidebar's tiles are above what scrolls).
     */
    private fun hold(place: Int, scrolls: Boolean = true) {
        val tab = compose.onAllNodesWithTag("tab")[place]
        if (scrolls) tab.performScrollTo()
        tab.performTouchInput { longClick() }
        compose.waitForIdle()
    }

    @Test
    fun aTabIsMovedAPlaceByAskingTheCoreWhichSaysWhichWayItCanGo() =
        runTest {
            val core = ScriptedCore(::windowScript)
            val model = BrowserModel(unconfinedSession(core))
            model.open()
            core.emit(said(listOf(tab(5, forward = true), tab(2, back = true))), subscription = 3)
            // By the core's word the first can go forward and the second back, and neither the
            // other way; no step is no way to go.
            assertTrue(canMove(model.tabs[0], RowStep.ROW_STEP_FORWARD))
            assertFalse(canMove(model.tabs[0], RowStep.ROW_STEP_BACK))
            assertTrue(canMove(model.tabs[1], RowStep.ROW_STEP_BACK))
            assertFalse(canMove(model.tabs[1], RowStep.ROW_STEP_FORWARD))
            assertFalse(canMove(model.tabs[1], RowStep.ROW_STEP_UNSPECIFIED))
            // The core is asked for a step: where the tab then is, is the core's to say.
            model.moveTab(2, RowStep.ROW_STEP_BACK)
            model.moveTab(5, RowStep.ROW_STEP_FORWARD)
            assertEquals(listOf("2 back", "5 forward"), asked(core))
        }

    @Test
    fun theCommandsWordsGoByHowTheRowIsDrawn() {
        fun said(moves: List<TabMove>): List<String> =
            moves.map { move ->
                val words =
                    when (move.words) {
                        R.string.tabs_move_left -> "left"
                        R.string.tabs_move_right -> "right"
                        R.string.tabs_move_up -> "up"
                        R.string.tabs_move_down -> "down"
                        else -> "other words"
                    }
                "$words: ${move.tag}"
            }
        // Along the top, where words run from left to right: left is towards the row's start.
        assertEquals(listOf("left: move-back", "right: move-forward"), said(tabMoves(atTheSide = false, rightToLeft = false)))
        // Where they run from right to left the row does too: left is towards its end. The
        // commands are listed in the same order.
        assertEquals(listOf("left: move-forward", "right: move-back"), said(tabMoves(atTheSide = false, rightToLeft = true)))
        // Down the side, up is towards the row's start, whichever way words run.
        assertEquals(listOf("up: move-back", "down: move-forward"), said(tabMoves(atTheSide = true, rightToLeft = false)))
        assertEquals(listOf("up: move-back", "down: move-forward"), said(tabMoves(atTheSide = true, rightToLeft = true)))
    }

    @Test
    fun aTabsOwnMenuMovesItAPlaceAndIsGreyTheWayItCannotGo() {
        show()
        compose.onAllNodesWithTag("tab").assertCountEquals(6)
        // The first of two tabs: it can go right, and not left.
        hold(3)
        compose.onNodeWithTag("move-back").assertTextEquals("Move Tab Left")
        compose.onNodeWithTag("move-back").assertIsNotEnabled()
        compose.onNodeWithTag("move-forward").assertTextEquals("Move Tab Right")
        compose.onNodeWithTag("move-forward").assertIsEnabled()
        compose.onNodeWithTag("move-back").performClick()
        compose.waitForIdle()
        assertEquals("nothing is asked the way it cannot go", emptyList<String>(), asked())
        compose.onNodeWithTag("move-forward").performClick()
        compose.waitForIdle()
        assertEquals(listOf("5 forward"), asked())
        compose.onAllNodesWithTag("move-forward").assertCountEquals(0)

        // The tab in front, between two: either way.
        hold(4)
        compose.onNodeWithTag("move-back").assertIsEnabled()
        compose.onNodeWithTag("move-forward").assertIsEnabled()
        compose.onNodeWithTag("move-back").performClick()
        compose.waitForIdle()
        assertEquals(listOf("5 forward", "2 back"), asked())

        // A Favourite's tile moves as any tab: the first of two, right.
        hold(0)
        compose.onNodeWithTag("move-back").assertIsNotEnabled()
        compose.onNodeWithTag("move-forward").performClick()
        compose.waitForIdle()
        assertEquals(listOf("5 forward", "2 back", "8 forward"), asked())

        // A tab alone in its part has nowhere to go: its menu has the rest, and no such
        // command.
        hold(2)
        compose.onAllNodesWithTag("keep-or-let-go").assertCountEquals(1)
        compose.onAllNodesWithTag("move-back").assertCountEquals(0)
        compose.onAllNodesWithTag("move-forward").assertCountEquals(0)
        compose.onNodeWithTag("keep-or-let-go").performClick()
        compose.waitForIdle()

        // An empty tab has nothing to keep, and a menu all the same when it has somewhere to
        // go: the commands that move it, and no other.
        hold(5)
        compose.onAllNodesWithTag("keep-or-let-go").assertCountEquals(0)
        compose.onNodeWithTag("move-forward").assertIsNotEnabled()
        compose.onNodeWithTag("move-back").performClick()
        compose.waitForIdle()
        assertEquals(listOf("5 forward", "2 back", "8 forward", "6 back"), asked())
    }

    @Test
    fun whereWordsRunFromRightToLeftLeftIsTowardsTheRowsEnd() {
        show(direction = LayoutDirection.Rtl)
        // The first of two tabs can go towards the row's end, which is on its left.
        hold(3)
        compose.onNodeWithTag("move-forward").assertTextEquals("Move Tab Left")
        compose.onNodeWithTag("move-forward").assertIsEnabled()
        compose.onNodeWithTag("move-back").assertTextEquals("Move Tab Right")
        compose.onNodeWithTag("move-back").assertIsNotEnabled()
        compose.onNodeWithTag("move-forward").performClick()
        compose.waitForIdle()
        assertEquals(listOf("5 forward"), asked())
    }

    @Test
    @Config(qualifiers = WIDE)
    fun downTheSideTheCommandsSayUpAndDownAndATilesSayLeftAndRight() {
        show(Layout.LAYOUT_SIDEBAR)
        compose.onAllNodesWithTag("sidebar").assertCountEquals(1)
        // A row of the sidebar is in a column: the first of two tabs can go down.
        hold(3)
        compose.onNodeWithTag("move-back").assertTextEquals("Move Tab Up")
        compose.onNodeWithTag("move-back").assertIsNotEnabled()
        compose.onNodeWithTag("move-forward").assertTextEquals("Move Tab Down")
        compose.onNodeWithTag("move-forward").performClick()
        compose.waitForIdle()
        assertEquals(listOf("5 forward"), asked())
        // A Favourite's tile is in a row there too: left and right.
        hold(1, scrolls = false)
        compose.onNodeWithTag("move-back").assertTextEquals("Move Tab Left")
        compose.onNodeWithTag("move-forward").assertTextEquals("Move Tab Right")
        compose.onNodeWithTag("move-back").performClick()
        compose.waitForIdle()
        assertEquals(listOf("5 forward", "9 back"), asked())
    }
}

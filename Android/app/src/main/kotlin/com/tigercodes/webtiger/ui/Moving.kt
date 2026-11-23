// Moving a tab in its row (045): the two commands that move a tab a place, with their words,
// which go by how the row is drawn. What a row's parts are, and how far a tab can go among
// those of its own, are the core's: it says of each tab which way a step would move it.
package com.tigercodes.webtiger.ui

import androidx.annotation.StringRes
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.core.v1.RowStep

/** One of the two commands that move a tab a place: the step it asks of the core, its words, and the name a test finds it by. */
internal class TabMove(val step: RowStep, @param:StringRes val words: Int) {
    val tag: String get() = if (step == RowStep.ROW_STEP_BACK) "move-back" else "move-forward"
}

/**
 * The two, in the order a menu lists them, for a tab as it is drawn. Down the side, up is
 * towards the row's start. Along the top (and for a Favourite's tile, which is in a row
 * wherever the tabs are) the row runs the way words do: where they run from right to left,
 * left is towards its end.
 */
internal fun tabMoves(atTheSide: Boolean, rightToLeft: Boolean): List<TabMove> =
    if (atTheSide) {
        listOf(TabMove(RowStep.ROW_STEP_BACK, R.string.tabs_move_up), TabMove(RowStep.ROW_STEP_FORWARD, R.string.tabs_move_down))
    } else {
        listOf(
            TabMove(if (rightToLeft) RowStep.ROW_STEP_FORWARD else RowStep.ROW_STEP_BACK, R.string.tabs_move_left),
            TabMove(if (rightToLeft) RowStep.ROW_STEP_BACK else RowStep.ROW_STEP_FORWARD, R.string.tabs_move_right),
        )
    }

/** Whether a step that way would move the tab: the core's word. */
internal fun canMove(tab: com.tigercodes.webtiger.core.v1.TabSummary, step: RowStep): Boolean =
    when (step) {
        RowStep.ROW_STEP_BACK -> tab.steps_back
        RowStep.ROW_STEP_FORWARD -> tab.steps_forward
        else -> false
    }

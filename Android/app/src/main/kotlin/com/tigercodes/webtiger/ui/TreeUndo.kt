// Undo for what is done to the tree (044), the shell's part: the command that undoes the newest
// thing done to arrange the row a window shows, with words that say what that was. What can be
// undone, and what undoing it puts back, are the core's: it names the kind, and a shell has the
// words for it. On a touch screen it is asked from the menu of a tab, and of a Group (WT-D32).
package com.tigercodes.webtiger.ui

import androidx.annotation.StringRes
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import com.tigercodes.webtiger.R

/** "Undo …" for a menu: what it says, and what it does. */
internal class TreeUndo(@param:StringRes val words: Int, val action: () -> Unit)

/**
 * The words of the command for what it would undo, by the core's name for it. A kind this
 * shell has no words for is undone all the same: the command says "Undo".
 */
@StringRes
internal fun undoWords(kind: String): Int =
    when (kind) {
        "site.kept" -> R.string.undo_site_kept
        "site.let_go" -> R.string.undo_site_let_go
        "group.made" -> R.string.undo_group_made
        "group.removed" -> R.string.undo_group_removed
        "group.renamed" -> R.string.undo_group_renamed
        "group.coloured" -> R.string.undo_group_coloured
        "group.moved" -> R.string.undo_group_moved
        "favourite.added" -> R.string.undo_favourite_added
        "favourite.removed" -> R.string.undo_favourite_removed
        "tab.moved" -> R.string.undo_tab_moved
        "group.reordered" -> R.string.undo_group_reordered
        else -> R.string.undo_arrangement
    }

/** The command for a menu, while there is something to undo. */
internal val BrowserModel.treeUndo: TreeUndo?
    get() = if (canUndoArrangement) TreeUndo(undoWords(undoArrangementKind), ::undoArrangement) else null

/** The command at the top of a menu, with a line under it, while there is something to undo. */
@Composable
internal fun TreeUndoItem(undo: TreeUndo?, onChosen: () -> Unit) {
    if (undo == null) return
    DropdownMenuItem(
        text = { Text(stringResource(undo.words)) },
        onClick = {
            onChosen()
            undo.action()
        },
        modifier = Modifier.testTag("undo-arrangement"),
    )
    HorizontalDivider()
}

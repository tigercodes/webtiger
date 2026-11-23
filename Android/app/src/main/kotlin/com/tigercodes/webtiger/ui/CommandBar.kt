// The command bar (048), the window's part: the address field's list has Spaces and commands
// in it. With each question the window sends the commands it can do now, by a name and its
// words; the core says which match what is typed. A row that is chosen is done here: a Space
// is shown, a command is what its menu's item is. What is typed is never done as a command.
package com.tigercodes.webtiger.ui

import androidx.annotation.StringRes
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.core.v1.AddressSuggestion
import com.tigercodes.webtiger.core.v1.OfferedCommand
import com.tigercodes.webtiger.core.v1.RowStep
import com.tigercodes.webtiger.core.v1.StepTab

/**
 * A command of the window's that needs nothing more than to be chosen (048 D6): the name the
 * core gives back when its words match, its words, and what it does.
 */
internal class BarCommand(val name: String, @param:StringRes val words: Int, val action: () -> Unit)

/**
 * What only the window that is drawn can do, as the command bar asks it of the model: say a
 * command's words in the person's language, open the lists of what was visited, and say which
 * way the row runs. A model whose window is not drawn has no words, and sends no commands.
 */
internal class WindowDoes(
    val words: ((Int) -> String)? = null,
    val showHistory: (() -> Unit)? = null,
    val rightToLeft: Boolean = false,
)

/**
 * The commands that can be done now, in the order their menus have them. One that can do
 * nothing now is not among them (048 D4): a row that does nothing when it is chosen is worse
 * than no row.
 */
internal fun BrowserModel.barCommands(): List<BarCommand> {
    if (windowId == null) return emptyList()
    val front = tabs.firstOrNull { it.tab_id == tab.tab_id }
    val commands = mutableListOf(BarCommand("tab.new", R.string.tabs_new, ::openTab))
    if (canReopenClosedTab) commands += BarCommand("tab.reopen", R.string.tabs_reopen, ::reopenClosedTab)
    commands += BarCommand("tab.close", R.string.tabs_close) { closeTab(tab.tab_id) }
    // The tab in front as a Site: kept or let go, a Favourite or none, sent home.
    if (canKeepOrLetGo) {
        commands += BarCommand(if (tab.kept) "site.let_go" else "site.keep", if (tab.kept) R.string.sites_let_go else R.string.sites_keep, ::keepOrLetGoTheTabInFront)
    }
    if (front != null && canFavourite(front)) {
        commands +=
            BarCommand(if (front.favourite) "favourite.remove" else "favourite.add", if (front.favourite) R.string.favourites_remove else R.string.favourites_add) {
                setFavourite(front.tab_id, !front.favourite)
            }
    }
    if (tab.away) commands += BarCommand("site.home", R.string.sites_return_site, ::sendHomeTheTabInFront)
    // How the window is drawn, and what it shows of its page.
    if (hasRoomForSidebar) {
        commands += BarCommand(if (showsSidebar) "layout.top" else "layout.side", if (showsSidebar) R.string.layout_top else R.string.layout_side, ::switchLayout)
    }
    if (hasPage) commands += BarCommand("page.find", R.string.find_open, ::openFind)
    if (zoom != null) commands += BarCommand("page.zoom", R.string.zoom_open, ::openZoom)
    windowDoes.showHistory?.let { show -> commands += BarCommand("history.show", R.string.history_open, show) }
    // The tab in front among the others: a place along the row, the next and the one before.
    if (front != null) {
        // A Favourite's tile is in a row wherever the tabs are.
        for (move in tabMoves(atTheSide = showsSidebar && !front.favourite, rightToLeft = windowDoes.rightToLeft)) {
            if (canMove(front, move.step)) {
                commands += BarCommand(if (move.step == RowStep.ROW_STEP_BACK) "tab.move_back" else "tab.move_forward", move.words) { moveTab(front.tab_id, move.step) }
            }
        }
    }
    if (tabs.size > 1) {
        commands += BarCommand("tab.next", R.string.tabs_next) { stepTab(StepTab.To.TO_NEXT) }
        commands += BarCommand("tab.previous", R.string.tabs_previous) { stepTab(StepTab.To.TO_PREVIOUS) }
    }
    return commands
}

/**
 * What a question carries for the command bar: the commands that can be done now, by their
 * names and their words. They are kept until the next question, so that the one chosen is the
 * one that was offered. None while the window is not drawn: it has no words to send.
 */
internal fun BrowserModel.commandsToAsk(): List<OfferedCommand> {
    val words = windowDoes.words
    val commands = if (words == null) emptyList() else barCommands()
    commandsAsked = commands
    return commands.map { OfferedCommand(name = it.name, words = words?.invoke(it.words).orEmpty()) }
}

/**
 * Does what a row of the command bar says, when it is a Space or a command. False for any
 * other row: a place, or a tab that is open.
 */
internal fun BrowserModel.chooseOfTheCommandBar(suggestion: AddressSuggestion): Boolean {
    if (suggestion.space_id != 0L) {
        isEditingAddress = false
        showSpace(suggestion.space_id)
        return true
    }
    if (suggestion.command.isEmpty()) return false
    isEditingAddress = false
    // The command as it was when it was offered. One the window did not send is done by nobody.
    commandsAsked.firstOrNull { it.name == suggestion.command }?.action?.invoke()
    return true
}

/** What a row says it is at its end, when it is no place: a tab that is open, a Space, a command. */
@StringRes
internal fun kindOf(suggestion: AddressSuggestion): Int? =
    when {
        suggestion.tab_id != 0L -> R.string.suggestions_switch_tab
        suggestion.space_id != 0L -> R.string.suggestions_space
        suggestion.command.isNotEmpty() -> R.string.suggestions_command
        else -> null
    }

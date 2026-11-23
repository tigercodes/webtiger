// Keys on Android with a keyboard (036): which key asks for what, and the asking. Nothing is
// decided here but that: each key calls what a button of the window calls, and which tab is the
// next one is the core's to say (035).
package com.tigercodes.webtiger.ui

import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEvent
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.isAltPressed
import androidx.compose.ui.input.key.isCtrlPressed
import androidx.compose.ui.input.key.isMetaPressed
import androidx.compose.ui.input.key.isShiftPressed
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.type
import com.tigercodes.webtiger.core.v1.NavigationCommand
import com.tigercodes.webtiger.core.v1.StepTab
import com.tigercodes.webtiger.core.v1.ZoomChange

/** What a key asks of the window. */
internal sealed interface KeyCommand {
    data object NewTab : KeyCommand

    data object CloseTab : KeyCommand

    data object ReopenClosedTab : KeyCommand

    /** Keep the tab in front as a Site, or let a Site go (039). */
    data object KeepOrLetGo : KeyCommand

    /** Send the Site in front back to its home (039). */
    data object SiteHome : KeyCommand

    data object NextTab : KeyCommand

    data object PreviousTab : KeyCommand

    data object LastTab : KeyCommand

    /** The tab at a place in the row, counted from one. */
    data class TabAt(val place: Int) : KeyCommand

    data object OpenLocation : KeyCommand

    data object Reload : KeyCommand

    data object Back : KeyCommand

    data object Forward : KeyCommand

    data object Find : KeyCommand

    data object History : KeyCommand

    data object ZoomIn : KeyCommand

    data object ZoomOut : KeyCommand

    data object ZoomReset : KeyCommand

    /** The tabs down the side, or back along the top (043). */
    data object SwitchLayout : KeyCommand
}

/** The keys that are the tab at a place: Ctrl+1 is the first. */
private val PLACES = listOf(Key.One, Key.Two, Key.Three, Key.Four, Key.Five, Key.Six, Key.Seven, Key.Eight)

/**
 * What a key asks for, with the modifiers held with it; null when it is not one of the
 * window's keys. As Chrome on Android and a desktop browser on a PC keyboard have them (036 §3).
 */
internal fun keyCommand(key: Key, ctrl: Boolean, shift: Boolean, alt: Boolean, meta: Boolean): KeyCommand? =
    when {
        meta -> null
        alt && !ctrl && !shift ->
            when (key) {
                Key.DirectionLeft -> KeyCommand.Back
                Key.DirectionRight -> KeyCommand.Forward
                // Alt+Home is a browser's key for its home page: here, the Site's own (039).
                Key.MoveHome -> KeyCommand.SiteHome
                else -> null
            }
        !ctrl || alt -> null
        shift ->
            when (key) {
                Key.T -> KeyCommand.ReopenClosedTab
                Key.Tab -> KeyCommand.PreviousTab
                // The tabs at the side, or at the top (043). With Shift and not with Alt: on
                // many keyboards Ctrl and Alt together are the key that types a third letter.
                Key.S -> KeyCommand.SwitchLayout
                // Plus is Shift and the key beside Backspace, on most keyboards.
                Key.Equals, Key.Plus -> KeyCommand.ZoomIn
                else -> null
            }
        else ->
            when (key) {
                Key.T -> KeyCommand.NewTab
                Key.W -> KeyCommand.CloseTab
                Key.D -> KeyCommand.KeepOrLetGo
                Key.Tab -> KeyCommand.NextTab
                Key.Nine -> KeyCommand.LastTab
                Key.L -> KeyCommand.OpenLocation
                Key.R -> KeyCommand.Reload
                Key.F -> KeyCommand.Find
                Key.H -> KeyCommand.History
                Key.Equals, Key.Plus, Key.NumPadAdd -> KeyCommand.ZoomIn
                Key.Minus, Key.NumPadSubtract -> KeyCommand.ZoomOut
                Key.Zero -> KeyCommand.ZoomReset
                else -> PLACES.indexOf(key).takeIf { it >= 0 }?.let { KeyCommand.TabAt(it + 1) }
            }
    }

/**
 * The window's answer to the keys of a keyboard, for the activity to ask before anything in
 * the window has a key (036): a key does the same with the keyboard in the page, in a field,
 * on a button, or nowhere at all.
 */
internal class WindowKeys {
    /**
     * What the window on the screen does for a key's command; false when it does nothing for
     * keys just now. None while no browser window is drawn.
     */
    var answer: ((KeyCommand) -> Boolean)? = null

    /** True when the key is one of the window's and the window answered it: nothing else is to have it then. */
    fun handle(event: android.view.KeyEvent): Boolean {
        val answer = answer ?: return false
        val pressed = KeyEvent(event)
        if (pressed.type != KeyEventType.KeyDown) return false
        val command =
            keyCommand(pressed.key, ctrl = pressed.isCtrlPressed, shift = pressed.isShiftPressed, alt = pressed.isAltPressed, meta = pressed.isMetaPressed)
                ?: return false
        // A key held down asks once: the keys it sends while it is held are nobody's.
        if (event.repeatCount > 0) return true
        return answer(command)
    }
}

/** The keys' way to the window that is drawn: the activity's. None in a window drawn without one. */
internal val LocalWindowKeys = staticCompositionLocalOf<WindowKeys?> { null }

/**
 * Does what a key asked for. What only the window can do (put the keyboard in the address
 * field, open the lists of what was visited) is the window's to give.
 */
internal fun BrowserModel.perform(command: KeyCommand, openLocation: () -> Unit, showHistory: () -> Unit) {
    when (command) {
        KeyCommand.NewTab -> openTab()
        KeyCommand.CloseTab -> closeTab(tab.tab_id)
        KeyCommand.ReopenClosedTab -> reopenClosedTab()
        KeyCommand.KeepOrLetGo -> keepOrLetGoTheTabInFront()
        KeyCommand.SiteHome -> sendHomeTheTabInFront()
        KeyCommand.NextTab -> stepTab(StepTab.To.TO_NEXT)
        KeyCommand.PreviousTab -> stepTab(StepTab.To.TO_PREVIOUS)
        KeyCommand.LastTab -> stepTab(StepTab.To.TO_LAST)
        is KeyCommand.TabAt -> stepTab(StepTab.To.TO_PLACE, command.place)
        KeyCommand.OpenLocation -> openLocation()
        KeyCommand.Reload -> perform(NavigationCommand.NAVIGATION_COMMAND_RELOAD)
        KeyCommand.Back -> perform(NavigationCommand.NAVIGATION_COMMAND_BACK)
        KeyCommand.Forward -> perform(NavigationCommand.NAVIGATION_COMMAND_FORWARD)
        KeyCommand.Find -> openFind()
        KeyCommand.History -> showHistory()
        KeyCommand.ZoomIn -> zoom(ZoomChange.ZOOM_CHANGE_IN)
        KeyCommand.ZoomOut -> zoom(ZoomChange.ZOOM_CHANGE_OUT)
        KeyCommand.ZoomReset -> zoom(ZoomChange.ZOOM_CHANGE_RESET)
        KeyCommand.SwitchLayout -> switchLayout()
    }
}

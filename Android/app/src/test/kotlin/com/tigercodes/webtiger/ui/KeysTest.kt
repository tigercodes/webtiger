// Keys on Android with a keyboard (036): the table of which key asks for what, as a function;
// and the window answering them, drawn by Compose on Robolectric's Android over a scripted core.
package com.tigercodes.webtiger.ui

import android.view.KeyEvent as NativeKey
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsFocused
import androidx.compose.ui.test.assertIsNotFocused
import androidx.compose.ui.test.junit4.v2.createComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.NavigationCommand
import com.tigercodes.webtiger.core.v1.StepTab
import com.tigercodes.webtiger.core.v1.TabView
import com.tigercodes.webtiger.core.v1.WindowSnapshot
import com.tigercodes.webtiger.core.v1.ZoomChange
import com.tigercodes.webtiger.core.v1.ZoomView
import com.tigercodes.webtiger.design.WebTigerTheme
import com.tigercodes.webtiger.webview.FakePages
import com.tigercodes.webtiger.webview.SystemWebViewEngine
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancel
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
class KeysTest {
    @get:Rule val compose = createComposeRule()

    private val core = ScriptedCore(::windowScript)
    private val scope = CoroutineScope(Dispatchers.Unconfined)
    private val session = BrowserSession(core, SystemWebViewEngine(FakePages()), scope)
    private val model = BrowserModel(session)

    @After
    fun end() {
        scope.cancel()
    }

    private fun withCtrl(key: Key, shift: Boolean = false) = keyCommand(key, ctrl = true, shift = shift, alt = false, meta = false)

    @Test
    fun eachKeyAsksForWhatABrowsersKeyAsksFor() {
        assertEquals(KeyCommand.NewTab, withCtrl(Key.T))
        assertEquals(KeyCommand.CloseTab, withCtrl(Key.W))
        assertEquals(KeyCommand.ReopenClosedTab, withCtrl(Key.T, shift = true))
        assertEquals(KeyCommand.KeepOrLetGo, withCtrl(Key.D))
        // Ctrl+Shift+S: the tabs down the side, or back along the top (043). Not with Alt, which
        // with Ctrl types a third letter on many keyboards; and plain Ctrl+S is nobody's here.
        assertEquals(KeyCommand.SwitchLayout, withCtrl(Key.S, shift = true))
        assertNull(withCtrl(Key.S))
        assertNull(keyCommand(Key.S, ctrl = true, shift = false, alt = true, meta = false))
        assertNull(keyCommand(Key.S, ctrl = true, shift = true, alt = true, meta = false))
        // Alt+Home: the Site in front goes back to its home (039).
        assertEquals(KeyCommand.SiteHome, keyCommand(Key.MoveHome, ctrl = false, shift = false, alt = true, meta = false))
        assertNull(keyCommand(Key.MoveHome, ctrl = false, shift = false, alt = false, meta = false))
        assertNull(keyCommand(Key.MoveHome, ctrl = true, shift = false, alt = false, meta = false))
        assertNull(keyCommand(Key.MoveHome, ctrl = false, shift = true, alt = true, meta = false))
        assertEquals(KeyCommand.NextTab, withCtrl(Key.Tab))
        assertEquals(KeyCommand.PreviousTab, withCtrl(Key.Tab, shift = true))
        assertEquals(KeyCommand.OpenLocation, withCtrl(Key.L))
        assertEquals(KeyCommand.Reload, withCtrl(Key.R))
        assertEquals(KeyCommand.Find, withCtrl(Key.F))
        assertEquals(KeyCommand.History, withCtrl(Key.H))
        // Zoom: Plus is Shift and Equals on most keyboards, and a key of its own on some.
        assertEquals(KeyCommand.ZoomIn, withCtrl(Key.Equals))
        assertEquals(KeyCommand.ZoomIn, withCtrl(Key.Equals, shift = true))
        assertEquals(KeyCommand.ZoomIn, withCtrl(Key.Plus))
        assertEquals(KeyCommand.ZoomIn, withCtrl(Key.NumPadAdd))
        assertEquals(KeyCommand.ZoomOut, withCtrl(Key.Minus))
        assertEquals(KeyCommand.ZoomOut, withCtrl(Key.NumPadSubtract))
        assertEquals(KeyCommand.ZoomReset, withCtrl(Key.Zero))
        // The tab at a place, counted from one; and Ctrl+9 is the last, not the ninth.
        assertEquals(
            (1..8).map { KeyCommand.TabAt(it) },
            listOf(Key.One, Key.Two, Key.Three, Key.Four, Key.Five, Key.Six, Key.Seven, Key.Eight).map { withCtrl(it) },
        )
        assertEquals(KeyCommand.LastTab, withCtrl(Key.Nine))
        // Back and forward are Alt and an arrow.
        assertEquals(KeyCommand.Back, keyCommand(Key.DirectionLeft, ctrl = false, shift = false, alt = true, meta = false))
        assertEquals(KeyCommand.Forward, keyCommand(Key.DirectionRight, ctrl = false, shift = false, alt = true, meta = false))
    }

    @Test
    fun aKeyThatIsNotTheWindowsIsLeftAlone() {
        // The letters by themselves are typing, and with Shift.
        for (key in listOf(Key.T, Key.W, Key.D, Key.L, Key.R, Key.F, Key.H, Key.Tab, Key.One, Key.Nine, Key.Zero, Key.DirectionLeft)) {
            assertNull("$key by itself", keyCommand(key, ctrl = false, shift = false, alt = false, meta = false))
            assertNull("$key with Shift", keyCommand(key, ctrl = false, shift = true, alt = false, meta = false))
            // With the system's own key held, the key is the system's.
            assertNull("$key with Meta", keyCommand(key, ctrl = true, shift = false, alt = false, meta = true))
            // Control and Alt together is how some keyboards type a third letter on a key.
            assertNull("$key with Control and Alt", keyCommand(key, ctrl = true, shift = false, alt = true, meta = false))
        }
        // Editing's keys stay the field's and the page's.
        for (key in listOf(Key.A, Key.C, Key.V, Key.X, Key.Z)) assertNull("$key", withCtrl(key))
        // Shift changes what a key asks for: it is not the same key with Shift.
        for (key in listOf(Key.W, Key.D, Key.L, Key.R, Key.F, Key.H, Key.One, Key.Nine)) assertNull("$key with Shift", withCtrl(key, shift = true))
        // An arrow with Control, or with Alt and Shift, is moving in text.
        assertNull(withCtrl(Key.DirectionLeft))
        assertNull(keyCommand(Key.DirectionLeft, ctrl = false, shift = true, alt = true, meta = false))
        assertNull(keyCommand(Key.T, ctrl = false, shift = false, alt = true, meta = false))
    }

    private val keys = WindowKeys()

    /** The window as the core says it is: a page in the tab in front, that can be zoomed, and a closed tab to bring back. */
    private fun show() {
        compose.setContent { WebTigerTheme { CompositionLocalProvider(LocalWindowKeys provides keys) { BrowserWindow(session, model) } } }
        compose.waitForIdle()
        compose.runOnIdle {
            core.emit(
                WindowSnapshot(
                    window_id = 1,
                    tab = TabView(tab_id = 2, web_view_id = 5, zoom = ZoomView(percent = 100, can_zoom_in = true, can_zoom_out = true)),
                    undo = "tab.closed",
                ),
                subscription = 3,
            )
        }
        compose.waitForIdle()
    }

    /** A key goes down, as the activity hears it from the system; true when the window answered it. */
    private fun press(code: Int, held: Int = 0, repeat: Int = 0, action: Int = NativeKey.ACTION_DOWN): Boolean {
        var answered = false
        compose.runOnIdle { answered = keys.handle(NativeKey(0L, 0L, action, code, repeat, held)) }
        compose.waitForIdle()
        return answered
    }

    /** What the core was asked, each request as a short word. */
    private fun asked(): List<String> =
        core.requests.mapNotNull { request ->
            request.open_tab?.let { "open tab in ${it.window_id}" }
                ?: request.undo?.let { "undo in ${it.window_id}" }
                ?: request.close_tab?.let { "close ${it.tab_id}" }
                ?: request.keep_tab?.let { "keep ${it.tab_id}" }
                ?: request.step_tab?.let { "step ${it.to} ${it.place} in ${it.window_id}" }
                ?: request.tab_command?.let { "tab ${it.tab_id} ${it.command}" }
                ?: request.zoom_page?.let { "zoom ${it.tab_id} ${it.change}" }
        }

    @Test
    fun theWindowAnswersTheKeysWithWhatItsButtonsAsk() {
        show()
        // Nothing in the window was given the keyboard's focus: the keys work all the same.
        val pressed =
            listOf(
                press(NativeKey.KEYCODE_T, CTRL),
                press(NativeKey.KEYCODE_T, CTRL or SHIFT),
                press(NativeKey.KEYCODE_W, CTRL),
                press(NativeKey.KEYCODE_D, CTRL),
                press(NativeKey.KEYCODE_TAB, CTRL),
                press(NativeKey.KEYCODE_TAB, CTRL or SHIFT),
                press(NativeKey.KEYCODE_3, CTRL),
                press(NativeKey.KEYCODE_9, CTRL),
                press(NativeKey.KEYCODE_R, CTRL),
                press(NativeKey.KEYCODE_DPAD_LEFT, ALT),
                press(NativeKey.KEYCODE_DPAD_RIGHT, ALT),
                press(NativeKey.KEYCODE_EQUALS, CTRL),
                press(NativeKey.KEYCODE_MINUS, CTRL),
                press(NativeKey.KEYCODE_0, CTRL),
            )
        assertEquals("each was the window's, and nothing else is to have it", List(14) { true }, pressed)
        assertEquals(
            listOf(
                "open tab in 1",
                "undo in 1",
                "close 2",
                "keep 2",
                "step ${StepTab.To.TO_NEXT} 0 in 1",
                "step ${StepTab.To.TO_PREVIOUS} 0 in 1",
                "step ${StepTab.To.TO_PLACE} 3 in 1",
                "step ${StepTab.To.TO_LAST} 0 in 1",
                "tab 2 ${NavigationCommand.NAVIGATION_COMMAND_RELOAD}",
                "tab 2 ${NavigationCommand.NAVIGATION_COMMAND_BACK}",
                "tab 2 ${NavigationCommand.NAVIGATION_COMMAND_FORWARD}",
                "zoom 2 ${ZoomChange.ZOOM_CHANGE_IN}",
                "zoom 2 ${ZoomChange.ZOOM_CHANGE_OUT}",
                "zoom 2 ${ZoomChange.ZOOM_CHANGE_RESET}",
            ),
            asked(),
        )
    }

    @Test
    fun aKeyThatIsNotTheWindowsIsLeftToWhateverHasTheKeyboard() {
        show()
        // Typing, editing's keys, a key with the system's own key held: not answered, so
        // the field or the page gets them.
        assertFalse(press(NativeKey.KEYCODE_T))
        assertFalse(press(NativeKey.KEYCODE_A, CTRL))
        assertFalse(press(NativeKey.KEYCODE_T, CTRL or NativeKey.META_META_ON))
        // A key coming up is nobody's command.
        assertFalse(press(NativeKey.KEYCODE_T, CTRL, action = NativeKey.ACTION_UP))
        assertEquals(emptyList<String>(), asked())

        // A key held down asks once: what it sends while held is taken, and asks nothing.
        assertTrue(press(NativeKey.KEYCODE_T, CTRL))
        assertTrue(press(NativeKey.KEYCODE_T, CTRL, repeat = 1))
        assertTrue(press(NativeKey.KEYCODE_T, CTRL, repeat = 2))
        assertEquals(listOf("open tab in 1"), asked())
    }

    @Test
    fun theKeysThatOnlyTheWindowCanAnswer() {
        show()
        // Ctrl+F: the find bar, as its button opens it.
        compose.onAllNodesWithTag("find-field").assertCountEquals(0)
        assertTrue(press(NativeKey.KEYCODE_F, CTRL))
        compose.onAllNodesWithTag("find-field").assertCountEquals(1)
        // Ctrl+L: the keyboard goes to the address field, from wherever it was.
        compose.onNodeWithTag("address-field").assertIsNotFocused()
        assertTrue(press(NativeKey.KEYCODE_L, CTRL))
        compose.onNodeWithTag("address-field").assertIsFocused()
        // Ctrl+H: the lists of what was visited. While they are over the window the window
        // answers no key: nothing is done under them, and the key is left to them.
        compose.onAllNodesWithTag("history-done").assertCountEquals(0)
        assertTrue(press(NativeKey.KEYCODE_H, CTRL))
        compose.onAllNodesWithTag("history-done").assertCountEquals(1)
        assertFalse(press(NativeKey.KEYCODE_T, CTRL))
        assertEquals(emptyList<String>(), asked())
    }

    @Test
    fun withNoBrowserWindowDrawnNoKeyIsAnswered() {
        // The activity asks; no window has said what it does for keys.
        assertFalse(press(NativeKey.KEYCODE_T, CTRL))
        // And a window that was drawn and is gone answers no more.
        show()
        assertTrue(press(NativeKey.KEYCODE_T, CTRL))
        compose.runOnIdle { keys.answer = null }
        assertFalse(press(NativeKey.KEYCODE_T, CTRL))
    }
}

private const val CTRL = NativeKey.META_CTRL_ON
private const val SHIFT = NativeKey.META_SHIFT_ON
private const val ALT = NativeKey.META_ALT_ON

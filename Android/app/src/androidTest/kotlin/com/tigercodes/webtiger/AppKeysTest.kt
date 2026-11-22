// Keys on Android with a keyboard (036), in the app itself on a device, in fake mode: the real
// core and the real WebView. The keys are sent as the system sends a keyboard's, to whatever
// has the keyboard's focus: the page, and then the address field. The window answers them
// before either.
package com.tigercodes.webtiger

import android.content.Context
import android.content.Intent
import android.os.SystemClock
import android.view.KeyEvent
import android.view.View
import android.view.ViewGroup
import android.webkit.WebView
import androidx.compose.ui.test.assertIsFocused
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performImeAction
import androidx.compose.ui.test.performTextReplacement
import androidx.test.core.app.ActivityScenario
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
internal class AppKeysTest {
    @get:Rule val compose = createEmptyComposeRule()

    private lateinit var scenario: ActivityScenario<MainActivity>
    private val context: Context get() = ApplicationProvider.getApplicationContext()
    private val site get() = AppWindowTest.site

    @Before
    fun launch() {
        InstrumentationRegistry.getInstrumentation().setInTouchMode(true)
        val intent =
            Intent(context, MainActivity::class.java)
                .putExtra(MainActivity.EXTRA_FAKE, true)
                // The process's one start address: the app's core starts once, and every window
                // of every test after this one opens on it.
                .putExtra(MainActivity.EXTRA_FAKE_START, site.url("/"))
        scenario = ActivityScenario.launch(intent)
    }

    @After
    fun close() {
        scenario.close()
    }

    private fun webViewIn(view: View): WebView? {
        if (view is WebView) return view
        if (view is ViewGroup) {
            for (index in 0 until view.childCount) webViewIn(view.getChildAt(index))?.let { return it }
        }
        return null
    }

    /** The title of the page the window shows; empty when it shows none. */
    private fun pageTitle(): String {
        var title = ""
        scenario.onActivity { activity -> title = webViewIn(activity.window.decorView)?.title.orEmpty() }
        return title
    }

    private fun awaitPage(title: String) {
        try {
            compose.waitUntil(15_000) { pageTitle() == title }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            throw AssertionError("the page <$title> never showed; the window shows <${pageTitle()}>")
        }
    }

    private fun count(tag: String): Int = compose.onAllNodesWithTag(tag).fetchSemanticsNodes().size

    private fun awaitCount(tag: String, count: Int) {
        try {
            compose.waitUntil(15_000) { count(tag) == count }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            throw AssertionError("there were never $count of <$tag>; there are ${count(tag)}")
        }
    }

    /** A key of a keyboard goes down and comes up, with others held, as the system sends it to the app. */
    private fun press(code: Int, held: Int = 0) {
        val instrumentation = InstrumentationRegistry.getInstrumentation()
        val now = SystemClock.uptimeMillis()
        instrumentation.sendKeySync(KeyEvent(now, now, KeyEvent.ACTION_DOWN, code, 0, held))
        instrumentation.sendKeySync(KeyEvent(now, now, KeyEvent.ACTION_UP, code, 0, held))
        instrumentation.waitForIdleSync()
    }

    @Test
    fun theKeysOfAKeyboardWorkWhereverTheKeyboardIs() {
        awaitPage("Home")
        compose.onNodeWithTag("address-field").performClick()
        compose.onNodeWithTag("address-field").performTextReplacement(site.url("/landed"))
        compose.onNodeWithTag("address-field").performImeAction()
        awaitPage("Landed")

        // With the keyboard in the page: back and forward.
        press(KeyEvent.KEYCODE_DPAD_LEFT, ALT)
        awaitPage("Home")
        press(KeyEvent.KEYCODE_DPAD_RIGHT, ALT)
        awaitPage("Landed")

        // A new tab, empty and in front; then from tab to tab, as the core says.
        press(KeyEvent.KEYCODE_T, CTRL)
        awaitCount("tab", 2)
        awaitCount("empty-tab", 1)
        press(KeyEvent.KEYCODE_TAB, CTRL or SHIFT)
        awaitPage("Landed")
        compose.onAllNodesWithTag("tab")[0].assertIsSelected()
        press(KeyEvent.KEYCODE_TAB, CTRL)
        awaitCount("empty-tab", 1)
        compose.onAllNodesWithTag("tab")[1].assertIsSelected()
        press(KeyEvent.KEYCODE_1, CTRL)
        awaitPage("Landed")
        press(KeyEvent.KEYCODE_9, CTRL)
        awaitCount("empty-tab", 1)
        press(KeyEvent.KEYCODE_1, CTRL)
        awaitPage("Landed")
        compose.onAllNodesWithTag("tab")[0].assertIsSelected()

        // The keyboard goes to the address field; and with the keyboard there, the tab in
        // front is closed, and brought back.
        press(KeyEvent.KEYCODE_L, CTRL)
        compose.waitForIdle()
        compose.onNodeWithTag("address-field").assertIsFocused()
        press(KeyEvent.KEYCODE_W, CTRL)
        awaitCount("tab", 1)
        awaitCount("empty-tab", 1)
        press(KeyEvent.KEYCODE_T, CTRL or SHIFT)
        awaitCount("tab", 2)
        awaitPage("Landed")
        compose.onAllNodesWithTag("tab")[0].assertIsSelected()
    }

    private companion object {
        const val CTRL = KeyEvent.META_CTRL_ON or KeyEvent.META_CTRL_LEFT_ON
        const val SHIFT = KeyEvent.META_SHIFT_ON or KeyEvent.META_SHIFT_LEFT_ON
        const val ALT = KeyEvent.META_ALT_ON or KeyEvent.META_ALT_LEFT_ON
    }
}

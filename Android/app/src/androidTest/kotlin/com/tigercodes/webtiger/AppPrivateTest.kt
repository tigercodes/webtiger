// A private window in the app itself on a device (037 §5.3), in fake mode: the real core and
// the real WebView. One is opened from the list of Spaces, signs in to the test site, is left
// and come back to, and closed: what it was signed in as is its own while it is open, is not
// the other window's, is in no history, and is gone with it.
package com.tigercodes.webtiger

import android.content.Context
import android.content.Intent
import android.view.View
import android.view.ViewGroup
import android.view.inputmethod.EditorInfo
import android.webkit.WebView
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performImeAction
import androidx.compose.ui.test.performTextReplacement
import androidx.test.core.app.ActivityScenario
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
internal class AppPrivateTest {
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
                // of every test after this one opens on it. A private window opens on it too.
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

    /** The title of the page the window on the screen shows; empty when it shows none. */
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

    private fun count(tag: String): Int = compose.onAllNodesWithTag(tag, useUnmergedTree = true).fetchSemanticsNodes().size

    private fun awaitCount(tag: String, count: Int) {
        try {
            compose.waitUntil(15_000) { count(tag) == count }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            throw AssertionError("there were never $count of <$tag>; there are ${count(tag)}")
        }
    }

    /** Sends the tab in front to a page of the test site, and waits for it by its title. */
    private fun go(path: String, title: String) {
        compose.onNodeWithTag("address-field").performClick()
        compose.onNodeWithTag("address-field").performTextReplacement(site.url(path))
        compose.onNodeWithTag("address-field").performImeAction()
        awaitPage(title)
    }

    /** What the chip says to a screen reader: the Space the window shows, or that it is private. */
    private fun chip(): String =
        compose
            .onNodeWithTag("space-chip")
            .fetchSemanticsNode()
            .config
            .getOrNull(SemanticsProperties.ContentDescription)
            .orEmpty()
            .joinToString()

    private fun awaitChip(says: String) {
        try {
            compose.waitUntil(15_000) { chip() == says }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            throw AssertionError("the chip never said <$says>; it says <${chip()}>")
        }
    }

    /** From an ordinary window: the list of Spaces, and its last row. */
    private fun openPrivate() {
        compose.onNodeWithTag("space-chip").performClick()
        awaitCount("space-private", 1)
        compose.onNodeWithTag("space-private").performClick()
        awaitChip("Private Window")
    }

    /** From a private window: the page behind its chip, and one of its buttons. */
    private fun fromPrivate(button: String) {
        compose.onNodeWithTag("space-chip").performClick()
        awaitCount("private-page", 1)
        compose.onNodeWithTag(button).performClick()
        awaitChip("Space: Home")
    }

    /**
     * What the keyboard is told of the field being typed in, as the system asks the view that
     * has the keyboard's focus: its options. Waited for: a field is the keyboard's a moment
     * after it is touched.
     */
    private fun toldTheKeyboard(what: String): Int {
        var told: Int? = null
        compose.waitUntil(15_000) {
            scenario.onActivity { activity ->
                val info = EditorInfo()
                told = activity.window.decorView.findFocus()?.onCreateInputConnection(info)?.let { info.imeOptions }
            }
            told != null
        }
        return checkNotNull(told) { "$what: no field for the keyboard" }
    }

    private fun asksNotToLearn(options: Int): Boolean = options and EditorInfo.IME_FLAG_NO_PERSONALIZED_LEARNING != 0

    /** Opens the test site's form in the window on the screen and puts the cursor in its first field with a finger. */
    private fun touchThePagesField() {
        go("/sign-in-form", "Sign in")
        var page: WebView? = null
        scenario.onActivity { activity -> page = webViewIn(activity.window.decorView) }
        PageTouch.tap(checkNotNull(page) { "no page is shown" }, "user")
    }

    /**
     * What is typed in a private window is not for the keyboard to learn from (037 §5.5): in
     * its address field, which the window draws, and in a page's field, which the web view
     * tells the keyboard about. In the other window the keyboard is asked nothing of the kind.
     * The app, with the real core: it is the core that says which login set is a private one's.
     */
    @Test
    fun aPrivateWindowAsksTheKeyboardNotToLearnAndTheOtherWindowDoesNot() {
        awaitPage("Home")
        compose.onNodeWithTag("address-field").performClick()
        assertFalse("the ordinary window's address field", asksNotToLearn(toldTheKeyboard("the ordinary window's address field")))
        touchThePagesField()
        assertFalse("a field of the ordinary window's page", asksNotToLearn(toldTheKeyboard("a field of the ordinary window's page")))

        openPrivate()
        awaitPage("Home")
        compose.onNodeWithTag("address-field").performClick()
        assertTrue("the private window's address field", asksNotToLearn(toldTheKeyboard("the private window's address field")))
        touchThePagesField()
        assertTrue("a field of the private window's page", asksNotToLearn(toldTheKeyboard("a field of the private window's page")))

        // And back in the other window, as before.
        fromPrivate("private-close")
        awaitPage("Sign in")
        compose.onNodeWithTag("address-field").performClick()
        assertFalse("the ordinary window's address field, afterwards", asksNotToLearn(toldTheKeyboard("the ordinary window's address field, afterwards")))
    }

    @Test
    fun aPrivateWindowsLoginsAreItsOwnAndGoWithIt() {
        awaitPage("Home")
        assertEquals("Space: Home", chip())

        // A private window, from the list of Spaces. It opens where a new window opens, and
        // signs in to the site.
        openPrivate()
        awaitPage("Home")
        go("/set?v=ada", "Set")
        go("/whoami", "who=ada")

        // Left open, and the other window shown: it is not signed in, and is where it was.
        fromPrivate("private-leave")
        awaitPage("Home")
        go("/whoami", "who=none")

        // The list of Spaces says a private window is open; its row shows it, as it was left.
        compose.onNodeWithTag("space-chip").performClick()
        awaitCount("space-private-open", 1)
        compose.onNodeWithTag("space-private").performClick()
        awaitChip("Private Window")
        awaitPage("who=ada")

        // Closed: the other window is shown, and no private window is open any more.
        fromPrivate("private-close")
        awaitPage("who=none")
        compose.onNodeWithTag("space-chip").performClick()
        awaitCount("space-private", 1)
        assertEquals("the row no longer says one is open", 0, count("space-private-open"))

        // Another private window knows nothing of the first.
        compose.onNodeWithTag("space-private").performClick()
        awaitChip("Private Window")
        awaitPage("Home")
        go("/whoami", "who=none")
        fromPrivate("private-close")

        // And nothing a private window visited is in the history the other window lists:
        // not the page that signed in, nor the one that said who was signed in.
        compose.onNodeWithTag("history").performClick()
        awaitCount("history-done", 1)
        compose.waitUntil(15_000) { compose.onAllNodesWithText("who=none").fetchSemanticsNodes().isNotEmpty() }
        assertEquals(0, compose.onAllNodesWithText("Set").fetchSemanticsNodes().size)
        assertEquals(0, compose.onAllNodesWithText("who=ada").fetchSemanticsNodes().size)
    }
}

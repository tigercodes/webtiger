// Undo (033) in the app itself on a device, in fake mode: the real core and the real WebView. A
// page's tab is closed; the button for a new tab, held, offers it back; chosen, the tab is where
// it was, in front, with its page.
//
// The app's core starts once in the tests' process, and keeps what every test before this one
// closed: that there is nothing to bring back when nothing was closed is the JVM's to show
// (UndoTest), not this test's.
package com.tigercodes.webtiger

import android.content.Context
import android.content.Intent
import android.view.View
import android.view.ViewGroup
import android.webkit.WebView
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.compose.ui.test.longClick
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performImeAction
import androidx.compose.ui.test.performTextReplacement
import androidx.compose.ui.test.performTouchInput
import androidx.test.core.app.ActivityScenario
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
internal class AppUndoTest {
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

    /** What the address field holds. These are a test site's addresses on the loopback address. */
    private fun field(): String =
        compose
            .onNodeWithTag("address-field")
            .fetchSemanticsNode()
            .config
            .getOrNull(SemanticsProperties.EditableText)
            ?.text
            .orEmpty()

    @Test
    fun theButtonForANewTabHeldBringsAClosedTabBack() {
        awaitPage("Home")
        // The window's tab goes to a page of this test's own; a second tab, empty, beside it.
        compose.onNodeWithTag("address-field").performClick()
        compose.onNodeWithTag("address-field").performTextReplacement(site.url("/landed"))
        compose.onNodeWithTag("address-field").performImeAction()
        awaitPage("Landed")
        compose.onNodeWithTag("new-tab").performClick()
        awaitCount("tab", 2)
        awaitCount("empty-tab", 1)

        // The page's tab is brought in front and closed: the empty one is what is left.
        compose.onAllNodesWithTag("tab")[0].performClick()
        awaitPage("Landed")
        compose.onAllNodesWithTag("close-tab")[0].performClick()
        awaitCount("tab", 1)
        awaitCount("empty-tab", 1)

        // The button for a new tab, held: the menu, and no new tab.
        compose.onNodeWithTag("new-tab").performTouchInput { longClick() }
        awaitCount("reopen-closed-tab", 1)
        compose.onAllNodesWithTag("tab").assertCountEquals(1)
        compose.onNodeWithTag("reopen-closed-tab").performClick()

        // Back where it was, the first of the two, in front, with its page and its address.
        awaitCount("tab", 2)
        awaitPage("Landed")
        compose.onAllNodesWithTag("tab")[0].assertIsSelected()
        compose.waitUntil(15_000) { field() == "http://127.0.0.1:${site.port}/landed" }
        assertEquals(0, count("reopen-closed-tab"))
    }
}

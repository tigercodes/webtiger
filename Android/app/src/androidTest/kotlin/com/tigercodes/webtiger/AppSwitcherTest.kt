// The phone's switcher (047) in the app, on a device, in fake mode: the real core and the real
// WebView. A button in the toolbar says how many tabs the row has, and opens the whole row over
// the window; a tab chosen there comes in front, and the switcher goes; the system's Back and
// its own button put it away with nothing changed.
package com.tigercodes.webtiger

import android.content.Context
import android.content.Intent
import android.view.View
import android.view.ViewGroup
import android.webkit.WebView
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.hasAnyAncestor
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.compose.ui.test.longClick
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performImeAction
import androidx.compose.ui.test.performScrollTo
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
internal class AppSwitcherTest {
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

    /** Sends the tab in front to a page of the test site, and waits for it by its title. */
    private fun go(path: String, title: String) {
        compose.onNodeWithTag("address-field").performClick()
        compose.onNodeWithTag("address-field").performTextReplacement(site.url(path))
        compose.onNodeWithTag("address-field").performImeAction()
        awaitPage(title)
    }

    /** What each tab of the strip says of itself beside its name, and its name: a tile's is what it says to a screen reader. */
    private fun tabs(): List<String> =
        compose.onAllNodesWithTag("tab").fetchSemanticsNodes().map { node ->
            val spoken = node.config.getOrNull(SemanticsProperties.ContentDescription).orEmpty().joinToString(" ")
            val name = spoken.ifEmpty { node.config.getOrNull(SemanticsProperties.Text).orEmpty().joinToString(" ") { it.text } }
            val said = node.config.getOrNull(SemanticsProperties.StateDescription).orEmpty()
            if (said.isEmpty()) name else "$name ($said)"
        }

    private fun awaitTabs(vararg expected: String) {
        try {
            compose.waitUntil(15_000) { tabs() == expected.toList() }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            throw AssertionError("the strip never was ${expected.toList()}; it is ${tabs()}")
        }
    }

    /** What the switcher's button says to a screen reader. */
    private fun button(): String =
        compose.onAllNodesWithTag("switcher").fetchSemanticsNodes().firstOrNull()?.config?.getOrNull(SemanticsProperties.ContentDescription).orEmpty().joinToString()

    private fun awaitButton(says: String) {
        try {
            compose.waitUntil(15_000) { button() == says }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            throw AssertionError("the switcher's button never said <$says>; it says <${button()}>")
        }
    }

    /** A thing of the switcher, by its name: the strip under it has things of the same names. */
    private fun listed(tag: String): SemanticsMatcher = hasTestTag(tag) and hasAnyAncestor(hasTestTag("switcher-sheet"))

    private fun isOpen(): Boolean = compose.onAllNodesWithTag("switcher-sheet").fetchSemanticsNodes().isNotEmpty()

    private fun open() {
        compose.onNodeWithTag("switcher").performClick()
        compose.waitUntil(15_000) { isOpen() }
    }

    private fun awaitShut() {
        try {
            compose.waitUntil(15_000) { !isOpen() }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            throw AssertionError("the switcher was never put away")
        }
    }

    @Test
    fun theWholeRowIsInTheSwitcherAndATabChosenThereComesInFront() {
        awaitPage("Home")
        awaitTabs("Home")
        awaitButton("Tabs: 1")
        // A second tab, somewhere else, in front: the button says two.
        compose.onNodeWithTag("new-tab").performScrollTo().performClick()
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("empty-tab").fetchSemanticsNodes().isNotEmpty() }
        go("/landed", "Landed")
        awaitTabs("Home", "Landed")
        awaitButton("Tabs: 2")

        // The switcher: both tabs are rows of it. The first, pressed, comes in front, and the
        // switcher goes.
        open()
        compose.waitUntil(15_000) { compose.onAllNodes(listed("tab")).fetchSemanticsNodes().size == 2 }
        compose.onAllNodes(listed("tab"))[0].performClick()
        awaitShut()
        awaitPage("Home")

        // The system's Back puts it away, and leaves the page and its tab as they were.
        open()
        scenario.onActivity { it.onBackPressedDispatcher.onBackPressed() }
        awaitShut()
        assertEquals("Home", pageTitle())
        awaitTabs("Home", "Landed")
        // And so does its own button.
        open()
        compose.onNodeWithTag("switcher-done").performClick()
        awaitShut()
        assertEquals("Home", pageTitle())

        // A tab closed from its row in the switcher: the switcher stays, and the button says
        // one fewer when it is put away.
        open()
        compose.onAllNodes(listed("close-tab"))[1].performClick()
        compose.waitUntil(15_000) { compose.onAllNodes(listed("tab")).fetchSemanticsNodes().size == 1 }
        assertEquals("the switcher stays while the row is put in order", true, isOpen())
        compose.onNodeWithTag("switcher-done").performClick()
        awaitShut()
        awaitButton("Tabs: 1")
        awaitTabs("Home")
    }
}

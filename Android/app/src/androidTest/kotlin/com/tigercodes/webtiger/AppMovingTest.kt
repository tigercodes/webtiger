// Moving a tab in its row (045) in the app, on a device, in fake mode: the real core and the
// real WebView. A tab changes places with the one beside it from its own menu; the command it
// cannot go by is grey; and the tab in front stays in front, with its page.
package com.tigercodes.webtiger

import android.content.Context
import android.content.Intent
import android.view.View
import android.view.ViewGroup
import android.webkit.WebView
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.assertTextEquals
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
internal class AppMovingTest {
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

    /** Holds a tab of the strip, and waits for its menu to have the commands that move it. */
    private fun hold(tab: Int) {
        compose.onAllNodesWithTag("tab")[tab].performScrollTo().performTouchInput { longClick() }
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("move-back").fetchSemanticsNodes().isNotEmpty() }
    }

    @Test
    fun aTabChangesPlacesWithTheOneBesideItFromItsOwnMenu() {
        awaitPage("Home")
        awaitTabs("Home")
        // A second tab, somewhere else, in front.
        compose.onNodeWithTag("new-tab").performScrollTo().performClick()
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("empty-tab").fetchSemanticsNodes().isNotEmpty() }
        go("/whoami", "who=none")
        awaitTabs("Home", "who=none")

        // The last tab can go left, and not right: the command it cannot go by is grey. Moved,
        // it has changed places with the one before it, and is in front still, with its page.
        hold(1)
        compose.onNodeWithTag("move-back").assertTextEquals("Move Tab Left")
        compose.onNodeWithTag("move-back").assertIsEnabled()
        compose.onNodeWithTag("move-forward").assertTextEquals("Move Tab Right")
        compose.onNodeWithTag("move-forward").assertIsNotEnabled()
        compose.onNodeWithTag("move-back").performClick()
        awaitTabs("who=none", "Home")
        assertEquals("it is still in front", "who=none", pageTitle())

        // Now it is the first: it can go right, and not left. Moved, the row is as it was.
        hold(0)
        compose.onNodeWithTag("move-back").assertIsNotEnabled()
        compose.onNodeWithTag("move-forward").assertIsEnabled()
        compose.onNodeWithTag("move-forward").performClick()
        awaitTabs("Home", "who=none")
        assertEquals("who=none", pageTitle())

        // A tab that is not in front moves as well, and stays behind: the one in front keeps
        // its page.
        hold(0)
        compose.onNodeWithTag("move-forward").performClick()
        awaitTabs("who=none", "Home")
        assertEquals("who=none", pageTitle())
    }
}

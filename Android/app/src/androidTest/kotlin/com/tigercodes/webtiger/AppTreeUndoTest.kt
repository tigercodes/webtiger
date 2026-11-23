// Undo for what is done to the tree (044) in the app, on a device, in fake mode: the real core
// and the real WebView. What was last done to the row is undone from a tab's own menu, where
// the command is at the top and says what it undoes; and with nothing to undo it is not there.
package com.tigercodes.webtiger

import android.content.Context
import android.content.Intent
import android.view.View
import android.view.ViewGroup
import android.webkit.WebView
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
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
internal class AppTreeUndoTest {
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

    /** Holds a tab of the strip, and presses the item of its menu that has this tag. */
    private fun fromTheMenuOf(tab: Int, item: String) {
        compose.onAllNodesWithTag("tab")[tab].performScrollTo().performTouchInput { longClick() }
        compose.waitUntil(15_000) { compose.onAllNodesWithTag(item).fetchSemanticsNodes().isNotEmpty() }
        compose.onNodeWithTag(item).performClick()
    }

    /** Holds a tab of the strip, and waits for its menu to have the item with this tag. */
    private fun hold(tab: Int, item: String) {
        compose.onAllNodesWithTag("tab")[tab].performScrollTo().performTouchInput { longClick() }
        compose.waitUntil(15_000) { compose.onAllNodesWithTag(item).fetchSemanticsNodes().isNotEmpty() }
    }

    @Test
    fun whatWasLastDoneToTheRowIsUndoneFromATabsOwnMenu() {
        awaitPage("Home")
        awaitTabs("Home")
        // Kept from its own menu: a Site. The menu then has the command at its top, saying
        // what it would undo; chosen, the tab is a tab like any other again.
        fromTheMenuOf(0, "keep-or-let-go")
        awaitTabs("Home (Site)")
        hold(0, "undo-arrangement")
        compose.onNodeWithTag("undo-arrangement").assertTextEquals("Undo Keep as Site")
        compose.onNodeWithTag("undo-arrangement").performClick()
        awaitTabs("Home")

        // Made a Favourite, which keeps it too: one thing done, and one undo takes it all
        // the way back.
        fromTheMenuOf(0, "favourite")
        awaitTabs("Home (Favourite)")
        hold(0, "undo-arrangement")
        compose.onNodeWithTag("undo-arrangement").assertTextEquals("Undo Add to Favourites")
        compose.onNodeWithTag("undo-arrangement").performClick()
        awaitTabs("Home")
        assertEquals("the page is as it was", "Home", pageTitle())

        // With nothing left to undo, the menu has no such command.
        hold(0, "keep-or-let-go")
        assertEquals(0, compose.onAllNodesWithTag("undo-arrangement").fetchSemanticsNodes().size)
    }
}

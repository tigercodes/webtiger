// Favourites (042) in the app, on a device, in fake mode: the real core and the real WebView. A
// tab is made a Favourite from its own menu and is a tile, first in the row, that says its
// whole name; put to sleep from its menu it stays; pressed it wakes; and it stops being one.
package com.tigercodes.webtiger

import android.content.Context
import android.content.Intent
import android.view.View
import android.view.ViewGroup
import android.webkit.WebView
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
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
internal class AppFavouritesTest {
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

    /** The letters on the row's tiles. */
    private fun letters(): List<String> =
        compose.onAllNodesWithTag("favourite-letter", useUnmergedTree = true).fetchSemanticsNodes().map { node ->
            node.config.getOrNull(SemanticsProperties.Text).orEmpty().joinToString { it.text }
        }

    @Test
    fun aTabIsMadeAFavouriteIsATileFirstInTheRowAndStopsBeingOne() {
        awaitPage("Home")
        awaitTabs("Home")
        // A second tab, somewhere else, in front.
        compose.onNodeWithTag("new-tab").performScrollTo().performClick()
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("empty-tab").fetchSemanticsNodes().isNotEmpty() }
        go("/whoami", "who=none")
        awaitTabs("Home", "who=none")

        // Made a Favourite from its own menu, the tab is kept, and is first in the row: a
        // tile with the first letter of its name, that says its whole name and that it is a
        // Favourite. It has no button at its end.
        fromTheMenuOf(1, "favourite")
        awaitTabs("who=none (Favourite)", "Home")
        assertEquals(listOf("W"), letters())
        assertEquals("a tile has no button at its end: only the other tab has one", 1, compose.onAllNodesWithTag("close-tab").fetchSemanticsNodes().size)
        assertEquals("it is still in front", "who=none", pageTitle())

        // The other tab in front; the tile, pressed, brings its page back.
        compose.onAllNodesWithTag("tab")[1].performScrollTo().performClick()
        awaitPage("Home")
        compose.onAllNodesWithTag("tab")[0].performScrollTo().performClick()
        awaitPage("who=none")

        // Put to sleep from the tile's own menu: it stays, asleep, and the other tab is in
        // front. Pressed, it wakes.
        fromTheMenuOf(0, "favourite-sleep")
        awaitPage("Home")
        awaitTabs("who=none (Favourite, Asleep)", "Home")
        compose.onAllNodesWithTag("tab")[0].performScrollTo().performClick()
        awaitPage("who=none")
        awaitTabs("who=none (Favourite)", "Home")

        // No longer a Favourite: a Site, with its name and the button at its end.
        fromTheMenuOf(0, "favourite")
        awaitTabs("who=none (Site)", "Home")
        assertEquals(emptyList<String>(), letters())
        assertEquals(2, compose.onAllNodesWithTag("close-tab").fetchSemanticsNodes().size)
    }
}

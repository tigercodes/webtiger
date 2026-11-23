// A site's icon (046) in the app, on a device, in fake mode: the real core and the real WebView.
// The WebView hands over the icon of a page that names one, the core keeps it for the page's
// host, and the window draws it: before the tab's name, and as the tile of a Favourite, which
// keeps it asleep.
//
// The page with an icon is asked for by the test site's other name, localhost: an icon is a
// host's, and the host every other test's pages are at (127.0.0.1) is to have none.
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
internal class AppIconsTest {
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

    /** Sends the tab in front to an address, and waits for its page by its title. */
    private fun go(address: String, title: String) {
        compose.onNodeWithTag("address-field").performClick()
        compose.onNodeWithTag("address-field").performTextReplacement(address)
        compose.onNodeWithTag("address-field").performImeAction()
        awaitPage(title)
    }

    /** How many of the row's tabs show an icon. */
    private fun icons(): Int = compose.onAllNodesWithTag("tab-icon", useUnmergedTree = true).fetchSemanticsNodes().size

    private fun awaitIcons(count: Int) {
        try {
            compose.waitUntil(15_000) { icons() == count }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            throw AssertionError("the row never showed $count icons; it shows ${icons()}")
        }
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
    fun aTabHasItsSitesIconAndAFavouritesTileIsItsIcon() {
        awaitPage("Home")
        awaitTabs("Home")
        assertEquals("the page names no icon, and its tab has none", 0, icons())

        // A second tab, at a page that names its icon: the WebView fetches it, the core keeps
        // it, and the tab shows it. The other tab's site has none.
        compose.onNodeWithTag("new-tab").performScrollTo().performClick()
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("empty-tab").fetchSemanticsNodes().isNotEmpty() }
        go(site.url("/with-icon").replace("127.0.0.1", "localhost"), "With icon")
        awaitTabs("Home", "With icon")
        awaitIcons(1)

        // Made a Favourite, its tile is its icon, and has no letter.
        fromTheMenuOf(1, "favourite")
        awaitTabs("With icon (Favourite)", "Home")
        awaitIcons(1)
        assertEquals("a tile with an icon has no letter", emptyList<String>(), letters())

        // Put to sleep, it keeps its icon: the picture is the core's, not the page's.
        fromTheMenuOf(0, "favourite-sleep")
        awaitPage("Home")
        awaitTabs("With icon (Favourite, Asleep)", "Home")
        assertEquals(1, icons())
        assertEquals(emptyList<String>(), letters())

        // No longer a Favourite: a Site, with its icon before its name.
        fromTheMenuOf(0, "favourite")
        awaitTabs("With icon (Site, Asleep)", "Home")
        assertEquals(1, icons())
    }
}

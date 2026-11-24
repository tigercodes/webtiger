// What is awake (056) in the app, on a device, in fake mode: the real core and the real
// WebView. The Library's fifth list (057) names the tabs that have a page loaded, each with
// where it is and why it is awake; "Sleep All Others" puts to sleep the ones that can be,
// which leave the list and say they are asleep in the strip; the tab on screen stays.
package com.tigercodes.webtiger

import android.content.Context
import android.content.Intent
import android.view.View
import android.view.ViewGroup
import android.webkit.WebView
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performImeAction
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performScrollToNode
import androidx.compose.ui.test.performTextReplacement
import androidx.test.core.app.ActivityScenario
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
internal class AppAwakeTest {
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

    /** Opens a tab in front of the others, at a page of the test site. */
    private fun openTab(path: String, title: String) {
        compose.onNodeWithTag("new-tab").performScrollTo().performClick()
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("empty-tab").fetchSemanticsNodes().isNotEmpty() }
        go(path, title)
    }

    /** What each tab of the strip says of itself beside its name, and its name. */
    private fun tabs(): List<String> =
        compose.onAllNodesWithTag("tab").fetchSemanticsNodes().map { node ->
            val name = node.config.getOrNull(SemanticsProperties.Text).orEmpty().joinToString(" ") { it.text }
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

    /** The tabs listed as awake: each the words of its row, joined. */
    private fun listed(): List<String> =
        compose.onAllNodesWithTag("awake-tab").fetchSemanticsNodes().map { node ->
            node.config.getOrNull(SemanticsProperties.Text).orEmpty().joinToString(" | ") { it.text }
        }

    /** How many tabs the list says are asleep: the number in "Asleep: 2". */
    private fun asleep(): Int {
        compose.onNodeWithTag("awake-list").performScrollToNode(hasTestTag("awake-asleep"))
        val words = compose.onNodeWithTag("awake-asleep").fetchSemanticsNode().config.getOrNull(SemanticsProperties.Text).orEmpty().joinToString("") { it.text }
        return words.filter(Char::isDigit).toInt()
    }

    @Test
    fun theLibraryListsWhatIsAwakeAndSleepAllOthersPutsTheRestToSleep() {
        awaitPage("Home")
        go("/landed", "Landed")
        openTab("/whoami", "who=none")
        openTab("/tall", "Tall")
        awaitTabs("Landed", "who=none", "Tall")

        // The Library, at its fifth list. The three are listed in their row's order, each with
        // where it is and why it is awake: the two behind are only waiting for their time, and
        // the third is on screen.
        compose.onNodeWithTag("history").performClick()
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("library-awake").fetchSemanticsNodes().isNotEmpty() }
        compose.onNodeWithTag("library-awake").performScrollTo().performClick()
        compose.waitUntil(15_000) { listed().size >= 3 }
        val mine = listed().filter { row -> listOf("Landed", "who=none", "Tall").any { row.startsWith("$it |") } }
        assertEquals("this window's three, in their row's order: ${listed()}", 3, mine.size)
        assertTrue("the first: ${mine[0]}", mine[0].startsWith("Landed |") && mine[0].endsWith("Will sleep by itself"))
        assertTrue("the second: ${mine[1]}", mine[1].startsWith("who=none |") && mine[1].endsWith("Will sleep by itself"))
        assertTrue("the third: ${mine[2]}", mine[2].startsWith("Tall |") && mine[2].endsWith("On screen"))
        assertTrue("each says where it is: ${mine[0]}", mine[0].contains("Home · 127.0.0.1"))
        val before = asleep()
        compose.onNodeWithTag("awake-sleep-others").assertIsEnabled()

        // Sleep All Others: the two behind sleep, and leave the list; the one on screen stays.
        compose.onNodeWithTag("awake-sleep-others").performClick()
        compose.waitUntil(15_000) { listed().none { it.startsWith("Landed |") || it.startsWith("who=none |") } }
        assertTrue("the one on screen is listed still: ${listed()}", listed().any { it.startsWith("Tall |") && it.endsWith("On screen") })
        assertEquals("two more are counted asleep", before + 2, asleep())
        compose.onNodeWithTag("awake-done").performClick()

        // The strip says the same: the two behind are asleep where they were, the third is not.
        awaitTabs("Landed (Asleep)", "who=none (Asleep)", "Tall")
        assertEquals("Tall", pageTitle())
    }
}

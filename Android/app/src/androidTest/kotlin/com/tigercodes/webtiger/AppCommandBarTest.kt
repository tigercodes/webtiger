// The command bar (048) in the app, on a device, in fake mode: the real core and the real
// WebView. A command found by its words is a row of the address field's list that says it is
// one, and chosen it is done; what can be done now is what is offered. Nothing is loaded.
//
// (A Space's row is not chosen here: the device suite's tests share one core, and a Space made
// by one of them is there for all that come after. It is chosen on the JVM, against a scripted
// core.)
package com.tigercodes.webtiger

import android.content.Context
import android.content.Intent
import android.view.View
import android.view.ViewGroup
import android.webkit.WebView
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
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
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
internal class AppCommandBarTest {
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

    /** What each row of the list says: its name, and at its end what it is. */
    private fun rows(): List<String> =
        compose.onAllNodesWithTag("suggestion").fetchSemanticsNodes().map { node ->
            node.config.getOrNull(SemanticsProperties.Text).orEmpty().joinToString(", ") { it.text }
        }

    /** Types in the address field, and presses the row of the list that says this. */
    private fun choose(row: String, afterTyping: String) {
        compose.onNodeWithTag("address-field").performClick()
        compose.onNodeWithTag("address-field").performTextReplacement(afterTyping)
        try {
            compose.waitUntil(15_000) { row in rows() }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            throw AssertionError("\"$afterTyping\" never offered <$row>; the list is ${rows()}")
        }
        compose.onAllNodesWithTag("suggestion")[rows().indexOf(row)].performClick()
    }

    @Test
    fun aCommandFoundByItsWordsIsDoneWhenItsRowIsChosen() {
        awaitPage("Home")
        awaitTabs("Home")
        val asked = site.requests.size

        // "keep": the command that keeps the tab, by the words its menu has; the row says it
        // is a command. Chosen, the tab is a Site, and the list has gone.
        choose("Keep as Site, Command", afterTyping = "keep")
        awaitTabs("Home (Site)")
        compose.waitUntil(15_000) { rows().isEmpty() }
        // The same words find the other command now: what can be done is what is offered.
        choose("Stop Keeping, Command", afterTyping = "keep")
        awaitTabs("Home")

        // "new t": a tab is opened, in front.
        choose("New Tab, Command", afterTyping = "new t")
        compose.waitUntil(15_000) { tabs().size == 2 }
        assertEquals("the page of the first tab is as it was, behind", 2, tabs().size)
        assertEquals("no command loaded anything", asked, site.requests.size)
    }
}

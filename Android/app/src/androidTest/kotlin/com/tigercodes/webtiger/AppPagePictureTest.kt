// A sleeping tab's picture (059) in the app, on a device, in fake mode: the real core and the
// real WebView. A tab at the suite's slow page, which answers after three seconds, is put to
// sleep from the Library ("Sleep All Others"): the core asked the WebView for a picture of
// its page as it put it to sleep. Touched, the tab is on its way to that page again for three
// seconds, and the page as it was stands where the page will be; when the page comes, it is
// gone.
package com.tigercodes.webtiger

import android.content.Context
import android.content.Intent
import android.view.View
import android.view.ViewGroup
import android.webkit.WebView
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.assertContentDescriptionEquals
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performImeAction
import androidx.compose.ui.test.performScrollTo
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
internal class AppPagePictureTest {
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

    /** What each tab of the strip says of itself beside its name, and its name. */
    private fun tabs(): List<String> =
        compose.onAllNodesWithTag("tab").fetchSemanticsNodes().map { node ->
            val name = node.config.getOrNull(SemanticsProperties.Text).orEmpty().joinToString(" ") { it.text }
            val said = node.config.getOrNull(SemanticsProperties.StateDescription).orEmpty()
            if (said.isEmpty()) name else "$name ($said)"
        }

    /** Which tabs of the strip say they are asleep, in the strip's order. */
    private fun asleep(): List<Boolean> = tabs().map { it.endsWith("(Asleep)") }

    private fun awaitAsleep(vararg expected: Boolean) {
        try {
            compose.waitUntil(15_000) { asleep() == expected.toList() }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            throw AssertionError("the strip's tabs never were asleep as ${expected.toList()}; the strip is ${tabs()}")
        }
    }

    /** How many pictures of a page as it was the window shows: one, or none. */
    private fun pictures(): Int = compose.onAllNodesWithTag("page-picture").fetchSemanticsNodes().size

    @Test
    fun aTabThatWakesShowsItsPageAsItWasUntilThePageComes() {
        awaitPage("Home")
        go("/slow", "Slow")
        assertEquals("a page that came by itself has no picture over it", 0, pictures())
        // Another tab in front of it.
        compose.onNodeWithTag("new-tab").performScrollTo().performClick()
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("empty-tab").fetchSemanticsNodes().isNotEmpty() }
        go("/whoami", "who=none")
        awaitAsleep(false, false)

        // The first is put to sleep, from the Library.
        compose.onNodeWithTag("history").performClick()
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("library-awake").fetchSemanticsNodes().isNotEmpty() }
        compose.onNodeWithTag("library-awake").performScrollTo().performClick()
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("awake-sleep-others").fetchSemanticsNodes().isNotEmpty() }
        compose.onNodeWithTag("awake-sleep-others").performClick()
        compose.onNodeWithTag("awake-done").performClick()
        awaitAsleep(true, false)
        assertEquals("a sleeping tab's picture is not over another tab's page", 0, pictures())

        // Touched, it is on its way to its page, which takes three seconds to answer: the page
        // as it was is where the page will be, and says what it is.
        compose.onAllNodesWithTag("tab")[0].performClick()
        try {
            compose.waitUntil(2_500) { pictures() == 1 }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            throw AssertionError("the page as it was never stood where the page will be; the strip is ${tabs()}")
        }
        compose.onNodeWithTag("page-picture").assertContentDescriptionEquals("The page as it was")

        // The page comes, and the picture is gone.
        awaitPage("Slow")
        compose.waitUntil(15_000) { pictures() == 0 }
        awaitAsleep(false, false)
        // And the tab is named by its page, as it was before it slept.
        compose.waitUntil(15_000) { tabs().firstOrNull() == "Slow" }
    }
}

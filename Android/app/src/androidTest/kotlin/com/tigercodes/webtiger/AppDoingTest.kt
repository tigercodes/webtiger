// Marks for what a tab is doing (058) in the app, on a device, in fake mode: the real core and
// the real WebView. A page of the suite's own plays a sound (silence, as a sound file) when its
// button is touched: its tab then says so in the strip and has its mark, in front and behind,
// and no longer once the sound is stopped. The camera's mark is not tried here: no test lets a
// page have a camera.
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
internal class AppDoingTest {
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

    /** The page the window shows, to be touched. */
    private fun page(): WebView {
        var found: WebView? = null
        scenario.onActivity { activity -> found = webViewIn(activity.window.decorView) }
        return found ?: throw AssertionError("the window shows no page")
    }

    /** How many marks for a sound the strip has. */
    private fun speakers(): Int = compose.onAllNodesWithTag("doing-sound", useUnmergedTree = true).fetchSemanticsNodes().size

    @Test
    fun aTabThatPlaysASoundSaysSoInTheStripUntilTheSoundStops() {
        awaitPage("Home")
        go("/doing", "Doing")
        awaitTabs("Doing")
        assertEquals("a tab whose page does nothing has no mark", 0, speakers())

        // The page's own button starts its sound: the tab says so, and has its mark.
        PageTouch.awaitShown(page())
        PageTouch.tap(page(), "play")
        awaitTabs("Doing (Playing sound)")
        assertEquals(1, speakers())

        // Another tab comes in front: the first says so still, from behind.
        openTab("/landed", "Landed")
        awaitTabs("Doing (Playing sound)", "Landed")
        assertEquals(1, speakers())

        // In front again, and its sound stopped: it says nothing, and its mark is gone.
        compose.onAllNodesWithTag("tab")[0].performScrollTo().performClick()
        awaitPage("Doing")
        PageTouch.awaitShown(page())
        PageTouch.tap(page(), "stop")
        awaitTabs("Doing", "Landed")
        assertEquals(0, speakers())
    }
}

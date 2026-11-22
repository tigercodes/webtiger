// Sites (039) in the app, on a device, in fake mode: the real core and the real WebView. A tab
// is kept from its own menu, says it is a Site, sleeps and stays when it is closed, wakes at
// its home when it is pressed, and is let go by the keyboard's key.
package com.tigercodes.webtiger

import android.content.Context
import android.content.Intent
import android.view.KeyEvent
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
internal class AppSitesTest {
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

    private fun awaitTabs(vararg expected: String) {
        try {
            compose.waitUntil(15_000) { tabs() == expected.toList() }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            throw AssertionError("the strip never was ${expected.toList()}; it is ${tabs()}")
        }
    }

    /** Asks for the one command from a tab's own menu: a long press on it. */
    private fun fromItsMenu(tab: Int) {
        compose.onAllNodesWithTag("tab")[tab].performScrollTo().performTouchInput { longClick() }
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("keep-or-let-go").fetchSemanticsNodes().isNotEmpty() }
        compose.onNodeWithTag("keep-or-let-go").performClick()
    }

    private fun askedForItsHome(): Int = site.requests.count { it == "/" }

    @Test
    fun aTabIsKeptSleepsWhenItIsClosedAndWakesAtItsHome() {
        awaitPage("Home")
        awaitTabs("Home")

        // Kept, from its own menu: it says it is a Site.
        fromItsMenu(0)
        awaitTabs("Home (Site)")

        // It wanders from its home; and a second tab is opened, somewhere else.
        go("/landed", "Landed")
        compose.onNodeWithTag("new-tab").performScrollTo().performClick()
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("empty-tab").fetchSemanticsNodes().isNotEmpty() }
        go("/whoami", "who=none")
        awaitTabs("Landed (Site)", "who=none")
        val asked = askedForItsHome()

        // The Site in front again, and closed by the button at its end: it is still there, by
        // the name it was kept with, asleep; the other tab has come in front; nothing was loaded.
        compose.onAllNodesWithTag("tab")[0].performScrollTo().performClick()
        awaitPage("Landed")
        compose.onAllNodesWithTag("close-tab")[0].performScrollTo().performClick()
        awaitPage("who=none")
        awaitTabs("Home (Site, Asleep)", "who=none")
        assertEquals("a sleeping Site has no page to close: only the other tab has the button", 1, compose.onAllNodesWithTag("close-tab").fetchSemanticsNodes().size)
        assertEquals(asked, askedForItsHome())

        // A press wakes it, at its home.
        compose.onAllNodesWithTag("tab")[0].performScrollTo().performClick()
        awaitPage("Home")
        awaitTabs("Home (Site)", "who=none")
        assertEquals("its home was loaded", asked + 1, askedForItsHome())

        // Ctrl+D, from a keyboard, lets the tab in front go; and keeps it again.
        val instrumentation = InstrumentationRegistry.getInstrumentation()
        fun ctrlD() {
            val now = android.os.SystemClock.uptimeMillis()
            instrumentation.sendKeySync(KeyEvent(now, now, KeyEvent.ACTION_DOWN, KeyEvent.KEYCODE_D, 0, KeyEvent.META_CTRL_ON or KeyEvent.META_CTRL_LEFT_ON))
            instrumentation.sendKeySync(KeyEvent(now, now, KeyEvent.ACTION_UP, KeyEvent.KEYCODE_D, 0, KeyEvent.META_CTRL_ON or KeyEvent.META_CTRL_LEFT_ON))
        }
        ctrlD()
        awaitTabs("Home", "who=none")
        ctrlD()
        awaitTabs("Home (Site)", "who=none")
        // And its own menu lets it go.
        fromItsMenu(0)
        awaitTabs("Home", "who=none")
    }
}

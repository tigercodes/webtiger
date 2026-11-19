// The app itself on a device, in fake mode (003 §7): the real core, the real WebView, the window
// on a real screen, and a site on the device's loopback address. What the Apple apps' UI tests
// show for their platforms, this shows for Android: a page loads through the core.
package com.tigercodes.webtiger

import android.app.Activity
import android.content.Intent
import android.content.pm.ActivityInfo
import android.view.View
import android.view.ViewGroup
import android.webkit.WebView
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.assertIsNotFocused
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.junit4.accessibility.disableAccessibilityChecks
import androidx.compose.ui.test.junit4.accessibility.enableAccessibilityChecks
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.tryPerformAccessibilityChecks
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performImeAction
import androidx.compose.ui.test.performTextReplacement
import androidx.compose.ui.text.TextRange
import androidx.test.core.app.ActivityScenario
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertSame
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
internal class AppWindowTest {
    @get:Rule val compose = createEmptyComposeRule()

    private lateinit var scenario: ActivityScenario<MainActivity>

    @Before
    fun launch() {
        launch(touch = true)
    }

    /**
     * Starts the app in fake mode. `touch` is the device's touch mode: with false, the device
     * behaves as one whose keyboard was used last, and the system gives a new window's first
     * focusable control the focus.
     */
    private fun launch(touch: Boolean) {
        InstrumentationRegistry.getInstrumentation().setInTouchMode(touch)
        val intent =
            Intent(ApplicationProvider.getApplicationContext(), MainActivity::class.java)
                .putExtra(MainActivity.EXTRA_FAKE, true)
                .putExtra(MainActivity.EXTRA_FAKE_START, site.url("/"))
        scenario = ActivityScenario.launch(intent)
    }

    @After
    fun close() {
        scenario.close()
    }

    // Helpers

    /** The WebView the window shows, found in the window itself: it is on screen, not only made. */
    private fun shownWebView(): WebView? {
        var found: WebView? = null
        scenario.onActivity { activity -> found = webViewIn(activity.window.decorView) }
        return found
    }

    private fun webViewIn(view: View): WebView? {
        if (view is WebView) return view
        if (view is ViewGroup) {
            for (index in 0 until view.childCount) webViewIn(view.getChildAt(index))?.let { return it }
        }
        return null
    }

    private fun pageTitle(): String {
        var title = ""
        scenario.onActivity { activity -> title = webViewIn(activity.window.decorView)?.title.orEmpty() }
        return title
    }

    /** Waits until the page on screen has `title`, for up to fifteen seconds. */
    private fun awaitPage(title: String) {
        compose.waitUntil(15_000) { pageTitle() == title }
    }

    /** What the address field holds, and where its selection is. */
    private fun field(): Pair<String, TextRange?> {
        val config = compose.onNodeWithTag("address-field").fetchSemanticsNode().config
        return config.getOrNull(SemanticsProperties.EditableText)?.text.orEmpty() to config.getOrNull(SemanticsProperties.TextSelectionRange)
    }

    private fun awaitAddress(text: String) {
        try {
            compose.waitUntil(15_000) { field().first == text }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            // These are a test site's addresses on the loopback address, not anyone's browsing.
            throw AssertionError("the address field never showed <$text>; it holds ${field()}")
        }
    }

    private fun type(address: String) {
        compose.onNodeWithTag("address-field").performClick()
        compose.onNodeWithTag("address-field").performTextReplacement(address)
        compose.onNodeWithTag("address-field").performImeAction()
    }

    /** What the address field shows for a page of the site: its scheme stays, as it is not https. */
    private fun shown(path: String): String = "http://127.0.0.1:${site.port}${if (path == "/") "" else path}"

    // Tests

    @Test
    fun aPageLoadsThroughTheCore() {
        awaitPage("Home")
        awaitAddress(shown("/"))
        compose.onNodeWithTag("page").assertExists()
        compose.onNodeWithTag("security-insecure").assertExists()
        compose.onNodeWithTag("reload").assertIsEnabled()
        compose.onNodeWithTag("back").assertIsNotEnabled()
        assertNotNull("the web view is in the window", shownWebView())
    }

    @Test
    fun typingAnAddressLoadsItAndBackGoesBack() {
        awaitPage("Home")
        type(site.url("/landed"))
        awaitPage("Landed")
        awaitAddress(shown("/landed"))
        compose.onNodeWithTag("back").assertIsEnabled().performClick()
        awaitPage("Home")
        awaitAddress(shown("/"))
        compose.onNodeWithTag("forward").assertIsEnabled()
    }

    @Test
    fun theSystemsBackGoesBackInThePage() {
        awaitPage("Home")
        type(site.url("/landed"))
        awaitPage("Landed")
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("back").fetchSemanticsNodes().isNotEmpty() }
        awaitAddress(shown("/landed"))
        scenario.onActivity { activity -> activity.onBackPressedDispatcher.onBackPressed() }
        awaitPage("Home")
    }

    @Test
    fun aFailureIsShownInWords() {
        awaitPage("Home")
        type("http://127.0.0.1:${TestSite.unusedPort()}/")
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("failed-page").fetchSemanticsNodes().isNotEmpty() }
        compose.onNodeWithText("This page didn’t load").assertExists()
        compose.onNodeWithTag("failed-words").assertTextEquals("The site couldn’t be reached.")
        // The engine's own error page is covered: its words are not the app's.
        compose.onNodeWithTag("page").assertDoesNotExist()
    }

    @Test
    fun atRestTheFieldShowsTheStartOfTheAddress() {
        awaitPage("Home")
        type(site.url("/landed"))
        awaitPage("Landed")
        awaitAddress(shown("/landed"))
        // A field scrolls to its caret. At rest the caret is at the start, so what is in view is
        // the host: an address too long for the field loses its tail, never its site.
        assertEquals(TextRange.Zero, field().second)
    }

    @Test
    fun aTapOnTheFieldSelectsTheWholeAddress() {
        awaitPage("Home")
        awaitAddress(shown("/"))
        compose.onNodeWithTag("address-field").performClick()
        // As every browser does: the whole address, selected, so typing replaces it.
        awaitAddress(site.url("/"))
        assertEquals(TextRange(0, site.url("/").length), field().second)
    }

    @Test
    fun aWindowThatOpensOnAPageGivesThePageTheKeyboard() {
        // Start again as a device with a keyboard does: out of touch mode, where the system
        // gives a new window's address field the focus before any page is there.
        scenario.close()
        launch(touch = false)
        awaitPage("Home")
        // The field had nothing typed in it, so it let go when the page arrived, and shows the
        // page: not what it held when the focus came, which was nothing.
        awaitAddress(shown("/"))
        compose.onNodeWithTag("address-field").assertIsNotFocused()
    }

    /**
     * Android's accessibility checks over each state of the window and over the catalogue
     * (000 §8.6): labels, contrast, the size of what can be touched. What the Apple apps' audit
     * is for them.
     */
    @Test
    fun everyStateOfTheWindowPassesTheAccessibilityChecks() {
        compose.enableAccessibilityChecks()
        try {
            // A page.
            awaitPage("Home")
            awaitAddress(shown("/"))
            compose.onRoot().tryPerformAccessibilityChecks()
            // A failed page.
            type("http://127.0.0.1:${TestSite.unusedPort()}/")
            compose.waitUntil(15_000) { compose.onAllNodesWithTag("failed-page").fetchSemanticsNodes().isNotEmpty() }
            compose.onRoot().tryPerformAccessibilityChecks()
            // The design catalogue, which has every state the window has.
            scenario.close()
            scenario = ActivityScenario.launch(Intent(ApplicationProvider.getApplicationContext(), MainActivity::class.java).putExtra(MainActivity.EXTRA_CATALOGUE, true))
            compose.waitUntil(15_000) { compose.onAllNodesWithTag("address-field").fetchSemanticsNodes().size > 1 }
            compose.onRoot().tryPerformAccessibilityChecks()
        } finally {
            compose.disableAccessibilityChecks()
        }
    }

    /**
     * Several tabs (005): a new one is empty and in front; each tab is where it was left; closing
     * the one in front brings the other forward.
     */
    @Test
    fun tabsOpenSwitchAndClose() {
        awaitPage("Home")
        compose.onAllNodesWithTag("tab").assertCountEquals(1)

        compose.onNodeWithTag("new-tab").performClick()
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("empty-tab").fetchSemanticsNodes().isNotEmpty() }
        compose.onAllNodesWithTag("tab").assertCountEquals(2)
        assertEquals("an empty tab has no web view in the window", null, shownWebView())
        // The new tab loads what is typed in it.
        compose.onNodeWithTag("address-field").performTextReplacement(site.url("/landed"))
        compose.onNodeWithTag("address-field").performImeAction()
        awaitPage("Landed")

        // The first tab is where it was left, and its web view is the one in the window again.
        compose.onAllNodesWithTag("tab")[0].performClick()
        awaitPage("Home")
        awaitAddress(shown("/"))

        // Closing the tab in front brings the other forward.
        compose.onAllNodesWithTag("close-tab")[0].performClick()
        awaitPage("Landed")
        compose.onAllNodesWithTag("tab").assertCountEquals(1)
    }

    @Test
    fun theDesignCatalogueOpensInPlaceOfABrowser() {
        scenario.close()
        val intent = Intent(ApplicationProvider.getApplicationContext(), MainActivity::class.java).putExtra(MainActivity.EXTRA_CATALOGUE, true)
        scenario = ActivityScenario.launch(intent)
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("catalogue").fetchSemanticsNodes().isNotEmpty() }
        compose.onNodeWithText("Design Catalogue").assertExists()
        compose.onNodeWithTag("catalogue-tokens").assertExists()
        // Each state is the window itself, with an address field of its own (004 §2).
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("address-field").fetchSemanticsNodes().size > 1 }
        assertEquals("no page is loaded for a catalogue", null, shownWebView())
    }

    @Test
    fun turningTheDeviceReloadsNothing() {
        awaitPage("Home")
        awaitAddress(shown("/"))
        val before = site.requests.count { it == "/" }
        val webView = shownWebView()
        var first: Activity? = null
        scenario.onActivity { activity ->
            first = activity
            activity.requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE
        }
        compose.waitUntil(15_000) {
            var wide = false
            scenario.onActivity { activity -> wide = activity.resources.configuration.orientation == android.content.res.Configuration.ORIENTATION_LANDSCAPE }
            wide
        }
        compose.waitForIdle()
        var second: Activity? = null
        scenario.onActivity { activity -> second = activity }
        assertSame("the activity is not made again (000 §6.5)", first, second)
        assertSame("the page's web view is the same one", webView, shownWebView())
        assertEquals("Home", pageTitle())
        assertEquals("the page was not asked for again", before, site.requests.count { it == "/" })
        awaitAddress(shown("/"))
    }

    companion object {
        /**
         * One site for the whole process: the app's core starts once, with the first window's
         * start address, and every later window opens the same one.
         */
        val site = TestSite()
    }
}

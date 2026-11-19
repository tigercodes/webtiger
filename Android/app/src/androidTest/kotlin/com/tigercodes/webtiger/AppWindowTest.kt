// The app itself on a device, in fake mode (003 §7): the real core, the real WebView, the window
// on a real screen, and a site on the device's loopback address. What the Apple apps' UI tests
// show for their platforms, this shows for Android: a page loads through the core.
package com.tigercodes.webtiger

import android.app.Activity
import android.content.Intent
import android.content.IntentFilter
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
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.assertTextContains
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.junit4.accessibility.disableAccessibilityChecks
import androidx.compose.ui.test.junit4.accessibility.enableAccessibilityChecks
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.compose.ui.test.longClick
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performImeAction
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.performTextReplacement
import androidx.compose.ui.test.performTouchInput
import androidx.compose.ui.test.tryPerformAccessibilityChecks
import androidx.compose.ui.text.TextRange
import androidx.test.core.app.ActivityScenario
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNotSame
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

    /** Waits until the find bar says `words` of what was found, for up to fifteen seconds. */
    private fun awaitFound(words: String) {
        fun said(): List<String> =
            compose.onAllNodesWithTag("find-outcome").fetchSemanticsNodes().map { node ->
                node.config.getOrNull(SemanticsProperties.Text).orEmpty().joinToString(" ") { it.text }
            }
        try {
            compose.waitUntil(15_000) { said() == listOf(words) }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            throw AssertionError("the find bar never said <$words>; it says ${said()}")
        }
    }

    /** The visits the History list shows, each as its words. */
    private fun visits(): List<String> =
        compose.onAllNodesWithTag("history-entry").fetchSemanticsNodes().map { node ->
            node.config.getOrNull(SemanticsProperties.Text).orEmpty().joinToString(" | ") { it.text }
        }

    /** Waits until the History list shows `count` visits, for up to fifteen seconds. */
    private fun awaitVisits(count: Int) {
        try {
            compose.waitUntil(15_000) { visits().size == count }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            // These are a test site's pages on the loopback address, not anyone's browsing.
            val open = compose.onAllNodesWithTag("history-done").fetchSemanticsNodes().isNotEmpty()
            val nothing = compose.onAllNodesWithTag("history-nothing").fetchSemanticsNodes().isNotEmpty()
            throw AssertionError("the History list never showed $count visits; it shows ${visits()} (the list is open: $open; it says there is nothing: $nothing)")
        }
    }

    /**
     * The app's core lives as long as its process, and the tests before this one browsed:
     * history starts from nothing, by the History list's own way of forgetting everything.
     */
    private fun startWithNoHistory() {
        compose.onNodeWithTag("history").performClick()
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("history-clear").fetchSemanticsNodes().isNotEmpty() }
        compose.onNodeWithTag("history-clear").performClick()
        compose.onNodeWithTag("history-clear-everything").performClick()
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("history-nothing").fetchSemanticsNodes().isNotEmpty() }
        compose.onNodeWithTag("history-nothing").assertTextContains("Pages you visit will be listed here.")
        compose.onNodeWithTag("history-done").performClick()
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("history-nothing").fetchSemanticsNodes().isEmpty() }
    }

    /** The places offered under the address field, each as its words. */
    private fun offered(): List<String> =
        compose.onAllNodesWithTag("suggestion").fetchSemanticsNodes().map { node ->
            node.config.getOrNull(SemanticsProperties.Text).orEmpty().joinToString(" | ") { it.text }
        }

    /** Waits until `count` places are offered, for up to fifteen seconds. */
    private fun awaitOffered(count: Int) {
        try {
            compose.waitUntil(15_000) { offered().size == count }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            // These are a test site's pages on the loopback address, not anyone's browsing.
            throw AssertionError("$count places were never offered; what is offered is ${offered()}, for a field holding ${field()}")
        }
    }

    /** What the zoom bar says the level is, as a screen reader is told it. Null when the bar is shut. */
    private fun level(): String? =
        compose.onAllNodesWithTag("zoom-level").fetchSemanticsNodes().firstOrNull()?.config?.getOrNull(SemanticsProperties.StateDescription)

    /** Waits until the zoom bar says `words`, for up to fifteen seconds. */
    private fun awaitLevel(words: String?) {
        try {
            compose.waitUntil(15_000) { level() == words }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            throw AssertionError("the zoom bar never said <$words>; it says <${level()}>")
        }
    }

    /** The size the WebView on screen draws text at, in per cent. */
    private fun textZoom(): Int? {
        var zoom: Int? = null
        scenario.onActivity { activity -> zoom = webViewIn(activity.window.decorView)?.settings?.textZoom }
        return zoom
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

    /**
     * A long press on the field at rest brings the keyboard and, in the same gesture, selects a
     * word in the text the field showed until then. That is not typing: the field holds the
     * whole address, and nothing is offered for it (011).
     */
    @Test
    fun aLongPressOnTheFieldAtRestIsNotTyping() {
        awaitPage("Home")
        awaitAddress(shown("/"))
        compose.onNodeWithTag("address-field").performTouchInput { longClick() }
        awaitAddress(site.url("/"))
        compose.waitForIdle()
        assertEquals(emptyList<String>(), offered())
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

    /**
     * A page that asks for a new window (006): a link with a target opens in a tab of its own,
     * in front, beside the tab it came from; closing it brings that tab forward again.
     */
    @Test
    fun aLinkThatOpensAWindowOpensATab() {
        awaitPage("Home")
        type(site.url("/opener"))
        awaitPage("Opener")
        // A tab after the first, so that "beside" is not "at the end", and "the tab it came
        // from" is not "the tab after it".
        compose.onNodeWithTag("new-tab").performClick()
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("empty-tab").fetchSemanticsNodes().isNotEmpty() }
        compose.onAllNodesWithTag("tab")[0].performClick()
        awaitPage("Opener")
        val opener = checkNotNull(shownWebView())

        // A finger on the page's link.
        PageTouch.tap(opener, "blank")

        awaitPage("Opened")
        awaitAddress(shown("/opened"))
        compose.onAllNodesWithTag("tab").assertCountEquals(3)
        compose.onAllNodesWithTag("tab")[1].assertIsSelected()
        assertNotSame("the link's page is in a web view of its own", opener, shownWebView())

        // Closing it brings forward the tab it came from, not the tab after it.
        compose.onAllNodesWithTag("close-tab")[1].performClick()
        awaitPage("Opener")
        awaitAddress(shown("/opener"))
        compose.onAllNodesWithTag("tab").assertCountEquals(2)
        compose.onAllNodesWithTag("tab")[0].assertIsSelected()
        assertSame("the tab's own web view, as it was left", opener, shownWebView())
    }

    /**
     * A window that is refused the only place it was going leaves no tab behind (006). The
     * page's link opens a window on a letter to write, which is for the system and not for a
     * browser: the WebView makes the window before the core's policy refuses its address, and
     * the core closes it.
     */
    @Test
    fun aWindowRefusedItsAddressLeavesNoTab() {
        awaitPage("Home")
        type(site.url("/opener"))
        awaitPage("Opener")
        val opener = checkNotNull(shownWebView())

        PageTouch.tap(opener, "letter")
        // Whatever came and went, the page is in front again, and its other link works.
        Thread.sleep(1_000)
        awaitPage("Opener")
        assertSame(opener, shownWebView())
        PageTouch.tap(opener, "blank")

        awaitPage("Opened")
        compose.onAllNodesWithTag("tab").assertCountEquals(2)
    }

    /**
     * A link for the system (007), with the real core: a touch on a link to write a letter
     * leaves the page as it was. In fake mode the shell opens nothing, so the suite starts no
     * mail app on the device it runs on; an activity that tried to start would be caught here.
     */
    @Test
    fun aLinkForTheSystemLeavesThePageAsItWas() {
        val instrumentation = InstrumentationRegistry.getInstrumentation()
        val letters =
            instrumentation.addMonitor(
                IntentFilter(Intent.ACTION_VIEW).apply {
                    addCategory(Intent.CATEGORY_BROWSABLE)
                    addDataScheme("mailto")
                },
                null,
                true,
            )
        try {
            awaitPage("Home")
            type(site.url("/contact"))
            awaitPage("Contact")
            val page = checkNotNull(shownWebView())

            PageTouch.tap(page, "letter")
            // Long enough for the core to have heard of the link and asked the shell to open it.
            Thread.sleep(1_500)

            awaitPage("Contact")
            awaitAddress(shown("/contact"))
            assertSame(page, shownWebView())
            compose.onAllNodesWithTag("tab").assertCountEquals(1)
            assertEquals("fake mode started no activity for the letter", 0, letters.hits)
        } finally {
            instrumentation.removeMonitor(letters)
        }
    }

    /**
     * History (008), with the real core and the real WebView: what was visited is listed, newest
     * first; it is found by a word; choosing a visit loads it again; a visit is forgotten, and
     * then all of them.
     */
    @Test
    fun historyListsWhatWasVisitedFindsItAndForgetsIt() {
        awaitPage("Home")
        startWithNoHistory()

        type(site.url("/landed"))
        awaitPage("Landed")
        type(site.url("/"))
        awaitPage("Home")

        compose.onNodeWithTag("history").performClick()
        awaitVisits(2)
        compose.onAllNodesWithTag("history-entry")[0].assertTextContains("Home")
        compose.onAllNodesWithTag("history-entry")[1].assertTextContains("Landed")
        compose.onRoot().tryPerformAccessibilityChecks()

        // A word of a title finds its visit, and only it.
        compose.onNodeWithTag("history-search").performTextInput("lan")
        awaitVisits(1)
        compose.onAllNodesWithTag("history-entry")[0].assertTextContains("Landed")

        // Choosing it loads it again, in the tab in front, and the list is gone.
        compose.onAllNodesWithTag("history-entry")[0].performClick()
        awaitPage("Landed")
        awaitAddress(shown("/landed"))
        compose.onAllNodesWithTag("history-entry").assertCountEquals(0)

        // That was a visit too. One is forgotten, and then all of them.
        compose.onNodeWithTag("history").performClick()
        awaitVisits(3)
        compose.onAllNodesWithTag("history-forget")[0].performClick()
        awaitVisits(2)
        compose.onNodeWithTag("history-clear").performClick()
        compose.onNodeWithTag("history-clear-everything").performClick()
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("history-nothing").fetchSemanticsNodes().isNotEmpty() }
        compose.onAllNodesWithTag("history-entry").assertCountEquals(0)

        // The system's Back closes the list, and the page is as it was.
        scenario.onActivity { activity -> activity.onBackPressedDispatcher.onBackPressed() }
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("history-nothing").fetchSemanticsNodes().isEmpty() }
        awaitPage("Landed")
    }

    /**
     * Places offered as the address field is typed in (011), with the real core: a page that
     * was visited is offered by the start of a word of its title; the system's Back puts the
     * list away and leaves what was typed; choosing the place goes there.
     */
    @Test
    fun aPlaceVisitedIsOfferedWhenPartOfItsNameIsTyped() {
        awaitPage("Home")
        startWithNoHistory()
        type(site.url("/landed"))
        awaitPage("Landed")
        type(site.url("/"))
        awaitPage("Home")
        awaitOffered(0)

        // The keyboard alone offers nothing: the field holds the page's own address. (This is
        // the third tap on the field in quick succession, which selects all of what it showed
        // at rest: that was once taken for typing.)
        compose.onNodeWithTag("address-field").performClick()
        awaitAddress(site.url("/"))
        awaitOffered(0)

        // The start of a word of the other page's title: that page, and only it.
        compose.onNodeWithTag("address-field").performTextReplacement("lan")
        awaitOffered(1)
        compose.onAllNodesWithTag("suggestion")[0].assertTextContains("Landed")
        compose.onAllNodesWithTag("suggestion")[0].assertTextContains(shown("/landed"))
        compose.onRoot().tryPerformAccessibilityChecks()

        // The system's Back puts the list away, and leaves what was typed and the page.
        scenario.onActivity { activity -> activity.onBackPressedDispatcher.onBackPressed() }
        awaitOffered(0)
        assertEquals("lan", field().first)
        assertEquals("Home", pageTitle())

        // Typing on asks again; choosing the place goes there, and the list is gone.
        compose.onNodeWithTag("address-field").performTextReplacement("land")
        awaitOffered(1)
        compose.onAllNodesWithTag("suggestion")[0].performClick()
        awaitPage("Landed")
        awaitAddress(shown("/landed"))
        awaitOffered(0)
    }

    /**
     * Zoom (012), with the real core and the real WebView: the bar is shut until asked for;
     * bigger and smaller go through the core, the bar says the level and the WebView draws its
     * text at it; the level is the site's, and is there on another page of it; the system's
     * Back closes the bar.
     */
    @Test
    fun zoomGoesThroughTheCoreAndTheBarSaysTheLevel() {
        awaitPage("Home")
        assertEquals(null, level())
        compose.onNodeWithTag("zoom").performClick()
        // The app's core lives as long as its process: the site is at actual size here
        // whatever a test before this one left it at.
        compose.waitUntil(15_000) { level() != null }
        if (level() != "100%") compose.onNodeWithTag("zoom-level").performClick()
        awaitLevel("100%")
        assertEquals(100, textZoom())

        compose.onNodeWithTag("zoom-in").performClick()
        awaitLevel("110%")
        compose.onNodeWithTag("zoom-in").performClick()
        awaitLevel("125%")
        compose.onNodeWithTag("zoom-out").performClick()
        awaitLevel("110%")
        compose.waitUntil(15_000) { textZoom() == 110 }
        compose.onRoot().tryPerformAccessibilityChecks()

        // Pressing the level puts the page back.
        compose.onNodeWithTag("zoom-level").performClick()
        awaitLevel("100%")
        compose.waitUntil(15_000) { textZoom() == 100 }

        // Left at 110%, the level is there on another page of the site; and the system's Back
        // closes the bar before it does anything to the page.
        compose.onNodeWithTag("zoom-in").performClick()
        awaitLevel("110%")
        scenario.onActivity { activity -> activity.onBackPressedDispatcher.onBackPressed() }
        awaitLevel(null)
        assertEquals("Home", pageTitle())
        type(site.url("/landed"))
        awaitPage("Landed")
        compose.onNodeWithTag("zoom").performClick()
        awaitLevel("110%")
        assertEquals(110, textZoom())

        // And the site is left as it was found, for the tests after this one.
        compose.onNodeWithTag("zoom-level").performClick()
        awaitLevel("100%")
    }

    /**
     * Find on the page (010), with the real core and the real WebView: the bar is shut until
     * asked for; a word that is there is counted, and the count follows next and previous; a
     * word that is not there is said not to be; the system's Back closes the bar.
     */
    @Test
    fun findOnThePageCountsAndSaysWhenTheTextIsNotThere() {
        awaitPage("Home")
        type(site.url("/find"))
        awaitPage("Find")
        compose.onNodeWithTag("find-field").assertDoesNotExist()

        compose.onNodeWithTag("find").performClick()
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("find-field").fetchSemanticsNodes().isNotEmpty() }
        compose.onNodeWithTag("find-field").performTextInput("needle")
        awaitFound("1 of 3")
        compose.onNodeWithTag("find-next").performClick()
        awaitFound("2 of 3")
        compose.onNodeWithTag("find-previous").performClick()
        awaitFound("1 of 3")
        compose.onRoot().tryPerformAccessibilityChecks()

        // Closing the bar ends the find; opened again it is empty.
        compose.onNodeWithTag("find-done").performClick()
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("find-field").fetchSemanticsNodes().isEmpty() }
        compose.onNodeWithTag("find").performClick()
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("find-field").fetchSemanticsNodes().isNotEmpty() }
        compose.onAllNodesWithTag("find-outcome").assertCountEquals(0)
        compose.onNodeWithTag("find-field").performTextInput("zebra")
        awaitFound("Not found")

        // The system's Back closes the bar, and the page is as it was.
        scenario.onActivity { activity -> activity.onBackPressedDispatcher.onBackPressed() }
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("find-field").fetchSemanticsNodes().isEmpty() }
        awaitPage("Find")
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

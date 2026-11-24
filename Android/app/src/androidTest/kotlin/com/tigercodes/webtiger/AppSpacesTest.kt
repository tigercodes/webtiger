// Several Spaces (024) in the app itself on a device, in fake mode: the real core, the real
// WebView, the window on a real screen. A Space is made from the chip in the strip of tabs, the
// window shows it, and its tabs and its logins are its own.
package com.tigercodes.webtiger

import android.content.Intent
import android.view.View
import android.view.ViewGroup
import android.webkit.WebView
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.assert
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsEnabled
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.junit4.accessibility.disableAccessibilityChecks
import androidx.compose.ui.test.junit4.accessibility.enableAccessibilityChecks
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performImeAction
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.performTextReplacement
import androidx.compose.ui.test.tryPerformAccessibilityChecks
import androidx.test.core.app.ActivityScenario
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
internal class AppSpacesTest {
    @get:Rule val compose = createEmptyComposeRule()

    private lateinit var scenario: ActivityScenario<MainActivity>

    /** The process's one site: the app's core starts once, and every window opens on it. */
    private val site get() = AppWindowTest.site

    @Before
    fun launch() {
        InstrumentationRegistry.getInstrumentation().setInTouchMode(true)
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

    private fun webViewIn(view: View): WebView? {
        if (view is WebView) return view
        if (view is ViewGroup) {
            for (index in 0 until view.childCount) webViewIn(view.getChildAt(index))?.let { return it }
        }
        return null
    }

    /** The title of the page the window shows: of the WebView that is on the screen. */
    private fun pageTitle(): String {
        var title = ""
        scenario.onActivity { activity -> title = webViewIn(activity.window.decorView)?.title.orEmpty() }
        return title
    }

    private fun awaitPage(title: String) {
        try {
            compose.waitUntil(15_000) { pageTitle() == title }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            // A test site's page on the loopback address, not anyone's browsing.
            throw AssertionError("the page <$title> never showed; the window shows <${pageTitle()}>")
        }
    }

    private fun type(address: String) {
        compose.onNodeWithTag("address-field").performClick()
        compose.onNodeWithTag("address-field").performTextReplacement(address)
        compose.onNodeWithTag("address-field").performImeAction()
    }

    /** What the chip says to a screen reader: "Space: " and the name of the Space the window shows. */
    private fun chip(): String =
        compose
            .onNodeWithTag("space-chip")
            .fetchSemanticsNode()
            .config
            .getOrNull(SemanticsProperties.ContentDescription)
            .orEmpty()
            .joinToString()

    private fun awaitSpace(name: String) {
        try {
            compose.waitUntil(15_000) { chip() == "Space: $name" }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            throw AssertionError("the window never showed the Space <$name>; its chip says <${chip()}>")
        }
    }

    private fun awaitTag(tag: String) {
        try {
            compose.waitUntil(15_000) { compose.onAllNodesWithTag(tag).fetchSemanticsNodes().isNotEmpty() }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            // The names of the window's own controls, never what is in them.
            throw AssertionError("<$tag> never showed; the window has ${tags()}")
        }
    }

    /** The test tags of everything the window shows. */
    private fun tags(): List<String> {
        val found = mutableListOf<String>()

        fun walk(node: androidx.compose.ui.semantics.SemanticsNode) {
            node.config.getOrNull(SemanticsProperties.TestTag)?.let(found::add)
            node.children.forEach(::walk)
        }
        compose.onAllNodes(androidx.compose.ui.test.isRoot(), useUnmergedTree = true).fetchSemanticsNodes().forEach(::walk)
        return found
    }

    /** Opens the list of Spaces, and waits for the core's answer to be in it: the app's own Space is always there. */
    private fun openSpaces() {
        compose.onNodeWithTag("space-chip").performClick()
        awaitTag("spaces-new")
        awaitTag("space-1")
    }

    /**
     * A Space is made with logins of its own: the window shows it, with one empty tab, and a
     * site that knows who is signed in in the first Space does not know them here. Back in the
     * first Space its tab is as it was left. A Space made to share the first one's logins is
     * signed in as the first is. Android's accessibility checks pass over the list and the page
     * that makes a Space.
     *
     * The cookie is one for the session only. The Spaces are this process's: the fake core's
     * store is in memory, and no other test makes one, so they are the second and the third.
     */
    @Test
    fun aSpaceIsMadeAndKeepsItsTabsAndItsLoginsApart() {
        compose.enableAccessibilityChecks()
        try {
            awaitPage("Home")
            type(site.url("/session?v=ada"))
            awaitPage("Session")
            awaitSpace("Home")

            // The list: the one Space there is, which the window shows.
            openSpaces()
            compose.onNodeWithTag("space-1").assertTextEquals("Home", "Shown")
            compose.onRoot().tryPerformAccessibilityChecks()

            // The page that makes one: nothing is made of no name.
            compose.onNodeWithTag("spaces-new").performClick()
            awaitTag("space-name")
            compose.onNodeWithTag("space-make").assertIsNotEnabled()
            compose.onNodeWithTag("space-name").performTextInput("Work")
            compose.onNodeWithTag("space-make").assertIsEnabled()
            compose.onRoot().tryPerformAccessibilityChecks()
            // (The page is longer than a phone's screen has room for above the keyboard since
            // it has the icons too, 050: the button is moved to before it is pressed.)
            compose.onNodeWithTag("space-make").performScrollTo().performClick()

            // The window shows it: one empty tab, and logins of its own.
            awaitSpace("Work")
            awaitTag("empty-tab")
            compose.onAllNodesWithTag("tab").assertCountEquals(1)
            compose.onAllNodesWithTag("spaces").assertCountEquals(0)
            type(site.url("/whoami"))
            awaitPage("who=none")

            // Back in the first Space, its tab is as it was left, signed in as it was.
            openSpaces()
            compose.onNodeWithTag("space-2").assertTextEquals("Work", "Shown")
            compose.onNodeWithTag("space-1").performClick()
            awaitSpace("Home")
            awaitPage("Session")
            compose.onAllNodesWithTag("tab").assertCountEquals(1)
            type(site.url("/whoami"))
            awaitPage("who=ada")

            // A Space that shares the first one's logins is signed in where the first is.
            openSpaces()
            compose.onNodeWithTag("spaces-new").performClick()
            awaitTag("space-name")
            compose.onNodeWithTag("space-name").performTextInput("Project")
            compose.onNodeWithTag("space-logins-shared").performClick()
            compose.onNodeWithTag("space-make").performScrollTo().performClick()
            awaitSpace("Project")
            awaitTag("empty-tab")
            type(site.url("/whoami"))
            awaitPage("who=ada")

            // The list says who shares; and Work is as it was left, signed in as nobody.
            openSpaces()
            compose.onNodeWithTag("space-1").assertTextEquals("Home", "Shares logins with Project")
            compose.onNodeWithTag("space-2").assertTextEquals("Work")
            compose.onNodeWithTag("space-3").assertTextEquals("Project", "Shares logins with Home", "Shown")
            compose.onNodeWithTag("space-2").performClick()
            awaitSpace("Work")
            awaitPage("who=none")

            // The Space the window shows is named: the field holds the name it has, and the
            // chip and the list say the new one.
            openSpaces()
            compose.onNodeWithTag("spaces-rename").performClick()
            awaitTag("space-save")
            compose.onNodeWithTag("space-name").assert(androidx.compose.ui.test.hasText("Work"))
            // It was given a colour nobody had when it was made (025): the first there is.
            compose.onNodeWithTag("space-colour").assertTextEquals("Colour: Red")
            compose.onRoot().tryPerformAccessibilityChecks()
            compose.onNodeWithTag("space-name").performTextReplacement("Office")
            compose.onNodeWithTag("space-colour-green").performScrollTo().performClick()
            compose.onNodeWithTag("space-colour").assertTextEquals("Colour: Green")
            compose.onNodeWithTag("space-save").performScrollTo().performClick()
            awaitSpace("Office")
            compose.onAllNodesWithTag("spaces").assertCountEquals(0)
            awaitPage("who=none")
            openSpaces()
            compose.onNodeWithTag("space-2").assertTextEquals("Office", "Shown")
            // And the colour it was given is the one it has.
            compose.onNodeWithTag("spaces-rename").performClick()
            awaitTag("space-save")
            compose.onNodeWithTag("space-colour").assertTextEquals("Colour: Green")
            compose.onNodeWithTag("space-colour-green").assertIsSelected()
        } finally {
            compose.disableAccessibilityChecks()
        }
    }
}

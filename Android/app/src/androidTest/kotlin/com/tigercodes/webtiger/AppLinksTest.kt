// A link pressed in another app (031), in the app itself on a device, in fake mode: the real
// core, the real WebView, the window on a real screen, and the system's own handling of a
// window that is asked for again. The window is asked as the system asks it for another app's
// link; no other app is started, and the addresses are the test site's.
package com.tigercodes.webtiger

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.view.View
import android.view.ViewGroup
import android.webkit.WebView
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
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
internal class AppLinksTest {
    @get:Rule val compose = createEmptyComposeRule()

    private lateinit var scenario: ActivityScenario<MainActivity>
    private val context: Context get() = ApplicationProvider.getApplicationContext()

    /** The process's one site: the app's core starts once, and every window opens on it. */
    private val site get() = AppWindowTest.site

    @Before
    fun launch() {
        InstrumentationRegistry.getInstrumentation().setInTouchMode(true)
        val intent =
            Intent(context, MainActivity::class.java)
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

    /** What the address field holds. */
    private fun field(): String =
        compose
            .onNodeWithTag("address-field")
            .fetchSemanticsNode()
            .config
            .getOrNull(SemanticsProperties.EditableText)
            ?.text
            .orEmpty()

    /**
     * Another app's link is pressed, and the system has chosen WebTiger for it: the window is
     * asked with the system's own request, from outside it, as the system asks.
     */
    private fun pressedElsewhere(address: String) {
        context.startActivity(
            Intent(Intent.ACTION_VIEW, Uri.parse(address))
                .addCategory(Intent.CATEGORY_BROWSABLE)
                .setClass(context, MainActivity::class.java)
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK),
        )
    }

    @Test
    fun aLinkPressedInAnotherAppOpensInANewTabOfTheWindowThereIs() {
        awaitPage("Home")
        compose.onAllNodesWithTag("tab").assertCountEquals(1)
        val asked = site.requests.size

        pressedElsewhere(site.url("/landed"))

        // The window that is there has it, in a new tab after the one that was in front, in
        // front: its page loads, and the field says where it is.
        awaitPage("Landed")
        compose.onAllNodesWithTag("tab").assertCountEquals(2)
        compose.onAllNodesWithTag("tab")[1].assertIsSelected()
        assertEquals(site.url("/landed"), field())

        // The same link pressed again is a new request, and opens again (028 found that a
        // second request could be taken for the first one over again).
        pressedElsewhere(site.url("/landed"))
        compose.waitUntil(15_000) { compose.onAllNodesWithTag("tab").fetchSemanticsNodes().size == 3 }
        compose.onAllNodesWithTag("tab")[2].assertIsSelected()
        awaitPage("Landed")

        // Several at once, as an app that opens a list of links sends them, with nothing
        // between; and among them what is no address of the web, handed to the window by
        // name as an app could. Every request reaches the window, in order: the core refuses
        // what is not of the web, which opens nothing, and the two pages of the site open,
        // each after the one before, the last in front.
        for (address in listOf("javascript:alert(1)", site.url("/landed?from=list"), "file:///sdcard/page.html", "about:blank", "intent://example.com/#Intent;scheme=https;end", site.url("/links"))) {
            pressedElsewhere(address)
        }
        try {
            awaitPage("Links")
        } catch (failure: AssertionError) {
            throw AssertionError("${failure.message}; tabs ${compose.onAllNodesWithTag("tab").fetchSemanticsNodes().size}; the site was asked ${site.requests.drop(asked)}")
        }
        compose.onAllNodesWithTag("tab").assertCountEquals(5)
        compose.onAllNodesWithTag("tab")[4].assertIsSelected()
        assertEquals(site.url("/links"), field())
        // The site was asked for the four pages (it keeps the path of what it is asked for),
        // and for nothing else but what a page shows.
        assertEquals(
            listOf("/landed", "/landed", "/landed", "/links"),
            site.requests.drop(asked).filter { it != "/favicon.ico" && it != "/pixel" },
        )
    }
}

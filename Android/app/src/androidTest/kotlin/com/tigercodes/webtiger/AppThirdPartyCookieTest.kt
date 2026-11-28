// Third-party cookies (070) in the app, on a device, in fake mode: the real core and the real
// WebView. A site that has kept a name is, in another site's page, asked with nothing, and
// finds nothing stored: the WebView shuts its cookies out, and the adapter keeps its storage
// apart on the core's word (071, AppThirdPartyStorageTest). What the WebView does is its
// adapter's suite's (SystemWebViewThirdPartyCookieTest). This is the app as it is put together.
package com.tigercodes.webtiger

import android.content.Context
import android.content.Intent
import android.view.View
import android.view.ViewGroup
import android.webkit.WebView
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
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
internal class AppThirdPartyCookieTest {
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

    private fun awaitPage(what: String, shown: (String) -> Boolean) {
        try {
            compose.waitUntil(15_000) { shown(pageTitle()) }
        } catch (_: androidx.compose.ui.test.ComposeTimeoutException) {
            throw AssertionError("the page that $what never showed; the window shows <${pageTitle()}>")
        }
    }

    /** Sends the tab in front to an address, and waits for its page by the title it comes to. */
    private fun go(address: String, what: String, shown: (String) -> Boolean) {
        compose.onNodeWithTag("address-field").performClick()
        compose.onNodeWithTag("address-field").performTextReplacement(address)
        compose.onNodeWithTag("address-field").performImeAction()
        awaitPage(what, shown)
    }

    /** The cookies each request for a path came with, from the first that this test made. */
    private fun cookies(path: String, after: Int): List<Set<String>> =
        site.headers(path).drop(after).map { said -> said["cookie"].orEmpty().split("; ").filter { it.isNotEmpty() }.toSet() }

    @Test
    fun aSiteInAnotherSitesPageIsNotSentItsCookies() {
        awaitPage("is the start") { it == "Home" }
        val kept = site.headers("/kept").size
        val pictures = site.headers("/pixel").size
        // The site, by its other name, keeps a name of this test's own: in a cookie, and in
        // one that says it may go anywhere. (A device keeps what pages keep between tests.)
        go("http://localhost:${site.port}/keep?v=grace", "says it has kept the name") { it == "Kept" }

        // Then a page of another site holds it in a frame, and a picture of it: its script
        // there has no cookie and can keep none, finds nothing stored (071), and by the
        // site's own record neither was asked for with a cookie.
        go(site.url("/holds-another-site"), "says what the frame has") { it == "frame: asked none/none, reads none, writes no, stored none" }
        assertEquals(listOf(emptySet<String>()), cookies("/kept", kept))
        assertEquals(listOf(emptySet<String>()), cookies("/pixel", pictures))

        // As itself, the site has what it kept. (And whatever other cookies an earlier test
        // left it by this name: a device keeps them.)
        go("http://localhost:${site.port}/kept", "says what the site has as itself") {
            it.startsWith("asked grace/grace, reads ") && it.endsWith(", writes yes, stored grace")
        }
        assertEquals(true, cookies("/kept", kept).last().containsAll(setOf("who=grace", "anywhere=grace")))
    }
}

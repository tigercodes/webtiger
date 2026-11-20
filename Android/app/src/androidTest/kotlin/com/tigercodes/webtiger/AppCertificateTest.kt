// Going on past a certificate that is not trusted (026) in the app itself on a device, in fake
// mode: the real core, the real WebView, the window on a real screen, and a site over https in
// this process whose certificate signs itself. The page is refused and says why; nothing of the
// site is asked for until the warning's button is pressed; then it loads, marked as not secure.
package com.tigercodes.webtiger

import android.content.Intent
import android.view.View
import android.view.ViewGroup
import android.webkit.WebView
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.assertTextEquals
import androidx.compose.ui.test.junit4.accessibility.enableAccessibilityChecks
import androidx.compose.ui.test.junit4.v2.createEmptyComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performImeAction
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTextReplacement
import androidx.compose.ui.test.tryPerformAccessibilityChecks
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
internal class AppCertificateTest {
    @get:Rule val compose = createEmptyComposeRule()

    private lateinit var scenario: ActivityScenario<MainActivity>

    /** The process's one site: the app's core starts once, and every window opens on it. */
    private val site get() = AppWindowTest.site

    /** A site over https whose certificate signs itself: this test's own, on a port of its own. */
    private val secure = SelfSignedSite()

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
        secure.close()
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

    private fun count(tag: String): Int = compose.onAllNodesWithTag(tag, useUnmergedTree = true).fetchSemanticsNodes().size

    /**
     * A site whose certificate signs itself is refused, and the page says why. Nothing on it
     * goes on by one press: "Advanced…" shows the warning, which names the site; and until the
     * button under the warning is pressed the site is never asked for its page. Pressed, the
     * page loads, and the mark beside its address says it is not secure. Android's
     * accessibility checks pass over the failed page and over the warning.
     *
     * The exception is this process's fake core's, for this test's own site on its own port,
     * which is closed when the test ends.
     */
    @Test
    fun aCertificateThatIsNotTrustedIsWarnedOfBeforeItIsGoneOnPast() {
        compose.enableAccessibilityChecks()
        awaitPage("Home")
        val address = "https://127.0.0.1:${secure.port}/"

        type(address)

        // Refused: the page says which trouble it is, and offers to try again or to say more.
        awaitTag("certificate-advanced")
        compose.onNodeWithTag("failed-words").assertTextEquals("This device doesn’t trust this site’s certificate.")
        compose.onAllNodesWithTag("certificate-go-on").assertCountEquals(0)
        // Nothing vouches for an address whose certificate was just refused.
        assertEquals("no padlock beside a refused certificate", 0, count("security-secure"))
        assertEquals("the site was not asked for its page", 0, secure.served.get())
        compose.onRoot().tryPerformAccessibilityChecks()

        compose.onNodeWithTag("certificate-advanced").performClick()

        // The warning, which names the site; and still nothing was asked of it.
        awaitTag("certificate-go-on")
        compose.onNodeWithTag("certificate-go-on").performScrollTo()
        compose.onNodeWithTag("certificate-go-on").assertTextEquals("Go On to 127.0.0.1 (Not Safe)")
        compose.onNodeWithTag("certificate-warning-words").assertTextEquals(
            "WebTiger can’t confirm that this is really 127.0.0.1. Someone could be pretending to be it, " +
                "and could read or change what you send and receive: passwords, messages, card numbers.",
        )
        assertEquals("a warning lets nothing through", 0, secure.served.get())
        compose.onRoot().tryPerformAccessibilityChecks()

        compose.onNodeWithTag("certificate-go-on").performClick()

        // Gone on: the site's page, asked for once, and marked as not secure.
        awaitPage("Trusted after all")
        assertEquals(1, secure.served.get())
        awaitTag("security-insecure")
        assertEquals("a page past a certificate is never shown as secure", 0, count("security-secure"))
        compose.onAllNodesWithTag("failed-page").assertCountEquals(0)
    }
}

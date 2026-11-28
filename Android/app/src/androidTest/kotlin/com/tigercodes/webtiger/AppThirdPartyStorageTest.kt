// A third party's storage (071) in the app, on a device, in fake mode: the real core and the
// real WebView. A site that has kept a name in every store it has finds none of it in another
// site's page, where the core says its frame is to be kept apart; with Guard turned off, the
// frame has what the WebView gives, which is everything; and turned on again, nothing. What
// the adapter does frame by frame is its suite's (SystemWebViewThirdPartyStorageTest). This is
// the app as it is put together.
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
import com.tigercodes.webtiger.core.CoreClient
import com.tigercodes.webtiger.core.v1.GuardSwitch
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.SetGuard
import com.tigercodes.webtiger.ui.LaunchOptions
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
internal class AppThirdPartyStorageTest {
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

    /** Sets Guard's switch for everywhere (067), as a person will from the window when it has a place for it. */
    private fun guardEverywhere(switch: GuardSwitch) {
        var running: CoreClient? = null
        scenario.onActivity { activity -> running = (activity.application as WebTigerApp).session(LaunchOptions(fake = true)).getOrNull()?.core }
        val core = checkNotNull(running) { "the app has no core" }
        // Asked from the test's own thread: the window's is left free to hear what follows.
        runBlocking { core.request(Request(set_guard = SetGuard(everywhere = true, switch = switch))) }
    }

    @Test
    fun aSiteInAnotherSitesPageHasNoneOfItsStoresUnlessGuardIsOff() {
        awaitPage("is the start") { it == "Home" }
        val other = "http://localhost:${site.port}"
        // The site, by its other name, keeps a name of this test's own in every store it
        // has, and says so when asked again. (A device keeps what pages keep between tests:
        // what it had before is not counted on.)
        go("$other/stores?v=hedy", "says what the site's stores had") { it.startsWith("local ") }
        go("$other/stores", "says what the site's stores have") { it == "local hedy, session hedy, database hedy, cache hedy, $THE_REST" }

        // In a page of another site, in the same tab: none of it, in any store.
        val holds = site.url("/holds-another-site?frame=stores")
        go(holds, "says the frame has nothing") { it == "frame: $APART" }
        try {
            // Guard off: the frame has what the WebView gives a third party, which is all
            // the site has as itself.
            guardEverywhere(GuardSwitch.GUARD_SWITCH_OFF)
            go(holds, "says the frame has what the site kept") { it == "frame: local hedy, session hedy, database hedy, cache hedy, $THE_REST" }
        } finally {
            // The app's core is every test's: left as it was found.
            guardEverywhere(GuardSwitch.GUARD_SWITCH_ON)
        }
        // On again: nothing, from the next page.
        go(holds, "says the frame has nothing again") { it == "frame: $APART" }
    }

    private companion object {
        /** What `/stores` says, after the stores a name is kept in, of a site as itself. */
        const val THE_REST = "files given, worker offered, shared absent"

        /** What `/stores` says in a frame that is kept apart. */
        const val APART = "local none, session none, database refused, cache refused, files refused, worker absent, shared absent"
    }
}

// The adapter suite's System WebView run (000 §8.3): the behaviours every engine adapter must
// show, against the device's real WebView and a site on its loopback address. The core is
// scripted, so what is tested is the adapter alone: what it reports, in what order, and what it
// refuses. The same behaviours as the WebKit run (Packages/Tests/WTWebKitTests).
package com.tigercodes.webtiger.webview

import androidx.test.ext.junit.runners.AndroidJUnit4
import com.tigercodes.webtiger.TestSite
import com.tigercodes.webtiger.core.NavigationVerdict
import com.tigercodes.webtiger.core.RequestVerdict
import com.tigercodes.webtiger.core.v1.EngineKind
import com.tigercodes.webtiger.core.v1.HostPing
import com.tigercodes.webtiger.core.v1.HostRequest
import com.tigercodes.webtiger.core.v1.NavigationCommand
import okio.ByteString.Companion.encodeUtf8
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
internal class SystemWebViewAdapterTest : AdapterTestCase() {
    // Event order

    @Test
    fun aLoadReportsStartedThenCommittedThenFinished() {
        create(1)
        assertNotNull("a load is acknowledged", load(1, site.url("/")).ack)
        await(1, "the page to finish") { finished(it) && hasTitle("Home")(it) }

        val order = names(reports(1)).filter { it in listOf("started", "committed", "finished") }
        assertEquals(listOf("started", "committed", "finished"), order)
        assertEquals(site.url("/"), reports(1).firstNotNullOf { it.navigation_started }.url)
        assertEquals(site.url("/"), reports(1).firstNotNullOf { it.navigation_committed }.url)
        assertEquals("Home", lastTitle(1))
        assertNotNull(webView(1))
    }

    @Test
    fun aRedirectCommitsWhereItLanded() {
        create(1)
        load(1, site.url("/redirect"))
        await(1, "the redirect to land") { finished(it) && hasTitle("Landed")(it) }
        assertEquals(listOf(site.url("/landed")), reports(1).mapNotNull { it.navigation_committed?.url })
    }

    @Test
    fun aPageChangingItsOwnAddressReportsAnAddressChange() {
        create(1)
        load(1, site.url("/push"))
        await(1, "the page to push its address") { reports -> reports.any { it.address_changed?.url == site.url("/pushed") } }
        assertEquals("a pushed address is not a new page", 1, reports(1).count { it.navigation_committed != null })
    }

    @Test
    fun backAndForwardReportHistory() {
        create(1)
        load(1, site.url("/"))
        await(1, "the first page") { finished(it) }
        load(1, site.url("/landed"))
        await(1, "the second page") { finished(it, times = 2) }
        val afterTwo = reports(1).last { it.history_changed != null }.history_changed
        assertEquals(true, afterTwo?.can_go_back)
        assertEquals(false, afterTwo?.can_go_forward)

        command(1, NavigationCommand.NAVIGATION_COMMAND_BACK)
        await(1, "going back") { reports -> reports.any { it.history_changed?.can_go_back == false && it.history_changed?.can_go_forward == true } }
        await(1, "the first page's title again") { lastTitle(1) == "Home" }
        assertEquals("going back is to a page that committed", site.url("/"), addresses(1).last())

        command(1, NavigationCommand.NAVIGATION_COMMAND_FORWARD)
        await(1, "going forward") { lastTitle(1) == "Landed" && addresses(1).last() == site.url("/landed") }
    }

    @Test
    fun aLinkThePageFollowsIsANewPage() {
        create(1)
        load(1, site.url("/"))
        await(1, "the page") { finished(it) }
        run(1, "document.getElementById('next').click();")
        await(1, "the link's page") { finished(it, times = 2) && hasTitle("Landed")(it) }
        assertEquals(listOf(site.url("/"), site.url("/landed")), reports(1).mapNotNull { it.navigation_committed?.url })
        assertEquals("the link's load is announced before it commits", 2, reports(1).count { it.navigation_started != null })
    }

    // Failures

    @Test
    fun aConnectionThatIsRefusedReportsACodeAndNoText() {
        create(1)
        val nowhere = "http://127.0.0.1:${TestSite.unusedPort()}/"
        load(1, nowhere)
        await(1, "the load to fail") { failed(it) }
        assertEquals("net.connect", reports(1).firstNotNullOf { it.navigation_failed }.code)
        // The engine then shows a page of its own. Whatever it reports about that, the tab is
        // not told that a page finished, and is given no title that is the engine's words.
        Thread.sleep(1_000)
        assertFalse("a failed load never finishes", names(reports(1)).contains("finished"))
        assertFalse("the engine's error page has no title of the page's", names(reports(1)).contains("title"))
        assertTrue("the only address reported is the one that failed", addresses(1).all { it == nowhere })
        assertEquals("the failure is reported once", 1, reports(1).count { it.navigation_failed != null })
    }

    @Test
    fun aNameThatDoesNotResolveReportsDns() {
        create(1)
        // RFC 6761 reserves .invalid: it never resolves.
        load(1, "http://webtiger-test.invalid/")
        await(1, "the load to fail") { failed(it) }
        assertEquals("net.dns", reports(1).firstNotNullOf { it.navigation_failed }.code)
    }

    @Test
    fun tryingAgainAfterAFailureLoadsTheAddressAgain() {
        create(1)
        val nowhere = "http://127.0.0.1:${TestSite.unusedPort()}/"
        load(1, nowhere)
        await(1, "the load to fail") { failed(it) }
        command(1, NavigationCommand.NAVIGATION_COMMAND_RELOAD)
        await(1, "the second failure") { reports -> reports.count { it.navigation_failed != null } == 2 }
        assertEquals(listOf(nowhere, nowhere), reports(1).mapNotNull { it.navigation_started?.url })
    }

    @Test
    fun stoppingALoadReportsItsEnd() {
        create(1)
        load(1, site.url("/slow"))
        await(1, "the load to start") { reports -> reports.any { it.navigation_started != null } }
        command(1, NavigationCommand.NAVIGATION_COMMAND_STOP)
        await(1, "the stop") { failed(it) }
        assertEquals("net.cancelled", reports(1).firstNotNullOf { it.navigation_failed }.code)
        Thread.sleep(500)
        assertFalse("a stopped load commits nothing", names(reports(1)).contains("committed"))
    }

    // The navigation policy

    @Test
    fun aNavigationTheCoreRefusesNeverCommits() {
        core.setVerdict(NavigationVerdict.CANCEL, site.url("/landed"))
        create(1)
        load(1, site.url("/"))
        await(1, "the page") { finished(it) }
        run(1, "document.getElementById('next').click();")
        await(1, "the refusal") { reports -> reports.any { it.navigation_failed?.code == "net.cancelled" } }
        Thread.sleep(500)
        assertEquals("the page on screen is still the page on screen", listOf(site.url("/")), addresses(1))
        assertEquals("the refused navigation is reported as ended: ${names(reports(1))}", "failed(net.cancelled)", names(reports(1)).last { it != "title" && it != "history" })
    }

    @Test
    fun aRedirectTheCoreRefusesEndsTheLoad() {
        core.setVerdict(NavigationVerdict.CANCEL, site.url("/landed"))
        create(1)
        load(1, site.url("/redirect"))
        await(1, "the refusal") { reports -> reports.any { it.navigation_failed?.code == "net.cancelled" } }
        Thread.sleep(500)
        assertEquals("an address that never committed is never reported as the page's", emptyList<String>(), addresses(1))
        assertFalse(names(reports(1)).contains("finished"))
    }

    @Test
    fun anExternalAddressIsNotLoaded() {
        core.setVerdict(NavigationVerdict.EXTERNAL, "mailto:ada@example.com")
        create(1)
        load(1, site.url("/"))
        await(1, "the page") { finished(it) }
        run(1, "location.href = 'mailto:ada@example.com';")
        await(1, "the refusal") { reports -> reports.any { it.navigation_failed?.code == "net.cancelled" } }
        Thread.sleep(300)
        assertEquals("the page on screen is still the page on screen", listOf(site.url("/")), addresses(1))
    }

    @Test
    fun anAddressTheCoreWouldNotLoadIsNotGivenToTheWebView() {
        // WebView.loadUrl runs a `javascript:` address as script in the page that is showing, so
        // the engine asks the policy itself before every load it is asked for.
        core.setVerdict(NavigationVerdict.CANCEL, "javascript:document.title='run'")
        create(1)
        load(1, site.url("/"))
        await(1, "the page") { finished(it) }
        assertEquals("webview.bad_address", load(1, "javascript:document.title='run'").error?.code)
        Thread.sleep(300)
        assertEquals("Home", lastTitle(1))
    }

    @Test
    fun anAddressIsReportedOnlyOnceItHasCommitted() {
        create(1)
        load(1, site.url("/"))
        await(1, "the first page") { finished(it) }
        load(1, site.url("/redirect"))
        await(1, "the second page") { finished(it, times = 2) }
        // Every address reported as the page's is one that committed: never the address asked
        // for, and never the redirect it passed through.
        assertEquals(listOf(site.url("/"), site.url("/landed")), addresses(1))
    }

    // Guard's callback

    @Test
    fun everyRequestAPageMakesIsPutToTheCore() {
        create(1)
        load(1, site.url("/tracked"))
        await(1, "the page") { finished(it) }
        assertTrue("with nothing blocked, the page's image is fetched", site.requests.contains("/pixel"))

        core.setVerdict(RequestVerdict.BLOCK, site.url("/pixel"))
        create(2)
        val before = site.requests.count { it == "/pixel" }
        load(2, site.url("/tracked"))
        await(2, "the page") { finished(it) }
        assertEquals("a request the core blocks never leaves the device", before, site.requests.count { it == "/pixel" })
    }

    // A renderer that dies

    @Test
    fun aRendererThatDiesIsReportedAndTheNextLoadMakesAnother() {
        create(1)
        load(1, site.url("/"))
        await(1, "the page") { finished(it) }
        val terminated = onMain { (engine.webView(1) as? android.webkit.WebView)?.webViewRenderProcess?.terminate() }
        assertEquals("the renderer could be terminated", true, terminated)
        await(1, "the renderer's death") { reports -> reports.any { it.renderer_gone != null } }
        assertNull("a web view whose renderer died is not shown again", webView(1))

        // Reload: the engine has nothing of its own to reload, so the address is loaded again,
        // in a new web view.
        command(1, NavigationCommand.NAVIGATION_COMMAND_RELOAD)
        await(1, "the page again") { finished(it, times = 2) }
        assertEquals(listOf(site.url("/"), site.url("/")), reports(1).mapNotNull { it.navigation_committed?.url })
        assertNotNull(webView(1))
    }

    // Host requests

    @Test
    fun requestsAboutWebViewsThatDoNotExistAreRefusedWithCodes() {
        assertEquals("webview.unknown", load(42, site.url("/")).error?.code)
        assertEquals("webview.unknown", command(42, NavigationCommand.NAVIGATION_COMMAND_BACK).error?.code)
        assertEquals("engine.unavailable", create(1, kind = EngineKind.ENGINE_KIND_WEBKIT).error?.code)
        assertNotNull(create(2).ack)
        assertEquals("webview.exists", create(2).error?.code)
        assertEquals("webview.bad_login_set", create(3, loginSet = "not-a-uuid").error?.code)
        assertEquals("webview.bad_command", command(2, NavigationCommand.NAVIGATION_COMMAND_UNSPECIFIED).error?.code)
    }

    @Test
    fun destroyingAWebViewReleasesIt() {
        create(1)
        assertEquals(1, onMain { engine.webViewCount })
        destroy(1)
        assertEquals(0, onMain { engine.webViewCount })
        assertNull(webView(1))
        assertNotNull("destroying it again is not an error", destroy(1).ack)
    }

    @Test
    fun thePingIsAnswered() {
        val answer = answer(HostRequest(ping = HostPing(payload = "hello".encodeUtf8())))
        assertEquals("hello".encodeUtf8(), answer.pong?.payload)
    }

    /**
     * Loads begun back to back, each the moment the one before is over (003 §8.1): every one
     * is a load, and none is said to have been cancelled. The engine's last words about a
     * load now and then arrive after the adapter has begun the next; they end nothing.
     */
    @Test
    fun loadsBegunBackToBackAreEachALoad() {
        create(1)
        val rounds = 100
        repeat(rounds) { round ->
            load(1, site.url("/landed"))
            // Looked at without a pause: the next load is begun the moment this one is over.
            val until = System.nanoTime() + 15_000_000_000
            while (reports(1).count { it.navigation_finished != null || it.navigation_failed != null } <= round) {
                if (System.nanoTime() > until) throw AssertionError("load ${round + 1} was never over; heard ${heard(1)}")
            }
            val failures = reports(1).mapNotNull { it.navigation_failed?.code }
            if (failures.isNotEmpty()) throw AssertionError("load ${round + 1} was said to have failed ($failures); heard ${heard(1)}")
        }
        assertEquals("each load finished, once", rounds, reports(1).count { it.navigation_finished != null })
        assertEquals("and each committed", rounds, reports(1).count { it.navigation_committed != null })
    }
}

// The adapter suite's System WebView run, for a link for the system (007): a letter to write, a
// number to ring, a message to send. The adapter refuses such an address in the web view, as
// before, and reports it; it opens nothing. Whether the address is handed to the system is the
// core's to say, and is tested there. The same behaviours as the WebKit run
// (WebKitOutsideLinkTests), with a finger on the page.
package com.tigercodes.webtiger.webview

import androidx.test.ext.junit.runners.AndroidJUnit4
import com.tigercodes.webtiger.core.NavigationVerdict
import com.tigercodes.webtiger.core.v1.OutsideLinkRequested
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
internal class SystemWebViewOutsideLinkTest : AdapterTestCase() {
    /** As the core's policy does: these are for the system. */
    private fun theSystems(vararg urls: String) {
        for (url in urls) core.setVerdict(NavigationVerdict.EXTERNAL, url)
    }

    /** The links for the system a web view's page has asked for, in order. */
    private fun outsideLinks(id: Long): List<OutsideLinkRequested> = reports(id).mapNotNull { it.outside_link_requested }

    private fun contact(id: Long = 1) {
        theSystems("mailto:ada@example.com", "tel:+15550100", "sms:+15550100")
        create(id)
        load(id, site.url("/contact"))
        await(id, "the page to load") { finished(it) && hasTitle("Contact")(it) }
    }

    @Test
    fun aTouchedLinkIsReportedBeforeItsRefusal() {
        contact()
        val before = names(reports(1)).size

        tap(1, "letter")
        await(1, "the link to be reported") { outsideLinks(1).isNotEmpty() }

        assertEquals(listOf("mailto:ada@example.com" to true), outsideLinks(1).map { it.url to it.user_gesture })
        // Reported first, then refused like any navigation the policy does not allow: the core
        // must hear of the link before the refusal that may close a window (006).
        await(1, "the refusal") { failed(it) }
        assertEquals(listOf("outside", "failed(net.cancelled)"), names(reports(1)).drop(before))
        assertEquals("said to be a refusal", true, reports(1).last { it.navigation_failed != null }.navigation_failed?.refused)
        // The page is where it was.
        assertEquals("Contact", lastTitle(1))
        assertEquals("\"/contact\"", run(1, "return location.pathname;"))
    }

    @Test
    fun aButtonWhoseScriptGoesThereIsReportedAsAskedFor() {
        contact()

        tap(1, "script")
        await(1, "the link to be reported") { outsideLinks(1).isNotEmpty() }

        // The WebView says a navigation a script starts while a finger is on the page was
        // asked for, which WebKit does not.
        assertEquals(listOf("tel:+15550100" to true), outsideLinks(1).map { it.url to it.user_gesture })
    }

    @Test
    fun aLinkInAFrameIsReportedToo() {
        contact()
        val before = names(reports(1)).size

        tap(1, "frame")
        await(1, "the link to be reported") { outsideLinks(1).isNotEmpty() }
        Thread.sleep(300)

        assertEquals(listOf("sms:+15550100" to true), outsideLinks(1).map { it.url to it.user_gesture })
        // Nothing of the page itself was going anywhere, and nothing is said of it.
        assertEquals(listOf("outside"), names(reports(1)).drop(before))
    }

    @Test
    fun aPageThatGoesThereByItselfIsReportedAsNotAskedFor() {
        theSystems("tel:+15550100")
        create(1)
        load(1, site.url("/rings"))
        await(1, "the page to have tried") { outsideLinks(1).isNotEmpty() }

        assertEquals("nobody touched it", listOf(false), outsideLinks(1).map { it.user_gesture })
    }

    @Test
    fun aLinkThatOpensAWindowOnSuchAnAddressReportsItInTheWindow() {
        theSystems("mailto:ada@example.com")
        create(1)
        load(1, site.url("/opener"))
        await(1, "the page to load") { finished(it) && hasTitle("Opener")(it) }

        tap(1, "letter")
        val new = awaitWindow(askedBy = 1).new_web_view_id
        await(new, "the refusal") { failed(it) }

        // The WebView makes the window first and asks about the address in it. The link is
        // reported there, before the refusal the core closes the window on; the touch that
        // opened the window is what the core counts (007 §3).
        assertEquals(listOf("mailto:ada@example.com"), outsideLinks(new).map { it.url })
        assertEquals(listOf("outside", "failed(net.cancelled)"), names(reports(new)).filter { it == "outside" || it.startsWith("failed") })
        assertTrue("the page that asked said only that it asked for a window", outsideLinks(1).isEmpty())
    }
}

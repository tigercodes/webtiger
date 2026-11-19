// The adapter suite's System WebView run, for a page that asks for a new window (006): the one
// web view an adapter makes unasked. What is tested is the adapter alone: that it makes the web
// view the WebView asks for, names it, reports it, and keeps the hold between the two pages.
// What becomes of the web view is the core's to say, and is tested there. The same behaviours
// as the WebKit run (WebKitNewWindowTests).
package com.tigercodes.webtiger.webview

import androidx.test.ext.junit.runners.AndroidJUnit4
import com.tigercodes.webtiger.core.NavigationVerdict
import com.tigercodes.webtiger.core.v1.NavigationCommand
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
internal class SystemWebViewNewWindowTest : AdapterTestCase() {
    /** A web view on the page that asks for windows when touched. */
    private fun opener(id: Long = 1, loginSet: String = newLoginSet()) {
        create(id, loginSet)
        load(id, site.url("/opener"))
        await(id, "the opener to load") { finished(it) && hasTitle("Opener")(it) }
    }

    @Test
    fun aLinkWithATargetAsksForAWindow() {
        opener()
        val before = names(reports(1))

        tap(1, "blank")
        val asked = awaitWindow(askedBy = 1)

        val new = asked.new_web_view_id
        assertNotEquals("the adapter named the web view, with an id of the shell's own", 0L, new and SystemWebViewEngine.OWN_ID)
        assertTrue("somebody touched the link", asked.user_gesture)
        assertEquals("the WebView does not say where the page wants the window to go", "", asked.url)
        assertNotNull("the web view is there before the core has said a word", webView(new))
        assertEquals(2, onMain { engine.webViewCount })

        // The address loads in the new web view, reported like any other's.
        await(new, "the new window's page") { finished(it) && hasTitle("Opened")(it) }
        val order = names(reports(new)).filter { it in listOf("started", "committed", "finished") }
        assertEquals(listOf("started", "committed", "finished"), order)
        assertEquals(listOf(site.url("/opened")), reports(new).mapNotNull { it.navigation_committed?.url })
        // A link gives the page it opens no hold on the page it was in: only window.open does.
        assertEquals("the new page found no opener", "Opened", lastTitle(new))

        // The web view that asked went nowhere: the one thing it has said since is that it asked.
        assertEquals(listOf("window"), names(reports(1)).drop(before.size))
        assertEquals("Opener", lastTitle(1))
    }

    @Test
    fun aWindowOpenedByScriptKeepsTheHoldOfThePageThatOpenedIt() {
        opener()

        tap(1, "open")
        val new = awaitWindow(askedBy = 1).new_web_view_id

        // The new page has an opener, and says so; and it reached the page that opened it.
        await(new, "the new page to find its opener") { hasTitle("Opened by a page")(it) }
        await(1, "the opener to hear from the window it opened") { hasTitle("heard=hello")(it) }
    }

    @Test
    fun theNewWebViewIsInTheLoginSetOfThePageThatAsked() {
        val loginSet = newLoginSet()
        create(1, loginSet)
        load(1, site.url("/set?v=ada"))
        await(1, "the sign-in") { finished(it) && hasTitle("Set")(it) }
        load(1, site.url("/opener"))
        await(1, "the opener to load") { hasTitle("Opener")(it) }

        tap(1, "open")
        val new = awaitWindow(askedBy = 1).new_web_view_id
        await(new, "the new window's page") { hasTitle("Opened by a page")(it) }

        // It is a web view like any other: the core's requests about it are answered, and its
        // pages are signed in as the page that asked is.
        assertNotNull("a load in it is acknowledged", load(new, site.url("/whoami")).ack)
        await(new, "the new window to say who is signed in") { hasTitle("who=ada")(it) }

        // Another login set's page that asks gets a web view in its own.
        opener(2)
        tap(2, "open")
        val other = awaitWindow(askedBy = 2).new_web_view_id
        assertNotEquals("no id is used twice", new, other)
        await(other, "the other window's page") { hasTitle("Opened by a page")(it) }
        load(other, site.url("/whoami"))
        await(other, "the other window to say nobody is signed in") { hasTitle("who=none")(it) }
    }

    @Test
    fun aPageAskingForItsWindowToBeClosedIsReported() {
        opener()
        tap(1, "open")
        val new = awaitWindow(askedBy = 1).new_web_view_id
        await(new, "the new page") { finished(it) && hasTitle("Opened by a page")(it) }

        tap(new, "close")

        await(new, "the page to ask for its window to be closed") { "close" in names(it) }
        // Asking is all a page can do: the web view is the core's to destroy.
        assertNotNull(webView(new))
    }

    @Test
    fun aWindowAskedForWithNobodyTouchingIsNotMade() {
        create(1)
        load(1, site.url("/pops"))
        // The page says what window.open gave it: nothing.
        await(1, "the page to have asked") { finished(it) && hasTitle("blocked")(it) }

        assertEquals("nothing was asked of the core", 0, windowsAsked(by = 1).size)
        assertEquals("and no web view was made", 1, onMain { engine.webViewCount })
    }

    @Test
    fun aPageThatClicksItsOwnLinkGetsNoWindow() {
        create(1)
        load(1, site.url("/clicks"))
        await(1, "the page to have clicked") { finished(it) && hasTitle("clicked")(it) }
        Thread.sleep(500)

        assertEquals("nothing was asked of the core", 0, windowsAsked(by = 1).size)
        assertEquals("and no web view was made", 1, onMain { engine.webViewCount })
    }

    @Test
    fun aWindowTheCoreRefusesIsDestroyed() {
        opener()
        tap(1, "open")
        val new = awaitWindow(askedBy = 1).new_web_view_id
        assertNotNull(webView(new))

        // What the core sends for a window it refuses.
        assertNotNull(destroy(new).ack)

        assertNull(webView(new))
        assertEquals(1, onMain { engine.webViewCount })
        // The page that asked can ask again; the next window has a new id. (That page is not
        // said to be as it was: the WebView makes a window and starts its page before the core
        // can refuse it, and once in a few dozen runs the page opened had already run, and
        // spoken to the page that opened it, by the time it was destroyed. 006 §8.)
        tap(1, "open")
        val next = awaitWindow(askedBy = 1, count = 2).new_web_view_id
        assertNotEquals(new, next)
        assertNotEquals(0L, next and SystemWebViewEngine.OWN_ID)
    }

    @Test
    fun theNavigationPolicyIsAskedAboutWhereANewWindowGoes() {
        opener()
        val before = names(reports(1))
        // The core's policy refuses where the link leads.
        core.setVerdict(NavigationVerdict.CANCEL, site.url("/opened"))

        tap(1, "blank")

        // The WebView asks for the window before it says where it goes, so the web view is
        // made; its load is then refused, and reported as cancelled, and as a refusal: the
        // core closes the tab of a window that was refused the only place it was going.
        val new = awaitWindow(askedBy = 1).new_web_view_id
        await(new, "the refused load to be reported") { reports -> reports.any { it.navigation_failed != null } }
        assertEquals(listOf("net.cancelled" to true), reports(new).mapNotNull { it.navigation_failed }.map { it.code to it.refused })
        assertEquals("nothing committed in it", emptyList<String>(), addresses(new))
        assertEquals("the page that asked went nowhere", listOf("window"), names(reports(1)).drop(before.size))
    }

    @Test
    fun aWindowRedirectedToARefusedAddressSaysItWasRefused() {
        opener()
        // Where the window is first sent is allowed; where the server sends it on to is not.
        core.setVerdict(NavigationVerdict.CANCEL, site.url("/landed"))
        show(1)
        run(1, "document.getElementById('open').setAttribute('onclick', \"window.open('/redirect')\");")

        tap(1, "open")
        val new = awaitWindow(askedBy = 1).new_web_view_id

        await(new, "the load's end") { reports -> reports.any { it.navigation_failed?.refused == true } }
        val failures = reports(new).mapNotNull { it.navigation_failed }
        assertTrue(failures.all { it.code == "net.cancelled" })
        assertEquals("the last word on the load is that it was refused", true, failures.last().refused)
        assertEquals(emptyList<String>(), addresses(new))
    }

    @Test
    fun aLoadThatIsStoppedIsNotARefusal() {
        create(1)
        load(1, site.url("/slow"))
        await(1, "the load to start") { reports -> reports.any { it.navigation_started != null } }

        command(1, NavigationCommand.NAVIGATION_COMMAND_STOP)

        await(1, "the load's end") { failed(it) }
        assertEquals(listOf("net.cancelled" to false), reports(1).mapNotNull { it.navigation_failed }.map { it.code to it.refused })
    }
}

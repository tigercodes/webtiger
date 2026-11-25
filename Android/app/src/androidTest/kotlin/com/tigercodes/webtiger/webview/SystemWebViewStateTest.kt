// A web view's state through the adapter (051), against the device's real WebView: what the
// core is handed when it asks for one, and what a web view made from it is. The core is
// scripted, and keeps nothing: the state goes from one web view to the next in the test's own
// hands, as the core's store would carry it. What the WebView's state is in itself is held by
// SystemWebViewRestoreTest; here it is the adapter's part: asking, handing back, falling back
// to the address, and what is reported on the way. The Apple suite's WebKitStateTests.swift,
// for Android.
package com.tigercodes.webtiger.webview

import com.tigercodes.webtiger.core.NavigationVerdict
import com.tigercodes.webtiger.core.v1.HistoryChanged
import com.tigercodes.webtiger.core.v1.HostRequest
import com.tigercodes.webtiger.core.v1.HostResponse
import com.tigercodes.webtiger.core.v1.NavigationCommand
import com.tigercodes.webtiger.core.v1.WebViewRestore
import com.tigercodes.webtiger.core.v1.WebViewSaveState
import okio.ByteString
import okio.ByteString.Companion.encodeUtf8
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

internal class SystemWebViewStateTest : AdapterTestCase() {
    /** The state of a web view, as the core asks for it. Null when the adapter answers with anything but a state. */
    private fun state(id: Long): ByteString? = answer(HostRequest(web_view_save_state = WebViewSaveState(web_view_id = id))).web_view_state?.state

    /** Has a web view made from a state, with an address to fall back on, as the core asks. */
    private fun restore(id: Long, state: ByteString, orLoad: String): HostResponse =
        answer(HostRequest(web_view_restore = WebViewRestore(web_view_id = id, state = state, url = orLoad)))

    private fun commits(id: Long): List<String> = reports(id).mapNotNull { it.navigation_committed?.url }

    /** What the adapter last said of a web view's Back and Forward. */
    private fun history(id: Long): HistoryChanged? = reports(id).lastOrNull { it.history_changed != null }?.history_changed

    /** A web view in a window, so that its page has a size and can be scrolled. */
    private fun createShown(id: Long, loginSet: String) {
        assertNotNull(create(id, loginSet).ack)
        show(id)
    }

    /** Goes through Home and Landed to the tall page in web view 1, each by a load of its own. */
    private fun goToTheTallPage(loginSet: String) {
        createShown(1, loginSet)
        listOf("/", "/landed", "/tall").forEachIndexed { number, path ->
            load(1, site.url(path))
            await(1, "$path to finish") { finished(it, number + 1) && commits(1).size == number + 1 }
        }
    }

    /** The state of web view 1 at the tall page, with web view 1 gone: the tab sleeps. */
    private fun asleepAtTheTallPage(loginSet: String): ByteString {
        goToTheTallPage(loginSet)
        val kept = checkNotNull(state(1)) { "a state is answered with a state" }
        destroy(1)
        return kept
    }

    // A state given and taken

    /**
     * A web view made from a state names its page after the page has committed: the core takes
     * a title to be the committed page's, and one said before the commit is lost. (The WebView
     * says the title of a page it makes from a state before that page commits, twice, and not
     * after: the adapter says it again.) A tab that woke was named by its address.
     */
    @Test
    fun aWebViewMadeFromAStateNamesItsPageAfterThePageHasCommitted() {
        val loginSet = newLoginSet()
        goToTheTallPage(loginSet)
        val kept = checkNotNull(state(1)) { "a state is answered with a state" }
        destroy(1)

        createShown(2, loginSet)
        assertNotNull("a restore is acknowledged", restore(2, kept, site.url("/tall")).ack)
        await(2, "the page it was at") { finished(it) && hasTitle("Tall")(it) }
        val said = names(reports(2)).filter { it in setOf("committed", "title") }
        assertEquals("one page committed", 1, said.count { it == "committed" })
        assertEquals("and the last thing said of its name is after it: $said", "title", said.last())
        assertEquals("Tall", lastTitle(2))
        // A page that is loaded names itself after its commit, as it did.
        createShown(3, loginSet)
        load(3, site.url("/landed"))
        await(3, "the page") { finished(it) && hasTitle("Landed")(it) }
        assertEquals(listOf("committed", "title"), names(reports(3)).filter { it in setOf("committed", "title") })
    }

    @Test
    fun aWebViewMadeFromAStateIsWhereTheOtherWasWithItsListAndItsScroll() {
        val loginSet = newLoginSet()
        goToTheTallPage(loginSet)
        assertEquals("2400", run(1, "window.scrollTo(0, 2400); return Math.round(window.scrollY);"))
        // The WebView writes where a page is scrolled to into its list about a second after the scroll.
        Thread.sleep(2500)
        val kept = checkNotNull(state(1)) { "a state is answered with a state" }
        assertTrue("the WebView has a state to give", kept.size > 0)
        assertTrue("under the most the core keeps (051 D3): ${kept.size} bytes", kept.size < 512 * 1024)
        // The tab sleeps: its web view is gone before another is made.
        destroy(1)

        createShown(2, loginSet)
        assertNotNull("a restore is acknowledged", restore(2, kept, site.url("/tall")).ack)
        await(2, "the page it was at") { finished(it) && hasTitle("Tall")(it) }

        // It arrives as any page does: a load that starts and commits, once, at the page the
        // other web view was at. Nothing said its address before it had committed.
        val order = names(reports(2)).filter { it in setOf("started", "committed", "finished", "address") }
        assertEquals(listOf("started", "committed", "finished"), order)
        assertEquals(listOf(site.url("/tall")), commits(2))
        // The list came with it: Back goes back, and there is nowhere forward.
        val said = checkNotNull(history(2)) { "its Back and Forward were said" }
        assertTrue(said.can_go_back)
        assertFalse(said.can_go_forward)
        // And it is where it was scrolled to.
        var down = "0"
        for (attempt in 0 until 300) {
            down = run(2, "return Math.round(window.scrollY);").orEmpty()
            if (down != "0" && down.isNotEmpty()) break
            Thread.sleep(10)
        }
        assertEquals("2400", down)

        // Back goes to the page before it, which a web view that was only loaded could not.
        command(2, NavigationCommand.NAVIGATION_COMMAND_BACK)
        await(2, "the page before") { commits(2).size == 2 && finished(it, 2) }
        assertEquals(listOf(site.url("/tall"), site.url("/landed")), commits(2))
        assertEquals(true, history(2)?.can_go_forward)
    }

    @Test
    fun aStateTakenInTheMiddleOfTheListHasSomewhereToGoForward() {
        val loginSet = newLoginSet()
        goToTheTallPage(loginSet)
        command(1, NavigationCommand.NAVIGATION_COMMAND_BACK)
        // Its fourth page: the one before, again.
        await(1, "the page before") { commits(1).size == 4 && finished(it, 4) }
        assertEquals(site.url("/landed"), commits(1).last())
        val kept = checkNotNull(state(1))
        destroy(1)

        createShown(2, loginSet)
        assertNotNull(restore(2, kept, site.url("/landed")).ack)
        await(2, "the page it was at") { finished(it) && hasTitle("Landed")(it) }
        assertEquals(listOf(site.url("/landed")), commits(2))
        val said = checkNotNull(history(2))
        assertTrue(said.can_go_back)
        assertTrue(said.can_go_forward)
    }

    // A state that is not followed

    @Test
    fun aStateTheWebViewWillNotTakeLoadsTheAddress() {
        val loginSet = newLoginSet()
        val kept = asleepAtTheTallPage(loginSet)
        // Bytes that are no state of anything; a state cut short; and none at all.
        val bad = listOf("no state of the WebView's".encodeUtf8(), kept.substring(0, kept.size / 2), ByteString.EMPTY)
        bad.forEachIndexed { number, state ->
            val id = 10L + number
            createShown(id, loginSet)
            assertNotNull("state $number: the tab wakes all the same", restore(id, state, site.url("/landed")).ack)
            await(id, "state $number: the address to load") { finished(it) && hasTitle("Landed")(it) }
            assertEquals("state $number", listOf(site.url("/landed")), commits(id))
            val said = checkNotNull(history(id))
            assertFalse("state $number: a web view that was loaded has been nowhere else", said.can_go_back)
            assertFalse(said.can_go_forward)
        }
    }

    /**
     * The WebView asks the app about no load that goes through its own list, so the adapter
     * follows a state only to the page the core named with it: the one its policy was asked
     * about. A state that is at another page is let go with the web view that took it.
     */
    @Test
    fun aStateThatIsAtAnotherPageThanTheOneNamedIsNotFollowed() {
        val loginSet = newLoginSet()
        val kept = asleepAtTheTallPage(loginSet)

        createShown(2, loginSet)
        assertNotNull(restore(2, kept, site.url("/landed")).ack)
        await(2, "the address to load") { finished(it) && hasTitle("Landed")(it) }
        assertEquals("the tall page, where the state is, was not gone to", listOf(site.url("/landed")), commits(2))
        val said = checkNotNull(history(2))
        assertFalse("nothing of the state's list is in the web view that loaded the address", said.can_go_back)
        assertFalse(said.can_go_forward)
    }

    @Test
    fun aPageTheCoresPolicyRefusesIsNotGoneToByAState() {
        val loginSet = newLoginSet()
        val kept = asleepAtTheTallPage(loginSet)

        core.setVerdict(NavigationVerdict.CANCEL, site.url("/tall"))
        createShown(2, loginSet)
        assertEquals("webview.bad_address", restore(2, kept, site.url("/tall")).error?.code)
        Thread.sleep(500)
        assertEquals(emptyList<String>(), commits(2))
        assertFalse("an address that never committed is never reported as the page's", names(reports(2)).contains("address"))
    }

    @Test
    fun aWebViewThereIsNotHasNoStateAndTakesNone() {
        val asked = answer(HostRequest(web_view_save_state = WebViewSaveState(web_view_id = 99)))
        assertEquals("webview.unknown", asked.error?.code)
        assertNull(asked.web_view_state)
        assertEquals("webview.unknown", restore(99, "anything".encodeUtf8(), site.url("/")).error?.code)
    }

    @Test
    fun aWebViewThatHasBeenNowhereHasNoState() {
        createShown(1, newLoginSet())
        assertEquals("answered, with none to give", 0, checkNotNull(state(1)).size)
    }

    // Kept apart

    @Test
    fun aStateDoesNotCarryItsLoginSetWithIt() {
        // A state is a list of pages, not who is signed in: made into a web view of another
        // login set, it is that login set's page.
        val ada = newLoginSet()
        val grace = newLoginSet()
        createShown(1, ada)
        load(1, site.url("/set?v=ada"))
        await(1, "the cookie to be set") { finished(it) }
        load(1, site.url("/whoami"))
        await(1, "who is asked") { hasTitle("who=ada")(it) && finished(it, 2) }
        val kept = checkNotNull(state(1))
        destroy(1)

        createShown(2, grace)
        assertNotNull(restore(2, kept, site.url("/whoami")).ack)
        await(2, "who is asked, in the other login set") { finished(it) && commits(2).isNotEmpty() }
        assertEquals("the other login set is signed in as nobody", "who=none", lastTitle(2))
        // And in its own, it is still signed in.
        createShown(3, ada)
        assertNotNull(restore(3, kept, site.url("/whoami")).ack)
        await(3, "who is asked, in its own") { hasTitle("who=ada")(it) }
    }
}

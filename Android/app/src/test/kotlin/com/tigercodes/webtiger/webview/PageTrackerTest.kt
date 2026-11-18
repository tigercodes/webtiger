// What a System WebView's callbacks become. Each test replays the callbacks the WebView makes
// in one situation, in the order its source gives them (003 §5), and says what the core is told.
// None of these sequences was recorded from a device: no device was available (003 §8).
package com.tigercodes.webtiger.webview

import com.tigercodes.webtiger.core.v1.EngineEvent
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

private const val A = "https://example.com/"
private const val B = "https://www.example.com/home"
private const val X = "https://other.example.org/page"

class PageTrackerTest {
    private val events = ArrayList<EngineEvent>()
    private val tracker = PageTracker(7) { events.add(it) }

    /** What has been reported since this was last asked, each event as a short word. */
    private fun told(): List<String> = events.map(::word).also { events.clear() }

    private fun word(event: EngineEvent): String {
        assertEquals("every report names the web view", 7L, event.web_view_id)
        event.navigation_started?.let { return "started ${it.url}".trim() }
        event.navigation_committed?.let { return "committed ${it.url}" }
        event.navigation_finished?.let { return "finished" }
        event.navigation_failed?.let { return "failed ${it.code} ${it.url}".trim() }
        event.address_changed?.let { return "address ${it.url}" }
        event.title_changed?.let { return "title ${it.title}" }
        event.progress_changed?.let { return "progress ${it.fraction}" }
        event.history_changed?.let { return "history ${it.can_go_back} ${it.can_go_forward}" }
        event.renderer_gone?.let { return "gone" }
        return "nothing"
    }

    /** A page the user loaded, all the way, as a current WebView reports it. */
    private fun load(url: String, canGoBack: Boolean = false) {
        tracker.willLoad(url)
        tracker.progress(10)
        tracker.pageStarted(url)
        tracker.committed(url, canGoBack, canGoForward = false)
        tracker.progress(100)
        tracker.pageFinished(url)
        events.clear()
    }

    @Test
    fun aLoadTheUserAskedFor() {
        tracker.willLoad(A)
        assertEquals(listOf("started $A"), told())
        tracker.progress(10)
        // A current WebView says "page started" when the page commits, not before.
        tracker.pageStarted(A)
        tracker.committed(A, canGoBack = false, canGoForward = false)
        tracker.title("Example Domain")
        tracker.progress(70)
        tracker.progress(100)
        tracker.pageFinished(A)
        assertEquals(
            listOf("progress 0.1", "committed $A", "history false false", "title Example Domain", "progress 0.7", "progress 1.0", "finished"),
            told(),
        )
        assertEquals(A, tracker.pageUrl)
    }

    @Test
    fun aLoadAsAnOlderWebViewReportsIt() {
        // "Page started" when the load begins, and again for the redirect.
        tracker.willLoad(A)
        tracker.pageStarted(A)
        tracker.navigationAllowed(B)
        tracker.pageStarted(B)
        tracker.committed(B, canGoBack = false, canGoForward = false)
        tracker.pageFinished(B)
        tracker.progress(100)
        assertEquals(listOf("started $A", "committed $B", "history false false", "finished"), told())
    }

    @Test
    fun noAddressIsReportedAsThePagesBeforeItCommits() {
        load(A)
        // Everything the engine says about where a load is heading, and nothing of a commit.
        tracker.navigationAllowed(X)
        tracker.progress(10)
        tracker.navigationAllowed(B) // redirected
        tracker.progress(40)
        val heading = told()
        assertEquals(listOf("started $X", "progress 0.1", "progress 0.4"), heading)
        assertEquals("the page is still the one on screen", A, tracker.pageUrl)
        tracker.pageStarted(B)
        tracker.committed(B, canGoBack = true, canGoForward = false)
        assertEquals(listOf("committed $B", "history true false"), told())
    }

    @Test
    fun aLinkThePageFollowed() {
        load(A)
        tracker.navigationAllowed(X)
        tracker.pageStarted(X)
        tracker.committed(X, canGoBack = true, canGoForward = false)
        tracker.pageFinished(X)
        assertEquals(listOf("started $X", "committed $X", "history true false", "finished"), told())
    }

    @Test
    fun thePageChangingItsOwnAddress() {
        load(A)
        // A fragment: committed, and "finished" with no "started" before it.
        tracker.committed("$A#section", canGoBack = true, canGoForward = false)
        tracker.pageFinished("$A#section")
        assertEquals(listOf("address $A#section", "history true false"), told())
        // The History API: committed, and nothing else.
        tracker.committed("${A}inbox", canGoBack = true, canGoForward = false)
        assertEquals(listOf("address ${A}inbox", "history true false"), told())
        // replaceState to the address it already has: only the history is read again.
        tracker.committed("${A}inbox", canGoBack = true, canGoForward = false)
        assertEquals(listOf("history true false"), told())
    }

    @Test
    fun aNavigationRefusedOnTheCoresVerdict() {
        load(A)
        tracker.navigationRefused("mailto:ada@example.com", isRedirect = false)
        assertEquals(listOf("failed net.cancelled mailto:ada@example.com"), told())
        // The engine reports the refused navigation's end as a finished page: not the page's.
        tracker.pageFinished("mailto:ada@example.com")
        tracker.progress(100)
        assertEquals(emptyList<String>(), told())
    }

    @Test
    fun aRedirectRefusedEndsTheLoadTheUserAskedFor() {
        tracker.willLoad(A)
        tracker.progress(10)
        told()
        tracker.navigationRefused("file:///etc/passwd", isRedirect = true)
        assertEquals(listOf("failed net.cancelled file:///etc/passwd"), told())
        tracker.pageFinished("file:///etc/passwd")
        tracker.progress(100)
        assertEquals("the load is over, and nothing more is said of it", emptyList<String>(), told())
    }

    @Test
    fun aNavigationRefusedWhileAnotherLoadGoesOn() {
        load(A)
        tracker.willLoad(B)
        told()
        // The page still on screen tries something the policy refuses; the user's load goes on.
        tracker.navigationRefused("javascript:void(0)", isRedirect = false)
        assertEquals(listOf("failed net.cancelled javascript:void(0)", "started $B"), told())
        tracker.pageStarted(B)
        tracker.committed(B, canGoBack = true, canGoForward = false)
        tracker.pageFinished(B)
        assertEquals(listOf("committed $B", "history true false", "finished"), told())
    }

    @Test
    fun aLoadThatFails() {
        tracker.willLoad(A)
        tracker.progress(10)
        told()
        // The engine: "started", the error, "finished"; then its own error page commits at the
        // failed address, gets a title, and finishes loading.
        tracker.pageStarted(A)
        tracker.failed("net.dns", A)
        tracker.pageFinished(A)
        tracker.committed(A, canGoBack = false, canGoForward = false)
        tracker.title("Web page not available")
        tracker.progress(80)
        tracker.progress(100)
        tracker.pageFinished(A)
        assertEquals(listOf("failed net.dns $A", "committed $A", "history false false"), told())
        assertEquals("a reload loads the address that failed", A, tracker.reloadTarget)
    }

    @Test
    fun aLoadThatFailsWithItsErrorPageCommittingFirst() {
        tracker.willLoad(A)
        told()
        tracker.committed(A, canGoBack = false, canGoForward = false)
        tracker.progress(100)
        tracker.pageStarted(A)
        tracker.failed("net.connect", A)
        tracker.pageFinished(A)
        val said = told()
        assertEquals("failed net.connect $A", said.last())
        assertEquals(listOf("committed $A", "history false false", "progress 1.0", "finished", "started $A", "failed net.connect $A"), said)
    }

    @Test
    fun tryingAgainAfterAFailure() {
        tracker.willLoad(A)
        tracker.failed("net.offline", A)
        tracker.committed(A, canGoBack = false, canGoForward = false)
        told()
        // The adapter loads the address again rather than reload the engine's error page.
        tracker.willLoad(checkNotNull(tracker.reloadTarget))
        tracker.progress(10)
        tracker.pageStarted(A)
        tracker.committed(A, canGoBack = false, canGoForward = false)
        tracker.title("Example Domain")
        tracker.progress(100)
        assertEquals(listOf("started $A", "progress 0.1", "committed $A", "history false false", "title Example Domain", "progress 1.0", "finished"), told())
        assertNull(tracker.reloadTarget)
    }

    @Test
    fun leavingAFailedPageByGoingBack() {
        load(A)
        tracker.navigationAllowed(X)
        tracker.failed("net.timeout", X)
        tracker.committed(X, canGoBack = true, canGoForward = false)
        told()
        tracker.willTraverse()
        tracker.progress(10)
        tracker.pageStarted(A)
        tracker.committed(A, canGoBack = false, canGoForward = true)
        tracker.pageFinished(A)
        assertEquals(listOf("started", "progress 0.1", "committed $A", "history false true", "finished"), told())
        assertNull(tracker.reloadTarget)
    }

    @Test
    fun aLoadStoppedBeforeItCommits() {
        load(A)
        tracker.willLoad(X)
        told()
        tracker.willStop()
        assertEquals(listOf("failed net.cancelled $X"), told())
        // Whatever the engine then says of the stopped load adds nothing.
        tracker.pageFinished(X)
        tracker.progress(100)
        assertEquals(emptyList<String>(), told())
        // The page still on screen changes its own address: that is all it is.
        tracker.committed("$A#top", canGoBack = true, canGoForward = false)
        assertEquals(listOf("address $A#top", "history true false"), told())
    }

    @Test
    fun aStopThatCameTooLate() {
        load(A)
        tracker.willLoad(X)
        tracker.willStop()
        told()
        // The page had committed in the engine before the stop reached it: a new document.
        tracker.committed(X, canGoBack = true, canGoForward = false)
        assertEquals(listOf("committed $X", "history true false"), told())
    }

    @Test
    fun aLoadStoppedAfterItCommitted() {
        tracker.willLoad(A)
        tracker.pageStarted(A)
        tracker.committed(A, canGoBack = false, canGoForward = false)
        told()
        tracker.willStop()
        assertEquals(listOf("finished"), told())
        tracker.pageFinished(A)
        tracker.progress(100)
        assertEquals(emptyList<String>(), told())
    }

    @Test
    fun aLinkThatTurnsOutToBeADownload() {
        load(A)
        tracker.navigationAllowed("${A}report.pdf")
        tracker.progress(10)
        // The engine ends it as a finished page, with no commit.
        tracker.pageFinished("${A}report.pdf")
        tracker.progress(100)
        assertEquals(listOf("started ${A}report.pdf", "progress 0.1", "failed net.cancelled ${A}report.pdf"), told())
        tracker.committed("${A}reports", canGoBack = true, canGoForward = false)
        assertEquals("the page is the same document still", listOf("address ${A}reports", "history true false"), told())
    }

    @Test
    fun anHttpErrorWithNothingToShow() {
        // The engine says "started" and "finished" at once, then commits its error page there.
        tracker.willLoad(A)
        tracker.pageStarted(A)
        tracker.pageFinished(A)
        tracker.committed(A, canGoBack = false, canGoForward = false)
        tracker.progress(100)
        assertEquals(listOf("started $A", "failed net.cancelled $A", "committed $A", "history false false"), told())
    }

    @Test
    fun goingBackToAPageTheEngineKept() {
        load(A)
        load(X, canGoBack = true)
        tracker.willTraverse()
        assertEquals(emptyList<String>(), told())
        // No "finished" for a page restored whole: the progress reaching its end is the end.
        tracker.progress(10)
        tracker.pageStarted(A)
        tracker.committed(A, canGoBack = false, canGoForward = true)
        tracker.progress(100)
        assertEquals(listOf("started", "progress 0.1", "committed $A", "history false true", "progress 1.0", "finished"), told())
    }

    @Test
    fun goingBackWithinOnePage() {
        load(A)
        tracker.committed("$A#two", canGoBack = true, canGoForward = false)
        told()
        tracker.willTraverse()
        tracker.committed(A, canGoBack = false, canGoForward = true)
        tracker.pageFinished(A)
        assertEquals(listOf("address $A", "history false true"), told())
    }

    @Test
    fun aCertificateTheSystemDoesNotTrust() {
        tracker.willLoad(A)
        told()
        tracker.certificateRefused(A)
        assertEquals(listOf("failed net.tls $A"), told())
        // However the engine then reports the cancelled load, it has been said.
        tracker.pageStarted(A)
        tracker.failed("net.failed", A)
        tracker.pageFinished(A)
        tracker.progress(100)
        assertEquals(emptyList<String>(), told())
    }

    @Test
    fun aCertificateRefusedForSomethingThePageLoadsIsNotThePagesFailure() {
        load(A)
        tracker.certificateRefused("https://cdn.example.net/script.js")
        assertEquals(emptyList<String>(), told())
        // Nor during the page's own load, when it is not the page's address.
        tracker.willLoad(B)
        told()
        tracker.certificateRefused("https://cdn.example.net/script.js")
        assertEquals(emptyList<String>(), told())
    }

    @Test
    fun aCertificateRefusedAfterARedirect() {
        tracker.willLoad(A)
        tracker.navigationAllowed(B)
        told()
        tracker.certificateRefused(B)
        assertEquals(listOf("failed net.tls $B"), told())
    }

    @Test
    fun theRendererDying() {
        load(A, canGoBack = true)
        tracker.rendererGone()
        assertEquals(listOf("gone", "history false false"), told())
        assertEquals("a reload loads the page again in a new web view", A, tracker.reloadTarget)
        tracker.willLoad(A)
        tracker.pageStarted(A)
        tracker.committed(A, canGoBack = false, canGoForward = false)
        tracker.pageFinished(A)
        assertEquals(listOf("started $A", "committed $A", "history false false", "finished"), told())
        assertNull(tracker.reloadTarget)
    }

    @Test
    fun aReload() {
        tracker.willReload()
        assertEquals("nothing has committed: the engine has nothing to reload", emptyList<String>(), told())
        load(A)
        tracker.willReload()
        tracker.pageStarted(A)
        tracker.committed(A, canGoBack = false, canGoForward = false)
        tracker.pageFinished(A)
        assertEquals(listOf("started $A", "committed $A", "history false false", "finished"), told())
    }

    @Test
    fun aLoadNothingAnnounced() {
        load(A)
        // A form posted: the policy is not asked, and the first word is the progress.
        tracker.progress(10)
        tracker.pageStarted("${A}sent")
        tracker.committed("${A}sent", canGoBack = true, canGoForward = false)
        tracker.pageFinished("${A}sent")
        tracker.progress(100)
        assertEquals(listOf("started", "progress 0.1", "committed ${A}sent", "history true false", "finished"), told())
    }

    @Test
    fun theSameThingSaidTwiceIsReportedOnce() {
        tracker.willLoad(A)
        tracker.pageStarted(A)
        tracker.pageStarted(A)
        tracker.committed(A, canGoBack = false, canGoForward = false)
        tracker.pageFinished(A)
        tracker.pageFinished(A)
        tracker.progress(100)
        val said = told()
        assertEquals(1, said.count { it.startsWith("started") })
        assertEquals(1, said.count { it == "finished" })
        assertTrue(said.none { it.startsWith("failed") })
    }
}

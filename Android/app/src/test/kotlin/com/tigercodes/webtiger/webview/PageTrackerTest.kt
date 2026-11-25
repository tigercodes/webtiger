// What a System WebView's callbacks become. Each test replays the callbacks the WebView makes
// in one situation and says what the core is told. The sequences were first taken from the
// WebView's source (003 §5); those marked "as observed" were then recorded from WebView 133 on
// an Android 16 emulator, where the device suite (src/androidTest) runs the same situations
// against the real thing.
package com.tigercodes.webtiger.webview

import com.tigercodes.webtiger.core.v1.EngineEvent
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
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
        event.navigation_failed?.let { return "failed ${it.code} ${it.url}".trim() + (if (it.refused) " (refused)" else "") + if (it.certificate_held) " (held)" else "" }
        event.address_changed?.let { return "address ${it.url}" }
        event.title_changed?.let { return "title ${it.title}" }
        event.progress_changed?.let { return "progress ${it.fraction}" }
        event.history_changed?.let { return "history ${it.can_go_back} ${it.can_go_forward}" }
        event.renderer_gone?.let { return "gone" }
        event.find_result?.let { return if (it.found) "found ${it.current} of ${it.matches}" + (if (it.counted) "" else " (not counted)") else "not found" }
        event.outside_link_requested?.let { return "outside ${it.url}" + if (it.user_gesture) " (asked)" else "" }
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

    /** As observed: a WebView made from a state says its page's title twice before the page commits, and not after. */
    @Test
    fun aTitleSaidBeforeItsPagesCommitIsSaidAgainAfterItWhenTheEngineCallsThePageSo() {
        tracker.willLoad(A)
        tracker.title("Tall")
        tracker.pageStarted(A)
        tracker.title("Tall")
        assertEquals(listOf("started $A", "title Tall", "title Tall"), told())
        tracker.committed(A, canGoBack = true, canGoForward = false, titled = "Tall")
        assertEquals("the page is named after it has committed", listOf("committed $A", "title Tall", "history true false"), told())
        // Once: what the engine calls the page later is nothing to say again.
        tracker.progress(100)
        tracker.pageFinished(A)
        assertEquals(listOf("progress 1.0", "finished"), told())
    }

    @Test
    fun aTitleSaidEarlyIsNotThePagesWhenTheEngineDoesNotCallItSo() {
        // The page before sets its title as it leaves; the page that commits is called by its
        // address, as a page that has said nothing is. The title is not handed on to it.
        load(A)
        tracker.willLoad(B)
        tracker.title("Your bank")
        tracker.pageStarted(B)
        told()
        tracker.committed(B, canGoBack = true, canGoForward = false, titled = "www.example.com/home")
        assertEquals(listOf("committed $B", "history true false"), told())
        // Nor when the adapter cannot say what the engine calls the page; nor an empty one.
        tracker.willLoad(X)
        tracker.title("Early")
        told()
        tracker.committed(X, canGoBack = true, canGoForward = false)
        assertEquals(listOf("committed $X", "history true false"), told())
        tracker.willLoad(A)
        tracker.title("")
        told()
        tracker.committed(A, canGoBack = true, canGoForward = false, titled = "")
        assertEquals(listOf("committed $A", "history true false"), told())
    }

    @Test
    fun aTitleSaidEarlyDoesNotOutliveItsLoad() {
        // Said of a load that committed: not said again at the next page's commit, though the
        // engine calls that one the same.
        tracker.willLoad(A)
        tracker.title("Same")
        tracker.committed(A, canGoBack = false, canGoForward = false, titled = "Same")
        tracker.progress(100)
        tracker.pageFinished(A)
        told()
        tracker.willLoad(B)
        tracker.pageStarted(B)
        told()
        tracker.committed(B, canGoBack = true, canGoForward = false, titled = "Same")
        assertEquals(listOf("committed $B", "history true false"), told())
        // A title said after a page has committed is the page's, as it was: said once.
        tracker.title("Late")
        assertEquals(listOf("title Late"), told())
        tracker.progress(100)
        tracker.pageFinished(B)
        // And is not a title said early: the next page is not named by it, whatever the engine calls it.
        tracker.willLoad(A)
        tracker.pageStarted(A)
        told()
        tracker.committed(A, canGoBack = true, canGoForward = false, titled = "Late")
        assertEquals(listOf("committed $A", "history true false"), told())
        tracker.progress(100)
        tracker.pageFinished(A)
        // Said of a load that failed: what commits next is the engine's error page, which is
        // not named by it.
        tracker.willLoad(X)
        tracker.title("Before it failed")
        tracker.failed("net.dns", X)
        told()
        tracker.pageStarted(X)
        tracker.committed(X, canGoBack = true, canGoForward = false, titled = "Before it failed")
        assertFalse("the error page is not named by the title of the load that failed", told().contains("title Before it failed"))
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
        // Cancelled, and said to be a refusal: nothing else is loading in the web view.
        assertEquals(listOf("failed net.cancelled mailto:ada@example.com (refused)"), told())
        // The engine reports the refused navigation's end as a finished page: not the page's.
        tracker.pageFinished("mailto:ada@example.com")
        tracker.progress(100)
        assertEquals(emptyList<String>(), told())
    }

    @Test
    fun aWindowAPageOpenedThatIsRefusedTheOnlyPlaceItWasGoing() {
        // A new web view, nothing loaded: the WebView asks about its first address, and the
        // policy says no (006).
        tracker.navigationRefused("mailto:ada@example.com", isRedirect = false)
        assertEquals(listOf("failed net.cancelled mailto:ada@example.com (refused)"), told())
        tracker.progress(10)
        tracker.progress(100)
        assertEquals("and nothing more is said of it", emptyList<String>(), told())
    }

    @Test
    fun aLinkForTheSystemIsReportedAndThenRefused() {
        load(A)
        // As the WebView's client calls them: the link first, then its refusal (007).
        tracker.outsideLink("mailto:ada@example.com", userGesture = true)
        tracker.navigationRefused("mailto:ada@example.com", isRedirect = false)
        assertEquals(listOf("outside mailto:ada@example.com (asked)", "failed net.cancelled mailto:ada@example.com (refused)"), told())
    }

    @Test
    fun aLinkForTheSystemNobodyAskedForSaysSo() {
        load(A)
        // In a frame: nothing of the page itself was going anywhere, and nothing more is said.
        tracker.outsideLink("tel:+15550100", userGesture = false)
        assertEquals(listOf("outside tel:+15550100"), told())
    }

    @Test
    fun whatAFindFoundIsCountedFromOne() {
        load(A)
        // The WebView counts matches from 0; the contract, as a person does, from 1.
        tracker.findResult(activeOrdinal = 0, matches = 3)
        tracker.findResult(activeOrdinal = 2, matches = 3)
        assertEquals(listOf("found 1 of 3", "found 3 of 3"), told())
        // Nothing found; and a number the WebView could not mean is not passed on.
        tracker.findResult(activeOrdinal = 0, matches = 0)
        tracker.findResult(activeOrdinal = 7, matches = 3)
        tracker.findResult(activeOrdinal = -1, matches = 2)
        assertEquals(listOf("not found", "found 3 of 3", "found 1 of 2"), told())
    }

    @Test
    fun aLoadThatIsStoppedIsNotARefusal() {
        tracker.willLoad(A)
        tracker.progress(10)
        told()
        tracker.willStop()
        assertEquals(listOf("failed net.cancelled $A"), told())
    }

    @Test
    fun aNavigationRefusedAsObserved() {
        load(A)
        // A link the page followed, refused: the WebView then reports progress and no page.
        tracker.navigationRefused(X, isRedirect = false)
        assertEquals(listOf("failed net.cancelled $X (refused)"), told())
        tracker.progress(10)
        tracker.progress(100)
        tracker.progress(100)
        assertEquals("the refused navigation's progress is not a load", emptyList<String>(), told())
        // The next load nothing announced is one again.
        tracker.progress(10)
        assertEquals(listOf("started", "progress 0.1"), told())
    }

    @Test
    fun anAddressRefusedOnceCanBeLoadedLater() {
        load(A)
        tracker.navigationRefused(X, isRedirect = false)
        told()
        // No end was reported for the refusal; the user then goes there, and now may.
        tracker.willLoad(X)
        tracker.pageStarted(X)
        tracker.committed(X, canGoBack = true, canGoForward = false)
        tracker.pageFinished(X)
        assertEquals(listOf("started $X", "committed $X", "history true false", "finished"), told())
    }

    @Test
    fun aRedirectRefusedEndsTheLoadTheUserAskedFor() {
        tracker.willLoad(A)
        tracker.progress(10)
        told()
        tracker.navigationRefused("file:///etc/passwd", isRedirect = true)
        // The load was sent on to the refused address, and ends with it.
        assertEquals(listOf("failed net.cancelled file:///etc/passwd (refused)"), told())
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
        // Not said to be a refusal: the web view is still loading something.
        assertEquals(listOf("failed net.cancelled javascript:void(0)", "started $B"), told())
        // The refused navigation's own progress does not end the load that is going on.
        tracker.progress(10)
        tracker.progress(100)
        assertEquals(listOf("progress 0.1"), told())
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
    fun aCertificateIsRefusedWithTheKindOfTroubleItWas() {
        tracker.willLoad(A)
        told()
        tracker.certificateRefused(A, "net.tls.date")
        assertEquals(listOf("failed net.tls.date $A"), told())
    }

    @Test
    fun aCertificateThatIsHeldIsSaidToBeAndOnlyForThePagesOwnLoad() {
        tracker.willLoad(A)
        told()
        // The page's own load: the failure says the adapter holds what it refused (026).
        assertTrue(tracker.certificateRefused(A, "net.tls.untrusted", held = true))
        assertEquals(listOf("failed net.tls.untrusted $A (held)"), told())

        // Something the page loads: not the page's failure, and nobody is told anything.
        load(B)
        assertFalse(tracker.certificateRefused("https://cdn.example.net/script.js", "net.tls.untrusted", held = true))
        assertEquals(emptyList<String>(), told())
        // Not held: said as before.
        tracker.willLoad(A)
        told()
        assertTrue(tracker.certificateRefused(A, "net.tls.untrusted"))
        assertEquals(listOf("failed net.tls.untrusted $A"), told())
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

    @Test
    fun aCommitSaysWhetherItIsANewPage() {
        val reported = ArrayList<EngineEvent>()
        val tracker = PageTracker(7) { reported.add(it) }
        tracker.willLoad("https://example.com/")
        tracker.pageStarted("https://example.com/")
        assertEquals("a load that commits is a new page", true, tracker.committed("https://example.com/", canGoBack = false, canGoForward = false))
        tracker.pageFinished("https://example.com/")

        // The page changes its own address: the document is the same one, and so is what
        // was found in it (010 §3).
        assertEquals(false, tracker.committed("https://example.com/#part", canGoBack = true, canGoForward = false))
        assertEquals(false, tracker.committed("https://example.com/pushed", canGoBack = true, canGoForward = false))

        tracker.pageStarted("https://example.com/next")
        assertEquals(true, tracker.committed("https://example.com/next", canGoBack = true, canGoForward = false))
    }

    @Test
    fun whatIsFoundWhileANewPageIsOnItsWayIsNotPassedOn() {
        val reported = ArrayList<EngineEvent>()
        val tracker = PageTracker(7) { reported.add(it) }
        tracker.willLoad("https://example.com/")
        tracker.pageStarted("https://example.com/")
        tracker.committed("https://example.com/", canGoBack = false, canGoForward = false)
        tracker.pageFinished("https://example.com/")
        tracker.findResult(activeOrdinal = 1, matches = 3)
        assertEquals("in a page that is there, a find's result is reported", 1, reported.count { it.find_result != null })

        // Another page is on its way: the WebView finds the old text in the new document
        // before it says the page committed, and nobody asked.
        tracker.pageStarted("https://example.com/next")
        tracker.findResult(activeOrdinal = 0, matches = 0)
        tracker.findResult(activeOrdinal = 0, matches = 3)
        assertEquals(1, reported.count { it.find_result != null })

        // Once it is there, a find in it is reported again.
        tracker.committed("https://example.com/next", canGoBack = true, canGoForward = false)
        tracker.findResult(activeOrdinal = 0, matches = 2)
        assertEquals(2, reported.count { it.find_result != null })
        assertEquals(2, reported.last { it.find_result != null }.find_result?.matches)
    }

    @Test
    fun aLoadThatBecameADownloadIsOverAndNoPageComesOfIt() {
        val reported = ArrayList<EngineEvent>()
        val tracker = PageTracker(7) { reported.add(it) }
        tracker.willLoad("https://example.com/")
        tracker.pageStarted("https://example.com/")
        tracker.committed("https://example.com/", canGoBack = false, canGoForward = false)
        tracker.pageFinished("https://example.com/")
        val before = reported.size

        // A link on the page leads to a file: the load begins, and becomes a download.
        tracker.navigationAllowed("https://example.com/file.zip")
        tracker.becameDownload("https://example.com/file.zip")

        val after = reported.drop(before)
        assertEquals("the load began, and stopped", 2, after.size)
        assertEquals("https://example.com/file.zip", after[0].navigation_started?.url)
        assertEquals("net.cancelled", after[1].navigation_failed?.code)
        assertEquals("the page on screen is the page", "https://example.com/", tracker.pageUrl)

        // A download with no load on its way (a link that says "download") ends nothing.
        tracker.becameDownload("https://example.com/other.zip")
        assertEquals(before + 2, reported.size)
    }

    // A load's last words come late (003 §8.1)

    /** What was reported, but for progress and the way back and forward, which these tests are not about. */
    private fun ends(): List<String> = told().filterNot { it.startsWith("progress") || it.startsWith("history") }

    @Test
    fun theLoadBeforesLastWordsDoNotEndTheLoadTheAdapterBegan() {
        // As observed (WebView 133, 2026-11-21): load, progress 10, started, commit,
        // progress 100, load, progress 100, finished, progress 10. The first load is over at
        // its first progress 100; the adapter begins another at once; then the engine says
        // progress 100 once more and "finished" for the first, and only then its first word
        // of the second.
        tracker.willLoad(A)
        tracker.progress(10)
        tracker.pageStarted(A)
        tracker.committed(A, canGoBack = false, canGoForward = false)
        tracker.progress(100)
        assertEquals(listOf("started $A", "committed $A", "finished"), ends())

        tracker.willLoad(A)
        tracker.progress(100)
        tracker.pageFinished(A)
        assertEquals("the first load's last words end nothing", listOf("started $A"), ends())

        tracker.progress(10)
        tracker.pageStarted(A)
        tracker.committed(A, canGoBack = false, canGoForward = false)
        tracker.progress(100)
        tracker.pageFinished(A)
        assertEquals("the second load is a load like any other, and ends once", listOf("committed $A", "finished"), ends())
    }

    @Test
    fun aLoadOverAtFinishedIsNotEndedAgainByItsLateProgress() {
        tracker.willLoad(A)
        tracker.progress(10)
        tracker.pageStarted(A)
        tracker.committed(A, canGoBack = false, canGoForward = false)
        tracker.pageFinished(A)
        assertEquals(listOf("started $A", "committed $A", "finished"), ends())

        // Another address this time, begun before the first load's progress 100 arrives.
        tracker.willLoad(B)
        tracker.progress(100)
        assertEquals(listOf("started $B"), ends())
        tracker.progress(10)
        tracker.committed(B, canGoBack = true, canGoForward = false)
        tracker.pageFinished(B)
        assertEquals(listOf("committed $B", "finished"), ends())
    }

    @Test
    fun aReloadBegunAtOnceIsNotEndedByTheLoadBefore() {
        tracker.willLoad(A)
        tracker.progress(10)
        tracker.committed(A, canGoBack = false, canGoForward = false)
        tracker.progress(100)
        ends()
        tracker.willReload()
        tracker.pageFinished(A)
        tracker.progress(100)
        assertEquals(listOf("started $A"), ends())
        tracker.progress(10)
        tracker.committed(A, canGoBack = false, canGoForward = false)
        tracker.progress(100)
        assertEquals(listOf("committed $A", "finished"), ends())
    }

    @Test
    fun aLoadBegunWhileAnErrorPageIsOnItsWayIsNotEndedByIt() {
        // A certificate is refused; the person goes on at once (026), and the adapter loads
        // the address again while the engine's own error page for the first load is still on
        // its way. The error page starts, commits and finishes like a page.
        tracker.willLoad(A)
        tracker.progress(10)
        assertTrue(tracker.certificateRefused(A, "net.tls.untrusted", held = true))
        assertEquals(listOf("started $A", "failed net.tls.untrusted $A (held)"), ends())

        tracker.willLoad(A)
        tracker.pageStarted(A)
        tracker.pageFinished(A)
        tracker.progress(100)
        assertEquals("the error page's words end nothing, and are no page", listOf("started $A"), ends())

        // The load itself: the certificate is refused again (it was another, or nobody said
        // to let it through), and that is said as this load's failure.
        tracker.progress(10)
        assertTrue("the failure is the load's own", tracker.certificateRefused(A, "net.tls.untrusted", held = true))
        assertEquals(listOf("failed net.tls.untrusted $A (held)"), ends())
    }

    @Test
    fun aLoadBegunWhileAnErrorPageIsOnItsWayLoadsItsPage() {
        tracker.willLoad(A)
        tracker.progress(10)
        tracker.failed("net.dns", A)
        ends()
        tracker.willLoad(A)
        // The error page's title is the engine's words: not the page's, now either.
        tracker.title("Webpage not available")
        tracker.pageFinished(A)
        assertEquals(listOf("started $A"), ends())
        tracker.progress(10)
        tracker.pageStarted(A)
        tracker.committed(A, canGoBack = false, canGoForward = false)
        tracker.title("Example")
        tracker.progress(100)
        tracker.pageFinished(A)
        assertEquals(listOf("committed $A", "title Example", "finished"), ends())
    }

    @Test
    fun aLoadTheEngineSaidSomethingOfEndsAsBefore() {
        // What the engine says of a load is what ends the wait: a load that stops with no
        // page is still said to be cancelled, and one that is stopped by the adapter too.
        tracker.willLoad(A)
        tracker.progress(10)
        tracker.pageFinished(A)
        assertEquals(listOf("started $A", "failed net.cancelled $A"), ends())

        tracker.willLoad(B)
        tracker.willStop()
        assertEquals(listOf("started $B", "failed net.cancelled $B"), ends())
        // And what the engine then says of the stopped load ends nothing more.
        tracker.pageFinished(B)
        tracker.progress(100)
        assertEquals(emptyList<String>(), ends())
    }
}

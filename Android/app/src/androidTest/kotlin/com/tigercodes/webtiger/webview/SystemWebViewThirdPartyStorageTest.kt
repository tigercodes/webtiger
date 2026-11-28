// The adapter suite's System WebView run, for a third party's storage (071). The WebView does
// not keep it apart (070 §5.1); the adapter does, with a script of its own in every frame,
// for the frames the core says. Here the core's part is played by the test: it says which
// frames are apart, and sees what was asked. A frame that is apart finds nothing its site
// stored, what it stores goes with its document, and the stores that last are refused. A
// frame the core says has its storage has what the WebView gives: said as it is, so that a
// WebView that begins to keep storage apart itself is noticed.
package com.tigercodes.webtiger.webview

import androidx.test.ext.junit.runners.AndroidJUnit4
import com.tigercodes.webtiger.core.FrameAsked
import com.tigercodes.webtiger.core.StorageVerdict
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
internal class SystemWebViewThirdPartyStorageTest : AdapterTestCase() {
    /** The address of a path of the site by its other name: another site, to an engine. */
    private fun other(path: String) = "http://localhost:${site.port}$path"

    private val otherOrigin get() = "http://localhost:${site.port}"
    private val ownOrigin get() = "http://127.0.0.1:${site.port}"

    /** A web view in a login set, at an address, waited for by the title its page comes to. */
    private fun show(id: Long, loginSet: String, url: String, title: String) {
        create(id, loginSet)
        go(id, url, title)
    }

    /** Sends a web view to an address, and waits for the title its page comes to. */
    private fun go(id: Long, url: String, title: String) {
        load(id, url)
        try {
            await(id, "the page titled $title") { hasTitle(title)(it) }
        } catch (late: AssertionError) {
            // What the page did say is what such a failure is read for.
            throw AssertionError("the page said <${lastTitle(id)}> and not <$title>", late)
        }
    }

    /** The core's part: every frame it is asked about is to be kept apart. */
    private fun everyFrameApart() {
        core.setStorage { StorageVerdict.APART }
    }

    @Test
    fun aSiteInAnotherSitesPageFindsNothingItStoredAndWhatItStoresThereGoesWithThePage() {
        everyFrameApart()
        val set = newLoginSet()
        show(1, set, other("/keep?v=ada"), "Kept")
        // In a page of another site, its script finds nothing stored.
        show(2, set, site.url("/holds-another-site"), "frame: $NOTHING_IN_A_FRAME")
        // The core was asked about that frame: who it is, the top frame its document has,
        // and the page the adapter knows; and about no other.
        assertEquals(listOf(FrameAsked(otherOrigin, ownOrigin, site.url("/holds-another-site"), 2)), core.frames)
        // As itself it has what it had.
        show(3, set, other("/kept"), ITSELF)
        // What it stores in a frame ("eve") is not there when it is visited as itself.
        show(4, set, site.url("/holds-another-site?frame=keep"), "frame: kept")
        show(5, set, other("/kept"), ITSELF)
        // Nor in the next page it is a frame of.
        show(6, set, site.url("/holds-another-site"), "frame: $NOTHING_IN_A_FRAME")
    }

    @Test
    fun everyStoreOfASiteInAnotherSitesPageIsItsDocumentsOwnOrRefused() {
        everyFrameApart()
        val set = newLoginSet()
        // As itself, the site keeps a name in every store it has.
        show(1, set, other("/stores?v=ada"), "local none, session none, database none, cache none, $THE_REST_AS_ITSELF")
        // In another site's page, in the very web view it was kept in: the page's storage
        // and the session's are empty, the stores that last are refused, and the workers
        // that outlive a page are not there.
        go(1, site.url("/holds-another-site?frame=stores"), "frame: $STORES_APART")
        // What it kept there ("eve") is nowhere: as itself it has the name it kept, in each.
        show(2, set, other("/stores"), "local ada, session none, database ada, cache ada, $THE_REST_AS_ITSELF")
        // And in a frame once more, nothing of either.
        show(3, set, site.url("/holds-another-site?frame=stores"), "frame: $STORES_APART")
    }

    @Test
    fun theStoreAFrameIsGivenWorksAsAPagesStorageDoes() {
        val set = newLoginSet()
        // The WebView's own storage passes the page's checks: they are of storage as it is.
        show(1, set, other("/storage-works"), WORKS)
        show(2, set, site.url("/holds-another-site?frame=storage-works"), "frame: $WORKS")
        // And so does the store a frame that is kept apart is given.
        everyFrameApart()
        show(3, set, site.url("/holds-another-site?frame=storage-works"), "frame: $WORKS")
        // Asked about each of the two frames, and not about the site as itself.
        assertEquals(2, core.frames.size)
    }

    @Test
    fun aFrameOfThePagesOwnSiteHasItsStorageAndNothingIsAsked() {
        everyFrameApart()
        val set = newLoginSet()
        show(1, set, site.url("/keep?v=ada"), "Kept")
        show(2, set, site.url("/holds-another-site?at=127.0.0.1"), "frame: $ITSELF")
        show(3, set, site.url("/holds-another-site?at=127.0.0.1&frame=stores"), "frame: local ada, session none, database none, cache none, $THE_REST_AS_ITSELF")
        // Neither for the page itself nor for a frame of its own origin.
        assertEquals(emptyList<FrameAsked>(), core.frames)
        // And the question's name is not in the page, for a script of the page's to ask by.
        assertEquals("\"undefined\"", run(3, "return typeof window.${ThirdPartyStorage.NAME};"))
    }

    /**
     * A frame the core says has its storage (as where Guard is off for the page) has what the
     * WebView gives a third party: every store its site has as itself. The gap of 070 §5.1,
     * said here store by store, so that a WebView that keeps any of them apart is noticed.
     */
    @Test
    fun aFrameTheCoreSaysHasItsStorageHasWhatTheWebViewGives() {
        val set = newLoginSet()
        show(1, set, other("/stores?v=ada"), "local none, session none, database none, cache none, $THE_REST_AS_ITSELF")
        go(1, site.url("/holds-another-site?frame=stores"), "frame: local ada, session ada, database ada, cache ada, $THE_REST_AS_ITSELF")
        // The core was asked, and said so.
        assertEquals(listOf(FrameAsked(otherOrigin, ownOrigin, site.url("/holds-another-site?frame=stores"), 1)), core.frames)
        // And what the frame kept ("eve") is the site's own from then on.
        show(2, set, other("/stores"), "local eve, session none, database eve, cache eve, $THE_REST_AS_ITSELF")
    }

    @Test
    fun aFrameWithinAFrameIsJudgedAgainstThePage() {
        // The core's part, as the core plays it: a frame of another origin than the top frame's is apart.
        core.setStorage { frame -> if (frame.origin == frame.topOrigin) StorageVerdict.OWN else StorageVerdict.APART }
        val set = newLoginSet()
        show(1, set, other("/keep?v=ada"), "Kept")
        // The page holds a frame of its own site, which holds the other site: judged against
        // the page, two frames up.
        show(2, set, site.url("/holds-a-holder"), "frame: $NOTHING_IN_A_FRAME")
        assertEquals(listOf(FrameAsked(otherOrigin, ownOrigin, site.url("/holds-a-holder"), 2)), core.frames)
        // The page holds the other site, which holds a frame of the page's own site: that
        // one is the page's, and has its storage, whatever is between them.
        show(3, set, site.url("/keep?v=bob"), "Kept")
        create(4, set)
        load(4, site.url("/holds-a-holder?middle=localhost&at=127.0.0.1"))
        await(4, "the innermost frame to say what it has") { reports -> reports.any { it.title_changed?.title.orEmpty().endsWith("stored bob") } }
        // Only the frame between was asked about: the innermost is of the page's own origin.
        assertEquals(FrameAsked(otherOrigin, ownOrigin, site.url("/holds-a-holder?middle=localhost&at=127.0.0.1"), 4), core.frames.last())
        assertEquals(2, core.frames.size)
    }

    /**
     * **It is not a wall, and these are two ways round it, said as they are** (071 §8), so
     * that a WebView in which either is closed is noticed. A frame with no address of its own
     * is of whoever made it, and the WebView runs no script of the adapter's in one: a site
     * that is kept apart, and reaches for its storage through an empty frame of its own
     * making, has it. And a worker has stores of its own, which nothing put in a page reaches.
     */
    @Test
    fun anEmptyFrameOfItsOwnMakingAndAWorkerAreWaysRound() {
        everyFrameApart()
        val set = newLoginSet()
        show(1, set, other("/stores?v=ada"), "local none, session none, database none, cache none, $THE_REST_AS_ITSELF")
        // The plain way finds nothing.
        show(2, set, site.url("/holds-another-site?frame=stores"), "frame: $STORES_APART")
        // Through an empty frame: the name, the moment the frame is made and after.
        show(3, set, site.url("/holds-another-site?frame=kept-through-a-frame"), "frame: at once ada, later ada")
        // From a worker: the name, in the site's database.
        show(4, set, site.url("/holds-another-site?frame=kept-through-a-worker"), "frame: worker: ada")
        // As itself, a worker of the site's has the same: the page is right about workers.
        show(5, set, other("/kept-through-a-worker"), "worker: ada")
    }

    private companion object {
        /** What `/kept` says of a site that kept "ada" (`/keep?v=ada`), asked as itself. */
        const val ITSELF = "asked ada/ada, reads anywhere+who, writes yes, stored ada"

        /** What it says in a frame of another site's page, kept apart: no cookie (070), and nothing stored. */
        const val NOTHING_IN_A_FRAME = "asked none/none, reads none, writes no, stored none"

        /** What `/stores` says, after the stores a name is kept in, of a site as itself. */
        const val THE_REST_AS_ITSELF = "files given, worker offered, shared absent"

        /** What `/stores` says in a frame that is kept apart. */
        const val STORES_APART = "local none, session none, database refused, cache refused, files refused, worker absent, shared absent"

        /** What `/storage-works` says of storage that works. */
        const val WORKS = "local works, session works"
    }
}

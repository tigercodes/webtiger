// The adapter suite's System WebView run, for third-party cookies (070). A site that is in
// another site's page (a frame, a picture) is not sent the cookies it has; what it sets there
// is not kept; and its script in a frame reads none and can write none. Visited as itself it
// has what it had. The test site answers by two names, 127.0.0.1 and localhost: two sites, to
// an engine. The same behaviours as the WebKit run (WebKitThirdPartyCookieTests), but for one
// thing, which is the WebView's and is said here as it is: **a page's storage is not kept
// apart.** A site in a frame of another site's page reads what it stored as itself, and what
// it stores there it has as itself (070 §5.1).
package com.tigercodes.webtiger.webview

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
internal class SystemWebViewThirdPartyCookieTest : AdapterTestCase() {
    /** The address of a path of the site by its other name: another site, to an engine. */
    private fun other(path: String) = "http://localhost:${site.port}$path"

    /** The cookies each request for a path came with, in the order asked: none is empty. */
    private fun cookies(path: String): List<Set<String>> =
        site.headers(path).map { said -> said["cookie"].orEmpty().split("; ").filter { it.isNotEmpty() }.toSet() }

    /** A web view in a login set, at an address, waited for by the title its page comes to. */
    private fun show(id: Long, loginSet: String, url: String, title: String, ephemeral: Boolean = false) {
        create(id, loginSet, ephemeral = ephemeral)
        load(id, url)
        try {
            await(id, "the page titled $title") { hasTitle(title)(it) }
        } catch (late: AssertionError) {
            // What the page did say is what such a failure is read for.
            throw AssertionError("the page said <${lastTitle(id)}> and not <$title>", late)
        }
    }

    @Test
    fun aSiteInAnotherSitesPageIsNotSentItsCookiesAndItsScriptHasNone() {
        val set = newLoginSet()
        show(1, set, other("/keep?v=ada"), "Kept")
        // As itself, it is asked with what it kept, and its script reads it.
        show(2, set, other("/kept"), ITSELF)

        // In a page of another site: its frame's page and its picture are asked for with
        // nothing, and its script in the frame has no cookie and can keep none. (Its storage
        // it has: the WebView's own doing.)
        show(3, set, site.url("/holds-another-site"), "frame: $IN_A_FRAME")
        assertEquals(listOf(setOf("who=ada", "anywhere=ada"), emptySet()), cookies("/kept"))
        assertEquals(listOf(emptySet<String>()), cookies("/pixel"))

        // And it has lost nothing: as itself again, it has what it had.
        show(4, set, other("/kept"), ITSELF)
        assertEquals(setOf("who=ada", "anywhere=ada"), cookies("/kept").last())
    }

    @Test
    fun whatASiteSetsInAFrameOfAnotherSitesPageIsNotKept() {
        val set = newLoginSet()
        // The frame's page is one that keeps a name every way it can, and says when it has.
        show(1, set, site.url("/holds-another-site?frame=keep"), "frame: kept")
        assertEquals("the frame's page was asked for", 1, site.requests.count { it == "/keep" })
        // As itself, the site has no cookie. (What the frame stored it has: the WebView's own
        // doing.)
        show(2, set, other("/kept"), "asked none/none, reads none, writes yes, stored eve")
        assertEquals(listOf(emptySet<String>()), cookies("/kept"))
    }

    @Test
    fun whatASiteSetsWithAPictureInAnotherSitesPageIsNotKept() {
        val set = newLoginSet()
        // The picture is no picture: its answer sets the cookies. The page says what its
        // frame said once the picture has been answered.
        show(1, set, site.url("/holds-another-site?picture=keep"), "frame: $NOBODY_IN_A_FRAME")
        assertEquals("the picture was asked for", 1, site.requests.count { it == "/keep" })
        show(2, set, other("/kept"), NOBODY)
        assertEquals(emptySet<String>(), cookies("/kept").last())
    }

    /**
     * A site a person has used as itself (a touch in its page) is shut out of another site's
     * page all the same: the story of 070 is a site a person is signed in to, and engines have
     * had rules that let such a site more than one a person only passed through.
     */
    @Test
    fun aSiteAPersonHasUsedIsShutOutOfAnotherSitesPageAllTheSame() {
        val set = newLoginSet()
        show(1, set, other("/keep?v=ada"), "Kept")
        tap(1, "use")
        await(1, "the touch to reach the site's page") { hasTitle("Used")(it) }

        show(2, set, site.url("/holds-another-site"), "frame: $IN_A_FRAME")
        assertEquals(listOf(emptySet<String>()), cookies("/kept"))
        assertEquals(listOf(emptySet<String>()), cookies("/pixel"))
        // And in the very web view the person used it in.
        load(1, site.url("/holds-another-site"))
        await(1, "the frame to say what it has") { hasTitle("frame: $IN_A_FRAME")(it) }
        assertEquals(listOf(emptySet<String>(), emptySet()), cookies("/kept"))
    }

    /**
     * The other way round, so that these tests are known to see a cookie where there is one:
     * a frame and a picture of the page's own site have that site's cookies.
     */
    @Test
    fun aFrameAndAPictureOfThePagesOwnSiteHaveItsCookies() {
        val set = newLoginSet()
        show(1, set, site.url("/keep?v=ada"), "Kept")
        show(2, set, site.url("/holds-another-site?at=127.0.0.1"), "frame: $ITSELF")
        assertEquals(listOf(setOf("who=ada", "anywhere=ada")), cookies("/kept"))
        assertEquals(listOf(setOf("who=ada", "anywhere=ada")), cookies("/pixel"))
    }

    private companion object {
        /**
         * What `/kept` says of a site that kept "ada" (`/keep?v=ada`), asked as itself: what it
         * was asked with, what its script reads and can write, and what its storage has.
         */
        const val ITSELF = "asked ada/ada, reads anywhere+who, writes yes, stored ada"

        /** What it says of a site that has kept nothing. */
        const val NOBODY = "asked none/none, reads none, writes yes, stored none"

        /**
         * What it says in a frame of another site's page, of a site that kept "ada": asked with
         * nothing, no cookie to read, none written. **Its storage is the one it has as itself:**
         * the System WebView does not keep a third party's storage apart (070 §5.1). Said as it
         * is, so that a WebView that begins to is noticed.
         */
        const val IN_A_FRAME = "asked none/none, reads none, writes no, stored ada"

        /** What a site that has kept nothing says in a frame of another site's page. */
        const val NOBODY_IN_A_FRAME = "asked none/none, reads none, writes no, stored none"
    }
}

// The adapter suite's System WebView run, for find on the page (010): what the WebView's own
// find does when the adapter asks it, and what the adapter reports of it. The WebView counts:
// how many matches there are, and which one is shown. The same behaviours as the WebKit run
// (WebKitFindTests), with the counts WebKit does not give.
package com.tigercodes.webtiger.webview

import androidx.test.ext.junit.runners.AndroidJUnit4
import com.tigercodes.webtiger.core.v1.FindResult
import com.tigercodes.webtiger.core.v1.HostRequest
import com.tigercodes.webtiger.core.v1.HostResponse
import com.tigercodes.webtiger.core.v1.WebViewFind
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
internal class SystemWebViewFindTest : AdapterTestCase() {
    /** A web view on the page with one word three times, shown as the tab in front is. */
    private fun page(id: Long = 1) {
        create(id)
        load(id, site.url("/find"))
        await(id, "the page to load") { finished(it) && hasTitle("Find")(it) }
        // A page that is not shown is not laid out, and the WebView finds nothing in it.
        show(id)
    }

    private fun find(text: String, backwards: Boolean = false, id: Long = 1): HostResponse =
        answer(HostRequest(web_view_find = WebViewFind(web_view_id = id, text = text, backwards = backwards)))

    /** What the adapter has reported of finds in a web view, in order. */
    private fun results(id: Long = 1): List<FindResult> = reports(id).mapNotNull { it.find_result }

    /** Finds, and waits for what was found: "which of how many", or "none". */
    private fun found(text: String, backwards: Boolean = false): String {
        val before = results().size
        assertNotNull("a find is acknowledged", find(text, backwards).ack)
        await(1, "the find's result") { results().size > before }
        val result = results().last()
        assertEquals("the WebView counts", true, result.counted)
        return if (result.found) "${result.current} of ${result.matches}" else "none"
    }

    @Test
    fun textThatIsThereIsFoundAndCounted() {
        page()
        // Whatever its case: the second is written with a capital.
        assertEquals("1 of 3", found("needle"))
    }

    @Test
    fun theSameTextAgainGoesToTheNextAndRoundToTheFirst() {
        page()
        assertEquals(listOf("1 of 3", "2 of 3", "3 of 3", "1 of 3"), List(4) { found("needle") })
    }

    @Test
    fun backwardsGoesToTheOneBeforeAndRoundToTheLast() {
        page()
        found("needle")
        assertEquals("2 of 3", found("needle"))
        assertEquals("1 of 3", found("needle", backwards = true))
        assertEquals("3 of 3", found("needle", backwards = true))
    }

    @Test
    fun textThatIsNotThereIsNotFound() {
        page()
        assertEquals("none", found("zebra"))
    }

    @Test
    fun otherTextStartsFromItsFirstMatch() {
        page()
        found("needle")
        assertEquals("2 of 3", found("needle"))
        // Another word, once on the page.
        assertEquals("1 of 1", found("haystack"))
        // And the first word again is a find of its own, from its first match.
        assertEquals("1 of 3", found("needle"))
    }

    @Test
    fun emptyTextStopsAndTheNextFindStartsAgain() {
        page()
        found("needle")
        assertEquals("2 of 3", found("needle"))

        assertNotNull("stopping is acknowledged", find("").ack)

        // The same word after stopping is found from its first match, not its third.
        assertEquals("1 of 3", found("needle"))
    }

    @Test
    fun aNewPageIsLookedInFromItsFirstMatch() {
        page()
        found("needle")
        assertEquals("2 of 3", found("needle"))
        val loads = reports(1).count { it.navigation_finished != null }

        load(1, site.url("/find"))
        await(1, "the page again") { finished(it, times = loads + 1) }

        assertEquals("the same word in a new page is a new find", "1 of 3", found("needle"))
    }

    @Test
    fun aNewPageIsNotLookedInUntilItIsAsked() {
        page()
        found("needle")
        assertEquals("2 of 3", found("needle"))
        val loads = reports(1).count { it.navigation_finished != null }
        val said = results().size

        load(1, site.url("/find"))
        await(1, "the page again") { finished(it, times = loads + 1) }
        // Long enough for a WebView that goes on finding by itself to have said so.
        Thread.sleep(1_500)

        assertEquals("nothing is found that nobody looked for; reported: ${names(reports(1)).takeLast(12)}", said, results().size)
    }

    @Test
    fun aWebViewThatIsNotThereIsRefusedWithACode() {
        assertEquals("webview.unknown", find("needle", id = 99).error?.code)
    }
}

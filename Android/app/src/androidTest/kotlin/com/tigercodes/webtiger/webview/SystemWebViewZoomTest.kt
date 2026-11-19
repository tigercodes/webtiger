// The adapter suite's System WebView run, for zoom (012): what the WebView does when the
// adapter asks it to zoom. The WebView has no public way to zoom a whole page; it scales the
// text, and the measure here is how wide the page's own words are.
package com.tigercodes.webtiger.webview

import com.tigercodes.webtiger.core.v1.HostRequest
import com.tigercodes.webtiger.core.v1.HostResponse
import com.tigercodes.webtiger.core.v1.WebViewSetZoom
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test

internal class SystemWebViewZoomTest : AdapterTestCase() {
    /**
     * A web view on the screen, as a tab's is before its page arrives, and then the page with
     * one line of words. (A page that arrives in a web view nobody has shown is laid out in no
     * room at all, and its words have no width until something makes it lay them out again.)
     */
    private fun page(id: Long = 1, path: String = "/sized") {
        create(id)
        show(id)
        load(id, site.url(path))
        await(id, "the page to load") { finished(it) && hasTitle("Sized")(it) }
        repeat(500) {
            if (measures(id).first > 0) return
            Thread.sleep(10)
        }
        throw AssertionError("the page was never laid out: its words have no width")
    }

    private fun zoom(percent: Int, id: Long = 1): HostResponse = answer(HostRequest(web_view_set_zoom = WebViewSetZoom(web_view_id = id, percent = percent)))

    /** How wide the page's words are, and how wide the page is laid out, in the page's own units. */
    private fun measures(id: Long = 1): Pair<Double, Double> {
        val said =
            run(id, "return [document.getElementById('words').getBoundingClientRect().width, document.documentElement.clientWidth].join(' ');")
                ?.trim('"')
                ?.split(' ')
                ?.mapNotNull { it.toDoubleOrNull() }
        check(said != null && said.size == 2) { "the page did not say its measures: $said" }
        return said[0] to said[1]
    }

    /** Waits until the words are `wide`, give or take a twentieth, and says how wide they were. */
    private fun wordsSettledAt(wide: Double, id: Long = 1): Double {
        var last = measures(id).first
        repeat(500) {
            if (kotlin.math.abs(last - wide) <= wide / 20) return last
            Thread.sleep(10)
            last = measures(id).first
        }
        return last
    }

    @Test
    fun aZoomedPagesTextIsBiggerAndThePageIsAsWideAsItWas() {
        page()
        val (words, room) = measures()
        assertTrue("the words are measured: $words", words > 20)

        assertNotNull("a zoom is acknowledged", zoom(200).ack)
        assertEquals("twice the size: words twice as wide", words * 2, wordsSettledAt(words * 2), words / 10)
        assertEquals("and the page is laid out in the room it had: the WebView scales text, not the page", room, measures().second, 1.0)

        zoom(50)
        assertEquals("half the size", words / 2, wordsSettledAt(words / 2), words / 20)

        zoom(100)
        assertEquals(words, wordsSettledAt(words), words / 20)
    }

    @Test
    fun theZoomStaysForThePageTheWebViewGoesOnTo() {
        page()
        val words = measures().first
        zoom(150)
        wordsSettledAt(words * 1.5)
        val loads = reports(1).count { it.navigation_finished != null }

        load(1, site.url("/sized?again=1"))
        await(1, "the page again") { finished(it, times = loads + 1) }

        assertEquals("the web view keeps its zoom: the core asks again only for another level", words * 1.5, wordsSettledAt(words * 1.5), words / 10)
    }

    @Test
    fun aZoomIsOneWebViewsOwn() {
        page(1)
        val words = measures(1).first
        zoom(200, id = 1)
        wordsSettledAt(words * 2, id = 1)

        page(2)

        assertEquals("another web view is at actual size", words, measures(2).first, words / 20)
    }

    @Test
    fun aLevelNoPageCanBeReadAtIsHeldToOneThatCan() {
        page()
        val words = measures().first

        zoom(5)
        assertEquals("no smaller than half", words / 2, wordsSettledAt(words / 2), words / 20)
    }

    @Test
    fun aWebViewThatIsNotThereIsRefusedWithACode() {
        assertEquals("webview.unknown", zoom(200, id = 99).error?.code)
    }
}

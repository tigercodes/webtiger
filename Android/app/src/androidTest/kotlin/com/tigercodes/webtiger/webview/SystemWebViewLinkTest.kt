// The adapter suite's System WebView run, for a link's menu (018): a finger pressed and held
// on a link of the test site. Who is told, with what address, and what a long press that is
// not on a link is left to be.
package com.tigercodes.webtiger.webview

import org.junit.Assert.assertEquals
import org.junit.Test
import java.util.concurrent.CopyOnWriteArrayList

internal class SystemWebViewLinkTest : AdapterTestCase() {
    /** The long presses on a link that the engine told of, in order: the web view, and where the link leads. */
    private val pressed = CopyOnWriteArrayList<Pair<Long, String>>()

    /** A web view on the page of links, with somebody to tell of a press. */
    private fun page(id: Long = 1) {
        onMain { engine.onLinkPressed = { webView, url -> pressed.add(webView to url) } }
        create(id)
        load(id, site.url("/links"))
        await(id, "the page to load") { finished(it) && hasTitle("Links")(it) }
    }

    private fun awaitPressed(count: Int = 1) {
        await(1, "a long press on a link to be told of; told: $pressed") { pressed.size >= count }
    }

    @Test
    fun aLongPressOnALinkInWordsIsToldWithWhereItLeads() {
        page()

        longPress(1, "words")

        awaitPressed()
        assertEquals(listOf(1L to site.url("/landed")), pressed.toList())
        // The press was the menu's: the page is where it was, and nothing was asked of the core.
        Thread.sleep(500)
        assertEquals(listOf(site.url("/links")), addresses(1))
        assertEquals(0, reports(1).count { it.link_open_requested != null })
    }

    @Test
    fun aLongPressOnAPictureThatIsALinkIsToldWithTheLinksAddressNotThePictures() {
        page()

        longPress(1, "picture")

        awaitPressed()
        assertEquals(listOf(1L to site.url("/landed?from=picture")), pressed.toList())
    }

    @Test
    fun aLongPressOnWordsThatAreNoLinkIsTheWebViewsOwn() {
        page()

        longPress(1, "plain")

        Thread.sleep(1_000)
        assertEquals("nobody is told", emptyList<Pair<Long, String>>(), pressed.toList())
        assertEquals(listOf(site.url("/links")), addresses(1))
    }

    @Test
    fun aTapOnTheLinkStillFollowsIt() {
        page()

        tap(1, "words")

        await(1, "the link's page") { hasTitle("Landed")(it) }
        assertEquals("a tap is no long press", emptyList<Pair<Long, String>>(), pressed.toList())
    }

    @Test
    fun aLinkChosenFromTheMenuIsReportedForTheCoreToOpen() {
        page()
        longPress(1, "words")
        awaitPressed()

        val (webView, url) = pressed.first()
        onMain { engine.openLink(webView, url, background = true) }

        val asked = reports(1).mapNotNull { it.link_open_requested }
        assertEquals(1, asked.size)
        assertEquals(site.url("/landed"), asked.first().url)
        assertEquals(true, asked.first().background)
        assertEquals(true, asked.first().user_gesture)
    }
}

// The adapter suite's System WebView run, for a link's menu (018) and a picture's (022): a
// finger pressed and held on a link or a picture of the test site. Who is told, with what
// addresses, and what a long press on anything else is left to be.
package com.tigercodes.webtiger.webview

import org.junit.Assert.assertEquals
import org.junit.Test
import java.util.concurrent.CopyOnWriteArrayList

internal class SystemWebViewLinkTest : AdapterTestCase() {
    /** The long presses that the engine told of, in order: the web view, and what was pressed. */
    private val pressed = CopyOnWriteArrayList<Pair<Long, Pressed>>()

    /** A web view on the page of links, with somebody to tell of a press. */
    private fun page(id: Long = 1) {
        onMain { engine.onPressed = { webView, what -> pressed.add(webView to what) } }
        create(id)
        load(id, site.url("/links"))
        await(id, "the page to load") { finished(it) && hasTitle("Links")(it) }
    }

    /** A web view on the page of pictures, with somebody to tell of a press. */
    private fun pictures(id: Long = 1) {
        onMain { engine.onPressed = { webView, what -> pressed.add(webView to what) } }
        create(id)
        load(id, site.url("/pictures"))
        await(id, "the page to load") { finished(it) && hasTitle("Pictures")(it) }
    }

    private fun awaitPressed(count: Int = 1) {
        await(1, "a long press on a link to be told of; told: $pressed") { pressed.size >= count }
    }

    @Test
    fun aLongPressOnALinkInWordsIsToldWithWhereItLeads() {
        page()

        longPress(1, "words")

        awaitPressed()
        assertEquals(listOf(1L to Pressed(link = site.url("/landed"))), pressed.toList())
        // The press was the menu's: the page is where it was, and nothing was asked of the core.
        Thread.sleep(500)
        assertEquals(listOf(site.url("/links")), addresses(1))
        assertEquals(0, reports(1).count { it.link_open_requested != null })
    }

    @Test
    fun aLongPressOnAPictureThatIsALinkIsToldWithTheLinksAddressAndThePictures() {
        page()

        longPress(1, "picture")

        awaitPressed()
        // Where the link leads is the link's address, not the picture's; the picture's is its own.
        assertEquals(listOf(1L to Pressed(link = site.url("/landed?from=picture"), picture = site.url("/pixel"))), pressed.toList())
    }

    @Test
    fun aLongPressOnAPictureIsToldWithThePicturesAddress() {
        pictures()

        longPress(1, "plain")

        awaitPressed()
        assertEquals(listOf(1L to Pressed(picture = site.url("/tiger.svg"))), pressed.toList())
        // The press was the menu's: the page is where it was, and nothing was said to the core.
        Thread.sleep(500)
        assertEquals(listOf(site.url("/pictures")), addresses(1))
        assertEquals(0, reports(1).count { it.link_open_requested != null || it.download_requested != null })
    }

    @Test
    fun aPictureChosenToBeSavedIsOfferedToTheCoreAsThePersonsOwn() {
        pictures()
        longPress(1, "plain")
        awaitPressed()

        val (webView, what) = pressed.first()
        onMain { engine.saveFile(webView, what.picture) }

        // The site is asked what the picture is before it is offered: not at once.
        await(1, "the picture to be offered") { reports -> reports.any { it.download_requested != null } }
        val asked = reports(1).mapNotNull { it.download_requested }
        assertEquals(1, asked.size)
        assertEquals(site.url("/tiger.svg"), asked.first().url)
        assertEquals("the person chose it: no page asked", true, asked.first().user_asked)
        assertEquals("the site suggests no name", "", asked.first().suggested_filename)
        assertEquals("what kind of file the site says it is", "image/svg+xml", asked.first().mime_type)
        // The page is where it was: nothing of its own became a download.
        assertEquals(listOf(site.url("/pictures")), addresses(1))
    }

    @Test
    fun aLongPressOnWordsThatAreNoLinkIsTheWebViewsOwn() {
        page()

        longPress(1, "plain")

        Thread.sleep(1_000)
        assertEquals("nobody is told", emptyList<Pair<Long, Pressed>>(), pressed.toList())
        assertEquals(listOf(site.url("/links")), addresses(1))
    }

    @Test
    fun aTapOnTheLinkStillFollowsIt() {
        page()

        tap(1, "words")

        await(1, "the link's page") { hasTitle("Landed")(it) }
        assertEquals("a tap is no long press", emptyList<Pair<Long, Pressed>>(), pressed.toList())
    }

    @Test
    fun aLinkChosenFromTheMenuIsReportedForTheCoreToOpen() {
        page()
        longPress(1, "words")
        awaitPressed()

        val (webView, what) = pressed.first()
        onMain { engine.openLink(webView, what.link, background = true) }

        val asked = reports(1).mapNotNull { it.link_open_requested }
        assertEquals(1, asked.size)
        assertEquals(site.url("/landed"), asked.first().url)
        assertEquals(true, asked.first().background)
        assertEquals(true, asked.first().user_gesture)
    }
}

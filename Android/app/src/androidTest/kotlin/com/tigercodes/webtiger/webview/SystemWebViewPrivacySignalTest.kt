// The adapter suite's System WebView run, for Global Privacy Control (069): the signal that says
// "do not sell or share what you learn of me". What the WebView lets a shell do, which is what
// the feature was built on: a page's scripts read the signal in every frame, from before the
// page's own scripts run, and cannot make it say otherwise; a load the shell begins carries the
// header; and a web view whose pages are not told the signal has neither. The same behaviours
// as the WebKit run (WebKitPrivacySignalTests).
package com.tigercodes.webtiger.webview

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
internal class SystemWebViewPrivacySignalTest : AdapterTestCase() {
    /** The header a request for a path came with, each time it was asked for: null for a request that had none. */
    private fun signal(path: String): List<String?> = site.headers(path).map { it["sec-gpc"] }

    /** Waits until the site has been asked for a path. */
    private fun awaitAsked(path: String) {
        repeat(1000) {
            if (path in site.requests) return
            Thread.sleep(10)
        }
        throw AssertionError("the site was never asked for $path: ${site.requests}")
    }

    @Test
    fun aPagesScriptsReadTheSignalAndCannotMakeItSayOtherwise() {
        create(1, privacySignal = true)
        load(1, site.url("/gpc"))
        // What the page read first, and what it read after trying to write it and to make it
        // again: true both times.
        await(1, "the page to say what it read") { hasTitle("gpc=true then true")(it) }
    }

    @Test
    fun aFrameOfAnotherOriginReadsTheSignalToo() {
        create(1, privacySignal = true)
        load(1, site.url("/gpc-in-a-frame"))
        await(1, "the frame to say what it read") { hasTitle("frame: gpc=true then true")(it) }
    }

    @Test
    fun aLoadTheShellBeginsCarriesTheHeader() {
        create(1, privacySignal = true)
        load(1, site.url("/tracked"))
        await(1, "the page") { finished(it) }
        assertEquals(listOf<String?>("1"), signal("/tracked"))
        // A second load the shell begins, in the same web view: with it again.
        load(1, site.url("/landed"))
        await(1, "the second page") { finished(it, times = 2) }
        assertEquals(listOf<String?>("1"), signal("/landed"))
    }

    @Test
    fun aWindowAPageOpensIsToldAsThePageThatOpenedItIs() {
        create(1, privacySignal = true)
        load(1, site.url("/opener"))
        await(1, "the opener to load") { finished(it) && hasTitle("Opener")(it) }
        tap(1, "open")
        val new = awaitWindow(askedBy = 1).new_web_view_id
        await(new, "the window's page") { hasTitle("Opened by a page")(it) }
        assertEquals("\"true\"", run(new, "return String(navigator.globalPrivacyControl);"))
        // And a load the shell begins in it carries the header.
        load(new, site.url("/landed"))
        await(new, "a page loaded in the window") { hasTitle("Landed")(it) }
        assertEquals(listOf<String?>("1"), signal("/landed"))

        // And one opened by a page that is not told, is not.
        create(2)
        load(2, site.url("/opener"))
        await(2, "the opener to load") { finished(it) && hasTitle("Opener")(it) }
        tap(2, "open")
        val other = awaitWindow(askedBy = 2).new_web_view_id
        await(other, "the window's page") { hasTitle("Opened by a page")(it) }
        assertEquals("\"undefined\"", run(other, "return String(navigator.globalPrivacyControl);"))
        load(other, site.url("/landed"))
        await(other, "a page loaded in the other window") { hasTitle("Landed")(it) }
        assertEquals(listOf("1", null), signal("/landed"))
    }

    @Test
    fun aWebViewWhosePagesAreNotToldHasNeither() {
        create(1)
        load(1, site.url("/gpc"))
        // Nothing to read at first; and what the page then writes is the page's own.
        await(1, "the page to say what it read") { hasTitle("gpc=undefined then false")(it) }
        assertEquals(listOf<String?>(null), signal("/gpc"))
    }

    /**
     * What the WebView fetches of its own accord, for a page the shell asked for with the
     * header: said here as it was seen, so that a WebView that does otherwise is noticed.
     */
    @Test
    fun whatThePageFetchesAndWhereItGoesByItself() {
        create(1, privacySignal = true)
        load(1, site.url("/tracked"))
        await(1, "the page") { finished(it) }
        awaitAsked("/pixel")
        val picture = signal("/pixel")

        // The page's own link, touched in the page: the WebView's navigation.
        load(1, site.url("/"))
        await(1, "the second page") { finished(it, times = 2) && hasTitle("Home")(it) }
        tap(1, "next")
        await(1, "the link's page") { hasTitle("Landed")(it) }
        val followed = signal("/landed")
        assertEquals("the page's picture; the link the page followed", listOf(listOf<String?>(null), listOf<String?>(null)), listOf(picture, followed))
    }
}

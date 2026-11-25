package com.tigercodes.webtiger.webview

import com.tigercodes.webtiger.core.ResourceType
import org.junit.Assert.assertEquals
import org.junit.Test

/**
 * What a request is for, told from what the System WebView says of it (065): the headers are
 * those the WebView was seen to give for each kind (WebView 133, on an emulator).
 */
class RequestKindTest {
    private val agent = "User-Agent" to "Mozilla/5.0 (Linux; Android 16) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/133.0 Mobile Safari/537.36"
    private val referer = "Referer" to "http://127.0.0.1:8765/kinds"
    private val page = "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7"
    private val picture = "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8"

    private fun kind(
        vararg headers: Pair<String, String>,
        mainFrame: Boolean = false,
    ) = RequestKind.of(mainFrame, mapOf(agent, *headers))

    @Test
    fun aRequestIsToldByWhatItAcceptsFirst() {
        assertEquals(ResourceType.DOCUMENT, kind("Accept" to page, mainFrame = true))
        assertEquals(ResourceType.SUBDOCUMENT, kind("Accept" to page, "Upgrade-Insecure-Requests" to "1", referer))
        assertEquals(ResourceType.IMAGE, kind("Accept" to picture, referer))
        assertEquals(ResourceType.STYLESHEET, kind("Accept" to "text/css,*/*;q=0.1", referer))
        // A film: it accepts anything, and asks for a part of the thing.
        assertEquals(
            ResourceType.MEDIA,
            kind("Accept" to "*/*", referer, "Accept-Encoding" to "identity;q=1, *;q=0", "Range" to "bytes=0-"),
        )
    }

    @Test
    fun whatAcceptsAnythingIsAKindTheEngineCannotName() {
        // A script, a fetch of the page's own, and a font (which says where it is asked from).
        assertEquals(ResourceType.OTHER, kind("Accept" to "*/*", referer))
        assertEquals(ResourceType.OTHER, kind("Origin" to "http://127.0.0.1:8765", "Accept" to "*/*", referer))
        // A request that says nothing of what it accepts, and one with no headers at all.
        assertEquals(ResourceType.OTHER, kind(referer))
        assertEquals(ResourceType.OTHER, RequestKind.of(mainFrame = false, headers = emptyMap()))
        // What a page's own script asks for by a kind it names is not a page, a picture or a stylesheet.
        assertEquals(ResourceType.OTHER, kind("Accept" to "application/json, text/plain, */*"))
    }

    @Test
    fun theTopFramesOwnDocumentIsOneWhateverItSays() {
        assertEquals(ResourceType.DOCUMENT, kind("Accept" to "*/*", mainFrame = true))
        assertEquals(ResourceType.DOCUMENT, RequestKind.of(mainFrame = true, headers = emptyMap()))
        assertEquals(ResourceType.DOCUMENT, kind("Accept" to picture, mainFrame = true))
    }

    @Test
    fun aHeaderIsReadWhateverTheCaseOfItsNameAndOfWhatItSays() {
        assertEquals(ResourceType.STYLESHEET, kind("accept" to "Text/CSS , */*;q=0.1"))
        assertEquals(ResourceType.IMAGE, kind("ACCEPT" to " image/webp;q=0.9,*/*"))
        assertEquals(ResourceType.MEDIA, kind("accept" to "*/*", "range" to "bytes=100-"))
        // What a request accepts first is what tells: a film's range does not make a picture a film.
        assertEquals(ResourceType.IMAGE, kind("Accept" to picture, "Range" to "bytes=0-"))
    }
}

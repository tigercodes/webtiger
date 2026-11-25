package com.tigercodes.webtiger.webview

import com.tigercodes.webtiger.core.ResourceType

/**
 * What a request is for, as far as the System WebView tells (065). It does not say: it says
 * whether a request is for the top frame, and the headers the page's engine has set by then.
 * What a request accepts names three kinds, and a request for a part of a thing is a film's or
 * a sound's. A script, a fetch of the page's own and a font all accept anything, and are not
 * told apart here: they are a kind the engine cannot name, and the core guesses from the
 * address (064, 065).
 *
 * As the WebView was seen to say them (WebView 133, on an emulator):
 * - a frame's page accepts `text/html` before anything else, as the top frame's does;
 * - a picture accepts `image/avif` and other pictures first;
 * - a stylesheet accepts `text/css` first;
 * - a film asks for `Range: bytes=0-`, and accepts anything.
 */
internal object RequestKind {
    fun of(
        mainFrame: Boolean,
        headers: Map<String, String>,
    ): ResourceType {
        if (mainFrame) return ResourceType.DOCUMENT
        val first =
            header(headers, "Accept")
                .substringBefore(',')
                .substringBefore(';')
                .trim()
                .lowercase()
        return when {
            first == "text/html" -> ResourceType.SUBDOCUMENT
            first.startsWith("image/") -> ResourceType.IMAGE
            first == "text/css" -> ResourceType.STYLESHEET
            header(headers, "Range").isNotEmpty() -> ResourceType.MEDIA
            else -> ResourceType.OTHER
        }
    }

    /** A header by its name, whatever the case it was written in; empty when it is not there. */
    private fun header(
        headers: Map<String, String>,
        name: String,
    ): String = headers.entries.firstOrNull { it.key.equals(name, ignoreCase = true) }?.value.orEmpty()
}

// A page says something, or asks (016): `alert`, `confirm` and `prompt`, on the System WebView.
// The WebView would show a dialog of its own; the adapter says no to that, reports what the
// page said, and waits: the core says what the page is answered. It decides nothing, and shows
// nothing.
package com.tigercodes.webtiger.webview

import com.tigercodes.webtiger.core.v1.DialogKind
import com.tigercodes.webtiger.core.v1.DialogRequested
import com.tigercodes.webtiger.core.v1.EngineEvent
import java.net.URI

/** The dialogs of one engine's pages that wait for the core's word. Used on the main thread, as the engine is. */
internal class Dialogs(private val report: (EngineEvent) -> Unit) {
    private class Waiting(val webViewId: Long, val answer: (Boolean, String) -> Unit)

    /**
     * How many dialogs this engine has named. Each has the next number, and none is used twice:
     * the ids are the shell's own (engine.proto, `DialogRequested`).
     */
    private var named = 0L
    private val waiting = HashMap<Long, Waiting>()

    /**
     * A page of `webViewId` says `message`, or asks, from a frame at `url`. `answer` gives the
     * page its answer, once: whether it was accepted, and for a prompt the text. Nothing is
     * answered until the core has said.
     */
    fun asked(webViewId: Long, url: String, kind: DialogKind, message: String, defaultText: String, answer: (Boolean, String) -> Unit) {
        named += 1
        waiting[named] = Waiting(webViewId, answer)
        report(
            EngineEvent(
                web_view_id = webViewId,
                dialog_requested = DialogRequested(request_id = named, origin = originOf(url), kind = kind, message = message, default_text = defaultText),
            ),
        )
    }

    /** The core's word on a dialog. One that no longer waits has no page to answer. */
    fun answer(id: Long, accepted: Boolean, text: String) {
        waiting.remove(id)?.answer(accepted, text)
    }

    /** A web view is going: what its pages said is dismissed, and nothing more is said of it. */
    fun dropped(webViewId: Long) {
        val gone = waiting.filterValues { it.webViewId == webViewId }
        for ((id, dialog) in gone) {
            waiting.remove(id)
            dialog.answer(false, "")
        }
    }

    internal companion object {
        /**
         * Who called, as the contract writes an origin: the scheme, host and port of the address
         * the WebView names, and nothing of its path. An address that is none of the web's has
         * no origin: it is passed as no origin at all, and the core dismisses the dialog.
         */
        fun originOf(url: String): String {
            val address =
                try {
                    URI(url)
                } catch (_: Exception) {
                    return ""
                }
            val scheme = address.scheme?.lowercase() ?: return ""
            val host = address.host ?: return ""
            if (scheme != "http" && scheme != "https") return ""
            return if (address.port >= 0) "$scheme://$host:${address.port}" else "$scheme://$host"
        }
    }
}

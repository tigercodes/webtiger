// A server asks who is there before it answers (017): HTTP authentication, on the System
// WebView. Left alone the WebView gives the server no answer. The adapter reports that it asks
// and waits: the core says whether it is answered, and with what. It decides nothing, shows
// nothing, and keeps nothing of what was typed.
package com.tigercodes.webtiger.webview

import com.tigercodes.webtiger.core.v1.EngineEvent
import com.tigercodes.webtiger.core.v1.SignInRequested

/** What a person typed for a server: a name and a password. */
internal class Typed(val name: String, val password: String)

/** The servers' questions of one engine's web views that wait for the core's word. Used on the main thread, as the engine is. */
internal class SignIns(private val report: (EngineEvent) -> Unit) {
    private class Waiting(val webViewId: Long, val answer: (Typed?) -> Unit)

    /**
     * How many questions this engine has named. Each has the next number, and none is used
     * twice: the ids are the shell's own (engine.proto, `SignInRequested`).
     */
    private var named = 0L
    private val waiting = HashMap<Long, Waiting>()

    /**
     * A server asks who is there in the web view `webViewId`. The WebView names its `host` and
     * what the server calls what it protects (`realm`), and nothing more of it; `heading` is
     * where the web view's load was going, and `again` that the server refused an answer
     * already. `answer` gives the WebView the server's answer, once: what was typed, or null
     * for none. Nothing is answered until the core has said.
     */
    fun asked(webViewId: Long, host: String, realm: String, heading: String, again: Boolean, answer: (Typed?) -> Unit) {
        named += 1
        waiting[named] = Waiting(webViewId, answer)
        report(
            EngineEvent(
                web_view_id = webViewId,
                sign_in_requested = SignInRequested(request_id = named, host = host, realm = realm, heading = heading, again = again),
            ),
        )
    }

    /** The core's word on a server's question. One that no longer waits has no server to answer. */
    fun answer(id: Long, accepted: Boolean, name: String, password: String) {
        waiting.remove(id)?.answer(if (accepted) Typed(name, password) else null)
    }

    /** A web view is going: the servers that asked in it are given no answer. */
    fun dropped(webViewId: Long) {
        val gone = waiting.filterValues { it.webViewId == webViewId }
        for ((id, asking) in gone) {
            waiting.remove(id)
            asking.answer(null)
        }
    }
}

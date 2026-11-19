// The History list's model (008): what the core's store says was visited, a page at a time,
// found by words, and forgotten. The shell keeps no history of its own: it asks.
package com.tigercodes.webtiger.ui

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.tigercodes.webtiger.core.CoreError
import com.tigercodes.webtiger.core.v1.ForgetHistory
import com.tigercodes.webtiger.core.v1.HistoryEntry
import com.tigercodes.webtiger.core.v1.QueryHistory
import com.tigercodes.webtiger.core.v1.Request
import kotlinx.coroutines.launch

/** The history of a window's Space. Used on the main thread, as the window's model is. */
class HistoryModel(private val session: BrowserSession, private val windowId: Long) {
    /** The visits shown, newest first. */
    var entries by mutableStateOf<List<HistoryEntry>>(emptyList())
        private set

    /** There are older visits than these to show. */
    var more by mutableStateOf(false)
        private set

    /**
     * The core has answered at least once. Until then the list says nothing at all, rather than
     * that nothing was visited.
     */
    var loaded by mutableStateOf(false)
        private set

    /** The words looked for. */
    var text by mutableStateOf("")
        private set

    /** Counts what was asked: an answer to anything but the last question is out of date. */
    private var asked = 0

    /** Looks for other words: asks again, unless they are the words already looked for. */
    fun search(words: String) {
        if (words == text) return
        text = words
        reload()
    }

    /** Asks for the newest visits that match the words. */
    fun reload() = ask(before = 0)

    /** Asks for the visits older than the last one shown. */
    fun showOlder() {
        if (!more) return
        val last = entries.lastOrNull() ?: return
        ask(before = last.visit_id)
    }

    /** Forgets one visit. It leaves the list at once; the store forgets it for good. */
    fun forget(entry: HistoryEntry) {
        session.send(Request(forget_history = ForgetHistory(window_id = windowId, visit_id = entry.visit_id)))
        entries = entries.filter { it.visit_id != entry.visit_id }
    }

    /** Forgets every visit made at or after a time, and asks again for what is left. */
    fun forgetSince(millis: Long) {
        session.send(Request(forget_history = ForgetHistory(window_id = windowId, since_ms = millis)))
        reload()
    }

    /** Forgets every visit. */
    fun forgetEverything() {
        session.send(Request(forget_history = ForgetHistory(window_id = windowId, everything = true)))
        reload()
    }

    private fun ask(before: Long) {
        asked += 1
        val question = asked
        val query = QueryHistory(window_id = windowId, text = text, before_visit_id = before)
        // Started after whatever was sent before it, on the same thread: a question that follows
        // a forgetting is answered with what is left.
        session.scope.launch {
            val page =
                try {
                    session.core.request(Request(query_history = query)).history_page
                } catch (_: CoreError) {
                    null
                } ?: return@launch
            if (question != asked) return@launch
            entries = if (before == 0L) page.entries else entries + page.entries
            more = page.more
            loaded = true
        }
    }
}

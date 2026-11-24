// The Archive's model (049): the tabs that were tidied out of the Space a window shows, newest
// first, a page at a time; brought back, and forgotten. The shell keeps no archive of its own:
// it asks.
package com.tigercodes.webtiger.ui

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.tigercodes.webtiger.core.CoreError
import com.tigercodes.webtiger.core.v1.ArchiveEntry
import com.tigercodes.webtiger.core.v1.ForgetArchived
import com.tigercodes.webtiger.core.v1.QueryArchive
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.RestoreArchived
import kotlinx.coroutines.launch

/** The Archive of a window's Space. Used on the main thread, as the window's model is. */
class ArchiveModel(private val session: BrowserSession, private val windowId: Long) {
    /** The tidied tabs shown, newest first. */
    var entries by mutableStateOf<List<ArchiveEntry>>(emptyList())
        private set

    /** There are older ones than these to show. */
    var more by mutableStateOf(false)
        private set

    /**
     * The core has answered at least once. Until then the list says nothing at all, rather than
     * that nothing was tidied.
     */
    var loaded by mutableStateOf(false)
        private set

    /** Counts what was asked: an answer to anything but the last question is out of date. */
    private var asked = 0

    /** Asks for the newest of the Archive. */
    fun reload() = ask(before = 0)

    /** Asks for the ones older than the last one shown. */
    fun showOlder() {
        if (!more) return
        val last = entries.lastOrNull() ?: return
        ask(before = last.id)
    }

    /**
     * Brings a tidied tab back: the core makes it a tab again, in front, and `onBack` is called.
     * It leaves the list at once. When the core will not (the Archive has it no longer: another
     * window brought it back, or forgot it), the list is asked for again.
     */
    fun bringBack(entry: ArchiveEntry, onBack: () -> Unit) {
        entries = entries.filter { it.id != entry.id }
        session.scope.launch {
            val opened =
                try {
                    session.core.request(Request(restore_archived = RestoreArchived(window_id = windowId, entry_id = entry.id))).tab_opened
                } catch (_: CoreError) {
                    null
                }
            if (opened != null) onBack() else reload()
        }
    }

    /** Forgets one tidied tab. It leaves the list at once; the store forgets it for good. */
    fun forget(entry: ArchiveEntry) {
        session.send(Request(forget_archived = ForgetArchived(window_id = windowId, entry_id = entry.id)))
        entries = entries.filter { it.id != entry.id }
    }

    /** Forgets all of the Space's Archive, and asks again for what is left: nothing. */
    fun forgetEverything() {
        session.send(Request(forget_archived = ForgetArchived(window_id = windowId, all = true)))
        reload()
    }

    private fun ask(before: Long) {
        asked += 1
        val question = asked
        val query = QueryArchive(window_id = windowId, before = before)
        // Started after whatever was sent before it, on the same thread: a question that follows
        // a forgetting is answered with what is left.
        session.scope.launch {
            val page =
                try {
                    session.core.request(Request(query_archive = query)).archive_page
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

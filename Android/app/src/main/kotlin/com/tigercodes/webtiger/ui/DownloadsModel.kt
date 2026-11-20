// The Downloads list's model (013): what the core says was downloaded, a page at a time, with
// how far each one that is coming has got. The shell keeps no list of its own: it asks.
package com.tigercodes.webtiger.ui

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.tigercodes.webtiger.core.CoreError
import com.tigercodes.webtiger.core.v1.CancelDownload
import com.tigercodes.webtiger.core.v1.DownloadEntry
import com.tigercodes.webtiger.core.v1.DownloadState
import com.tigercodes.webtiger.core.v1.ForgetDownloads
import com.tigercodes.webtiger.core.v1.QueryDownloads
import com.tigercodes.webtiger.core.v1.Request
import kotlinx.coroutines.launch

/** The downloads of a window's Space. Used on the main thread, as the window's model is. */
class DownloadsModel(private val session: BrowserSession, private val windowId: Long) {
    /** The downloads shown, newest first. */
    var entries by mutableStateOf<List<DownloadEntry>>(emptyList())
        private set

    /** There are older downloads than these to show. */
    var more by mutableStateOf(false)
        private set

    /**
     * The core has answered at least once. Until then the list says nothing at all, rather than
     * that nothing was downloaded.
     */
    var loaded by mutableStateOf(false)
        private set

    /** Counts what was asked: an answer to anything but the last question is out of date. */
    private var asked = 0

    /** Something in the list is still coming. */
    val isComing: Boolean get() = entries.any { it.state == DownloadState.DOWNLOAD_STATE_COMING }

    /** Asks for the list again, as far down as it is shown. */
    fun reload() = ask(before = 0)

    /** Asks for the downloads older than the last one shown. */
    fun showOlder() {
        if (!more) return
        val last = entries.lastOrNull() ?: return
        ask(before = last.id)
    }

    /** Stops a download that is coming. What the core then says of it is what the list shows. */
    fun stop(entry: DownloadEntry) {
        session.send(Request(cancel_download = CancelDownload(download_id = entry.id)))
        reload()
    }

    /** Takes one download off the list. It leaves at once; its file stays where it is. */
    fun forget(entry: DownloadEntry) {
        session.send(Request(forget_downloads = ForgetDownloads(window_id = windowId, download_id = entry.id)))
        entries = entries.filter { it.id != entry.id }
    }

    /** Takes every download off the list. */
    fun forgetEverything() {
        session.send(Request(forget_downloads = ForgetDownloads(window_id = windowId, everything = true)))
        reload()
    }

    private fun ask(before: Long) {
        asked += 1
        val question = asked
        // A list that has been taken further down is asked for whole again, so that what it
        // shows of the older ones is as fresh as the rest.
        val limit = if (before == 0L && entries.size > PAGE) entries.size.coerceAtMost(MOST) else 0
        val query = QueryDownloads(window_id = windowId, before_id = before, limit = limit)
        // Started after whatever was sent before it, on the same thread: a question that follows
        // a forgetting is answered with what is left.
        session.scope.launch {
            val page =
                try {
                    session.core.request(Request(query_downloads = query)).downloads_page
                } catch (_: CoreError) {
                    null
                } ?: return@launch
            if (question != asked) return@launch
            entries = if (before == 0L) page.downloads else entries + page.downloads
            more = page.more
            loaded = true
        }
    }

    private companion object {
        /** How many downloads the core gives when it is not told. */
        const val PAGE = 100

        /** The most it gives at once. */
        const val MOST = 500
    }
}

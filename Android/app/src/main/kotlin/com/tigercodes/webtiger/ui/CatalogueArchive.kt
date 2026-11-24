// The Archive (049) in the design catalogue: the list under the switch it has in the window,
// with tidied tabs in it, and with none. What a core would answer, with no core behind it.
package com.tigercodes.webtiger.ui

import android.content.Context
import androidx.annotation.StringRes
import androidx.compose.foundation.layout.Box
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.Ack
import com.tigercodes.webtiger.core.v1.ArchiveEntry
import com.tigercodes.webtiger.core.v1.ArchivePage
import com.tigercodes.webtiger.core.v1.Response
import com.tigercodes.webtiger.webview.SystemWebViewEngine
import java.time.ZoneOffset
import java.util.Locale
import kotlinx.coroutines.CoroutineScope

/** One state of the Archive (049). */
internal enum class CatalogueArchive(@StringRes val title: Int) {
    LIST(R.string.catalogue_state_archive_list),
    EMPTY(R.string.catalogue_state_archive_empty),
    ;

    /**
     * What a core would answer for the state: two tabs tidied at one look, one whose name is
     * longer than the list is wide; one with no name, tidied half a day before; and one of last
     * week. The addresses are the documentation's own, and the times are fixed: midday on the
     * first of January 2026, and before.
     */
    private val page: ArchivePage
        get() =
            if (this == LIST) {
                ArchivePage(
                    entries =
                        listOf(
                            tidied(4, "Example Domain", "https://example.com/", "example.com", hoursAgo = 0),
                            tidied(3, "A page whose title is longer than the list is wide, and then goes on", "https://example.org/a/long/path", "example.org/a/long/path", hoursAgo = 0),
                            tidied(2, "", "https://example.net/plain.txt", "example.net/plain.txt", hoursAgo = 12),
                            tidied(1, "Last week’s page", "http://example.com/last-week", "http://example.com/last-week", hoursAgo = 24 * 7),
                        ),
                    more = true,
                )
            } else {
                ArchivePage()
            }

    private fun tidied(id: Long, title: String, url: String, display: String, hoursAgo: Long): ArchiveEntry =
        ArchiveEntry(id = id, title = title, url = url, display = display, tidied_at = 1_767_268_800_000 - hoursAgo * 3_600_000)

    /** A model over a core that answers with this state, and nothing else. */
    fun model(context: Context, scope: CoroutineScope): ArchiveModel {
        val answer = page
        val core = ScriptedCore { request -> if (request.query_archive != null) Response(archive_page = answer) else Response(ack = Ack()) }
        return ArchiveModel(BrowserSession(core, SystemWebViewEngine(context), scope), windowId = 1)
    }
}

/**
 * The Archive in one state: [ArchiveList] over a scripted core that answers with it, under the
 * switch it has in the window. Its times are shown as one place and one language show them, so
 * a picture of it is the same wherever it is drawn.
 */
@Composable
internal fun CatalogueArchiveList(state: CatalogueArchive, modifier: Modifier = Modifier) {
    val context = LocalContext.current.applicationContext
    val scope = rememberCoroutineScope()
    val model = remember(state) { state.model(context, scope) }
    Box(modifier = modifier) {
        ArchiveList(model, onBroughtBack = {}, onDone = {}, zone = ZoneOffset.UTC, locale = Locale.UK, switcher = { LibrarySwitch(LibraryPart.ARCHIVE) {} })
    }
}

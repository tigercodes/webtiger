// The Downloads list (013): what was downloaded, newest first; how far each one that is coming
// has got, and how each one that ended did. One that is coming can be stopped; one that has
// ended can be taken off the list, which touches no file.
package com.tigercodes.webtiger.ui

import android.content.Context
import android.text.format.Formatter
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.LocalMinimumInteractiveComponentSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.core.v1.DownloadEntry
import com.tigercodes.webtiger.core.v1.DownloadState
import com.tigercodes.webtiger.design.Tokens
import kotlinx.coroutines.delay

/**
 * The list over `model`. `onDone` is called when a person is finished. With a `switcher`, it
 * stands where the title would, on a row of its own: the list is then one of two a person can
 * choose between. `watching` is whether the list asks again by itself: not a picture of one.
 */
@Composable
fun DownloadsList(
    model: DownloadsModel,
    onDone: () -> Unit,
    modifier: Modifier = Modifier,
    switcher: (@Composable () -> Unit)? = null,
    watching: Boolean = true,
) {
    // Asked again every second for as long as the list is shown: how far a download has got is
    // not pushed, and one that starts while the list is open belongs in it.
    LaunchedEffect(model, watching) {
        model.reload()
        while (watching) {
            delay(ASK_AGAIN_MS)
            model.reload()
        }
    }
    Surface(modifier = modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background) {
        Column(modifier = Modifier.fillMaxSize()) {
            ListHeader(
                switcher = switcher,
                title = R.string.downloads_title,
            ) {
                TextButton(onClick = model::forgetEverything, enabled = model.entries.isNotEmpty(), modifier = Modifier.testTag("downloads-clear")) {
                    Text(stringResource(R.string.downloads_clear))
                }
                TextButton(onClick = onDone, modifier = Modifier.testTag("downloads-done")) { Text(stringResource(R.string.downloads_done)) }
            }
            HorizontalDivider(color = Tokens.Palette.hairline.current)
            when {
                // Nothing is said until the core has answered: not that nothing was downloaded.
                !model.loaded -> Unit
                model.entries.isEmpty() ->
                    Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                        Text(
                            text = stringResource(R.string.downloads_empty),
                            textAlign = TextAlign.Center,
                            modifier = Modifier.padding(Tokens.Space.l).testTag("downloads-nothing"),
                        )
                    }
                else ->
                    LazyColumn(modifier = Modifier.fillMaxSize()) {
                        items(model.entries, key = { it.id }) { entry ->
                            DownloadRow(entry = entry, onStop = { model.stop(entry) }, onForget = { model.forget(entry) })
                            HorizontalDivider(color = Tokens.Palette.hairline.current, modifier = Modifier.padding(start = Tokens.Space.m))
                        }
                        if (model.more) {
                            item {
                                TextButton(onClick = model::showOlder, modifier = Modifier.fillMaxWidth().testTag("downloads-older")) {
                                    Text(stringResource(R.string.downloads_older))
                                }
                            }
                        }
                    }
            }
        }
    }
}

/**
 * The top of a list in the sheet: its title, or the switch that stands in for it, and the
 * list's own buttons. With a switch, the buttons have the row below it: a switch and two
 * buttons are too many for one row on a phone (as Apple's audit said of the iPhone's, 013 §6).
 */
@Composable
internal fun ListHeader(switcher: (@Composable () -> Unit)?, title: Int, buttons: @Composable () -> Unit) {
    if (switcher != null) {
        Column(modifier = Modifier.fillMaxWidth().padding(start = Tokens.Space.m, end = Tokens.Space.xs)) {
            switcher()
            Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                Spacer(modifier = Modifier.weight(1f))
                buttons()
            }
        }
    } else {
        Row(modifier = Modifier.fillMaxWidth().padding(start = Tokens.Space.m, end = Tokens.Space.xs), verticalAlignment = Alignment.CenterVertically) {
            Text(
                text = stringResource(title),
                style = MaterialTheme.typography.titleLarge,
                fontWeight = FontWeight.SemiBold,
                modifier = Modifier.weight(1f).semantics { heading() },
            )
            buttons()
        }
    }
}

/** One download: its file's name, where it came from and how it stands; a button stops it, or takes it off the list. */
@Composable
private fun DownloadRow(entry: DownloadEntry, onStop: () -> Unit, onForget: () -> Unit) {
    val coming = entry.state == DownloadState.DOWNLOAD_STATE_COMING
    val standing = standing(LocalContext.current, entry)
    Row(modifier = Modifier.fillMaxWidth().padding(start = Tokens.Space.m), verticalAlignment = Alignment.CenterVertically) {
        Column(
            modifier =
                Modifier
                    .weight(1f)
                    .heightIn(min = LocalMinimumInteractiveComponentSize.current)
                    .padding(vertical = Tokens.Space.xs)
                    // One thing to a screen reader: the file, its site, and how it stands.
                    .semantics(mergeDescendants = true) {}
                    .testTag("download-entry"),
            verticalArrangement = Arrangement.Center,
        ) {
            // Each on a line of its own, as in the History list. A name may be any length: it
            // is cut to two lines, in its middle, where its start and its ending both stay.
            Text(text = entry.file_name, maxLines = 2, overflow = TextOverflow.MiddleEllipsis)
            if (entry.host.isNotEmpty()) {
                Text(text = entry.host, maxLines = 1, overflow = TextOverflow.MiddleEllipsis, style = MaterialTheme.typography.bodySmall)
            }
            if (standing.isNotEmpty()) Text(text = standing, style = MaterialTheme.typography.bodySmall)
            // How far it has got, drawn, when its size is known. One whose size is not known
            // has only its words: a bar that moved without saying anything would be the one
            // thing in the list that never stood still.
            if (coming && entry.total_bytes > 0) {
                LinearProgressIndicator(
                    progress = { (entry.received_bytes.toFloat() / entry.total_bytes.toFloat()).coerceIn(0f, 1f) },
                    modifier = Modifier.fillMaxWidth().padding(top = Tokens.Space.xs, end = Tokens.Space.m).clearAndSetSemantics { },
                )
            }
        }
        if (coming) {
            IconButton(onClick = onStop, modifier = Modifier.testTag("download-stop")) {
                Icon(painter = painterResource(R.drawable.ic_close), contentDescription = stringResource(R.string.downloads_stop))
            }
        } else {
            IconButton(onClick = onForget, modifier = Modifier.testTag("download-forget")) {
                Icon(painter = painterResource(R.drawable.ic_delete), contentDescription = stringResource(R.string.downloads_forget))
            }
        }
    }
}

/**
 * How a download stands, in words: how much of it has arrived, how big it is, or why it
 * failed. The core speaks codes; the words are here.
 */
internal fun standing(context: Context, entry: DownloadEntry): String =
    when (entry.state) {
        DownloadState.DOWNLOAD_STATE_COMING ->
            if (entry.total_bytes > 0) {
                context.getString(R.string.downloads_coming, size(context, entry.received_bytes), size(context, entry.total_bytes))
            } else {
                context.getString(R.string.downloads_coming_so_far, size(context, entry.received_bytes))
            }
        DownloadState.DOWNLOAD_STATE_FINISHED -> if (entry.total_bytes > 0) size(context, entry.total_bytes) else ""
        else -> context.getString(wordsFor(entry.error_code))
    }

/**
 * A failure's words, by its code. A code this build has no words for still says something, and
 * never shows the code's own text.
 */
internal fun wordsFor(code: String): Int =
    when (code) {
        "download.cancelled" -> R.string.error_download_cancelled
        "download.network" -> R.string.error_download_network
        "download.disk" -> R.string.error_download_disk
        "download.interrupted" -> R.string.error_download_interrupted
        else -> R.string.error_download_failed
    }

/** A number of bytes as a person reads a file's size. */
internal fun size(context: Context, bytes: Long): String = Formatter.formatShortFileSize(context, bytes.coerceAtLeast(0))

/** How long between two questions about the list while it is shown. */
private const val ASK_AGAIN_MS = 1_000L

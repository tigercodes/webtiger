// The Archive (049): the tabs that were tidied out of this Space, newest first. One that is
// chosen is a tab again; one is forgotten, or all of them. The fourth of the Library's lists.
package com.tigercodes.webtiger.ui

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LocalMinimumInteractiveComponentSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.core.v1.ArchiveEntry
import com.tigercodes.webtiger.design.Tokens
import java.time.Instant
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.time.format.FormatStyle
import java.util.Locale

/**
 * The list over `model`. `onBroughtBack` is called when a tab a person chose is a tab again;
 * `onDone` when they are finished. Times are shown in `zone` and as `locale` writes them: the
 * person's own, unless a picture of the list is to be the same wherever it is drawn. With a
 * `switcher`, it stands where the title would, on a row of its own.
 */
@Composable
fun ArchiveList(
    model: ArchiveModel,
    onBroughtBack: () -> Unit,
    onDone: () -> Unit,
    modifier: Modifier = Modifier,
    zone: ZoneId = ZoneId.systemDefault(),
    locale: Locale = LocalConfiguration.current.locales[0],
    switcher: (@Composable () -> Unit)? = null,
) {
    LaunchedEffect(model) { model.reload() }
    // The question of whether to forget them all is up.
    var asking by remember { mutableStateOf(false) }
    val times = remember(zone, locale) { DateTimeFormatter.ofLocalizedDateTime(FormatStyle.MEDIUM, FormatStyle.SHORT).withLocale(locale).withZone(zone) }
    Surface(modifier = modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background) {
        Column(modifier = Modifier.fillMaxSize()) {
            ListHeader(switcher = switcher, title = R.string.archive_title) {
                // There is nothing to forget in an Archive that lists nothing.
                TextButton(onClick = { asking = true }, enabled = model.entries.isNotEmpty(), modifier = Modifier.testTag("archive-clear")) {
                    Text(stringResource(R.string.archive_clear))
                }
                TextButton(onClick = onDone, modifier = Modifier.testTag("archive-done")) { Text(stringResource(R.string.archive_done)) }
            }
            HorizontalDivider(color = Tokens.Palette.hairline.current)
            when {
                // Nothing is said until the core has answered: not that nothing was tidied.
                !model.loaded -> Unit
                model.entries.isEmpty() ->
                    Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                        Column(
                            modifier = Modifier.padding(Tokens.Space.l),
                            verticalArrangement = Arrangement.spacedBy(Tokens.Space.s),
                            horizontalAlignment = Alignment.CenterHorizontally,
                        ) {
                            Text(text = stringResource(R.string.archive_empty), textAlign = TextAlign.Center, modifier = Modifier.testTag("archive-nothing"))
                            // How a Space comes to be tidied: nothing is, until a person says so.
                            Text(text = stringResource(R.string.archive_empty_detail), textAlign = TextAlign.Center, style = MaterialTheme.typography.bodyMedium)
                        }
                    }
                else ->
                    LazyColumn(modifier = Modifier.fillMaxSize()) {
                        items(model.entries, key = { it.id }) { entry ->
                            ArchiveRow(
                                entry = entry,
                                tidied = stringResource(R.string.archive_tidied, times.format(Instant.ofEpochMilli(entry.tidied_at))),
                                onBringBack = { model.bringBack(entry, onBroughtBack) },
                                onForget = { model.forget(entry) },
                            )
                            HorizontalDivider(color = Tokens.Palette.hairline.current, modifier = Modifier.padding(start = Tokens.Space.m))
                        }
                        if (model.more) {
                            item {
                                TextButton(onClick = model::showOlder, modifier = Modifier.fillMaxWidth().testTag("archive-older")) {
                                    Text(stringResource(R.string.archive_older))
                                }
                            }
                        }
                    }
            }
        }
    }
    if (asking) {
        AlertDialog(
            onDismissRequest = { asking = false },
            title = { Text(stringResource(R.string.archive_clear_title)) },
            text = { Text(stringResource(R.string.archive_clear_detail)) },
            confirmButton = {
                TextButton(
                    onClick = {
                        asking = false
                        model.forgetEverything()
                    },
                    modifier = Modifier.testTag("archive-clear-everything"),
                ) { Text(stringResource(R.string.archive_clear_everything)) }
            },
            dismissButton = { TextButton(onClick = { asking = false }) { Text(stringResource(R.string.archive_clear_cancel)) } },
        )
    }
}

/** One tidied tab: its name, its address and when it was tidied; choosing it brings it back; a button forgets it. */
@Composable
private fun ArchiveRow(entry: ArchiveEntry, tidied: String, onBringBack: () -> Unit, onForget: () -> Unit) {
    Row(modifier = Modifier.fillMaxWidth().padding(start = Tokens.Space.m), verticalAlignment = Alignment.CenterVertically) {
        Column(
            modifier =
                Modifier
                    .weight(1f)
                    .clickable(role = Role.Button, onClick = onBringBack)
                    .heightIn(min = LocalMinimumInteractiveComponentSize.current)
                    .padding(vertical = Tokens.Space.xs)
                    .testTag("archive-entry"),
            verticalArrangement = Arrangement.Center,
        ) {
            // Each on lines of its own, as a visit's are (008). A page with no name is named by
            // its address.
            Text(text = entry.title.ifEmpty { entry.display }, maxLines = 2, overflow = TextOverflow.Ellipsis)
            if (entry.title.isNotEmpty()) {
                Text(text = entry.display, maxLines = 2, overflow = TextOverflow.MiddleEllipsis, style = MaterialTheme.typography.bodySmall)
            }
            Text(text = tidied, style = MaterialTheme.typography.bodySmall)
        }
        IconButton(onClick = onForget, modifier = Modifier.testTag("archive-forget")) {
            Icon(painter = painterResource(R.drawable.ic_delete), contentDescription = stringResource(R.string.archive_forget))
        }
    }
}

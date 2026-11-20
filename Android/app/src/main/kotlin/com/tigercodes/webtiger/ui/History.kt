// The History list (008): the pages visited, newest first, found by words of their titles and
// addresses, opened again by choosing one, and forgotten one at a time or many at once.
package com.tigercodes.webtiger.ui

import androidx.compose.foundation.background
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
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
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
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.core.v1.HistoryEntry
import com.tigercodes.webtiger.design.Tokens
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.time.format.FormatStyle
import java.util.Locale

/**
 * The list over `model`. `onOpen` is called with the visit a person chose; `onDone` when they
 * are finished. Times are shown in `zone` and as `locale` writes them: the person's own, unless
 * a picture of the list is to be the same wherever it is drawn. With a `switcher`, it stands
 * where the title would, on a row of its own: the list is then one of two a person can choose
 * between (013).
 */
@Composable
fun HistoryList(
    model: HistoryModel,
    onOpen: (HistoryEntry) -> Unit,
    onDone: () -> Unit,
    modifier: Modifier = Modifier,
    zone: ZoneId = ZoneId.systemDefault(),
    locale: Locale = LocalConfiguration.current.locales[0],
    switcher: (@Composable () -> Unit)? = null,
) {
    LaunchedEffect(model) { model.reload() }
    // The question of which visits to forget is up.
    var choosing by remember { mutableStateOf(false) }
    val times = remember(zone, locale) { DateTimeFormatter.ofLocalizedDateTime(FormatStyle.MEDIUM, FormatStyle.SHORT).withLocale(locale).withZone(zone) }
    Surface(modifier = modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background) {
        Column(modifier = Modifier.fillMaxSize()) {
            ListHeader(switcher = switcher, title = R.string.history_title) {
                // A button that asks, and not a menu: forgetting is chosen in two steps.
                TextButton(onClick = { choosing = true }, modifier = Modifier.testTag("history-clear")) { Text(stringResource(R.string.history_clear)) }
                TextButton(onClick = onDone, modifier = Modifier.testTag("history-done")) { Text(stringResource(R.string.history_done)) }
            }
            SearchField(model)
            HorizontalDivider(color = Tokens.Palette.hairline.current)
            when {
                // Nothing is said until the core has answered: not that nothing was visited.
                !model.loaded -> Unit
                model.entries.isEmpty() ->
                    Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                        Text(
                            text = stringResource(if (model.text.isEmpty()) R.string.history_empty else R.string.history_none_found),
                            textAlign = TextAlign.Center,
                            modifier = Modifier.padding(Tokens.Space.l).testTag("history-nothing"),
                        )
                    }
                else ->
                    LazyColumn(modifier = Modifier.fillMaxSize()) {
                        items(model.entries, key = { it.visit_id }) { entry ->
                            HistoryRow(
                                entry = entry,
                                time = times.format(Instant.ofEpochMilli(entry.visited_at_ms)),
                                onOpen = { onOpen(entry) },
                                onForget = { model.forget(entry) },
                            )
                            HorizontalDivider(color = Tokens.Palette.hairline.current, modifier = Modifier.padding(start = Tokens.Space.m))
                        }
                        if (model.more) {
                            item {
                                TextButton(onClick = model::showOlder, modifier = Modifier.fillMaxWidth().testTag("history-older")) {
                                    Text(stringResource(R.string.history_older))
                                }
                            }
                        }
                    }
            }
        }
    }
    if (choosing) {
        WhatToForget(
            onDismiss = { choosing = false },
            onChosen = { forget ->
                choosing = false
                forget()
            },
            model = model,
            zone = zone,
        )
    }
}

/** The field for words: each change asks the core again. */
@Composable
private fun SearchField(model: HistoryModel) {
    val label = stringResource(R.string.history_search)
    BasicTextField(
        value = model.text,
        onValueChange = model::search,
        singleLine = true,
        textStyle = MaterialTheme.typography.bodyLarge.copy(color = MaterialTheme.colorScheme.onBackground),
        cursorBrush = SolidColor(MaterialTheme.colorScheme.onBackground),
        keyboardOptions = KeyboardOptions(capitalization = KeyboardCapitalization.None, autoCorrectEnabled = false, imeAction = ImeAction.Search),
        modifier = Modifier.fillMaxWidth().padding(Tokens.Space.m).semantics { contentDescription = label }.testTag("history-search"),
        decorationBox = { field ->
            Box(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .heightIn(min = LocalMinimumInteractiveComponentSize.current)
                        .clip(RoundedCornerShape(Tokens.Radius.field))
                        .background(Tokens.Palette.fieldFill.current)
                        .padding(horizontal = Tokens.Space.m),
                contentAlignment = Alignment.CenterStart,
            ) {
                // Words a person must be able to read: the primary colour (001 F14).
                if (model.text.isEmpty()) Text(text = label, style = MaterialTheme.typography.bodyLarge)
                field()
            }
        },
    )
}

/** One visit: its name, its address and when; choosing it opens it; a button forgets it. */
@Composable
private fun HistoryRow(entry: HistoryEntry, time: String, onOpen: () -> Unit, onForget: () -> Unit) {
    Row(modifier = Modifier.fillMaxWidth().padding(start = Tokens.Space.m), verticalAlignment = Alignment.CenterVertically) {
        Column(
            modifier =
                Modifier
                    .weight(1f)
                    .clickable(role = Role.Button, onClick = onOpen)
                    .heightIn(min = LocalMinimumInteractiveComponentSize.current)
                    .padding(vertical = Tokens.Space.xs)
                    .testTag("history-entry"),
            verticalArrangement = Arrangement.Center,
        ) {
            // Each on lines of its own, so that at the largest text sizes none is cut short by
            // its neighbour: the name, the address, and when. A page with no title is named by
            // its address. An address may be any length: it is cut to two lines, in the middle,
            // so that its site and its end are both there.
            Text(text = entry.title.ifEmpty { entry.display }, maxLines = 2, overflow = TextOverflow.Ellipsis)
            if (entry.title.isNotEmpty()) {
                Text(text = entry.display, maxLines = 2, overflow = TextOverflow.MiddleEllipsis, style = MaterialTheme.typography.bodySmall)
            }
            Text(text = time, style = MaterialTheme.typography.bodySmall)
        }
        IconButton(onClick = onForget, modifier = Modifier.testTag("history-forget")) {
            Icon(painter = painterResource(R.drawable.ic_delete), contentDescription = stringResource(R.string.history_forget))
        }
    }
}

/** Asks which visits to forget: the last hour, today, or everything. */
@Composable
private fun WhatToForget(onDismiss: () -> Unit, onChosen: (() -> Unit) -> Unit, model: HistoryModel, zone: ZoneId) {
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(stringResource(R.string.history_clear_title)) },
        text = {
            Column {
                Text(stringResource(R.string.history_clear_detail))
                TextButton(onClick = { onChosen { model.forgetSince(System.currentTimeMillis() - 3_600_000) } }) {
                    Text(stringResource(R.string.history_clear_hour))
                }
                TextButton(onClick = { onChosen { model.forgetSince(LocalDate.now(zone).atStartOfDay(zone).toInstant().toEpochMilli()) } }) {
                    Text(stringResource(R.string.history_clear_today))
                }
                TextButton(onClick = { onChosen(model::forgetEverything) }, modifier = Modifier.testTag("history-clear-everything")) {
                    Text(stringResource(R.string.history_clear_everything))
                }
            }
        },
        confirmButton = {},
        dismissButton = { TextButton(onClick = onDismiss) { Text(stringResource(R.string.history_clear_cancel)) } },
    )
}

// What is awake (056): every tab that has a page loaded, wherever it is, with why it is awake;
// and "Sleep All Others". The fifth of the Library's lists (057). It is asked for when it is
// opened, and again whenever the window's own row changes while it is open: there is no timer.
package com.tigercodes.webtiger.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.LocalMinimumInteractiveComponentSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.core.v1.AwakeTab
import com.tigercodes.webtiger.core.v1.KeepAwake
import com.tigercodes.webtiger.design.Tokens

/**
 * The list over `model`. `onDone` is called when the person is finished. `watching` is the
 * window's row: the list is asked for again when it changes. With a `switcher`, it stands where
 * the title would, on a row of its own.
 */
@Composable
fun AwakeList(
    model: AwakeModel,
    onDone: () -> Unit,
    modifier: Modifier = Modifier,
    watching: Any? = null,
    switcher: (@Composable () -> Unit)? = null,
) {
    LaunchedEffect(model, watching) { model.reload() }
    Surface(modifier = modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background) {
        Column(modifier = Modifier.fillMaxSize()) {
            ListHeader(switcher = switcher, title = R.string.awake_title) {
                // Nothing that is listed could be put to sleep: there are no others.
                TextButton(onClick = model::sleepOthers, enabled = model.othersCouldSleep, modifier = Modifier.testTag("awake-sleep-others")) {
                    Text(stringResource(R.string.awake_sleep_others))
                }
                TextButton(onClick = onDone, modifier = Modifier.testTag("awake-done")) { Text(stringResource(R.string.awake_done)) }
            }
            HorizontalDivider(color = Tokens.Palette.hairline.current)
            when {
                // Nothing is said until the core has answered: not that no tab is awake.
                !model.loaded -> Unit
                model.tabs.isEmpty() ->
                    Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                        Column(
                            modifier = Modifier.padding(Tokens.Space.l),
                            verticalArrangement = Arrangement.spacedBy(Tokens.Space.s),
                            horizontalAlignment = Alignment.CenterHorizontally,
                        ) {
                            Text(text = stringResource(R.string.awake_empty), textAlign = TextAlign.Center, modifier = Modifier.testTag("awake-nothing"))
                            AwakeCounts(model)
                        }
                    }
                else ->
                    LazyColumn(modifier = Modifier.fillMaxSize().testTag("awake-list")) {
                        items(model.tabs, key = { it.tab_id }) { tab ->
                            AwakeRow(tab)
                            HorizontalDivider(color = Tokens.Palette.hairline.current, modifier = Modifier.padding(start = Tokens.Space.m))
                        }
                        item {
                            Column(modifier = Modifier.fillMaxWidth().padding(horizontal = Tokens.Space.m, vertical = Tokens.Space.s)) { AwakeCounts(model) }
                        }
                    }
            }
        }
    }
}

/** How many are asleep; and, in an ordinary window, how many are awake in private ones. */
@Composable
private fun AwakeCounts(model: AwakeModel) {
    Text(text = stringResource(R.string.awake_asleep, model.asleep), style = MaterialTheme.typography.bodySmall, modifier = Modifier.testTag("awake-asleep"))
    if (model.privateTabs > 0) {
        Text(
            text = stringResource(R.string.awake_private, model.privateTabs),
            style = MaterialTheme.typography.bodySmall,
            modifier = Modifier.testTag("awake-private"),
        )
    }
}

/** A tab that is awake: its name, where it is, and why it is awake. Not a button: a tab is gone to where it is, in its row. */
@Composable
private fun AwakeRow(tab: AwakeTab) {
    val name = awakeName(tab, untitled = stringResource(R.string.tabs_untitled))
    val place = awakePlace(tab, home = stringResource(R.string.space_home), privateWindow = stringResource(R.string.private_window))
    val reasons = awakeReasons(tab).map { stringResource(it) }
    val why = awakeWhy(tab, reasons, onScreen = stringResource(R.string.awake_why_in_sight), waiting = stringResource(R.string.awake_why_waiting))
    Column(
        modifier =
            Modifier
                .fillMaxWidth()
                // One thing to a screen reader: the tab, where it is, and why it is awake.
                .semantics(mergeDescendants = true) {}
                .heightIn(min = LocalMinimumInteractiveComponentSize.current)
                .padding(horizontal = Tokens.Space.m, vertical = Tokens.Space.xs)
                .testTag("awake-tab"),
        verticalArrangement = Arrangement.Center,
    ) {
        Text(text = name, maxLines = 2, overflow = TextOverflow.Ellipsis)
        Text(text = place, maxLines = 2, overflow = TextOverflow.MiddleEllipsis, style = MaterialTheme.typography.bodySmall)
        if (why.isNotEmpty()) Text(text = why, style = MaterialTheme.typography.bodySmall, modifier = Modifier.testTag("awake-why"))
    }
}

/** A tab's name: its page's title, or its site, or the words for a tab with no page. */
internal fun awakeName(tab: AwakeTab, untitled: String): String = tab.title.ifEmpty { tab.host.ifEmpty { untitled } }

/**
 * Where a tab is: its Space by name, with the shell's own word for the Space the app made, or
 * that it is a private window's; and its site, when its name is not that.
 */
internal fun awakePlace(tab: AwakeTab, home: String, privateWindow: String): String {
    val space = if (tab.private_window) privateWindow else tab.space_name.ifEmpty { home }
    return if (tab.host.isEmpty() || tab.title.isEmpty()) space else "$space · ${tab.host}"
}

/**
 * The words for the core's reasons, in the core's order. A reason this shell has no words for
 * (a newer core's) is not said, rather than said wrongly.
 */
internal fun awakeReasons(tab: AwakeTab): List<Int> = tab.keeps_awake.mapNotNull(::awakeWord)

/** The words for one of the core's reasons. */
internal fun awakeWord(reason: KeepAwake): Int? =
    when (reason) {
        KeepAwake.KEEP_AWAKE_SOUND -> R.string.awake_why_sound
        KeepAwake.KEEP_AWAKE_CAPTURE -> R.string.awake_why_capture
        KeepAwake.KEEP_AWAKE_TYPED -> R.string.awake_why_typed
        KeepAwake.KEEP_AWAKE_DOWNLOAD -> R.string.awake_why_download
        KeepAwake.KEEP_AWAKE_QUESTION -> R.string.awake_why_question
        KeepAwake.KEEP_AWAKE_LOADING -> R.string.awake_why_loading
        KeepAwake.KEEP_AWAKE_UNSPECIFIED -> null
    }

/**
 * Why a tab is awake, in words: that it is on screen, and each of the core's reasons, in the
 * core's order. With nothing to keep it awake, that it will sleep by itself.
 */
internal fun awakeWhy(tab: AwakeTab, reasons: List<String>, onScreen: String, waiting: String): String {
    if (!tab.in_sight && tab.keeps_awake.isEmpty()) return waiting
    return (listOfNotNull(onScreen.takeIf { tab.in_sight }) + reasons).joinToString(", ")
}

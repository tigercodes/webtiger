// Where searches go (020): the list of search engines a person chooses among. It is the first
// run's question, over the window before the window is used; and the same list again when an
// empty tab's "Change…" is pressed, with the engine in use marked.
package com.tigercodes.webtiger.ui

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.LocalMinimumInteractiveComponentSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.text.font.FontWeight
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.core.v1.SearchSettings
import com.tigercodes.webtiger.design.Tokens

/**
 * `onChoose` is told the engine that was pressed, by the core's name for it. `onKeep` closes
 * the list with nothing changed, and so does the system's Back: at the first run that keeps
 * the default, which the button names.
 */
@Composable
internal fun SearchEngines(settings: SearchSettings, onChoose: (String) -> Unit, onKeep: () -> Unit, modifier: Modifier = Modifier) {
    BackHandler(onBack = onKeep)
    Column(
        modifier =
            modifier
                .fillMaxSize()
                .background(MaterialTheme.colorScheme.background)
                .verticalScroll(rememberScrollState())
                .padding(vertical = Tokens.Space.l)
                .testTag("search-engines"),
    ) {
        Column(modifier = Modifier.padding(horizontal = Tokens.Space.l), verticalArrangement = Arrangement.spacedBy(Tokens.Space.s)) {
            Text(
                text = stringResource(R.string.search_question_title),
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.SemiBold,
                modifier = Modifier.semantics { heading() },
            )
            Text(text = stringResource(R.string.search_question_detail), color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        Column(modifier = Modifier.padding(top = Tokens.Space.m)) {
            for (engine in settings.engines) {
                // The engine searches go to is marked, in words: "In use" where somebody chose it,
                // "Default" at the first run, while nobody has.
                val inUse = engine.id == settings.chosen_id
                val said = if (!inUse) "" else stringResource(if (settings.chosen) R.string.search_in_use else R.string.search_default)
                HorizontalDivider(color = Tokens.Palette.hairline.current)
                Row(
                    modifier =
                        Modifier
                            .fillMaxWidth()
                            .clickable(role = Role.Button) { onChoose(engine.id) }
                            .heightIn(min = LocalMinimumInteractiveComponentSize.current)
                            .padding(horizontal = Tokens.Space.l, vertical = Tokens.Space.s)
                            .semantics(mergeDescendants = true) { if (inUse) stateDescription = said }
                            .testTag("search-engine-${engine.id}"),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(Tokens.Space.m),
                ) {
                    Column(modifier = Modifier.weight(1f)) {
                        Text(text = engine.name, style = MaterialTheme.typography.bodyLarge)
                        // Where the words are sent.
                        Text(text = engine.host, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                    if (inUse) Text(text = said, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.testTag("search-in-use"))
                }
            }
            HorizontalDivider(color = Tokens.Palette.hairline.current)
        }
        TextButton(onClick = onKeep, modifier = Modifier.padding(horizontal = Tokens.Space.s, vertical = Tokens.Space.s).testTag("search-keep")) {
            // "Cancel" where one is in use by somebody's choice; at the first run, "Keep
            // DuckDuckGo": the default, by its name.
            val default = settings.engines.firstOrNull { it.id == settings.chosen_id }?.name.orEmpty()
            Text(if (settings.chosen) stringResource(R.string.search_keep) else stringResource(R.string.search_keep_default, default))
        }
    }
}

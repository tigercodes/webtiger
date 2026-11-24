// A Space's time (049): after how long out of sight its tabs are tidied into its Archive, or
// never. The times are the core's, in seconds; the words are the strings', by the number of
// seconds. The shell times nothing: it says which time was chosen, and the core tidies.
package com.tigercodes.webtiger.ui

import android.annotation.SuppressLint
import android.content.res.Resources
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.selection.selectable
import androidx.compose.material3.LocalMinimumInteractiveComponentSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.design.Tokens

/**
 * The words for a time ("After 12 hours"; "Never" for none). Nothing for a time this build has
 * no words for: a number of seconds is never shown in their place.
 */
@SuppressLint("DiscouragedApi") // The times are the core's; res/raw/keep.xml keeps the strings.
internal fun tidyTimeWords(resources: Resources, packageName: String, seconds: Int): String {
    if (seconds < 0) return ""
    val id = resources.getIdentifier("space_tidy_$seconds", "string", packageName)
    return if (id != 0) resources.getString(id) else ""
}

/**
 * The times to choose among, each a row to press: `times` are the core's, in its order; one
 * this build has no words for is left out. The one chosen is marked, and said to be selected.
 * With fewer than two to choose among (a core that lists none) nothing is drawn.
 */
@Composable
internal fun SpaceTidyTimes(times: List<Int>, chosen: Int, onChoose: (Int) -> Unit, modifier: Modifier = Modifier) {
    val context = LocalContext.current
    val offered = times.map { it to tidyTimeWords(context.resources, context.packageName, it) }.filter { it.second.isNotEmpty() }
    if (offered.size < 2) return
    Column(modifier = modifier) {
        Text(
            text = stringResource(R.string.space_tidy),
            style = MaterialTheme.typography.titleMedium,
            modifier = Modifier.padding(horizontal = Tokens.Space.l).semantics { heading() }.testTag("space-tidy"),
        )
        Text(
            text = stringResource(R.string.space_tidy_detail),
            style = MaterialTheme.typography.bodyMedium,
            modifier = Modifier.padding(horizontal = Tokens.Space.l, vertical = Tokens.Space.xs),
        )
        for ((time, words) in offered) {
            Row(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .selectable(selected = time == chosen, role = Role.RadioButton, onClick = { onChoose(time) })
                        .heightIn(min = LocalMinimumInteractiveComponentSize.current)
                        .padding(horizontal = Tokens.Space.l)
                        .testTag("space-tidy-$time"),
                horizontalArrangement = Arrangement.spacedBy(Tokens.Space.s),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                RadioButton(selected = time == chosen, onClick = null)
                Text(text = words, style = MaterialTheme.typography.bodyLarge)
            }
        }
    }
}

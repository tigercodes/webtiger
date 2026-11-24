// A Space's icon as the shell shows it (050): the icons to choose among where a Space is made
// or edited, and the icon where it stands beside a name or in a name's place. The icons are
// the core's, each an emoji: it is what is drawn, by the system's own font, and what a screen
// reader names. There is no paint for one and no word.
package com.tigercodes.webtiger.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.sizeIn
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.LocalMinimumInteractiveComponentSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.design.Tokens

/**
 * The icons to choose among, each to be pressed: `icons` are the core's, in its order. `chosen`
 * is the one chosen, or empty; the first choice is none. The one chosen is ringed, and said to
 * be selected. With no icons to offer (a core that lists none) nothing is drawn.
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun SpaceIcons(icons: List<String>, chosen: String, onChoose: (String) -> Unit, modifier: Modifier = Modifier) {
    if (icons.isEmpty()) return
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(Tokens.Space.xs)) {
        Text(
            text = stringResource(R.string.space_icon),
            style = MaterialTheme.typography.titleMedium,
            modifier = Modifier.semantics { heading() }.testTag("space-icon"),
        )
        FlowRow {
            IconChoice("", number = 0, isChosen = chosen.isEmpty(), onChoose = onChoose)
            icons.forEachIndexed { place, icon -> IconChoice(icon, number = place + 1, isChosen = icon == chosen, onChoose = onChoose) }
        }
    }
}

/**
 * One choice: an icon, or none. A test knows it by its place in the core's list, from one: an
 * emoji is no name for a thing to press.
 */
@Composable
private fun IconChoice(icon: String, number: Int, isChosen: Boolean, onChoose: (String) -> Unit) {
    val touch = LocalMinimumInteractiveComponentSize.current
    val none = stringResource(R.string.space_no_icon)
    val ink = MaterialTheme.colorScheme.onBackground
    Box(
        modifier =
            Modifier
                .sizeIn(minWidth = touch, minHeight = touch)
                .selectable(selected = isChosen, role = Role.RadioButton) { onChoose(icon) }
                // "No icon" is said in words; an emoji is named by the screen reader itself.
                .semantics { if (icon.isEmpty()) contentDescription = none }
                .testTag(if (icon.isEmpty()) "space-icon-none" else "space-icon-$number"),
        contentAlignment = Alignment.Center,
    ) {
        // The ring of the one that is chosen, in the colour of the words.
        Box(
            modifier =
                Modifier
                    .size(40.dp)
                    .then(if (isChosen) Modifier.border(2.dp, ink, RoundedCornerShape(Tokens.Radius.field)) else Modifier),
            contentAlignment = Alignment.Center,
        ) {
            if (icon.isEmpty()) {
                // None: a ring with a stroke through it.
                Canvas(modifier = Modifier.size(22.dp)) {
                    val line = 1.5.dp.toPx()
                    drawCircle(color = ink, radius = size.minDimension / 2 - line / 2, style = Stroke(width = line))
                    drawLine(color = ink, start = Offset(size.width * 0.18f, size.height * 0.18f), end = Offset(size.width * 0.82f, size.height * 0.82f), strokeWidth = line)
                }
            } else {
                Text(text = icon, style = MaterialTheme.typography.titleLarge)
            }
        }
    }
}

/**
 * A Space's icon where it stands beside a name or in a name's place: the emoji, drawn. What a
 * screen reader says there is the Space's name: the icon is not its name (050 D5).
 */
@Composable
internal fun SpaceIconMark(icon: String, modifier: Modifier = Modifier) {
    Text(text = icon, modifier = modifier.testTag("space-icon-mark"))
}

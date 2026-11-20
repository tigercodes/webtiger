// A Space's colour as the shell shows it (025 §6): a mark of its paint beside the Space's name,
// and the colours to choose among where a Space is made or edited. The colours are the core's,
// by its names for them; the paints are the design tokens', and the words the strings'.
package com.tigercodes.webtiger.ui

import android.annotation.SuppressLint
import android.content.res.Resources
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.sizeIn
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.LocalMinimumInteractiveComponentSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.design.Tokens

private val colourName = Regex("^[a-z]+$")

/**
 * The word for a colour ("Blue"): what a screen reader says of its swatch, and what is written
 * beside the heading when it is chosen. Nothing for a name this build has no word for: a code's
 * own text is never shown.
 */
@SuppressLint("DiscouragedApi") // The names are the core's; res/raw/keep.xml keeps the strings.
internal fun spaceColourWord(resources: Resources, packageName: String, name: String): String {
    if (!colourName.matches(name)) return ""
    val id = resources.getIdentifier("space_colour_$name", "string", packageName)
    return if (id != 0) resources.getString(id) else ""
}

/**
 * The mark of a Space's colour: a dot of its paint. A Space with none, or with a colour this
 * build has no paint for, has a ring where the dot would be, so that names stay in line.
 */
@Composable
internal fun SpaceMark(colour: String, modifier: Modifier = Modifier, size: Dp = 10.dp) {
    val paint = Tokens.SpaceColour.paint(colour)
    Box(
        modifier =
            modifier
                .size(size)
                .then(
                    if (paint != null) {
                        Modifier.clip(CircleShape).background(paint.current).testTag("space-mark-$colour")
                    } else {
                        Modifier.border(1.5.dp, Tokens.Palette.hairline.current, CircleShape)
                    },
                ),
    )
}

/**
 * The colours to choose among, each a swatch to press: `colours` are the core's, in its order;
 * one this build has no paint for is left out. `chosen` is the one chosen, or empty. With
 * `offersNone` there is a choice of no colour too.
 *
 * The one chosen is ringed, and named in words beside the heading: it is not told by its colour
 * alone.
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun SpaceSwatches(colours: List<String>, offersNone: Boolean, chosen: String, onChoose: (String) -> Unit, modifier: Modifier = Modifier) {
    val context = LocalContext.current
    val word = { name: String -> spaceColourWord(context.resources, context.packageName, name) }
    val chosenWord = word(chosen)
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(Tokens.Space.xs)) {
        Text(
            text = if (chosenWord.isEmpty()) stringResource(R.string.space_colour) else stringResource(R.string.space_chosen_colour, chosenWord),
            style = MaterialTheme.typography.titleMedium,
            modifier = Modifier.semantics { heading() }.testTag("space-colour"),
        )
        FlowRow {
            if (offersNone) Swatch("", stringResource(R.string.space_no_colour), chosen.isEmpty(), onChoose)
            for (colour in colours.filter { Tokens.SpaceColour.paint(it) != null }) {
                Swatch(colour, word(colour), colour == chosen, onChoose)
            }
        }
    }
}

@Composable
private fun Swatch(colour: String, word: String, isChosen: Boolean, onChoose: (String) -> Unit) {
    val touch = LocalMinimumInteractiveComponentSize.current
    Box(
        modifier =
            Modifier
                .sizeIn(minWidth = touch, minHeight = touch)
                .selectable(selected = isChosen, role = Role.RadioButton) { onChoose(colour) }
                .semantics { contentDescription = word }
                .testTag("space-colour-${colour.ifEmpty { "none" }}"),
        contentAlignment = Alignment.Center,
    ) {
        // The ring of the one that is chosen, in the colour of the words.
        Box(
            modifier =
                Modifier
                    .size(34.dp)
                    .then(if (isChosen) Modifier.border(2.dp, MaterialTheme.colorScheme.onBackground, CircleShape) else Modifier),
            contentAlignment = Alignment.Center,
        ) {
            SpaceMark(colour, size = 22.dp)
        }
    }
}

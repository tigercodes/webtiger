// What a page says, or asks (016 §6): the bar a tab shows for `alert`, `confirm` and `prompt`.
// The site's name, in the core's words, on a line of its own; under it the page's words, which
// are the page's and are shown as such; for a prompt, a field; and the answers.
package com.tigercodes.webtiger.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.LocalMinimumInteractiveComponentSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.core.v1.DialogKind
import com.tigercodes.webtiger.core.v1.DialogPrompt
import com.tigercodes.webtiger.design.Tokens

/**
 * `onAnswer` is told what the person chose: OK or not, the text of a prompt's field with OK,
 * and whether the page's messages are to be stopped. Whether the answer came too soon to be
 * the person's is the core's to say (014 §3): the bar stays until the core takes it away.
 *
 * No button has the keyboard, and the bar takes it from nothing by appearing; in a prompt's
 * field, which the person has to go to themselves, the keyboard's Done is OK.
 */
@Composable
internal fun DialogBar(promptId: Long, dialog: DialogPrompt, onAnswer: (Boolean, String, Boolean) -> Unit) {
    val asksForText = dialog.kind == DialogKind.DIALOG_KIND_PROMPT
    // What is in the field: the text the page offers, until the person changes it. Another
    // prompt starts from its own.
    var text by remember(promptId) { mutableStateOf(dialog.default_text) }
    Column(
        modifier =
            Modifier
                .fillMaxWidth()
                .background(Tokens.Palette.fieldFill.current)
                .padding(start = Tokens.Space.m, end = Tokens.Space.xs, top = Tokens.Space.s)
                .testTag("prompt"),
    ) {
        // Who says it, which only the core can write. Said to a screen reader when the bar
        // appears: a page that waits says nothing by itself.
        Text(
            text = stringResource(R.string.prompt_says, dialog.site),
            fontWeight = FontWeight.SemiBold,
            modifier = Modifier.fillMaxWidth().semantics { liveRegion = LiveRegionMode.Polite }.testTag("prompt-site"),
        )
        // The page's words: as many lines as they need, up to a few; beyond that they are
        // moved up and down, and the answers stay where they are.
        Box(modifier = Modifier.fillMaxWidth().padding(end = Tokens.Space.s).heightIn(max = MOST_WORDS).verticalScroll(rememberScrollState())) {
            Text(text = dialog.message, modifier = Modifier.testTag("prompt-words"))
        }
        if (asksForText) {
            val label = stringResource(R.string.prompt_answer)
            BasicTextField(
                value = text,
                onValueChange = { text = it },
                singleLine = true,
                textStyle = MaterialTheme.typography.bodyLarge.copy(color = MaterialTheme.colorScheme.onBackground),
                cursorBrush = SolidColor(MaterialTheme.colorScheme.onBackground),
                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
                keyboardActions = KeyboardActions(onDone = { onAnswer(true, text, false) }),
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .padding(top = Tokens.Space.s, end = Tokens.Space.s)
                        .semantics { contentDescription = label }
                        .testTag("prompt-field"),
                decorationBox = { field ->
                    Box(
                        modifier =
                            Modifier
                                .fillMaxWidth()
                                .heightIn(min = LocalMinimumInteractiveComponentSize.current)
                                .clip(RoundedCornerShape(Tokens.Radius.field))
                                .background(MaterialTheme.colorScheme.background)
                                .padding(horizontal = Tokens.Space.m),
                        contentAlignment = Alignment.CenterStart,
                    ) {
                        field()
                    }
                },
            )
        }
        // A page that said something before can be stopped: on a row of its own, apart from
        // the answers to what it says now.
        if (dialog.may_silence) {
            TextButton(onClick = { onAnswer(false, "", true) }, modifier = Modifier.testTag("prompt-silence")) {
                Text(stringResource(R.string.prompt_silence))
            }
        }
        Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            Spacer(modifier = Modifier.weight(1f))
            // Something said is only acknowledged: there is nothing to refuse.
            if (dialog.kind != DialogKind.DIALOG_KIND_ALERT) {
                TextButton(onClick = { onAnswer(false, "", false) }, modifier = Modifier.testTag("prompt-cancel")) { Text(stringResource(R.string.prompt_cancel)) }
            }
            TextButton(onClick = { onAnswer(true, if (asksForText) text else "", false) }, modifier = Modifier.testTag("prompt-ok")) {
                Text(stringResource(R.string.prompt_ok))
            }
        }
    }
}

/** How tall the page's words are let be before they scroll: about six lines. */
private val MOST_WORDS = 132.dp

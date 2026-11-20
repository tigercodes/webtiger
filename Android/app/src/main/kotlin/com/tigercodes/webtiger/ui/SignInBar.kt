// A server asks who is there (017 §6): the bar a tab shows for a name and a password. Every
// word in it is the app's or the core's: the server's own are not shown. What is typed is here
// only until it is sent: it is in no snapshot and no saved state.
package com.tigercodes.webtiger.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
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
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.core.v1.SignInPrompt
import com.tigercodes.webtiger.design.Tokens

/**
 * `onAnswer` is told what the person chose: to sign in or not, and with the first the name and
 * the password; Cancel passes on nothing that was typed. Whether the answer came too soon to
 * be the person's is the core's to say (014 §3): the bar stays until the core takes it away.
 *
 * No button has the keyboard, and the bar takes it from nothing by appearing. In the name's
 * field the keyboard's Next goes to the password; in the password's, its Done signs in.
 */
@Composable
internal fun SignInBar(promptId: Long, signIn: SignInPrompt, onAnswer: (Boolean, String, String) -> Unit) {
    // Not saved with the window's state: a password is not written down to be restored.
    // Another prompt starts empty.
    var name by remember(promptId) { mutableStateOf("") }
    var password by remember(promptId) { mutableStateOf("") }
    Column(
        modifier =
            Modifier
                .fillMaxWidth()
                .background(Tokens.Palette.fieldFill.current)
                .padding(start = Tokens.Space.m, end = Tokens.Space.xs, top = Tokens.Space.s)
                .testTag("prompt"),
    ) {
        // Who gets what is typed, which only the core can write. Said to a screen reader when
        // the bar appears: a load that waits says nothing by itself.
        Text(
            text = stringResource(R.string.prompt_sign_in, signIn.site),
            fontWeight = FontWeight.SemiBold,
            modifier = Modifier.fillMaxWidth().semantics { liveRegion = LiveRegionMode.Polite }.testTag("prompt-site"),
        )
        if (signIn.again) {
            Text(text = stringResource(R.string.prompt_sign_in_again), modifier = Modifier.padding(end = Tokens.Space.s).testTag("prompt-again"))
        }
        // Said unless the core says the connection is private: a shell told nothing warns.
        if (!signIn.confidential) {
            Text(text = stringResource(R.string.prompt_sign_in_not_private), modifier = Modifier.padding(end = Tokens.Space.s).testTag("prompt-not-private"))
        }
        TypedField(
            value = name,
            onChange = { name = it },
            label = stringResource(R.string.prompt_sign_in_name),
            tag = "prompt-name",
            keyboard = KeyboardOptions(capitalization = KeyboardCapitalization.None, autoCorrectEnabled = false, keyboardType = KeyboardType.Email, imeAction = ImeAction.Next),
        )
        TypedField(
            value = password,
            onChange = { password = it },
            label = stringResource(R.string.prompt_sign_in_password),
            tag = "prompt-password",
            // Shown as dots, and told to the keyboard and to assistive services as a password.
            hidden = true,
            keyboard = KeyboardOptions(autoCorrectEnabled = false, keyboardType = KeyboardType.Password, imeAction = ImeAction.Done),
            onDone = { onAnswer(true, name, password) },
        )
        Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            Spacer(modifier = Modifier.weight(1f))
            TextButton(onClick = { onAnswer(false, "", "") }, modifier = Modifier.testTag("prompt-cancel")) { Text(stringResource(R.string.prompt_cancel)) }
            TextButton(onClick = { onAnswer(true, name, password) }, modifier = Modifier.testTag("prompt-sign-in")) {
                Text(stringResource(R.string.prompt_sign_in_go))
            }
        }
    }
}

/** One of the bar's two fields: its label is shown in it while it is empty, and read as its name. */
@Composable
private fun TypedField(
    value: String,
    onChange: (String) -> Unit,
    label: String,
    tag: String,
    keyboard: KeyboardOptions,
    hidden: Boolean = false,
    onDone: (() -> Unit)? = null,
) {
    BasicTextField(
        value = value,
        onValueChange = onChange,
        singleLine = true,
        textStyle = MaterialTheme.typography.bodyLarge.copy(color = MaterialTheme.colorScheme.onBackground),
        cursorBrush = SolidColor(MaterialTheme.colorScheme.onBackground),
        visualTransformation = if (hidden) PasswordVisualTransformation() else VisualTransformation.None,
        keyboardOptions = keyboard,
        keyboardActions = KeyboardActions(onDone = onDone?.let { done -> { done() } }),
        modifier =
            Modifier
                .fillMaxWidth()
                .padding(top = Tokens.Space.s, end = Tokens.Space.s)
                .semantics { contentDescription = label }
                .testTag(tag),
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
                if (value.isEmpty()) {
                    Text(text = label, style = MaterialTheme.typography.bodyLarge, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1)
                }
                field()
            }
        },
    )
}

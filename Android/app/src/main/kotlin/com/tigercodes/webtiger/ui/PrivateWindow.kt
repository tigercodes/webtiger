// A private window on a phone (037 §5.3). The app has one window on the screen: a private
// window is a second window of the core's, shown in place of the first, which stays as it was
// behind it. Here: what a window can ask about the private one, and the page behind a private
// window's chip, which says what private means and how to leave it.
package com.tigercodes.webtiger.ui

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.design.Tokens

/**
 * What a window can ask about the private window, where the app shows one window at a time.
 *
 * `isOpen`: there is a private window, shown or behind. `show` opens one, or shows the one
 * there is. `leave` shows the other window again, and the private one stays open. `close`
 * closes the private window, with everything of it, and shows the other. `other` is the name
 * of the Space the other window shows, as it was given: empty for one nobody named.
 */
class PrivateWindows(val isOpen: Boolean, val other: String, val show: () -> Unit, val leave: () -> Unit, val close: () -> Unit)

/** The page behind a private window's chip: what private means, what it does not, and the ways on. */
@Composable
internal fun PrivatePage(private: PrivateWindows, onDone: () -> Unit, modifier: Modifier = Modifier) {
    BackHandler(onBack = onDone)
    Column(
        modifier =
            modifier
                .fillMaxSize()
                .background(MaterialTheme.colorScheme.background)
                .verticalScroll(rememberScrollState())
                .padding(Tokens.Space.l)
                .testTag("private-page"),
        verticalArrangement = Arrangement.spacedBy(Tokens.Space.m),
    ) {
        Text(
            text = stringResource(R.string.private_window),
            style = MaterialTheme.typography.headlineSmall,
            fontWeight = FontWeight.SemiBold,
            modifier = Modifier.semantics { heading() },
        )
        Text(text = stringResource(R.string.private_keeps))
        Text(text = stringResource(R.string.private_hides), color = MaterialTheme.colorScheme.onSurfaceVariant)
        Column {
            TextButton(onClick = private.close, modifier = Modifier.testTag("private-close")) { Text(stringResource(R.string.private_close)) }
            TextButton(onClick = private.leave, modifier = Modifier.testTag("private-leave")) {
                Text(stringResource(R.string.private_back, spaceName(private.other)))
            }
            TextButton(onClick = onDone, modifier = Modifier.testTag("private-done")) { Text(stringResource(R.string.spaces_done)) }
        }
    }
}

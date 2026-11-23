// The phone's switcher (047): in a window with no room for a sidebar, a button in the toolbar
// that says how many tabs the row has, and over the window the row as the sidebar has it. The
// switcher is the sidebar shown another way: it has no model of its own, and every row asks
// what it asks in the sidebar. A tab brought in front puts it away.
package com.tigercodes.webtiger.ui

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.minimumInteractiveComponentSize
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.design.Tokens

/** What the switcher's button shows: the number of tabs, while it has room for it. */
internal fun switcherCount(count: Int): String = if (count > 99) "99+" else count.toString()

/**
 * The button that opens the switcher: a square with the number of tabs the row has. The number
 * is the core's: those of shut Groups are among them. A screen reader is told what it is for.
 */
@Composable
internal fun SwitcherButton(count: Int, onClick: () -> Unit) {
    val said = stringResource(R.string.switcher_open, count)
    Box(
        contentAlignment = Alignment.Center,
        modifier =
            Modifier
                .minimumInteractiveComponentSize()
                .testTag("switcher")
                .clickable(role = Role.Button, onClick = onClick)
                // One thing to a screen reader, which says what it opens and how many there
                // are: the number drawn in it is not said a second time.
                .clearAndSetSemantics { contentDescription = said },
    ) {
        Box(
            contentAlignment = Alignment.Center,
            modifier = Modifier.size(24.dp).border(BorderStroke(1.5.dp, LocalContentColor.current), RoundedCornerShape(5.dp)),
        ) {
            Text(text = switcherCount(count), fontSize = if (count > 99) 9.sp else 12.sp, fontWeight = FontWeight.SemiBold, maxLines = 1)
        }
    }
}

/**
 * The switcher itself: the Space at its head, its title and the way out; then the row as the
 * sidebar has it, with no toolbar and none of the window's buttons.
 */
@Composable
internal fun Switcher(model: BrowserModel, onSpaces: () -> Unit, modifier: Modifier = Modifier) {
    Surface(modifier = Modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background) {
        Column(modifier = modifier.fillMaxSize().testTag("switcher-sheet")) {
            Box(modifier = Modifier.fillMaxWidth().padding(horizontal = Tokens.Space.xs, vertical = Tokens.Space.xs)) {
                Row(modifier = Modifier.align(Alignment.CenterStart), horizontalArrangement = Arrangement.spacedBy(Tokens.Space.xs)) {
                    // The Space the row is of (024); a private window says what it is (037).
                    if (model.isPrivate) {
                        SpaceChip(name = stringResource(R.string.private_chip), compact = false, onClick = onSpaces, saidAs = stringResource(R.string.private_window))
                    } else {
                        SpaceChip(name = model.spaceName, compact = false, onClick = onSpaces, colour = model.spaceColour)
                    }
                }
                Text(
                    text = stringResource(R.string.switcher_title),
                    style = MaterialTheme.typography.titleMedium,
                    modifier = Modifier.align(Alignment.Center).semantics { heading() },
                )
                TextButton(onClick = model::closeSwitcher, modifier = Modifier.align(Alignment.CenterEnd).testTag("switcher-done")) {
                    Text(stringResource(R.string.switcher_done))
                }
            }
            SidebarRows(model, Modifier.weight(1f).fillMaxWidth(), onChosen = model::closeSwitcher)
        }
    }
}

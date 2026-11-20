// Several Spaces (024 §6.2): the chip that names the Space a window shows, the list of Spaces it
// opens, and the page that makes one. The core decides everything here: the list is its answer,
// a name is tidied by it, and what it refuses is said in its words.
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
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.LocalMinimumInteractiveComponentSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.core.v1.SpaceSummary
import com.tigercodes.webtiger.design.Tokens
import java.text.BreakIterator

/** A Space's name as it is shown: its own, or the shell's word for the Space the app made while nobody has named it. */
@Composable
internal fun spaceName(name: String): String = name.ifEmpty { stringResource(R.string.space_home) }

/**
 * The first letter of a name as it is shown, as a capital: what stands for the Space where
 * there is no room for its name. A name that begins with a picture has the picture.
 */
internal fun spaceInitial(shown: String): String {
    if (shown.isEmpty()) return ""
    val letters = BreakIterator.getCharacterInstance()
    letters.setText(shown)
    return shown.substring(0, letters.next()).uppercase()
}

/** A window narrower than this is a phone's (000 §6.5: by the window's size, never by the device). */
internal const val COMPACT_WIDTH_DP = 600

/**
 * The chip that names the Space the window shows, and opens the list of Spaces.
 *
 * In a window as narrow as a phone's (`compact`) the strip has no room for a name beside the
 * tabs: with one, a second tab is pushed out of reach. There the chip is the Space's initial,
 * and its name is what a screen reader says and what the list shows.
 */
@Composable
internal fun SpaceChip(name: String, compact: Boolean, onClick: () -> Unit, modifier: Modifier = Modifier) {
    val shown = spaceName(name)
    val said = stringResource(R.string.space_chip, shown)
    val touch = LocalMinimumInteractiveComponentSize.current
    Row(
        modifier =
            modifier
                .clip(RoundedCornerShape(Tokens.Radius.field))
                .background(Tokens.Palette.fieldFill.current)
                .clickable(role = Role.Button, onClick = onClick)
                .heightIn(min = touch)
                .then(if (compact) Modifier.widthIn(min = touch) else Modifier.padding(horizontal = Tokens.Space.m))
                .semantics(mergeDescendants = true) { contentDescription = said }
                .testTag("space-chip"),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(Tokens.Space.xs, Alignment.CenterHorizontally),
    ) {
        if (compact) {
            Text(text = spaceInitial(shown), fontWeight = FontWeight.SemiBold)
        } else {
            // A long name is cut: the chip leaves the tabs their room.
            Text(text = shown, maxLines = 1, overflow = TextOverflow.Ellipsis, fontWeight = FontWeight.SemiBold, modifier = Modifier.widthIn(max = 120.dp))
            Icon(painter = painterResource(R.drawable.ic_down), contentDescription = null, modifier = Modifier.size(18.dp))
        }
    }
}

/**
 * The list of Spaces, and the page that makes one, over the window. `spaces` are the Spaces as
 * the core listed them, null until it has; `shownId` is the one the window shows; `refusal` is
 * why the core would not make one, by its code.
 *
 * `making`, `name` and `sharingWith` open it on the page that makes a Space, with that name
 * typed and that Space's logins chosen: for a picture of it (004).
 */
@Composable
internal fun Spaces(
    spaces: List<SpaceSummary>?,
    shownId: Long,
    refusal: String?,
    onShow: (Long) -> Unit,
    onMake: (String, Long?) -> Unit,
    onDone: () -> Unit,
    modifier: Modifier = Modifier,
    making: Boolean = false,
    name: String = "",
    sharingWith: Long? = null,
) {
    var isMaking by rememberSaveable { mutableStateOf(making) }
    // The system's Back leaves the page for the list, and the list for the window.
    BackHandler { if (isMaking) isMaking = false else onDone() }
    Column(
        modifier =
            modifier
                .fillMaxSize()
                .background(MaterialTheme.colorScheme.background)
                .verticalScroll(rememberScrollState())
                .padding(vertical = Tokens.Space.l)
                .testTag("spaces"),
    ) {
        if (isMaking) {
            // Whose logins a new Space would share, should the person choose to: those of the
            // Space the window shows.
            val all = spaces.orEmpty()
            NewSpace(
                owners = owners(all),
                startsSharingWith = sharingWith,
                wouldShareWith = owner(all, shownId),
                startsNamed = name,
                // A picture of the page takes no keyboard.
                takesKeyboard = !making,
                refusal = refusal,
                onMake = onMake,
                onBack = { isMaking = false },
            )
        } else {
            SpaceList(spaces.orEmpty(), shownId, onShow = onShow, onNew = { isMaking = true }, onDone = onDone)
        }
    }
}

/** The Spaces a new one can share logins with: the first of each that has logins of its own. (Sharing with one that shares already is sharing with whoever it shares with.) */
private fun owners(spaces: List<SpaceSummary>): List<SpaceSummary> = spaces.distinctBy { it.logins }

/** The first Space with the logins of the Space `id`: the one a new Space that shares them is said to share with. */
private fun owner(spaces: List<SpaceSummary>, id: Long): Long {
    val logins = spaces.firstOrNull { it.space_id == id }?.logins
    val all = owners(spaces)
    return (all.firstOrNull { it.logins == logins } ?: all.firstOrNull())?.space_id ?: 0
}

@Composable
private fun SpaceList(spaces: List<SpaceSummary>, shownId: Long, onShow: (Long) -> Unit, onNew: () -> Unit, onDone: () -> Unit) {
    Column(modifier = Modifier.padding(horizontal = Tokens.Space.l), verticalArrangement = Arrangement.spacedBy(Tokens.Space.s)) {
        Text(
            text = stringResource(R.string.spaces_title),
            style = MaterialTheme.typography.headlineSmall,
            fontWeight = FontWeight.SemiBold,
            modifier = Modifier.semantics { heading() },
        )
        Text(text = stringResource(R.string.spaces_detail), color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
    Column(modifier = Modifier.padding(top = Tokens.Space.m)) {
        for (space in spaces) {
            val isShown = space.space_id == shownId
            // Said in words, and to a screen reader: not by a colour alone.
            val said = if (isShown) stringResource(R.string.spaces_shown) else ""
            // Another Space with the same logins: the first there is.
            val other = spaces.firstOrNull { it.logins == space.logins && it.space_id != space.space_id }
            HorizontalDivider(color = Tokens.Palette.hairline.current)
            Row(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .clickable(role = Role.Button) { onShow(space.space_id) }
                        .heightIn(min = LocalMinimumInteractiveComponentSize.current)
                        .padding(horizontal = Tokens.Space.l, vertical = Tokens.Space.s)
                        .semantics(mergeDescendants = true) { if (isShown) stateDescription = said }
                        .testTag("space-${space.space_id}"),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(Tokens.Space.m),
            ) {
                Column(modifier = Modifier.weight(1f)) {
                    Text(text = spaceName(space.name), style = MaterialTheme.typography.bodyLarge)
                    if (other != null) {
                        Text(
                            text = stringResource(R.string.spaces_shares, spaceName(other.name)),
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
                if (isShown) Text(text = said, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.testTag("space-shown"))
            }
        }
        HorizontalDivider(color = Tokens.Palette.hairline.current)
    }
    Row(modifier = Modifier.padding(horizontal = Tokens.Space.s, vertical = Tokens.Space.s), horizontalArrangement = Arrangement.spacedBy(Tokens.Space.s)) {
        TextButton(onClick = onNew, modifier = Modifier.testTag("spaces-new")) { Text(stringResource(R.string.spaces_new)) }
        TextButton(onClick = onDone, modifier = Modifier.testTag("spaces-done")) { Text(stringResource(R.string.spaces_done)) }
    }
}

/** The page that makes a Space: its name, and whose logins it has, each choice with what it means in a sentence. */
@Composable
private fun NewSpace(
    owners: List<SpaceSummary>,
    startsSharingWith: Long?,
    wouldShareWith: Long,
    startsNamed: String,
    takesKeyboard: Boolean,
    refusal: String?,
    onMake: (String, Long?) -> Unit,
    onBack: () -> Unit,
) {
    // The name is what is asked first: the keyboard is in its field when the page opens.
    val nameField = remember { FocusRequester() }
    LaunchedEffect(Unit) { if (takesKeyboard) nameField.requestFocus() }
    var name by rememberSaveable { mutableStateOf(startsNamed) }
    var shares by rememberSaveable { mutableStateOf(startsSharingWith != null) }
    var shareWith by rememberSaveable { mutableStateOf(startsSharingWith ?: wouldShareWith) }
    // The Space whose logins are shared: the one chosen, while it is one that can be.
    val sharedWith = owners.firstOrNull { it.space_id == shareWith } ?: owners.firstOrNull()
    val submit = { if (name.isNotBlank()) onMake(name, sharedWith?.space_id.takeIf { shares }) }
    Column(modifier = Modifier.padding(horizontal = Tokens.Space.l), verticalArrangement = Arrangement.spacedBy(Tokens.Space.m)) {
        Text(
            text = stringResource(R.string.space_new_title),
            style = MaterialTheme.typography.headlineSmall,
            fontWeight = FontWeight.SemiBold,
            modifier = Modifier.semantics { heading() },
        )
        OutlinedTextField(
            value = name,
            onValueChange = { name = it },
            label = { Text(stringResource(R.string.space_new_name)) },
            singleLine = true,
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
            keyboardActions = KeyboardActions(onDone = { submit() }),
            modifier = Modifier.fillMaxWidth().focusRequester(nameField).testTag("space-name"),
        )
        Text(text = stringResource(R.string.space_new_logins), style = MaterialTheme.typography.titleMedium, modifier = Modifier.semantics { heading() })
    }
    Choice(R.string.space_new_own, stringResource(R.string.space_new_own_detail), chosen = !shares, tag = "space-logins-own") { shares = false }
    Choice(
        R.string.space_new_shared,
        stringResource(R.string.space_new_shared_detail, spaceName(sharedWith?.name.orEmpty())),
        chosen = shares,
        tag = "space-logins-shared",
    ) { shares = true }
    // Which Space, where there is more than one to choose among.
    if (shares && owners.size > 1) {
        Column(modifier = Modifier.padding(start = Tokens.Space.xl).testTag("space-share-with")) {
            Text(
                text = stringResource(R.string.space_new_share_with),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(horizontal = Tokens.Space.l),
            )
            for (owner in owners) {
                val chosen = owner.space_id == sharedWith?.space_id
                Row(
                    modifier =
                        Modifier
                            .fillMaxWidth()
                            .selectable(selected = chosen, role = Role.RadioButton) { shareWith = owner.space_id }
                            .heightIn(min = LocalMinimumInteractiveComponentSize.current)
                            .padding(horizontal = Tokens.Space.l)
                            .testTag("space-share-with-${owner.space_id}"),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(Tokens.Space.s),
                ) {
                    RadioButton(selected = chosen, onClick = null)
                    Text(text = spaceName(owner.name), maxLines = 1, overflow = TextOverflow.Ellipsis)
                }
            }
        }
    }
    if (refusal != null) {
        val context = LocalContext.current
        Text(
            text = spaceRefusalWords(context.resources, context.packageName, refusal),
            modifier = Modifier.padding(horizontal = Tokens.Space.l, vertical = Tokens.Space.s).testTag("space-refusal"),
        )
    }
    Row(modifier = Modifier.padding(horizontal = Tokens.Space.l, vertical = Tokens.Space.s), horizontalArrangement = Arrangement.spacedBy(Tokens.Space.s)) {
        TextButton(onClick = onBack, modifier = Modifier.testTag("space-back")) { Text(stringResource(R.string.space_new_back)) }
        // Nothing to make of no name. (The core says so too: this only spares the press.)
        Button(onClick = submit, enabled = name.isNotBlank(), modifier = Modifier.testTag("space-make")) { Text(stringResource(R.string.space_new_make)) }
    }
}

/** One of the two choices of logins: its name, what it means in a sentence, and whether it is the one chosen. */
@Composable
private fun Choice(title: Int, detail: String, chosen: Boolean, tag: String, onChoose: () -> Unit) {
    Row(
        modifier =
            Modifier
                .fillMaxWidth()
                .selectable(selected = chosen, role = Role.RadioButton, onClick = onChoose)
                .heightIn(min = LocalMinimumInteractiveComponentSize.current)
                .padding(horizontal = Tokens.Space.l, vertical = Tokens.Space.s)
                .testTag(tag),
        horizontalArrangement = Arrangement.spacedBy(Tokens.Space.s),
    ) {
        RadioButton(selected = chosen, onClick = null)
        Column {
            Text(text = stringResource(title), style = MaterialTheme.typography.bodyLarge)
            Text(text = detail, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}

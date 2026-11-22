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
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.sizeIn
import androidx.compose.foundation.shape.CircleShape
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
 *
 * A Space with a colour (025) has its mark here: a dot before the name, or, where the chip is
 * only the initial, a bar under it, which takes none of the strip's room.
 */
@Composable
internal fun SpaceChip(name: String, compact: Boolean, onClick: () -> Unit, modifier: Modifier = Modifier, colour: String = "", saidAs: String? = null) {
    val shown = spaceName(name)
    // What a screen reader says: the Space, by name; or what the chip was given to say (a
    // private window's is no Space, 037).
    val said = saidAs ?: stringResource(R.string.space_chip, shown)
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
        val paint = Tokens.SpaceColour.paint(colour)
        if (compact) {
            Box(modifier = Modifier.sizeIn(minWidth = touch, minHeight = touch), contentAlignment = Alignment.Center) {
                Text(text = spaceInitial(shown), fontWeight = FontWeight.SemiBold)
                if (paint != null) {
                    Box(
                        modifier =
                            Modifier
                                .align(Alignment.BottomCenter)
                                .padding(bottom = 8.dp)
                                .size(width = 18.dp, height = 3.dp)
                                .clip(CircleShape)
                                .background(paint.current)
                                .testTag("space-mark-$colour"),
                    )
                }
            }
        } else {
            if (paint != null) SpaceMark(colour)
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
 * `colours` are the colours a Space can have, as the core lists them (025). `onMake` is told
 * a new Space's name, the Space whose logins it shares if any, and its colour, or none for the
 * core to choose. `onEdit` is told the name typed for the Space the window shows, when it is
 * not the one it has, and its colour, when another was chosen: what is null stays as it is.
 *
 * `making`, `name` and `sharingWith` open it on the page that makes a Space, with that name
 * typed and that Space's logins chosen; `renaming` on the page that names the Space shown:
 * for a picture of either (004).
 */
@Composable
internal fun Spaces(
    spaces: List<SpaceSummary>?,
    shownId: Long,
    refusal: String?,
    onShow: (Long) -> Unit,
    onMake: (String, Long?, String) -> Unit,
    onEdit: (String?, String?) -> Unit,
    onDone: () -> Unit,
    modifier: Modifier = Modifier,
    colours: List<String> = emptyList(),
    /** The way to the private window (037), on a phone: the list's last row. None: no row. */
    private: PrivateWindows? = null,
    making: Boolean = false,
    name: String = "",
    sharingWith: Long? = null,
    renaming: Boolean = false,
) {
    val pictured = making || renaming
    var page by rememberSaveable { mutableStateOf(if (making) SpacesPage.NEW else if (renaming) SpacesPage.RENAME else SpacesPage.LIST) }
    // The system's Back leaves a page for the list, and the list for the window.
    BackHandler { if (page != SpacesPage.LIST) page = SpacesPage.LIST else onDone() }
    Column(
        modifier =
            modifier
                .fillMaxSize()
                .background(MaterialTheme.colorScheme.background)
                .verticalScroll(rememberScrollState())
                .padding(vertical = Tokens.Space.l)
                .testTag("spaces"),
    ) {
        val all = spaces.orEmpty()
        when (page) {
            SpacesPage.NEW ->
                // Whose logins a new Space would share, should the person choose to: those of
                // the Space the window shows.
                NewSpace(
                    owners = owners(all),
                    startsSharingWith = sharingWith,
                    wouldShareWith = owner(all, shownId),
                    startsNamed = name,
                    colours = colours,
                    // A picture of the page takes no keyboard.
                    takesKeyboard = !pictured,
                    refusal = refusal,
                    onMake = onMake,
                    onBack = { page = SpacesPage.LIST },
                )
            SpacesPage.RENAME ->
                // The name the Space has: none, for the one the app made while nobody has named
                // it. The shell's word for that one is not a name it was given.
                SpaceRename(
                    named = all.firstOrNull { it.space_id == shownId }?.name.orEmpty(),
                    coloured = all.firstOrNull { it.space_id == shownId }?.colour.orEmpty(),
                    colours = colours,
                    takesKeyboard = !pictured,
                    refusal = refusal,
                    onSave = onEdit,
                    onBack = { page = SpacesPage.LIST },
                )
            SpacesPage.LIST ->
                SpaceList(all, shownId, onShow = onShow, onNew = { page = SpacesPage.NEW }, onRename = { page = SpacesPage.RENAME }, onDone = onDone, private = private)
        }
    }
}

/** What the list of Spaces shows: the list itself, the page that makes a Space, or the page that names one. */
private enum class SpacesPage { LIST, NEW, RENAME }

/** The Spaces a new one can share logins with: the first of each that has logins of its own. (Sharing with one that shares already is sharing with whoever it shares with.) */
private fun owners(spaces: List<SpaceSummary>): List<SpaceSummary> = spaces.distinctBy { it.logins }

/** The first Space with the logins of the Space `id`: the one a new Space that shares them is said to share with. */
private fun owner(spaces: List<SpaceSummary>, id: Long): Long {
    val logins = spaces.firstOrNull { it.space_id == id }?.logins
    val all = owners(spaces)
    return (all.firstOrNull { it.logins == logins } ?: all.firstOrNull())?.space_id ?: 0
}

@Composable
private fun SpaceList(
    spaces: List<SpaceSummary>,
    shownId: Long,
    onShow: (Long) -> Unit,
    onNew: () -> Unit,
    onRename: () -> Unit,
    onDone: () -> Unit,
    private: PrivateWindows? = null,
) {
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
                // Its colour (025): a mark, and a ring for a Space with none.
                SpaceMark(space.colour)
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
        // The private window (037), which is no Space: the last row. It opens one, or shows
        // the one that is open, and says so in words.
        if (private != null) {
            val open = if (private.isOpen) stringResource(R.string.private_row_open) else ""
            Row(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .clickable(role = Role.Button, onClick = private.show)
                        .heightIn(min = LocalMinimumInteractiveComponentSize.current)
                        .padding(horizontal = Tokens.Space.l, vertical = Tokens.Space.s)
                        .semantics(mergeDescendants = true) { if (private.isOpen) stateDescription = open }
                        .testTag("space-private"),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(Tokens.Space.m),
            ) {
                // No mark: it is no Space. The room of one, so that its name is under theirs.
                Spacer(modifier = Modifier.size(10.dp))
                Column(modifier = Modifier.weight(1f)) {
                    Text(text = stringResource(R.string.private_window), style = MaterialTheme.typography.bodyLarge)
                    Text(
                        text = stringResource(R.string.private_row_detail),
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                if (private.isOpen) Text(text = open, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.testTag("space-private-open"))
            }
            HorizontalDivider(color = Tokens.Palette.hairline.current)
        }
    }
    Row(modifier = Modifier.padding(horizontal = Tokens.Space.s, vertical = Tokens.Space.s), horizontalArrangement = Arrangement.spacedBy(Tokens.Space.s)) {
        TextButton(onClick = onNew, modifier = Modifier.testTag("spaces-new")) { Text(stringResource(R.string.spaces_new)) }
        // Names the Space the window shows: there is none to name until the core has listed it.
        TextButton(onClick = onRename, enabled = spaces.any { it.space_id == shownId }, modifier = Modifier.testTag("spaces-rename")) {
            Text(stringResource(R.string.spaces_rename))
        }
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
    colours: List<String>,
    takesKeyboard: Boolean,
    refusal: String?,
    onMake: (String, Long?, String) -> Unit,
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
    // Its colour, by the core's name for it; none, for the core to choose.
    var colour by rememberSaveable { mutableStateOf("") }
    val submit = { if (name.isNotBlank()) onMake(name, sharedWith?.space_id.takeIf { shares }, colour) }
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
    Column(modifier = Modifier.padding(horizontal = Tokens.Space.l, vertical = Tokens.Space.s)) {
        SpaceSwatches(colours, offersNone = false, chosen = colour, onChoose = { colour = it })
        Text(
            text = stringResource(R.string.space_automatic_colour),
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
    Refusal(refusal, R.string.error_space_failed)
    Row(modifier = Modifier.padding(horizontal = Tokens.Space.l, vertical = Tokens.Space.s), horizontalArrangement = Arrangement.spacedBy(Tokens.Space.s)) {
        TextButton(onClick = onBack, modifier = Modifier.testTag("space-back")) { Text(stringResource(R.string.space_new_back)) }
        // Nothing to make of no name. (The core says so too: this only spares the press.)
        Button(onClick = submit, enabled = name.isNotBlank(), modifier = Modifier.testTag("space-make")) { Text(stringResource(R.string.space_new_make)) }
    }
}

/** Why the core would not do what a page asked, in words: above the page's buttons. */
@Composable
private fun Refusal(code: String?, unknown: Int) {
    if (code == null) return
    val context = LocalContext.current
    Text(
        text = spaceRefusalWords(context.resources, context.packageName, code, unknown),
        modifier = Modifier.padding(horizontal = Tokens.Space.l, vertical = Tokens.Space.s).testTag("space-refusal"),
    )
}

/**
 * The page that edits the Space the window shows (024 §6.3, 025): a field that holds the name
 * it has, and its colour among the colours there are. What is saved is what changed: the name,
 * when one is typed that is not the one it has; the colour, when it is another. A Space nobody
 * named may keep no name; one that has a name is not given none.
 */
@Composable
private fun SpaceRename(
    named: String,
    coloured: String,
    colours: List<String>,
    takesKeyboard: Boolean,
    refusal: String?,
    onSave: (String?, String?) -> Unit,
    onBack: () -> Unit,
) {
    val nameField = remember { FocusRequester() }
    LaunchedEffect(Unit) { if (takesKeyboard) nameField.requestFocus() }
    var name by rememberSaveable { mutableStateOf(named) }
    var colour by rememberSaveable { mutableStateOf(coloured) }
    val mayBeSaved = name.isNotBlank() || named.isEmpty()
    val submit = {
        if (mayBeSaved) onSave(name.takeIf { it.isNotBlank() && it != named }, colour.takeIf { it != coloured })
    }
    Column(modifier = Modifier.padding(horizontal = Tokens.Space.l), verticalArrangement = Arrangement.spacedBy(Tokens.Space.m)) {
        Text(
            text = stringResource(R.string.space_rename_title),
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
    }
    SpaceSwatches(colours, offersNone = true, chosen = colour, onChoose = { colour = it }, modifier = Modifier.padding(horizontal = Tokens.Space.l, vertical = Tokens.Space.s))
    Refusal(refusal, R.string.error_space_rename_failed)
    Row(modifier = Modifier.padding(horizontal = Tokens.Space.l, vertical = Tokens.Space.s), horizontalArrangement = Arrangement.spacedBy(Tokens.Space.s)) {
        TextButton(onClick = onBack, modifier = Modifier.testTag("space-back")) { Text(stringResource(R.string.space_new_back)) }
        // A Space that has a name is not given none. (The core says so too: this only spares the press.)
        Button(onClick = submit, enabled = mayBeSaved, modifier = Modifier.testTag("space-save")) { Text(stringResource(R.string.space_rename_save)) }
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

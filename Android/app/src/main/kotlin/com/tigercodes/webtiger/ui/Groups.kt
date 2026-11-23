// Groups (041) as the shell shows them: a Group's chip in the strip, where its Sites follow it,
// the choices of "Move to Group" in a tab's menu, and the page that makes or edits one. Which
// Groups there are, what is in them and what a shut one hides are the core's; the colours are a
// Space's (025).
package com.tigercodes.webtiger.ui

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.LocalMinimumInteractiveComponentSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
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
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.core.v1.GroupSummary
import com.tigercodes.webtiger.core.v1.TabSummary
import com.tigercodes.webtiger.design.Tokens

/** What the Group page was opened for: a new Group for a tab to be put in, or a Group there is. */
sealed interface GroupAsk {
    data class New(val tab: Long) : GroupAsk

    data class Edit(val group: GroupSummary) : GroupAsk
}

/**
 * What the menu of a tab that can be kept needs beyond keeping it: for "Move to Group" (041),
 * the Groups of the row, and what choosing one, or a new one, asks; for the Favourites (042),
 * whether the command has something to do now, and what it asks.
 */
internal class Grouping(
    val groups: List<GroupSummary>,
    val onMove: (Long) -> Unit,
    val onNew: () -> Unit,
    val canFavourite: Boolean = false,
    val onFavourite: () -> Unit = {},
)

/** What the strip draws, in order: a tab, or a Group's chip. */
internal sealed interface StripItem {
    data class Tab(val tab: TabSummary) : StripItem

    data class Group(val group: GroupSummary) : StripItem
}

/**
 * The row as the core's two lists say it (041 §5): the tabs, in the core's order and never in
 * another, with each Group's chip before the first tab that says it is in that Group. A Group
 * with no tab to show (shut, or empty) stands after the Group before it; and the first of the
 * Groups stands after the Sites that are in no Group, which is before the first tab that is no
 * Site. A tab that names a Group the list does not have is drawn as one in none.
 */
internal fun stripRow(tabs: List<TabSummary>, groups: List<GroupSummary>): List<StripItem> =
    buildList {
        // How many of the Groups have their chip in the row.
        var drawn = 0

        fun drawThrough(last: Int) {
            while (drawn <= last && drawn < groups.size) add(StripItem.Group(groups[drawn++]))
        }
        for (tab in tabs) {
            val place = groups.indexOfFirst { it.group_id == tab.group_id }
            // A Group's first tab has the Group's chip before it, and before that the chips of
            // the Groups before it that have none yet. The first tab that is no Site has every
            // chip there is before it.
            if (place >= 0) drawThrough(place) else if (!tab.kept) drawThrough(groups.lastIndex)
            add(StripItem.Tab(tab))
        }
        drawThrough(groups.lastIndex)
    }

/**
 * A Group's chip in the strip: its mark, in its colour when it has one, and its name; shut, how
 * many Sites it holds. A press shuts or opens it, and a long press has its own menu: edit it, or
 * remove it.
 */
@OptIn(ExperimentalFoundationApi::class)
@Composable
internal fun GroupChip(group: GroupSummary, onToggle: () -> Unit, onEdit: () -> Unit, onRemove: () -> Unit, undo: TreeUndo? = null) {
    var offering by remember { mutableStateOf(false) }
    // What is said of a Group beside its name: that it is one, and whether it is open or shut;
    // shut, how many Sites it holds.
    val said = if (group.shut) stringResource(R.string.group_said_shut, group.sites) else stringResource(R.string.group_said_open)
    val edit = stringResource(R.string.group_edit)
    val shape = RoundedCornerShape(Tokens.Radius.field)
    Box {
        Row(
            modifier =
                Modifier
                    .heightIn(min = LocalMinimumInteractiveComponentSize.current)
                    .clip(shape)
                    // A line round it: a Group is not a tab, and none of them is ever "in front".
                    .border(1.dp, Tokens.Palette.hairline.current, shape)
                    .combinedClickable(role = Role.Button, onClick = onToggle, onLongClick = { offering = true }, onLongClickLabel = edit)
                    .semantics { stateDescription = said }
                    .padding(horizontal = Tokens.Space.m)
                    .testTag("group"),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(Tokens.Space.xs),
        ) {
            // An outline while it is open, filled when it is shut: what is in it is put away.
            val (mark, markTag) = if (group.shut) R.drawable.ic_group_shut to "group-mark-shut" else R.drawable.ic_group to "group-mark"
            Icon(
                painter = painterResource(mark),
                contentDescription = null,
                tint = Tokens.SpaceColour.paint(group.colour)?.current ?: LocalContentColor.current,
                modifier = Modifier.size(16.dp).testTag(markTag),
            )
            Text(
                // Its name; shut, with how many Sites it holds.
                text = if (group.shut) stringResource(R.string.group_shut_label, group.name, group.sites) else group.name,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                fontWeight = FontWeight.Medium,
                modifier = Modifier.widthIn(max = 180.dp),
            )
        }
        DropdownMenu(expanded = offering, onDismissRequest = { offering = false }) {
            // What was last done to the row, undone (044): at the top, while there is something.
            TreeUndoItem(undo, onChosen = { offering = false })
            DropdownMenuItem(
                text = { Text(edit) },
                onClick = {
                    offering = false
                    onEdit()
                },
                modifier = Modifier.testTag("group-edit"),
            )
            DropdownMenuItem(
                text = { Text(stringResource(R.string.group_remove)) },
                onClick = {
                    offering = false
                    onRemove()
                },
                modifier = Modifier.testTag("group-remove"),
            )
        }
    }
}

/**
 * "Move to Group" in a tab's menu: a heading; the Groups of the row by name and "No Group", one
 * of which is the tab's and is marked so; and "New Group…". `current` is the Group the tab is
 * in, 0 for none; `onChosen` closes the menu.
 */
@Composable
internal fun MoveToGroupItems(groups: List<GroupSummary>, current: Long, onMove: (Long) -> Unit, onNew: () -> Unit, onChosen: () -> Unit) {
    HorizontalDivider()
    Text(
        text = stringResource(R.string.groups_move),
        style = MaterialTheme.typography.labelLarge,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        modifier = Modifier.padding(horizontal = 12.dp, vertical = Tokens.Space.s).semantics { heading() },
    )
    if (groups.isNotEmpty()) {
        for (group in groups) {
            MoveToGroupItem(group.name, isCurrent = group.group_id == current, tag = "move-to-group") {
                onChosen()
                onMove(group.group_id)
            }
        }
        MoveToGroupItem(stringResource(R.string.groups_none), isCurrent = current == 0L, tag = "group-none") {
            onChosen()
            onMove(0L)
        }
    }
    DropdownMenuItem(
        text = { Text(stringResource(R.string.groups_new)) },
        onClick = {
            onChosen()
            onNew()
        },
        modifier = Modifier.testTag("group-new"),
    )
}

/** One choice of "Move to Group": the one that is the tab's is marked, and said to be the one chosen. */
@Composable
private fun MoveToGroupItem(name: String, isCurrent: Boolean, tag: String, onClick: () -> Unit) {
    DropdownMenuItem(
        text = { Text(name, maxLines = 1, overflow = TextOverflow.Ellipsis) },
        trailingIcon = if (isCurrent) ({ Icon(painter = painterResource(R.drawable.ic_check), contentDescription = null, modifier = Modifier.size(18.dp)) }) else null,
        onClick = onClick,
        modifier = Modifier.semantics { selected = isCurrent }.testTag(tag),
    )
}

/**
 * The page that makes a Group, or edits one: its name, its colour or none, and two buttons.
 * Over the window, as the list of Spaces is. `startsNamed` opens it with that name typed and
 * takes no keyboard: for a picture of it (004).
 */
@Composable
internal fun GroupPage(
    asked: GroupAsk,
    colours: List<String>,
    refusal: String?,
    onDone: (String, String) -> Unit,
    onCancel: () -> Unit,
    modifier: Modifier = Modifier,
    startsNamed: String? = null,
) {
    val group = (asked as? GroupAsk.Edit)?.group
    // The name is what is asked first: the keyboard is in its field when the page opens.
    val nameField = remember { FocusRequester() }
    LaunchedEffect(Unit) { if (startsNamed == null) nameField.requestFocus() }
    var name by rememberSaveable { mutableStateOf(startsNamed ?: group?.name.orEmpty()) }
    var colour by rememberSaveable { mutableStateOf(group?.colour.orEmpty()) }
    val submit = { if (name.isNotBlank()) onDone(name, colour) }
    // The system's Back leaves the page for the window, with nothing made or changed.
    BackHandler(onBack = onCancel)
    Column(
        modifier =
            modifier
                .fillMaxSize()
                .background(MaterialTheme.colorScheme.background)
                .verticalScroll(rememberScrollState())
                .padding(vertical = Tokens.Space.l)
                .testTag("group-page"),
    ) {
        Column(modifier = Modifier.padding(horizontal = Tokens.Space.l), verticalArrangement = Arrangement.spacedBy(Tokens.Space.m)) {
            Text(
                text = stringResource(if (group == null) R.string.group_new_title else R.string.group_edit_title),
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.SemiBold,
                modifier = Modifier.semantics { heading() }.testTag("group-title"),
            )
            OutlinedTextField(
                value = name,
                onValueChange = { name = it },
                label = { Text(stringResource(R.string.group_name)) },
                singleLine = true,
                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
                keyboardActions = KeyboardActions(onDone = { submit() }),
                modifier = Modifier.fillMaxWidth().focusRequester(nameField).testTag("group-name"),
            )
        }
        SpaceSwatches(colours, offersNone = true, chosen = colour, onChoose = { colour = it }, modifier = Modifier.padding(horizontal = Tokens.Space.l, vertical = Tokens.Space.s))
        if (refusal != null) {
            // Why the core would not, in words: the code's own, or what is said when no more is known.
            val context = LocalContext.current
            Text(
                text = spaceRefusalWords(context.resources, context.packageName, refusal, if (group == null) R.string.error_group_failed else R.string.error_group_change_failed),
                modifier = Modifier.padding(horizontal = Tokens.Space.l, vertical = Tokens.Space.s).testTag("group-refusal"),
            )
        }
        Row(modifier = Modifier.padding(horizontal = Tokens.Space.l, vertical = Tokens.Space.s), horizontalArrangement = Arrangement.spacedBy(Tokens.Space.s)) {
            TextButton(onClick = onCancel, modifier = Modifier.testTag("group-cancel")) { Text(stringResource(R.string.group_cancel)) }
            // A Group has a name. (The core says so too: this only spares the press.)
            Button(onClick = submit, enabled = name.isNotBlank(), modifier = Modifier.testTag("group-done")) {
                Text(stringResource(if (group == null) R.string.group_make else R.string.group_save))
            }
        }
    }
}

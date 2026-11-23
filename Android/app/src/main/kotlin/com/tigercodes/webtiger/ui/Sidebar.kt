// The sidebar (043): the window's tabs down its side. The same row the strip draws along the
// top, in the core's order: the Favourites as tiles, then the tree of Sites and Groups, then
// under a line the tabs that are no Sites. Nothing here is decided: every row asks what its chip
// in the strip asks.
package com.tigercodes.webtiger.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.HorizontalDivider
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.core.v1.GroupSummary
import com.tigercodes.webtiger.core.v1.TabSummary
import com.tigercodes.webtiger.design.Tokens

/** How wide the sidebar is: as wide as a name needs. */
private val SIDEBAR_WIDTH = 280.dp

/** What the sidebar draws, in its three parts. */
internal class SidebarParts(
    /** The Favourites (042): a row of tiles. */
    val favourites: List<TabSummary>,
    /** The Sites that are in no Group, and each Group with its Sites after it (041). */
    val tree: List<StripItem>,
    /** The tabs that are no Sites. */
    val others: List<TabSummary>,
)

/**
 * The row in the core's order ([stripRow]), taken apart by what each thing is: nothing is put
 * in another order within a part.
 */
internal fun sidebarParts(tabs: List<TabSummary>, groups: List<GroupSummary>): SidebarParts {
    val favourites = ArrayList<TabSummary>()
    val tree = ArrayList<StripItem>()
    val others = ArrayList<TabSummary>()
    for (item in stripRow(tabs, groups)) {
        when {
            item is StripItem.Tab && item.tab.favourite -> favourites += item.tab
            item is StripItem.Tab && !item.tab.kept -> others += item.tab
            else -> tree += item
        }
    }
    return SidebarParts(favourites, tree, others)
}

/**
 * The sidebar: back, forward and reload with the address field under them; the Favourites; the
 * tree; under a line the other tabs and the button that opens one; and at its foot the Space
 * and the window's buttons.
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun Sidebar(
    model: BrowserModel,
    addressField: FocusRequester,
    onSubmitted: () -> Unit,
    onFind: () -> Unit,
    onZoom: () -> Unit,
    onHistory: () -> Unit,
    onSpaces: () -> Unit,
) {
    val list = stringResource(R.string.tabs_list)
    val parts = sidebarParts(model.tabs, model.groups)
    Column(modifier = Modifier.width(SIDEBAR_WIDTH).fillMaxHeight()) {
        Row(modifier = Modifier.padding(horizontal = Tokens.Space.xs), verticalAlignment = Alignment.CenterVertically) { NavigationButtons(model) }
        AddressField(model, addressField, onSubmitted, Modifier.fillMaxWidth().padding(horizontal = Tokens.Space.s))
        if (parts.favourites.isNotEmpty()) {
            FlowRow(
                modifier = Modifier.padding(horizontal = Tokens.Space.s, vertical = Tokens.Space.xs),
                horizontalArrangement = Arrangement.spacedBy(Tokens.Space.xs),
                verticalArrangement = Arrangement.spacedBy(Tokens.Space.xs),
            ) {
                for (tab in parts.favourites) Entry(model, tab)
            }
        }
        // The rows are made as they come into view: a row of thousands is not drawn to show the
        // few that fit (043 §8a).
        val rows = rememberLazyListState()
        val hasLine = parts.tree.isNotEmpty() || parts.favourites.isNotEmpty()
        // The tab that comes in front is brought into view, when it is one of the rows and is
        // not in view already.
        val front = model.tab.tab_id
        LaunchedEffect(front) {
            val inTree = parts.tree.indexOfFirst { it is StripItem.Tab && it.tab.tab_id == front }
            val among = parts.others.indexOfFirst { it.tab_id == front }
            val place = if (inTree >= 0) inTree else if (among >= 0) parts.tree.size + (if (hasLine) 1 else 0) + among else -1
            if (place >= 0 && rows.layoutInfo.visibleItemsInfo.none { it.index == place }) rows.scrollToItem(place)
        }
        LazyColumn(
            state = rows,
            modifier = Modifier.weight(1f).fillMaxWidth().semantics { contentDescription = list }.testTag("sidebar"),
            contentPadding = PaddingValues(horizontal = Tokens.Space.s, vertical = Tokens.Space.xs),
        ) {
            items(
                parts.tree,
                key = { item ->
                    when (item) {
                        is StripItem.Tab -> "tab-${item.tab.tab_id}"
                        is StripItem.Group -> "group-${item.group.group_id}"
                    }
                },
            ) { item ->
                when (item) {
                    // A Group's Site is set in under its Group.
                    is StripItem.Tab -> Entry(model, item.tab, Modifier.padding(start = if (item.tab.group_id == 0L) 0.dp else Tokens.Space.l), fills = true)
                    is StripItem.Group -> {
                        val group = item.group
                        GroupChip(group, onToggle = { model.shutOrOpen(group) }, onEdit = { model.askToEdit(group) }, onRemove = { model.removeGroup(group.group_id) })
                    }
                }
            }
            // Under a line, the tabs that are only passing through.
            if (hasLine) item(key = "line") { HorizontalDivider(modifier = Modifier.padding(vertical = Tokens.Space.xs)) }
            items(parts.others, key = { tab -> "tab-${tab.tab_id}" }) { tab -> Entry(model, tab, fills = true) }
            item(key = "new-tab") { NewTabButton(model) }
        }
        Row(modifier = Modifier.fillMaxWidth().padding(horizontal = Tokens.Space.xs), verticalAlignment = Alignment.CenterVertically) {
            // The Space the window shows (024), by its name: there is room for it here. A
            // private window says what it is (037).
            if (model.isPrivate) {
                SpaceChip(name = stringResource(R.string.private_chip), compact = false, onClick = onSpaces, saidAs = stringResource(R.string.private_window))
            } else {
                SpaceChip(name = model.spaceName, compact = false, onClick = onSpaces, colour = model.spaceColour)
            }
            Spacer(modifier = Modifier.weight(1f))
            WindowButtons(model, onFind, onZoom, onHistory)
        }
    }
}

/** A tab of the sidebar: the strip's chip, a tile for a Favourite, and for any other a row as wide as the sidebar. */
@Composable
private fun Entry(model: BrowserModel, tab: TabSummary, modifier: Modifier = Modifier, fills: Boolean = false) {
    TabChip(
        tab,
        inFront = tab.tab_id == model.tab.tab_id,
        onActivate = { model.activate(tab.tab_id) },
        onClose = { model.closeTab(tab.tab_id) },
        onKeepOrLetGo = { model.keepOrLetGo(tab.tab_id) },
        onSendHome = { model.sendHome(tab.tab_id) },
        onMove = { model.moveTab(tab.tab_id, it) },
        grouping =
            if (model.canBeGrouped(tab)) {
                Grouping(
                    model.groups,
                    onMove = { model.moveToGroup(tab.tab_id, it) },
                    onNew = { model.askForANewGroup(tab.tab_id) },
                    canFavourite = model.canFavourite(tab),
                    onFavourite = { model.setFavourite(tab.tab_id, !tab.favourite) },
                )
            } else {
                null
            },
        modifier = modifier,
        fills = fills,
    )
}

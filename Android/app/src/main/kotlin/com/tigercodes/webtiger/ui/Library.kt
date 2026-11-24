// The Library (057): what the button beside the tabs opens. The lists about the browser that
// are opened now and then, with a switch between them: History (008), Downloads (013),
// Permissions (014), the Archive (049), and what is awake (056). One sheet and one button, so
// that the row of tabs gains none.
package com.tigercodes.webtiger.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.relocation.BringIntoViewRequester
import androidx.compose.foundation.relocation.bringIntoViewRequester
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.LocalMinimumInteractiveComponentSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.core.v1.HistoryEntry
import com.tigercodes.webtiger.design.Tokens

/** Which of the sheet's lists is shown. */
enum class LibraryPart { HISTORY, DOWNLOADS, PERMISSIONS, ARCHIVE, AWAKE }

/**
 * `onOpen` is called with a visit a person chose; `onDone` when they are finished, and when a
 * tab they chose in the Archive is a tab again. `archive` is the Archive of the window's Space
 * (049); with none (a private window has none) the switch has no part for it. `awake` is what
 * is awake (056), the last of the parts, asked for again whenever `row`, the window's own row
 * of tabs, changes.
 */
@Composable
fun Library(
    history: HistoryModel,
    downloads: DownloadsModel,
    permissions: PermissionsModel,
    onOpen: (HistoryEntry) -> Unit,
    onDone: () -> Unit,
    modifier: Modifier = Modifier,
    showing: LibraryPart = LibraryPart.HISTORY,
    archive: ArchiveModel? = null,
    awake: AwakeModel? = null,
    row: Any? = null,
) {
    var part by rememberSaveable { mutableStateOf(showing) }
    val switcher: @Composable () -> Unit = { LibrarySwitch(part, archive = archive != null, awake = awake != null) { part = it } }
    when (part) {
        LibraryPart.HISTORY -> HistoryList(model = history, onOpen = onOpen, onDone = onDone, modifier = modifier, switcher = switcher)
        LibraryPart.DOWNLOADS -> DownloadsList(model = downloads, onDone = onDone, modifier = modifier, switcher = switcher)
        LibraryPart.PERMISSIONS -> PermissionsList(model = permissions, onDone = onDone, modifier = modifier, switcher = switcher)
        LibraryPart.ARCHIVE -> archive?.let { ArchiveList(model = it, onBroughtBack = onDone, onDone = onDone, modifier = modifier, switcher = switcher) }
        LibraryPart.AWAKE -> awake?.let { AwakeList(model = it, onDone = onDone, modifier = modifier, watching = row, switcher = switcher) }
    }
}

/**
 * The lists to choose between, of which the one that is shown is marked: where a list's title
 * would be. On a narrow window, or with large text, the row is moved sideways to reach its end.
 * It opens with the name of the list that is shown in sight. The Archive (049) is one of them
 * where the window has one; what is awake (056) is the last.
 */
@Composable
internal fun LibrarySwitch(part: LibraryPart, archive: Boolean = true, awake: Boolean = true, onChoose: (LibraryPart) -> Unit) {
    Row(
        modifier = Modifier.horizontalScroll(rememberScrollState()),
        horizontalArrangement = Arrangement.spacedBy(Tokens.Space.xs),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Choice(R.string.history_title, chosen = part == LibraryPart.HISTORY, tag = "library-history") { onChoose(LibraryPart.HISTORY) }
        Choice(R.string.downloads_title, chosen = part == LibraryPart.DOWNLOADS, tag = "library-downloads") { onChoose(LibraryPart.DOWNLOADS) }
        Choice(R.string.permissions_title, chosen = part == LibraryPart.PERMISSIONS, tag = "library-permissions") { onChoose(LibraryPart.PERMISSIONS) }
        if (archive) Choice(R.string.archive_title, chosen = part == LibraryPart.ARCHIVE, tag = "library-archive") { onChoose(LibraryPart.ARCHIVE) }
        if (awake) Choice(R.string.awake_title, chosen = part == LibraryPart.AWAKE, tag = "library-awake") { onChoose(LibraryPart.AWAKE) }
    }
}

@Composable
private fun Choice(title: Int, chosen: Boolean, tag: String, onChoose: () -> Unit) {
    // Where the names are more than the row is wide, the one shown is brought in sight: once
    // the row has been laid out, which it has not when it is first composed.
    val place = remember { BringIntoViewRequester() }
    LaunchedEffect(chosen) {
        if (chosen) {
            withFrameNanos { }
            place.bringIntoView()
        }
    }
    Box(
        modifier =
            Modifier
                .bringIntoViewRequester(place)
                .clip(RoundedCornerShape(Tokens.Radius.field))
                // Which list is shown is said by the fill, and to a screen reader as "selected".
                .background(if (chosen) Tokens.Palette.fieldFill.current else MaterialTheme.colorScheme.background)
                .selectable(selected = chosen, role = Role.Tab, onClick = onChoose)
                .heightIn(min = LocalMinimumInteractiveComponentSize.current)
                .padding(horizontal = Tokens.Space.m)
                .testTag(tag),
        contentAlignment = Alignment.Center,
    ) {
        // Words a person must be able to read: the primary colour (001 F14).
        Text(text = stringResource(title), style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold)
    }
}

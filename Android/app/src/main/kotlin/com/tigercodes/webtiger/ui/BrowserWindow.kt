// A browser window: its tabs in a strip, a toolbar, a progress line, and the page of the tab in
// front (005). The strip is one plain row on every size of window; the sidebar and the tab
// strip proper arrive in W2.
package com.tigercodes.webtiger.ui

import android.icu.text.ListFormatter
import androidx.activity.compose.BackHandler
import androidx.annotation.DrawableRes
import androidx.annotation.StringRes
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.focusable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.relocation.BringIntoViewRequester
import androidx.compose.foundation.relocation.bringIntoViewRequester
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LocalMinimumInteractiveComponentSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalResources
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.paneTitle
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.core.v1.AddressView
import com.tigercodes.webtiger.core.v1.NavigationCommand
import com.tigercodes.webtiger.core.v1.PermissionKind
import com.tigercodes.webtiger.core.v1.PromptView
import com.tigercodes.webtiger.core.v1.TabSummary
import com.tigercodes.webtiger.core.v1.ZoomChange
import com.tigercodes.webtiger.core.v1.ZoomView
import com.tigercodes.webtiger.design.Tokens

/** One window over `model`. The system's Back goes back in the page while it can. */
@Composable
fun BrowserWindow(session: BrowserSession, model: BrowserModel) {
    LaunchedEffect(model) { model.open() }
    BackHandler(enabled = model.tab.can_go_back) { model.perform(NavigationCommand.NAVIGATION_COMMAND_BACK) }
    // Where the keyboard's focus goes when an address has been submitted: to the page.
    val page = remember { FocusRequester() }
    // A window that opens on a page gives the page the keyboard. Out of touch mode the system
    // hands a new window's focus to its first control that can take it, which is the address
    // field; with nothing typed in it, the field lets go when the page arrives.
    LaunchedEffect(model.hasPage) {
        if (model.hasPage && model.isEditingAddress && !model.hasTypedAddress) page.requestFocus()
    }
    // Another tab came in front. An empty one is given the keyboard in its address field: there
    // is nothing else to do in an empty tab. One with a page keeps the keyboard out of the
    // field. (The window's first snapshot names its first tab, and is no switch.)
    val addressField = remember { FocusRequester() }
    var shownTab by remember { mutableLongStateOf(0L) }
    LaunchedEffect(model.tab.tab_id) {
        if (shownTab != 0L) {
            if (model.hasPage) page.requestFocus() else addressField.requestFocus()
        }
        shownTab = model.tab.tab_id
    }
    // The History list is open over the window (008).
    var showingHistory by rememberSaveable { mutableStateOf(false) }
    Surface(modifier = Modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background) {
        // The window draws edge to edge; its content keeps clear of the system's bars, the
        // cutout and the keyboard.
        Column(modifier = Modifier.fillMaxSize().windowInsetsPadding(WindowInsets.safeDrawing)) {
            TabStrip(model, onFind = model::openFind, onZoom = model::openZoom, onHistory = { showingHistory = true })
            Toolbar(model, addressField, onSubmitted = { page.requestFocus() })
            // What the tab asks the person (014): in the tab, under the toolbar, and not over
            // the window. It takes the keyboard from nothing by appearing.
            model.prompt?.let { asked -> PromptBar(asked, onAnswer = model::answerPrompt) }
            if (model.isFinding) FindBar(model)
            if (model.isZooming) ZoomBar(model)
            ProgressLine(model)
            Box(modifier = Modifier.fillMaxWidth().weight(1f).focusRequester(page).focusable()) {
                when {
                    model.hasFailed ->
                        FailedPage(code = model.tab.error_code, onRetry = { model.perform(NavigationCommand.NAVIGATION_COMMAND_RELOAD) })
                    model.hasPage -> WebViewHost(session.engine, model.tab.web_view_id, Modifier.fillMaxSize().testTag("page"))
                    else -> EmptyTab()
                }
                // What is offered for what is typed (011): under the toolbar, over the page.
                if (model.suggestions.isNotEmpty()) SuggestionList(model, onChosen = { page.requestFocus() })
            }
        }
        // Over the window, which stays as it is underneath: the page's web view is not taken
        // out of it. A visit chosen loads in the tab in front; the system's Back closes the
        // list before it does anything to the page.
        val windowId = model.windowId
        if (showingHistory && windowId != null) {
            val history = remember(windowId) { HistoryModel(session, windowId) }
            val downloads = remember(windowId) { DownloadsModel(session, windowId) }
            val permissions = remember(windowId) { PermissionsModel(session, windowId) }
            BackHandler { showingHistory = false }
            Library(
                history = history,
                downloads = downloads,
                permissions = permissions,
                onOpen = { visit ->
                    showingHistory = false
                    model.load(visit.url)
                },
                onDone = { showingHistory = false },
                modifier = Modifier.windowInsetsPadding(WindowInsets.safeDrawing),
            )
        }
    }
}

/**
 * The window's tabs in a row: each by its name, the one in front marked, a button to close each
 * and one to open another (005 §5).
 */
@Composable
private fun TabStrip(model: BrowserModel, onFind: () -> Unit, onZoom: () -> Unit, onHistory: () -> Unit) {
    val list = stringResource(R.string.tabs_list)
    Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
        Row(
            modifier =
                Modifier
                    .weight(1f)
                    .horizontalScroll(rememberScrollState())
                    .padding(horizontal = Tokens.Space.xs)
                    .semantics { contentDescription = list }
                    .testTag("tab-strip"),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(Tokens.Space.xs),
        ) {
            for (tab in model.tabs) {
                key(tab.tab_id) {
                    TabChip(tab, inFront = tab.tab_id == model.tab.tab_id, onActivate = { model.activate(tab.tab_id) }, onClose = { model.closeTab(tab.tab_id) })
                }
            }
            IconButton(onClick = model::openTab, modifier = Modifier.testTag("new-tab")) {
                Icon(painter = painterResource(R.drawable.ic_add), contentDescription = stringResource(R.string.tabs_new))
            }
        }
        // The find bar's button (010), the zoom bar's (012) and the History list's (008):
        // beside the tabs and not among them, where they stay when the tabs scroll. There is
        // nothing to find in a tab with no page, and nothing to zoom without one that can be.
        IconButton(onClick = onFind, enabled = model.hasPage, modifier = Modifier.testTag("find")) {
            Icon(painter = painterResource(R.drawable.ic_search), contentDescription = stringResource(R.string.find_open))
        }
        IconButton(onClick = onZoom, enabled = model.zoom != null, modifier = Modifier.testTag("zoom")) {
            Icon(painter = painterResource(R.drawable.ic_zoom), contentDescription = stringResource(R.string.zoom_open))
        }
        // A download is coming (013): a dot says so, where the list of them opens, and a
        // screen reader is told.
        val downloading = if (model.downloadsActive > 0) stringResource(R.string.downloads_active) else ""
        IconButton(onClick = onHistory, modifier = Modifier.semantics { stateDescription = downloading }.testTag("history")) {
            Box {
                Icon(painter = painterResource(R.drawable.ic_history), contentDescription = stringResource(R.string.history_open))
                if (model.downloadsActive > 0) {
                    Box(
                        modifier =
                            Modifier
                                .align(Alignment.TopEnd)
                                .size(Tokens.Space.s)
                                .clip(CircleShape)
                                .background(MaterialTheme.colorScheme.primary)
                                .testTag("downloading"),
                    )
                }
            }
        }
    }
}

/** One tab in the strip: its name, which brings it in front, and a button that closes it. */
@Composable
private fun TabChip(tab: TabSummary, inFront: Boolean, onActivate: () -> Unit, onClose: () -> Unit) {
    // The tab that comes in front is brought into view.
    val place = remember { BringIntoViewRequester() }
    LaunchedEffect(inFront) { if (inFront) place.bringIntoView() }
    Row(
        modifier =
            Modifier
                .bringIntoViewRequester(place)
                .clip(RoundedCornerShape(Tokens.Radius.field))
                .background(if (inFront) Tokens.Palette.fieldFill.current else Color.Transparent),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Row(
            modifier =
                Modifier
                    .heightIn(min = LocalMinimumInteractiveComponentSize.current)
                    .selectable(selected = inFront, role = Role.Tab, onClick = onActivate)
                    .padding(start = Tokens.Space.m)
                    .testTag("tab"),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(Tokens.Space.xs),
        ) {
            if (tab.loading) {
                // A dot, not a spinner: a tab that is loading is marked, and stays still.
                Box(modifier = Modifier.size(6.dp).clip(CircleShape).background(Tokens.Palette.accent.current))
            }
            Text(
                // A tab's name: its page's title, or its site, or the words for a tab with no page.
                text = tab.title.ifEmpty { tab.host.ifEmpty { stringResource(R.string.tabs_untitled) } },
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                fontWeight = if (inFront) FontWeight.SemiBold else FontWeight.Normal,
                modifier = Modifier.widthIn(max = 180.dp),
            )
        }
        IconButton(onClick = onClose, modifier = Modifier.testTag("close-tab")) {
            Icon(painter = painterResource(R.drawable.ic_close), contentDescription = stringResource(R.string.tabs_close), modifier = Modifier.size(18.dp))
        }
    }
}

@Composable
private fun Toolbar(model: BrowserModel, addressField: FocusRequester, onSubmitted: () -> Unit) {
    Row(
        modifier = Modifier.fillMaxWidth().heightIn(min = Tokens.Size.toolbarHeight).padding(horizontal = Tokens.Space.xs, vertical = Tokens.Space.xs),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        ToolbarButton(R.string.toolbar_back, R.drawable.ic_back, "back", enabled = model.tab.can_go_back) {
            model.perform(NavigationCommand.NAVIGATION_COMMAND_BACK)
        }
        ToolbarButton(R.string.toolbar_forward, R.drawable.ic_forward, "forward", enabled = model.tab.can_go_forward) {
            model.perform(NavigationCommand.NAVIGATION_COMMAND_FORWARD)
        }
        if (model.isLoading) {
            ToolbarButton(R.string.toolbar_stop, R.drawable.ic_stop, "stop", enabled = true) {
                model.perform(NavigationCommand.NAVIGATION_COMMAND_STOP)
            }
        } else {
            ToolbarButton(R.string.toolbar_reload, R.drawable.ic_reload, "reload", enabled = model.hasPage) {
                model.perform(NavigationCommand.NAVIGATION_COMMAND_RELOAD)
            }
        }
        AddressField(model, addressField, onSubmitted, Modifier.weight(1f).padding(start = Tokens.Space.xs))
    }
}

/**
 * The find bar (010): a field, what was found, the match before and the next, and Done. The
 * system's Back closes it before it does anything to the page.
 */
@Composable
private fun FindBar(model: BrowserModel) {
    BackHandler(onBack = model::closeFind)
    val label = stringResource(R.string.find_field)
    val field = remember { FocusRequester() }
    // The bar takes the keyboard when it opens, unless it is only a picture of one.
    LaunchedEffect(Unit) { if (model.findBarTakesKeyboard) runCatching { field.requestFocus() } }
    Row(
        modifier = Modifier.fillMaxWidth().heightIn(min = Tokens.Size.toolbarHeight).padding(horizontal = Tokens.Space.xs),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        // The field and what was found share one filled shape, and are two things to a screen
        // reader: what was found is not part of what was typed.
        Row(
            modifier =
                Modifier
                    .weight(1f)
                    .padding(start = Tokens.Space.xs)
                    .heightIn(min = Tokens.Size.fieldHeight)
                    .clip(RoundedCornerShape(Tokens.Radius.field))
                    .background(Tokens.Palette.fieldFill.current)
                    .padding(horizontal = Tokens.Space.m),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(Tokens.Space.s),
        ) {
            BasicTextField(
                value = model.findText,
                onValueChange = model::find,
                singleLine = true,
                textStyle = MaterialTheme.typography.bodyLarge.copy(color = MaterialTheme.colorScheme.onBackground),
                cursorBrush = SolidColor(MaterialTheme.colorScheme.onBackground),
                keyboardOptions = KeyboardOptions(capitalization = KeyboardCapitalization.None, autoCorrectEnabled = false, imeAction = ImeAction.Search),
                // Return goes to the next match, and the keyboard stays in the field.
                keyboardActions = KeyboardActions(onSearch = { model.findNext() }),
                modifier = Modifier.weight(1f).focusRequester(field).semantics { contentDescription = label }.testTag("find-field"),
                decorationBox = { text ->
                    Box(contentAlignment = Alignment.CenterStart) {
                        if (model.findText.isEmpty()) Text(text = label, style = MaterialTheme.typography.bodyLarge, maxLines = 1)
                        text()
                    }
                },
            )
            // What was found, in words: how many where the engine counts, and that there is
            // none.
            when (val outcome = model.findOutcome) {
                BrowserModel.FindOutcome.Nothing -> Unit
                BrowserModel.FindOutcome.NotFound -> Text(text = stringResource(R.string.find_none), maxLines = 1, modifier = Modifier.testTag("find-outcome"))
                is BrowserModel.FindOutcome.Match ->
                    Text(text = stringResource(R.string.find_count, outcome.current, outcome.of), maxLines = 1, modifier = Modifier.testTag("find-outcome"))
            }
        }
        ToolbarButton(R.string.find_previous, R.drawable.ic_up, "find-previous", enabled = model.findText.isNotEmpty(), onClick = model::findPrevious)
        ToolbarButton(R.string.find_next, R.drawable.ic_down, "find-next", enabled = model.findText.isNotEmpty(), onClick = model::findNext)
        TextButton(onClick = model::closeFind, modifier = Modifier.testTag("find-done")) { Text(stringResource(R.string.find_done)) }
    }
}

/**
 * The zoom bar (012 §6): smaller, the level, bigger, and Done. What it says and which of its
 * buttons can be pressed is the core's: the level of the page in front, and whether there is
 * one above and one below. Pressing the level puts the page back to actual size.
 */
@Composable
private fun ZoomBar(model: BrowserModel) {
    // The system's Back closes the bar before it does anything to the page.
    BackHandler(onBack = model::closeZoom)
    val zoom = model.zoom ?: ZoomView()
    val level = stringResource(R.string.zoom_level, zoom.percent)
    val reset = stringResource(R.string.zoom_reset)
    Row(
        modifier = Modifier.fillMaxWidth().heightIn(min = Tokens.Size.toolbarHeight).padding(horizontal = Tokens.Space.xs),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        ToolbarButton(R.string.zoom_out, R.drawable.ic_remove, "zoom-out", enabled = zoom.can_zoom_out, onClick = { model.zoom(ZoomChange.ZOOM_CHANGE_OUT) })
        // What a screen reader is told: what pressing it does, and the level as its state.
        TextButton(
            onClick = { model.zoom(ZoomChange.ZOOM_CHANGE_RESET) },
            enabled = zoom.percent != ACTUAL_SIZE,
            modifier =
                Modifier
                    .semantics {
                        contentDescription = reset
                        stateDescription = level
                    }
                    .testTag("zoom-level"),
        ) {
            Text(text = level, maxLines = 1, modifier = Modifier.clearAndSetSemantics { })
        }
        ToolbarButton(R.string.zoom_in, R.drawable.ic_add, "zoom-in", enabled = zoom.can_zoom_in, onClick = { model.zoom(ZoomChange.ZOOM_CHANGE_IN) })
        Spacer(modifier = Modifier.weight(1f))
        TextButton(onClick = model::closeZoom, modifier = Modifier.testTag("zoom-done")) { Text(stringResource(R.string.zoom_done)) }
    }
}

/**
 * What a tab asks the person (014 §7): a sentence that names who asks, in the core's words for
 * the site, and what for; and the two answers. Nothing in it is the page's. Neither answer is
 * given by a key: a press on one of the buttons is the only way.
 */
@Composable
internal fun PromptBar(prompt: PromptView, onAnswer: (Boolean) -> Unit) {
    val words = promptWords(prompt)
    Column(
        modifier =
            Modifier
                .fillMaxWidth()
                .background(Tokens.Palette.fieldFill.current)
                .padding(start = Tokens.Space.m, end = Tokens.Space.xs, top = Tokens.Space.s)
                .testTag("prompt"),
    ) {
        // Said to a screen reader when it appears: a page that waits for an answer says
        // nothing by itself.
        Text(text = words, modifier = Modifier.fillMaxWidth().semantics { liveRegion = LiveRegionMode.Polite }.testTag("prompt-words"))
        Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            Spacer(modifier = Modifier.weight(1f))
            TextButton(onClick = { onAnswer(false) }, modifier = Modifier.testTag("prompt-refuse")) { Text(stringResource(R.string.prompt_refuse)) }
            TextButton(onClick = { onAnswer(true) }, modifier = Modifier.testTag("prompt-allow")) { Text(stringResource(R.string.prompt_allow)) }
        }
    }
}

/**
 * A prompt's sentence. What a page wants is named in the person's language and joined as the
 * language joins a list; a kind this build has no words for is left out of the list, and a
 * prompt of a kind it does not know says nothing.
 */
@Composable
internal fun promptWords(prompt: PromptView): String {
    // A download a site was not yet allowed (015): the site and the file, both in the core's
    // words. The name is in quotation marks, so that where it begins and ends can be seen.
    prompt.download?.let { download -> return stringResource(R.string.prompt_download, download.site, download.file_name) }
    val permission = prompt.permission ?: return ""
    val wanted =
        permission.kinds.mapNotNull { kind ->
            when (kind) {
                PermissionKind.PERMISSION_KIND_CAMERA -> stringResource(R.string.prompt_kind_camera)
                PermissionKind.PERMISSION_KIND_MICROPHONE -> stringResource(R.string.prompt_kind_microphone)
                PermissionKind.PERMISSION_KIND_LOCATION -> stringResource(R.string.prompt_kind_location)
                else -> null
            }
        }
    val locale = LocalConfiguration.current.locales[0]
    return stringResource(R.string.prompt_permission, permission.site, ListFormatter.getInstance(locale).format(wanted))
}

/** Actual size, in per cent: the level at which there is nothing to put back. */
private const val ACTUAL_SIZE = 100

@Composable
private fun ToolbarButton(@StringRes title: Int, @DrawableRes icon: Int, tag: String, enabled: Boolean, onClick: () -> Unit) {
    IconButton(onClick = onClick, enabled = enabled, modifier = Modifier.testTag(tag)) {
        Icon(painter = painterResource(icon), contentDescription = stringResource(title))
    }
}

/**
 * The address field with its security mark. At rest it shows what the core says the page is;
 * with the keyboard it holds the whole address, selected, so typing replaces it.
 */
@Composable
private fun AddressField(model: BrowserModel, focus: FocusRequester, onSubmitted: () -> Unit, modifier: Modifier = Modifier) {
    val label = stringResource(R.string.address_label)
    // The text is the model's; the selection and what the keyboard is composing are the field's.
    var field by remember { mutableStateOf(TextFieldValue(model.addressText)) }
    val text = model.addressText
    val shown =
        when {
            // A field scrolls to its caret. At rest the caret is at the start, so what stays in
            // view of an address too long for the field is its site, and what is cut is its tail.
            !model.isEditingAddress -> TextFieldValue(text, TextRange.Zero)
            field.text == text -> field
            // The page moved on under a field nobody has typed in: its whole address, selected,
            // as when the keyboard came.
            !model.hasTypedAddress -> TextFieldValue(text, TextRange(0, text.length))
            else -> TextFieldValue(text, TextRange(text.length))
        }
    // The tap that gives the field the keyboard also places a caret where it fell, in the text
    // the field showed until then; a second tap close behind it selects a word there, and a
    // third all of it. None of that is typing, and all of it is dropped: the whole address
    // stays selected, and nothing is offered for an address nobody typed (011).
    val tap = remember { FocusingTap() }
    SideEffect {
        if (field != shown) field = shown
        // Drawn again since the focus came: whatever arrives now is the user's own.
        tap.restText = null
    }

    Row(
        modifier =
            modifier
                .heightIn(min = LocalMinimumInteractiveComponentSize.current)
                .clip(RoundedCornerShape(Tokens.Radius.field))
                .background(Tokens.Palette.fieldFill.current)
                .padding(horizontal = Tokens.Space.s),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(Tokens.Space.xs),
    ) {
        SecurityMark(model.fieldSecurity)
        Box(modifier = Modifier.weight(1f), contentAlignment = Alignment.CenterStart) {
            if (shown.text.isEmpty()) {
                Text(
                    text = stringResource(R.string.address_placeholder),
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                )
            }
            BasicTextField(
                value = shown,
                onValueChange = { changed ->
                    if (changed.text == tap.restText) {
                        // Still the text the field showed at rest: the focusing gesture's own
                        // caret or selection, until the field is drawn again.
                    } else {
                        // Typing is editing, wherever the focus was last said to be.
                        if (!model.isEditingAddress) model.isEditingAddress = true
                        field = changed
                        model.addressText = changed.text
                    }
                },
                singleLine = true,
                textStyle = MaterialTheme.typography.bodyLarge.copy(color = MaterialTheme.colorScheme.onSurface),
                cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
                keyboardOptions =
                    KeyboardOptions(
                        capitalization = KeyboardCapitalization.None,
                        autoCorrectEnabled = false,
                        keyboardType = KeyboardType.Uri,
                        imeAction = ImeAction.Go,
                    ),
                keyboardActions =
                    KeyboardActions(
                        onGo = {
                            model.submit()
                            onSubmitted()
                        },
                    ),
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .focusRequester(focus)
                        .onFocusChanged { state ->
                            if (state.isFocused == model.isEditingAddress) return@onFocusChanged
                            val atRest = model.addressText
                            model.isEditingAddress = state.isFocused
                            if (state.isFocused) {
                                // As every browser does: the whole address, selected.
                                val whole = model.addressText
                                field = TextFieldValue(whole, TextRange(0, whole.length))
                                tap.restText = atRest
                            }
                        }
                        .testTag("address-field")
                        .semantics { contentDescription = label },
            )
        }
    }
}

/**
 * The places offered for what is typed in the address field (011 §5): each by its title and its
 * address. Choosing one goes there, and the page takes the keyboard from the field.
 */
@Composable
private fun SuggestionList(model: BrowserModel, onChosen: () -> Unit) {
    val label = stringResource(R.string.suggestions_list)
    // The system's Back puts the list away before it does anything to the page.
    BackHandler { model.putSuggestionsAway() }
    Column(
        modifier =
            Modifier
                .fillMaxWidth()
                .background(MaterialTheme.colorScheme.background)
                .semantics { paneTitle = label }
                .testTag("suggestions"),
    ) {
        model.suggestions.forEach { place ->
            Column(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .clickable(role = Role.Button) {
                            model.choose(place)
                            onChosen()
                        }
                        .heightIn(min = LocalMinimumInteractiveComponentSize.current)
                        .padding(horizontal = Tokens.Space.m, vertical = Tokens.Space.xs)
                        .testTag("suggestion"),
                verticalArrangement = Arrangement.Center,
            ) {
                // A place with no title is named by its address. Each is one line: a long
                // title is cut at its end, a long address in its middle, where its site and
                // its end both stay.
                Text(text = place.title.ifEmpty { place.display }, maxLines = 1, overflow = TextOverflow.Ellipsis)
                if (place.title.isNotEmpty()) {
                    Text(text = place.display, maxLines = 1, overflow = TextOverflow.MiddleEllipsis, style = MaterialTheme.typography.bodySmall)
                }
            }
            HorizontalDivider()
        }
    }
}

/** What the address field showed when the keyboard's focus came to it, until it is drawn again. */
private class FocusingTap {
    var restText: String? = null
}

@Composable
private fun SecurityMark(security: AddressView.Security) {
    val size = Modifier.size(18.dp)
    when (security) {
        AddressView.Security.SECURITY_SECURE ->
            Icon(
                painter = painterResource(R.drawable.ic_secure),
                contentDescription = stringResource(R.string.address_secure),
                tint = Tokens.Palette.secure.current,
                modifier = size.testTag("security-secure"),
            )
        AddressView.Security.SECURITY_INSECURE ->
            Icon(
                painter = painterResource(R.drawable.ic_warning),
                contentDescription = stringResource(R.string.address_insecure),
                tint = Tokens.Palette.insecure.current,
                modifier = size.testTag("security-insecure"),
            )
        AddressView.Security.SECURITY_UNSPECIFIED, AddressView.Security.SECURITY_LOCAL ->
            Icon(
                painter = painterResource(R.drawable.ic_search),
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = size,
            )
    }
}

/** A thin line under the toolbar: how far the load has come. It says nothing to a screen reader. */
@Composable
private fun ProgressLine(model: BrowserModel) {
    val target = if (model.isLoading) model.tab.progress.toFloat().coerceIn(0.05f, 1f) else 0f
    val fraction by animateFloatAsState(targetValue = target, label = "progress")
    Box(
        modifier =
            Modifier
                .fillMaxWidth()
                .height(Tokens.Size.progressHeight)
                .background(Tokens.Palette.hairline.current)
                .clearAndSetSemantics {},
    ) {
        Box(modifier = Modifier.fillMaxHeight().fillMaxWidth(fraction).background(Tokens.Palette.accent.current))
    }
}

/** A tab with no page yet. */
@Composable
internal fun EmptyTab() {
    Column(
        // Words in the middle of the space they have, which scroll when it is too small for
        // them: at the largest text sizes they would otherwise run off both ends of it.
        modifier = Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(Tokens.Space.xl).testTag("empty-tab"),
        verticalArrangement = Arrangement.spacedBy(Tokens.Space.s, Alignment.CenterVertically),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text(
            text = stringResource(R.string.tab_empty_title),
            style = MaterialTheme.typography.headlineLarge,
            fontWeight = FontWeight.SemiBold,
            textAlign = TextAlign.Center,
        )
        Text(
            text = stringResource(R.string.tab_empty_hint),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
        )
    }
}

/** A page that failed to load. The core gave a code; the words are the shell's. */
@Composable
internal fun FailedPage(code: String, onRetry: () -> Unit) {
    val resources = LocalResources.current
    val packageName = LocalContext.current.packageName
    Column(
        modifier = Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(Tokens.Space.xl).testTag("failed-page"),
        verticalArrangement = Arrangement.spacedBy(Tokens.Space.m, Alignment.CenterVertically),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Icon(
            painter = painterResource(R.drawable.ic_warning),
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.size(44.dp),
        )
        Text(
            text = stringResource(R.string.error_title),
            style = MaterialTheme.typography.titleLarge,
            fontWeight = FontWeight.SemiBold,
            textAlign = TextAlign.Center,
        )
        Text(
            text = errorWords(resources, packageName, code),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
            modifier = Modifier.testTag("failed-words"),
        )
        Button(onClick = onRetry, modifier = Modifier.testTag("retry")) {
            Text(text = stringResource(R.string.error_retry))
        }
    }
}

/** Shown instead of a window when the core would not start. */
@Composable
fun StartFailed(code: String) {
    Surface(modifier = Modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background) {
        Column(
            modifier =
                Modifier
                    .fillMaxSize()
                    .windowInsetsPadding(WindowInsets.safeDrawing)
                    .verticalScroll(rememberScrollState())
                    .padding(Tokens.Space.xl)
                    .testTag("start-failed"),
            verticalArrangement = Arrangement.spacedBy(Tokens.Space.m, Alignment.CenterVertically),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Text(
                text = stringResource(R.string.error_start_title),
                style = MaterialTheme.typography.titleLarge,
                fontWeight = FontWeight.SemiBold,
                textAlign = TextAlign.Center,
            )
            Text(
                text = stringResource(R.string.error_start_detail, code),
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                textAlign = TextAlign.Center,
            )
        }
    }
}

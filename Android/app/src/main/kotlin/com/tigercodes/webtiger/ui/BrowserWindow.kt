// A browser window: its tabs in a strip, a toolbar, a progress line, and the page of the tab in
// front (005). The strip is one plain row on every size of window; the sidebar and the tab
// strip proper arrive in W2.
package com.tigercodes.webtiger.ui

import androidx.activity.compose.BackHandler
import androidx.annotation.DrawableRes
import androidx.annotation.StringRes
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.focusable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.relocation.BringIntoViewRequester
import androidx.compose.foundation.relocation.bringIntoViewRequester
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.layout.widthIn
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.style.TextOverflow
import com.tigercodes.webtiger.core.v1.TabSummary
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
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
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalResources
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.core.v1.AddressView
import com.tigercodes.webtiger.core.v1.NavigationCommand
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
            TabStrip(model, onFind = model::openFind, onHistory = { showingHistory = true })
            Toolbar(model, addressField, onSubmitted = { page.requestFocus() })
            if (model.isFinding) FindBar(model)
            ProgressLine(model)
            Box(modifier = Modifier.fillMaxWidth().weight(1f).focusRequester(page).focusable()) {
                when {
                    model.hasFailed ->
                        FailedPage(code = model.tab.error_code, onRetry = { model.perform(NavigationCommand.NAVIGATION_COMMAND_RELOAD) })
                    model.hasPage -> WebViewHost(session.engine, model.tab.web_view_id, Modifier.fillMaxSize().testTag("page"))
                    else -> EmptyTab()
                }
            }
        }
        // Over the window, which stays as it is underneath: the page's web view is not taken
        // out of it. A visit chosen loads in the tab in front; the system's Back closes the
        // list before it does anything to the page.
        val windowId = model.windowId
        if (showingHistory && windowId != null) {
            val history = remember(windowId) { HistoryModel(session, windowId) }
            BackHandler { showingHistory = false }
            HistoryList(
                model = history,
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
private fun TabStrip(model: BrowserModel, onFind: () -> Unit, onHistory: () -> Unit) {
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
        // The find bar's button (010) and the History list's (008): beside the tabs and not
        // among them, where they stay when the tabs scroll. There is nothing to find in a tab
        // with no page.
        IconButton(onClick = onFind, enabled = model.hasPage, modifier = Modifier.testTag("find")) {
            Icon(painter = painterResource(R.drawable.ic_search), contentDescription = stringResource(R.string.find_open))
        }
        IconButton(onClick = onHistory, modifier = Modifier.testTag("history")) {
            Icon(painter = painterResource(R.drawable.ic_history), contentDescription = stringResource(R.string.history_open))
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
    // the field showed until then. That caret is dropped: the whole address stays selected.
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
        SecurityMark(model.address.security)
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
                    if (changed.text == tap.restText && changed.selection.collapsed) {
                        tap.restText = null
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

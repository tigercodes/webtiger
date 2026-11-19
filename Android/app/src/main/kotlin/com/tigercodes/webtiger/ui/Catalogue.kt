// The design catalogue (004): every design token, and the window in each of its states. A
// designer opens it (`--ez webtiger.catalogue true`, in a debuggable build); the snapshot tests
// draw the same composables.
package com.tigercodes.webtiger.ui

import android.content.Context
import androidx.activity.OnBackPressedDispatcher
import androidx.activity.OnBackPressedDispatcherOwner
import androidx.activity.compose.LocalOnBackPressedDispatcherOwner
import androidx.annotation.StringRes
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalLifecycleOwner
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.lifecycle.Lifecycle
import com.tigercodes.webtiger.R
import com.tigercodes.webtiger.core.ScriptedCore
import com.tigercodes.webtiger.core.v1.Ack
import com.tigercodes.webtiger.core.v1.AddressSuggestion
import com.tigercodes.webtiger.core.v1.AddressView
import com.tigercodes.webtiger.core.v1.LoadState
import com.tigercodes.webtiger.core.v1.Response
import com.tigercodes.webtiger.core.v1.FindView
import com.tigercodes.webtiger.core.v1.HistoryEntry
import com.tigercodes.webtiger.core.v1.HistoryPage
import com.tigercodes.webtiger.core.v1.Subscribed
import com.tigercodes.webtiger.core.v1.TabSummary
import com.tigercodes.webtiger.core.v1.TabView
import com.tigercodes.webtiger.core.v1.WindowOpened
import com.tigercodes.webtiger.core.v1.WindowSnapshot
import com.tigercodes.webtiger.design.Tokens
import com.tigercodes.webtiger.webview.SystemWebViewEngine
import java.time.ZoneOffset
import java.util.Locale
import kotlinx.coroutines.CoroutineScope

/** One state of the browser window: the snapshot a core would publish for it. */
internal enum class CatalogueState(@StringRes val title: Int) {
    EMPTY(R.string.catalogue_state_empty),
    LOADING(R.string.catalogue_state_loading),
    SECURE(R.string.catalogue_state_secure),
    INSECURE(R.string.catalogue_state_insecure),
    LONG(R.string.catalogue_state_long),
    FAILED(R.string.catalogue_state_failed),
    HISTORY(R.string.catalogue_state_history),
    TABS(R.string.catalogue_state_tabs),
    FIND(R.string.catalogue_state_find),
    FIND_NONE(R.string.catalogue_state_find_none),
    SUGGESTIONS(R.string.catalogue_state_suggestions),
    ;

    /** What is typed in the address field in this state, when something is. */
    val typing: String? get() = if (this == SUGGESTIONS) "exa" else null

    /**
     * The places offered for it: one with a title, one whose title is longer than the list is
     * wide, one with none.
     */
    val offered: List<AddressSuggestion>
        get() =
            if (this == SUGGESTIONS) {
                listOf(
                    AddressSuggestion(url = "https://example.com/", title = "Example Domain", display = "example.com"),
                    AddressSuggestion(
                        url = "https://example.org/a/long/path",
                        title = "A page whose title is longer than the list is wide, and then goes on",
                        display = "example.org/a/long/path",
                    ),
                    AddressSuggestion(url = "https://example.net/plain.txt", display = "example.net/plain.txt"),
                )
            } else {
                emptyList()
            }

    /** What the window's find bar is open on in this state, when it is. */
    val finding: String?
        get() =
            when (this) {
                FIND -> "needle"
                FIND_NONE -> "zebra"
                else -> null
            }

    /** What a core would publish for the state. The addresses are the documentation's own. */
    val snapshot: WindowSnapshot
        get() {
            val tab =
                when (this) {
                    EMPTY -> TabView(tab_id = 2, load_state = LoadState.LOAD_STATE_IDLE)
                    LOADING -> page("https://example.com/", "example.com").copy(load_state = LoadState.LOAD_STATE_LOADING, progress = 0.4)
                    SECURE -> page("https://example.com/", "example.com")
                    INSECURE -> page("http://example.com/", "http://example.com", AddressView.Security.SECURITY_INSECURE)
                    LONG ->
                        "example.com/a/path/that/is/longer/than/the/field/is/wide/and/then/goes/on/for/a/while?with=a&query=too"
                            .let { path -> page("https://$path", path) }
                    FAILED -> page("https://nowhere.example.com/", "nowhere.example.com").copy(load_state = LoadState.LOAD_STATE_FAILED, error_code = "net.dns")
                    HISTORY -> page("https://example.com/second", "example.com/second").copy(can_go_back = true, can_go_forward = true)
                    TABS -> page("https://example.com/", "example.com").copy(title = "Example Domain")
                    // An engine that counts: the third of twelve.
                    FIND ->
                        page("https://example.com/", "example.com")
                            .copy(find = FindView(text = "needle", answered = true, found = true, counted = true, matches = 12, current = 3))
                    FIND_NONE -> page("https://example.com/", "example.com").copy(find = FindView(text = "zebra", answered = true))
                    SUGGESTIONS -> page("https://example.org/", "example.org")
                }
            // The strip names every tab: the one in front, and for "several tabs" its neighbours,
            // one with a title, one still loading and known only by its site, one with no page.
            val front =
                TabSummary(
                    tab_id = tab.tab_id,
                    title = tab.title,
                    host = tab.address?.host.orEmpty(),
                    loading = tab.load_state == LoadState.LOAD_STATE_LOADING,
                )
            val tabs =
                if (this == TABS) {
                    listOf(
                        TabSummary(tab_id = 11, title = "A page whose title is longer than a tab is wide"),
                        front,
                        TabSummary(tab_id = 12, host = "example.org", loading = true),
                        TabSummary(tab_id = 13),
                    )
                } else {
                    listOf(front)
                }
            return WindowSnapshot(window_id = 1, tab = tab, tabs = tabs)
        }

    private fun page(url: String, display: String, security: AddressView.Security = AddressView.Security.SECURITY_SECURE): TabView =
        TabView(
            tab_id = 2,
            web_view_id = 4,
            load_state = LoadState.LOAD_STATE_IDLE,
            address = AddressView(url = url, display = display, host = java.net.URI(url.substringBefore('?')).host.orEmpty(), registrable_domain = "example.com", security = security),
        )

    /**
     * A session whose core opens one window and publishes this state for it, and nothing else:
     * no Rust core, and no web engine doing anything.
     */
    fun session(context: Context, scope: CoroutineScope): BrowserSession {
        val published = snapshot
        lateinit var core: ScriptedCore
        core =
            ScriptedCore { request ->
                when {
                    request.open_window != null -> Response(window_opened = WindowOpened(window_id = 1, tab_id = 2))
                    request.subscribe != null -> {
                        // As the core does: a subscription's first snapshot comes with its answer.
                        core.emit(published, subscription = 3)
                        Response(subscribed = Subscribed(subscription_id = 3))
                    }
                    else -> Response(ack = Ack())
                }
            }
        return BrowserSession(core, SystemWebViewEngine(context), scope)
    }
}

/** One state of the History list (008). */
internal enum class CatalogueHistory(@StringRes val title: Int) {
    LIST(R.string.catalogue_state_history_list),
    EMPTY(R.string.catalogue_state_history_empty),
    NONE_FOUND(R.string.catalogue_state_history_none_found),
    ;

    /**
     * What a core would answer for the state. The addresses are the documentation's own, and
     * the times are fixed: midday on the first of January 2026, and before.
     */
    private val page: HistoryPage
        get() =
            if (this == LIST) {
                HistoryPage(
                    entries =
                        listOf(
                            visit(4, "Example Domain", "https://example.com/", "example.com", minutesAgo = 0),
                            visit(3, "A page whose title is longer than the list is wide, and then goes on", "https://example.org/a/long/path", "example.org/a/long/path", minutesAgo = 7),
                            visit(2, "", "https://example.net/plain.txt", "example.net/plain.txt", minutesAgo = 95),
                            visit(1, "Yesterday’s page", "http://example.com/yesterday", "http://example.com/yesterday", minutesAgo = 60 * 26),
                        ),
                    more = true,
                )
            } else {
                HistoryPage()
            }

    private fun visit(id: Long, title: String, url: String, display: String, minutesAgo: Long): HistoryEntry =
        HistoryEntry(visit_id = id, title = title, url = url, display = display, visited_at_ms = 1_767_268_800_000 - minutesAgo * 60_000)

    /** A model over a core that answers with this state, and nothing else. */
    fun model(context: Context, scope: CoroutineScope): HistoryModel {
        val answer = page
        val core = ScriptedCore { request -> if (request.query_history != null) Response(history_page = answer) else Response(ack = Ack()) }
        val model = HistoryModel(BrowserSession(core, SystemWebViewEngine(context), scope), windowId = 1)
        if (this == NONE_FOUND) model.search("zebra")
        return model
    }
}

/** The catalogue: the tokens, then the window in each state, then the start failure. */
@Composable
fun CatalogueScreen() {
    Surface(modifier = Modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background) {
        Column(
            modifier =
                Modifier
                    .fillMaxSize()
                    // Kept clear of the system's bars here, once: the windows inside are given no
                    // insets of their own to keep clear of.
                    .windowInsetsPadding(WindowInsets.safeDrawing)
                    .verticalScroll(rememberScrollState())
                    .padding(Tokens.Space.xl)
                    .testTag("catalogue"),
            verticalArrangement = Arrangement.spacedBy(Tokens.Space.xl),
        ) {
            Text(
                text = stringResource(R.string.catalogue_title),
                style = MaterialTheme.typography.headlineLarge,
                fontWeight = FontWeight.SemiBold,
                modifier = Modifier.semantics { heading() },
            )
            CatalogueTokens()
            CatalogueHeading(R.string.catalogue_states)
            for (state in CatalogueState.entries) {
                Column(verticalArrangement = Arrangement.spacedBy(Tokens.Space.s)) {
                    Text(text = stringResource(state.title), style = MaterialTheme.typography.titleMedium)
                    CatalogueWindow(state, Modifier.fillMaxWidth().height(CatalogueWindowHeight).framed())
                }
            }
            for (state in CatalogueHistory.entries) {
                Column(verticalArrangement = Arrangement.spacedBy(Tokens.Space.s)) {
                    Text(text = stringResource(state.title), style = MaterialTheme.typography.titleMedium)
                    CatalogueHistoryList(state, Modifier.fillMaxWidth().height(CatalogueWindowHeight).framed())
                }
            }
            CatalogueHeading(R.string.catalogue_start_failed)
            Box(modifier = Modifier.fillMaxWidth().height(CatalogueWindowHeight).framed()) { StartFailed("store.open") }
        }
    }
}

/** Tall enough for the failed page, the tallest of the states. */
internal val CatalogueWindowHeight = 320.dp

@Composable
private fun Modifier.framed(): Modifier =
    clip(RoundedCornerShape(Tokens.Radius.tile)).border(1.dp, Tokens.Palette.hairline.current, RoundedCornerShape(Tokens.Radius.tile))

@Composable
private fun CatalogueHeading(@StringRes title: Int) {
    Text(
        text = stringResource(title),
        style = MaterialTheme.typography.titleLarge,
        fontWeight = FontWeight.SemiBold,
        modifier = Modifier.semantics { heading() },
    )
}

/** The window itself in one state: [BrowserWindow] over a scripted core that publishes it. */
@Composable
internal fun CatalogueWindow(state: CatalogueState, modifier: Modifier = Modifier) {
    val context = LocalContext.current.applicationContext
    val scope = rememberCoroutineScope()
    val session = remember(state) { state.session(context, scope) }
    val model = remember(session) { BrowserModel(session, finding = state.finding, typing = state.typing, offered = state.offered) }
    // A window takes the system's Back for its page. These are pictures of windows: Back leaves
    // the catalogue, whatever state one of them shows.
    val lifecycle = LocalLifecycleOwner.current.lifecycle
    val nobody = remember(lifecycle) { DetachedBack(lifecycle) }
    Box(modifier = modifier) {
        CompositionLocalProvider(LocalOnBackPressedDispatcherOwner provides nobody) { BrowserWindow(session, model) }
    }
}

/**
 * The History list in one state: [HistoryList] over a scripted core that answers with it. Its
 * times are shown as one place and one language show them, so a picture of it is the same
 * wherever it is drawn.
 */
@Composable
internal fun CatalogueHistoryList(state: CatalogueHistory, modifier: Modifier = Modifier) {
    val context = LocalContext.current.applicationContext
    val scope = rememberCoroutineScope()
    val model = remember(state) { state.model(context, scope) }
    Box(modifier = modifier) { HistoryList(model, onOpen = {}, onDone = {}, zone = ZoneOffset.UTC, locale = Locale.UK) }
}

/** A Back dispatcher nothing ever calls. */
private class DetachedBack(override val lifecycle: Lifecycle) : OnBackPressedDispatcherOwner {
    override val onBackPressedDispatcher = OnBackPressedDispatcher()
}

/** Every design token, by name: the colours as swatches, the measures drawn to scale. */
@Composable
internal fun CatalogueTokens(modifier: Modifier = Modifier) {
    Column(modifier = modifier.testTag("catalogue-tokens"), verticalArrangement = Arrangement.spacedBy(Tokens.Space.l)) {
        TokenSection(R.string.catalogue_colours) {
            for ((name, colour) in Tokens.Catalogue.colors) {
                TokenRow(name) {
                    Box(
                        modifier =
                            Modifier
                                .size(width = 96.dp, height = 28.dp)
                                .clip(RoundedCornerShape(Tokens.Radius.field))
                                .background(colour.current)
                                .border(1.dp, Tokens.Palette.hairline.current, RoundedCornerShape(Tokens.Radius.field)),
                    )
                }
            }
        }
        TokenSection(R.string.catalogue_space) {
            for ((name, value) in Tokens.Catalogue.space) {
                TokenRow(name, value) { Box(Modifier.size(width = value, height = 12.dp).background(Tokens.Palette.accent.current)) }
            }
        }
        TokenSection(R.string.catalogue_radius) {
            for ((name, value) in Tokens.Catalogue.radius) {
                TokenRow(name, value) {
                    Box(
                        modifier =
                            Modifier
                                .size(width = 96.dp, height = 28.dp)
                                .clip(RoundedCornerShape(value))
                                .background(Tokens.Palette.fieldFill.current)
                                .border(1.dp, Tokens.Palette.hairline.current, RoundedCornerShape(value)),
                    )
                }
            }
        }
        TokenSection(R.string.catalogue_size) {
            for ((name, value) in Tokens.Catalogue.size) {
                TokenRow(name, value) { Box(Modifier.size(width = 12.dp, height = value).background(Tokens.Palette.accent.current)) }
            }
        }
    }
}

@Composable
private fun TokenSection(@StringRes title: Int, rows: @Composable () -> Unit) {
    Column(verticalArrangement = Arrangement.spacedBy(Tokens.Space.s)) {
        CatalogueHeading(title)
        rows()
    }
}

/** One token: its name, its value when it is a measure, and the token itself drawn. */
@Composable
private fun TokenRow(name: String, value: Dp? = null, drawn: @Composable () -> Unit) {
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(Tokens.Space.m)) {
        // A token's name is its identifier in Design/tokens.json: not words, so not translated.
        Text(text = name, fontFamily = FontFamily.Monospace, modifier = Modifier.width(150.dp))
        // A colour has no number. Its place is kept empty, with nothing in it for a screen reader
        // to stop at.
        if (value != null) Text(text = value.value.toInt().toString(), modifier = Modifier.width(36.dp)) else Spacer(Modifier.width(36.dp))
        // The drawing is not part of what is read out: a faint token is drawn faintly on purpose.
        Box(modifier = Modifier.clearAndSetSemantics {}) { drawn() }
    }
}

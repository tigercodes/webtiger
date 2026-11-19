// A window's model: what the core last said about it, and the user's intents sent back.
package com.tigercodes.webtiger.ui

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.tigercodes.webtiger.core.CoreError
import com.tigercodes.webtiger.core.v1.ActivateTab
import com.tigercodes.webtiger.core.v1.AddressSuggestion
import com.tigercodes.webtiger.core.v1.AddressView
import com.tigercodes.webtiger.core.v1.CloseTab
import com.tigercodes.webtiger.core.v1.FindInPage
import com.tigercodes.webtiger.core.v1.LoadState
import com.tigercodes.webtiger.core.v1.NavigationCommand
import com.tigercodes.webtiger.core.v1.OpenTab
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.SubmitAddress
import com.tigercodes.webtiger.core.v1.SuggestAddresses
import com.tigercodes.webtiger.core.v1.TabCommand
import com.tigercodes.webtiger.core.v1.TabSummary
import com.tigercodes.webtiger.core.v1.TabView
import com.tigercodes.webtiger.core.v1.WindowSnapshot
import kotlinx.coroutines.launch

/**
 * One window's state, for Compose to draw. Used on the main thread.
 *
 * A model made with `finding`, or with `typing` and what is `offered` for it, is a picture of a
 * window (004): its find bar is open on that text, or its address field holds it with those
 * places listed, before anything has been asked of the core.
 */
class BrowserModel(
    private val session: BrowserSession,
    finding: String? = null,
    typing: String? = null,
    offered: List<AddressSuggestion> = emptyList(),
) {
    /** The tab in front, as the core last published it. The window's controls act on it. */
    var tab by mutableStateOf(TabView())
        private set

    /** Every tab of the window, in the order they are shown. */
    var tabs by mutableStateOf<List<TabSummary>>(emptyList())
        private set

    private var fieldText by mutableStateOf(typing.orEmpty())

    /**
     * What is in the address field. It follows the core until the user types in it. Each change
     * while it is typed in asks what to offer for it (011).
     */
    var addressText: String
        get() = fieldText
        set(value) {
            if (value == fieldText) return
            fieldText = value
            suggest()
        }

    /** What the model last put in the field. While the field still holds it, nothing was typed. */
    private var shownText = ""

    private var editing by mutableStateOf(typing != null)

    /** A picture of a field being typed in: its field never had the keyboard, and cannot lose it. */
    private val pictured = typing != null

    /** The address field has the keyboard: what the user types is not overwritten. */
    var isEditingAddress: Boolean
        get() = editing
        set(value) {
            if (pictured) return
            editing = value
            show()
            if (!value) putSuggestionsAway()
        }

    /** The user has changed what the field was given. */
    val hasTypedAddress: Boolean get() = addressText != shownText

    /**
     * What the mark beside the address field says. It is the page's while the field shows the
     * page's address; once something else has been typed there, the mark says nothing: a
     * padlock must not vouch for what a person typed.
     */
    val fieldSecurity: AddressView.Security
        get() = if (hasTypedAddress) AddressView.Security.SECURITY_UNSPECIFIED else address.security

    private var window: OpenedWindow? = null
    private var opening = false

    /** What the address field is told to show. Never null here: an empty tab has an empty one. */
    val address: AddressView get() = tab.address ?: AddressView()

    /** Opens the window in the core. Call once, when the window appears. */
    suspend fun open() {
        if (window != null || opening) return
        opening = true
        try {
            window = session.openWindow(::apply)
        } catch (_: CoreError) {
            // The core would not open a window: the tab stays empty.
        } finally {
            opening = false
        }
    }

    fun close() {
        val closing = window ?: return
        window = null
        session.closeWindow(closing)
    }

    /**
     * The tab the window's controls act on: the one in front. Until the first snapshot says
     * which that is, it is the tab the window opened with.
     */
    private val frontTabId: Long? get() = tab.tab_id.takeIf { it != 0L } ?: window?.tabId

    /** Sends the address field's text to the core, which decides what it is. */
    fun submit() {
        val front = frontTabId ?: return
        val text = addressText
        isEditingAddress = false
        session.send(Request(submit_address = SubmitAddress(tab_id = front, text = text)))
    }

    fun perform(command: NavigationCommand) {
        val front = frontTabId ?: return
        session.send(Request(tab_command = TabCommand(tab_id = front, command = command)))
    }

    /** The window's id in the core, once it has opened: what its history is asked by (008). */
    val windowId: Long? get() = window?.windowId

    /**
     * Sends the tab in front to an address the app has, as if it had been typed: a visit chosen
     * from history.
     */
    fun load(address: String) {
        val front = frontTabId ?: return
        isEditingAddress = false
        session.send(Request(submit_address = SubmitAddress(tab_id = front, text = address)))
    }

    /** Opens an empty tab, in front, at the end of the window's tabs. */
    fun openTab() {
        val open = window ?: return
        session.send(Request(open_tab = OpenTab(window_id = open.windowId)))
    }

    /** Closes a tab. The core says which tab comes forward, and never leaves the window empty. */
    fun closeTab(id: Long) {
        session.send(Request(close_tab = CloseTab(tab_id = id)))
    }

    /** Brings a tab in front. */
    fun activate(id: Long) {
        if (id == tab.tab_id) return
        session.send(Request(activate_tab = ActivateTab(tab_id = id)))
    }

    val isLoading: Boolean get() = tab.load_state == LoadState.LOAD_STATE_LOADING

    val hasFailed: Boolean get() = tab.load_state == LoadState.LOAD_STATE_FAILED

    val hasPage: Boolean get() = tab.web_view_id != 0L

    // Places offered as the address field is typed in (011)

    /**
     * The places the core offers for what is typed, best first. The model keeps none of its own:
     * each is the core's answer to what is now in the field.
     */
    var suggestions by mutableStateOf(offered)
        private set

    /** Counts what was asked: an answer to anything but the last question is out of date. */
    private var suggestionsAsked = 0

    /** Asks what to offer for what is in the field: only while it is being typed in. */
    private fun suggest() {
        val open = window
        if (!editing || !hasTypedAddress || addressText.isEmpty() || open == null) {
            putSuggestionsAway()
            return
        }
        suggestionsAsked += 1
        val question = suggestionsAsked
        val ask = SuggestAddresses(window_id = open.windowId, text = addressText)
        session.scope.launch {
            val answer =
                try {
                    session.core.request(Request(suggest_addresses = ask)).address_suggestions
                } catch (_: CoreError) {
                    null
                } ?: return@launch
            if (question == suggestionsAsked) suggestions = answer.suggestions
        }
    }

    /** Puts the list away: nothing is offered, and an answer still on its way is for nothing. */
    fun putSuggestionsAway() {
        suggestionsAsked += 1
        if (suggestions.isNotEmpty()) suggestions = emptyList()
    }

    /** Goes to a place that was offered, in the tab in front. */
    fun choose(suggestion: AddressSuggestion) = load(suggestion.url)

    // Find on the page (010)

    /**
     * The find bar is open. A model made with `finding` starts with its bar open on that text,
     * before anything has been asked of the core: for a picture of the bar (004).
     */
    var isFinding by mutableStateOf(finding != null)
        private set

    /** What is typed in the find bar. */
    var findText by mutableStateOf(finding.orEmpty())
        private set

    /**
     * Whether the find bar takes the keyboard when it opens. Not when the model was made with
     * its bar open: that is a picture of a bar, and a picture has no caret in it.
     */
    val findBarTakesKeyboard: Boolean = finding == null

    /** The tab the find is in: where it must be stopped when the bar closes or another tab comes in front. */
    private var findingTabId: Long? = null

    /** What the find bar says of what was found. */
    sealed interface FindOutcome {
        /** Nothing to say: nothing looked for, nothing known yet, or found by an engine that does not count. */
        data object Nothing : FindOutcome

        data object NotFound : FindOutcome

        /** Which match is shown, and how many there are. */
        data class Match(val current: Int, val of: Int) : FindOutcome
    }

    /** What the core says was found of what is now in the bar. The model keeps no count of its own. */
    val findOutcome: FindOutcome
        get() {
            val find = tab.find
            if (!isFinding || find == null || !find.answered || find.text != findText) return FindOutcome.Nothing
            if (!find.found) return FindOutcome.NotFound
            return if (find.counted) FindOutcome.Match(find.current, find.matches) else FindOutcome.Nothing
        }

    /** Opens the find bar. There is nothing to find in a tab with no page. */
    fun openFind() {
        if (hasPage) isFinding = true
    }

    /** Closes the find bar: the find stops, and its marks go. */
    fun closeFind() {
        if (!isFinding) return
        isFinding = false
        findText = ""
        stopFinding()
    }

    /** Looks for other text, at once. The same text again asks nothing: that is what next is for. */
    fun find(text: String) {
        if (text == findText) return
        findText = text
        if (isFinding) look(backwards = false)
    }

    fun findNext() = look(backwards = false)

    fun findPrevious() = look(backwards = true)

    private fun look(backwards: Boolean) {
        val front = frontTabId ?: return
        if (!isFinding || !hasPage) return
        // Empty text is "stop": the tab then has no find to stop later.
        findingTabId = if (findText.isEmpty()) null else front
        session.send(Request(find_in_page = FindInPage(tab_id = front, text = findText, backwards = backwards)))
    }

    private fun stopFinding() {
        val finding = findingTabId ?: return
        findingTabId = null
        session.send(Request(find_in_page = FindInPage(tab_id = finding)))
    }

    private fun apply(snapshot: WindowSnapshot) {
        val front = snapshot.tab ?: TabView()
        // Another tab came in front: what was typed was typed for the tab that was there. (The
        // window's first snapshot names its first tab, and is no switch: what was typed before
        // it arrived stays.)
        val switched = tab.tab_id != 0L && front.tab_id != tab.tab_id
        if (switched) {
            // The find was in the tab that was in front: it stops there, and the bar closes.
            closeFind()
            // What was offered was offered for what was typed for that tab.
            putSuggestionsAway()
        }
        tab = front
        tabs = snapshot.tabs
        // A field with the keyboard still follows the page until something is typed in it: one
        // that took the focus by itself, at launch, must not go on showing what was there then.
        if (switched || !editing || !hasTypedAddress) show()
    }

    /** Puts the page's address in the field: whole while it is edited, the short form at rest. */
    private fun show() {
        // A picture's field holds what the picture was made with.
        if (pictured) return
        shownText = if (editing && address.url.isNotEmpty()) address.url else address.display
        addressText = shownText
    }
}

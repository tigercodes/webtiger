// A window's model: what the core last said about it, and the user's intents sent back.
package com.tigercodes.webtiger.ui

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.tigercodes.webtiger.core.CoreError
import com.tigercodes.webtiger.core.v1.ActivateTab
import com.tigercodes.webtiger.core.v1.AddressSuggestion
import com.tigercodes.webtiger.core.v1.AddressView
import com.tigercodes.webtiger.core.v1.AnswerPrompt
import com.tigercodes.webtiger.core.v1.ChooseSearchEngine
import com.tigercodes.webtiger.core.v1.CloseTab
import com.tigercodes.webtiger.core.v1.CreateSpace
import com.tigercodes.webtiger.core.v1.DialogKind
import com.tigercodes.webtiger.core.v1.FindInPage
import com.tigercodes.webtiger.core.v1.LoadState
import com.tigercodes.webtiger.core.v1.NavigationCommand
import com.tigercodes.webtiger.core.v1.OpenTab
import com.tigercodes.webtiger.core.v1.PromptView
import com.tigercodes.webtiger.core.v1.QuerySearch
import com.tigercodes.webtiger.core.v1.QuerySpaces
import com.tigercodes.webtiger.core.v1.RenameSpace
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.SearchSettings
import com.tigercodes.webtiger.core.v1.SpaceSummary
import com.tigercodes.webtiger.core.v1.SubmitAddress
import com.tigercodes.webtiger.core.v1.SuggestAddresses
import com.tigercodes.webtiger.core.v1.SwitchSpace
import com.tigercodes.webtiger.core.v1.TabCommand
import com.tigercodes.webtiger.core.v1.TabSummary
import com.tigercodes.webtiger.core.v1.TabView
import com.tigercodes.webtiger.core.v1.WindowSnapshot
import com.tigercodes.webtiger.core.v1.ZoomChange
import com.tigercodes.webtiger.core.v1.ZoomPage
import com.tigercodes.webtiger.core.v1.ZoomView
import com.tigercodes.webtiger.webview.Pressed
import kotlinx.coroutines.launch

/**
 * One window's state, for Compose to draw. Used on the main thread.
 *
 * A model made with `finding`, with `typing` and what is `offered` for it, or `zooming`, is a
 * picture of a window (004): its find bar is open on that text, its address field holds it with
 * those places listed, or its zoom bar is open, before anything has been asked of the core.
 */
class BrowserModel(
    private val session: BrowserSession,
    finding: String? = null,
    typing: String? = null,
    offered: List<AddressSuggestion> = emptyList(),
    zooming: Boolean = false,
    pressing: Pressed? = null,
    searching: SearchSettings? = null,
    changingSearch: Boolean = false,
) {
    /** The tab in front, as the core last published it. The window's controls act on it. */
    var tab by mutableStateOf(TabView())
        private set

    /** Every tab of the window, in the order they are shown. */
    var tabs by mutableStateOf<List<TabSummary>>(emptyList())
        private set

    /** How many downloads of the window's Space are coming (013): the core's count. */
    var downloadsActive by mutableStateOf(0)
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
        // From now this window is told when a link of a page is pressed and held (018).
        session.engine.onPressed = ::pressed
        try {
            window = session.openWindow(::apply)
            // Where searches go, and whether anybody chose it (020): asked once, when the
            // window has opened.
            askWhereSearchesGo()
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

    // Zoom (012)

    /** The zoom bar is open. */
    var isZooming by mutableStateOf(zooming)
        private set

    /**
     * What the page of the tab in front is zoomed to: the core's word for it, with whether there
     * is a level above and one below. Null while there is no page that can be zoomed.
     */
    val zoom: ZoomView? get() = tab.zoom

    /**
     * Opens the zoom bar, in place of the find bar: one bar at a time. There is nothing to zoom
     * without a page.
     */
    fun openZoom() {
        if (zoom == null) return
        closeFind()
        isZooming = true
    }

    fun closeZoom() {
        isZooming = false
    }

    /**
     * Asks the core to make the page of the tab in front bigger or smaller, or to put it back.
     * Which level that is, and which other tabs have it, is the core's.
     */
    fun zoom(change: ZoomChange) {
        val front = frontTabId ?: return
        if (zoom == null) return
        session.send(Request(zoom_page = ZoomPage(tab_id = front, change = change)))
    }

    // What the tab asks the person (014)

    /**
     * What the tab in front asks: the core's, in the core's words for who asks. Null when it
     * asks nothing; and for a prompt this build has no words for, which is not put before a
     * person as two buttons and no question: its page waits.
     */
    val prompt: PromptView? get() = tab.prompt?.takeIf { asked -> asked.download != null || asked.permission?.kinds.orEmpty().isNotEmpty() || asked.dialog?.kind in SPOKEN || asked.sign_in != null }

    /**
     * The person's answer to the prompt the tab shows. What it lets the page do, what is
     * remembered, and whether it came too soon after the prompt appeared to be the person's
     * answer to it, are the core's: a refusal leaves the prompt where it is. For what a page
     * says or asks (016), `text` is what a prompt's field held, and `silence` stops the page's
     * messages. For a server that asks who is there (017), `text` is the name and `secret` the
     * password: they are sent, and kept nowhere here.
     */
    fun answerPrompt(allow: Boolean, text: String = "", silence: Boolean = false, secret: String = "") {
        val front = frontTabId ?: return
        val shown = prompt ?: return
        session.send(Request(answer_prompt = AnswerPrompt(tab_id = front, prompt_id = shown.prompt_id, allow = allow, text = text, silence = silence, secret = secret)))
    }

    // Where searches go (020)

    /**
     * What the core says of where searches go: the engines, the one in use, and whether anybody
     * chose it. Null until the core has said. A model made with `searching` starts with that:
     * for a picture (004).
     */
    var search by mutableStateOf(searching)
        private set

    /** The list is open to change where searches go: an empty tab's "Change…" was pressed. */
    var isChangingSearch by mutableStateOf(changingSearch)
        private set

    /**
     * The list of engines is before the person: because nobody has chosen yet (the first run's
     * question, which stays until one is chosen), or because they asked to change.
     */
    val asksWhereSearchesGo: Boolean get() = search.let { it != null && (!it.chosen || isChangingSearch) }

    /** The name of the engine searches go to now. Null until the core has said. */
    val searchEngineName: String? get() = search?.let { settings -> settings.engines.firstOrNull { it.id == settings.chosen_id }?.name }

    /** Asks the core where searches go. Its answer is what the window then shows. */
    private fun askWhereSearchesGo() {
        session.scope.launch {
            val answer =
                try {
                    session.core.request(Request(query_search = QuerySearch())).search_settings
                } catch (_: CoreError) {
                    null
                }
            if (answer != null) search = answer
        }
    }

    /** The person's choice of an engine. The list goes when the core has taken it; a choice it refuses leaves the list as it is. */
    fun chooseSearchEngine(id: String) {
        session.scope.launch {
            try {
                session.core.request(Request(choose_search_engine = ChooseSearchEngine(id = id)))
            } catch (_: CoreError) {
                return@launch
            }
            isChangingSearch = false
            // The core's own word on what is now in use, and by whose choice.
            askWhereSearchesGo()
        }
    }

    /** An empty tab's "Change…". */
    fun changeSearch() {
        isChangingSearch = true
    }

    /** The list is closed with nothing changed. */
    fun keepSearch() {
        isChangingSearch = false
    }

    // Several Spaces (024 §6)

    /** The Space the window shows, by the core's name for it. 0 until the core has said. */
    var spaceId by mutableStateOf(0L)
        private set

    /** That Space's name. Empty for the Space the app made, until somebody names it. */
    var spaceName by mutableStateOf("")
        private set

    /** The Spaces as the core last listed them: asked for when the list is opened. Null until it has answered. */
    var spaces by mutableStateOf<List<SpaceSummary>?>(null)
        private set

    /** The list of Spaces is open over the window. */
    var showingSpaces by mutableStateOf(false)
        private set

    /** Why the core would not make a Space, by its code: said on the page that asked. */
    var spaceRefusal by mutableStateOf<String?>(null)
        private set

    /**
     * Names the Space the window shows (024 §6.3), and closes the list. The name goes as it was
     * typed. A name the core will not take leaves the list open, with the core's code in
     * [spaceRefusal]; the chip says the new name when the core's snapshot does.
     */
    fun renameSpace(name: String) {
        val shown = spaceId.takeIf { it != 0L } ?: return
        spaceRefusal = null
        session.scope.launch {
            try {
                session.core.request(Request(rename_space = RenameSpace(space_id = shown, name = name)))
            } catch (refused: CoreError.Refused) {
                spaceRefusal = refused.code
                return@launch
            } catch (_: CoreError) {
                spaceRefusal = "space.rename_failed"
                return@launch
            }
            showingSpaces = false
        }
    }

    /** Opens the list of Spaces: asked for anew, with nothing said of an earlier refusal. */
    fun openSpaces() {
        spaceRefusal = null
        showingSpaces = true
        session.scope.launch {
            val answer =
                try {
                    session.core.request(Request(query_spaces = QuerySpaces())).spaces_page
                } catch (_: CoreError) {
                    null
                }
            if (answer != null) spaces = answer.spaces
        }
    }

    /** The list is closed with nothing changed. */
    fun closeSpaces() {
        showingSpaces = false
    }

    /** The window is to show another Space, and the list is closed. The core's snapshot says when it shows it. */
    fun showSpace(id: Long) {
        val window = windowId ?: return
        session.send(Request(switch_space = SwitchSpace(window_id = window, space_id = id)))
        showingSpaces = false
    }

    /**
     * Makes a Space, with logins of its own or those of the Space `sharing`, and shows it in the
     * window. The name goes as it was typed: tidying it is the core's. A Space the core will not
     * make leaves the list open, with the core's code in [spaceRefusal].
     */
    fun makeSpace(name: String, sharing: Long?) {
        spaceRefusal = null
        session.scope.launch {
            val made =
                try {
                    session.core.request(Request(create_space = CreateSpace(name = name, share_logins_with = sharing ?: 0))).space_created
                } catch (refused: CoreError.Refused) {
                    spaceRefusal = refused.code
                    return@launch
                } catch (_: CoreError) {
                    null
                }
            if (made == null) {
                spaceRefusal = "space.failed"
                return@launch
            }
            showSpace(made.space_id)
        }
    }

    // A link's menu (018)

    /**
     * What the open menu is of: a link or a picture the person pressed and held in the page in
     * front (018, 022). Null when no menu is. A model made with `pressing` starts with the
     * menu of that open: for a picture of it (004).
     */
    internal var pressedLink by mutableStateOf(pressing?.let { PressedLink(webViewId = PICTURED, url = it.link, picture = it.picture) })
        private set

    /** The engine says a link or a picture of a page was pressed and held. Only of the page in front: a page behind is not being touched. */
    private fun pressed(webViewId: Long, pressed: Pressed) {
        if (webViewId != 0L && webViewId == tab.web_view_id) pressedLink = PressedLink(webViewId, pressed.link, pressed.picture)
    }

    /** What each choice in the menu does. */
    internal val linkChoices =
        LinkChoices(
            open = ::openPressedLink,
            download = ::downloadPressedLink,
            copy = ::copyPressedLink,
            openPicture = ::openPressedPicture,
            downloadPicture = ::downloadPressedPicture,
            dismiss = ::dismissLinkMenu,
        )

    /** "Download Linked File" (022): what the link leads to is offered to the core, as a download the person asked for. */
    fun downloadPressedLink() {
        val link = pressedLink ?: return
        pressedLink = null
        session.engine.saveFile(link.webViewId, link.url)
    }

    /** "Open Image in New Tab" (022): as a link is, behind this tab; whether a tab opens is the core's to say. */
    fun openPressedPicture() {
        val link = pressedLink ?: return
        pressedLink = null
        session.engine.openLink(link.webViewId, link.picture, background = true)
    }

    /** "Download Image" (022). */
    fun downloadPressedPicture() {
        val link = pressedLink ?: return
        pressedLink = null
        session.engine.saveFile(link.webViewId, link.picture)
    }

    /** "Open in New Tab": the core is asked, through the engine that saw the press, and opens the tab behind this one, or does not. */
    fun openPressedLink() {
        val link = pressedLink ?: return
        pressedLink = null
        session.engine.openLink(link.webViewId, link.url, background = true)
    }

    /** "Copy Link". */
    fun copyPressedLink() {
        val link = pressedLink ?: return
        pressedLink = null
        session.clipboard.copy(link.url)
    }

    fun dismissLinkMenu() {
        pressedLink = null
    }

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

    /**
     * Opens the find bar, in place of the zoom bar: one bar at a time. There is nothing to find
     * in a tab with no page.
     */
    fun openFind() {
        if (!hasPage) return
        isZooming = false
        isFinding = true
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
        // A link's menu is of the page it was pressed in: with another web view in front, or
        // none, it is put away. (A picture's menu stays in its picture.)
        if (pressedLink?.let { it.webViewId != PICTURED && it.webViewId != front.web_view_id } == true) pressedLink = null
        tab = front
        tabs = snapshot.tabs
        downloadsActive = snapshot.downloads_active
        spaceId = snapshot.space_id
        spaceName = snapshot.space_name
        // The zoom bar is the window's, and says the level of the page in front: with no page
        // there that can be zoomed, it has nothing to say.
        if (isZooming && front.zoom == null) isZooming = false
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

/** The web view a picture's link menu is said to be of: none of the core's, whose ids count up from one, and none of a shell's. */
private const val PICTURED = -1L

/** What a page may say or ask that this build has a bar for (016): a later contract's kind is not shown. */
private val SPOKEN = setOf(DialogKind.DIALOG_KIND_ALERT, DialogKind.DIALOG_KIND_CONFIRM, DialogKind.DIALOG_KIND_PROMPT)

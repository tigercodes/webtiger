// A window's model: what the core last said about it, and the user's intents sent back.
package com.tigercodes.webtiger.ui

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.graphics.ImageBitmap
import com.tigercodes.webtiger.core.CoreError
import com.tigercodes.webtiger.core.v1.ActivateTab
import com.tigercodes.webtiger.core.v1.AddressSuggestion
import com.tigercodes.webtiger.core.v1.AddressView
import com.tigercodes.webtiger.core.v1.AnswerPrompt
import com.tigercodes.webtiger.core.v1.ChooseSearchEngine
import com.tigercodes.webtiger.core.v1.CloseTab
import com.tigercodes.webtiger.core.v1.ColourSpace
import com.tigercodes.webtiger.core.v1.CreateSpace
import com.tigercodes.webtiger.core.v1.DialogKind
import com.tigercodes.webtiger.core.v1.FindInPage
import com.tigercodes.webtiger.core.v1.ColourGroup
import com.tigercodes.webtiger.core.v1.GroupSummary
import com.tigercodes.webtiger.core.v1.KeepTab
import com.tigercodes.webtiger.core.v1.Layout
import com.tigercodes.webtiger.core.v1.LetGoSite
import com.tigercodes.webtiger.core.v1.MakeGroup
import com.tigercodes.webtiger.core.v1.MoveToGroup
import com.tigercodes.webtiger.core.v1.RemoveGroup
import com.tigercodes.webtiger.core.v1.RenameGroup
import com.tigercodes.webtiger.core.v1.ShutGroup
import com.tigercodes.webtiger.core.v1.LoadState
import com.tigercodes.webtiger.core.v1.NavigationCommand
import com.tigercodes.webtiger.core.v1.OpenAddress
import com.tigercodes.webtiger.core.v1.OpenTab
import com.tigercodes.webtiger.core.v1.ProceedPastCertificate
import com.tigercodes.webtiger.core.v1.PromptView
import com.tigercodes.webtiger.core.v1.QuerySearch
import com.tigercodes.webtiger.core.v1.QuerySpaces
import com.tigercodes.webtiger.core.v1.RenameSpace
import com.tigercodes.webtiger.core.v1.MoveTab
import com.tigercodes.webtiger.core.v1.QueryIcons
import com.tigercodes.webtiger.core.v1.Request
import com.tigercodes.webtiger.core.v1.RowStep
import com.tigercodes.webtiger.core.v1.SearchSettings
import com.tigercodes.webtiger.core.v1.SearchWords
import com.tigercodes.webtiger.core.v1.SendSiteHome
import com.tigercodes.webtiger.core.v1.SetFavourite
import com.tigercodes.webtiger.core.v1.SetLayout
import com.tigercodes.webtiger.core.v1.SpaceSummary
import com.tigercodes.webtiger.core.v1.SubmitAddress
import com.tigercodes.webtiger.core.v1.SuggestAddresses
import com.tigercodes.webtiger.core.v1.SwitchSpace
import com.tigercodes.webtiger.core.v1.TabCommand
import com.tigercodes.webtiger.core.v1.TabSummary
import com.tigercodes.webtiger.core.v1.StepTab
import com.tigercodes.webtiger.core.v1.TabView
import com.tigercodes.webtiger.core.v1.Undo
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
 * those places listed, or its zoom bar is open, before anything has been asked of the core. One
 * made `warned` is the picture of the warning before going on past a certificate (026).
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
    warned: Boolean = false,
    /** The window is a private one (037): the core is asked to open it so. */
    private val asPrivate: Boolean = false,
) {
    /**
     * The window is a private one (037): asked for as one, or said to be one by the core.
     * Nothing it visits is kept, and it says so in words.
     */
    var isPrivate by mutableStateOf(asPrivate)
        private set

    /** The tab in front, as the core last published it. The window's controls act on it. */
    var tab by mutableStateOf(TabView())
        private set

    /** Every tab of the window, in the order they are shown. */
    var tabs by mutableStateOf<List<TabSummary>>(emptyList())
        private set

    /** The Groups of the row the window shows (041), in their order. */
    var groups by mutableStateOf<List<GroupSummary>>(emptyList())
        private set

    /** The row has as many Favourites as it can (042): the core says so, and no more is offered. */
    var favouritesFull by mutableStateOf(false)
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
            window = session.openWindow(private = asPrivate, update = ::apply)
            // Where searches go, and whether anybody chose it (020): asked once, when the
            // window has opened.
            askWhereSearchesGo()
            // Words another app handed over before there was a window to search in (028).
            wordsWaiting.also { wordsWaiting = null }?.let(::searchFor)
            addressesWaiting.toList().also { addressesWaiting.clear() }.forEach(::openFromOutside)
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

    /** Words handed over while the window was not open yet: searched for when it is (028). */
    private var wordsWaiting: String? = null

    /**
     * Another app handed the app words to search for (028). The core is told, with the window
     * they are for: what they are, and where the search goes, is the core's. Nothing is made
     * of them here.
     */
    fun searchFor(words: String) {
        val opened = window
        if (opened == null) {
            // The last words asked for are the ones searched for.
            wordsWaiting = words
            return
        }
        session.send(Request(search_words = SearchWords(window_id = opened.windowId, text = words)))
    }

    /** Addresses handed over while the window was not open yet: opened when it is, in order (031). */
    private val addressesWaiting = mutableListOf<String>()

    /**
     * The system handed the app an address from another app (031): a link that was pressed
     * there. The core is told, with the window it is for, and the address as it was given:
     * whether it is one to open, and where it opens, is the core's. Nothing is made of it here.
     */
    fun openFromOutside(address: String) {
        val opened = window
        if (opened == null) {
            // Each of them is a link somebody pressed: none is dropped for a later one.
            addressesWaiting += address
            return
        }
        session.send(Request(open_address = OpenAddress(window_id = opened.windowId, url = address)))
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

    /**
     * Asks the core to bring another tab of the window in front, by where it is in the row
     * (035): the next, the one before, the last, or the tab at `place`, counted from one.
     * Which tab that is, is the core's to say.
     */
    fun stepTab(to: StepTab.To, place: Int = 0) {
        val open = window ?: return
        session.send(Request(step_tab = StepTab(window_id = open.windowId, to = to, place = place)))
    }

    /**
     * Brings a tab in front. `chosenWhileTyping`: it was chosen from what was offered while
     * typing in the tab in front (038); the core then closes that tab when it is an empty one.
     */
    fun activate(id: Long, chosenWhileTyping: Boolean = false) {
        if (id == tab.tab_id) return
        session.send(Request(activate_tab = ActivateTab(tab_id = id, chosen_while_typing = chosenWhileTyping)))
    }

    val isLoading: Boolean get() = tab.load_state == LoadState.LOAD_STATE_LOADING

    val hasFailed: Boolean get() = tab.load_state == LoadState.LOAD_STATE_FAILED

    val hasPage: Boolean get() = tab.web_view_id != 0L

    // Sites (039)

    /** The tab in front can be kept, or is a Site that can be let go: the one command has something to do. */
    val canKeepOrLetGo: Boolean get() = tab.kept || hasPage

    /**
     * Keeps a tab as a Site, or lets it go when it is one (039). What either means, and where
     * the tab then sits, is the core's.
     */
    fun keepOrLetGo(id: Long) {
        val kept = if (id == tab.tab_id) tab.kept else tabs.firstOrNull { it.tab_id == id }?.kept == true
        session.send(if (kept) Request(let_go_site = LetGoSite(tab_id = id)) else Request(keep_tab = KeepTab(tab_id = id)))
    }

    /** The same for the tab in front: what the key asks. */
    fun keepOrLetGoTheTabInFront() {
        val front = frontTabId ?: return
        if (canKeepOrLetGo) keepOrLetGo(front)
    }

    /**
     * Sends a Site back to its home (039). Whether the tab is one that is away is the core's
     * to say: for any other it does nothing.
     */
    fun sendHome(id: Long) {
        session.send(Request(send_site_home = SendSiteHome(tab_id = id)))
    }

    /** The same for the tab in front, when it is a Site that is away: what the key asks. */
    fun sendHomeTheTabInFront() {
        val front = frontTabId ?: return
        if (tab.away) sendHome(front)
    }

    // The layout (043)

    /** How the window lays out its tabs: the core's word. */
    var layout by mutableStateOf(Layout.LAYOUT_STRIP)
        private set

    /**
     * The window is wide enough for a sidebar beside a page: the window says so, by its width
     * and never by the device. A phone's is not.
     */
    var hasRoomForSidebar by mutableStateOf(false)

    /**
     * The window draws its tabs down the side: the core says the sidebar, and the window has
     * room for one. Where it has none, the strip is drawn whatever the core says.
     */
    val showsSidebar: Boolean get() = hasRoomForSidebar && layout == Layout.LAYOUT_SIDEBAR

    /**
     * Switches the window's layout: to the other of what the core last said it is. Not where
     * there is no room for a sidebar: there is nothing to switch to.
     */
    fun switchLayout() {
        if (!hasRoomForSidebar) return
        val open = windowId ?: return
        val other = if (layout == Layout.LAYOUT_SIDEBAR) Layout.LAYOUT_STRIP else Layout.LAYOUT_SIDEBAR
        session.send(Request(set_layout = SetLayout(window_id = open, layout = other)))
    }

    // Favourites (042)

    /**
     * A tab can be made a Favourite, or is one that can stop being one: the command has
     * something to do. A tab with nothing to keep, a private window's tab, and any tab while the
     * core says the row is full, cannot be made one.
     */
    fun canFavourite(tab: TabSummary): Boolean = tab.favourite || (canBeGrouped(tab) && !favouritesFull)

    /** Marks a tab as a Favourite, or unmarks it. A tab that is no Site is kept by it: the core's rule. */
    fun setFavourite(id: Long, favourite: Boolean) {
        session.send(Request(set_favourite = SetFavourite(tab_id = id, favourite = favourite)))
    }

    // Icons (046)

    /** The pictures of the tabs' icons, by the numbers the core gives them, as far as this window has them. */
    var icons by mutableStateOf<Map<Long, ImageBitmap>>(emptyMap())
        private set

    /** The numbers asked for already: each is asked for once, whatever is answered. */
    private val iconsAsked = HashSet<Long>()

    /**
     * Asks the core for the pictures of the tabs' icons that this window has not got: once for
     * each number, and no more to a question than an answer carries. A window that has kept
     * many keeps only those its tabs show now.
     */
    private fun askForIcons(tabs: List<TabSummary>) {
        val shown = tabs.mapTo(HashSet()) { it.icon_id }
        if (icons.size > ICONS_KEPT) {
            icons = icons.filterKeys { it in shown }
            iconsAsked.retainAll(shown)
        }
        val wanted = shown.filter { it != 0L && iconsAsked.add(it) }.sorted()
        for (numbers in wanted.chunked(ICONS_ASKED)) {
            session.scope.launch {
                val answer =
                    try {
                        session.core.request(Request(query_icons = QueryIcons(icon_ids = numbers))).icons_page
                    } catch (_: CoreError) {
                        null
                    }
                val pictures = answer?.icons.orEmpty().mapNotNull { icon -> iconPicture(icon.png.toByteArray())?.let { icon.icon_id to it } }
                if (pictures.isNotEmpty()) icons = icons + pictures
            }
        }
    }

    // Moving in the row (045)

    /** Moves a tab one place among the tabs of its part of the row: how far it can go is the core's. */
    fun moveTab(id: Long, step: RowStep) {
        session.send(Request(move_tab = MoveTab(tab_id = id, by_ = step)))
    }

    // Groups (041)

    /** What the Group page is open for: a new Group for a tab, or a Group there is. Null while it is not open. */
    var groupAsked by mutableStateOf<GroupAsk?>(null)
        private set

    /** Why the core would not make a Group, or change one, by its code: said on that page. */
    var groupRefusal by mutableStateOf<String?>(null)
        private set

    /**
     * A tab can be put in a Group: it has something to keep, or is kept already, and the window
     * is not a private one, which has no Groups (037).
     */
    fun canBeGrouped(tab: TabSummary): Boolean = !isPrivate && (tab.kept || tab.host.isNotEmpty())

    /** Shuts a Group, or opens it: the other of what the core last said of it. */
    fun shutOrOpen(group: GroupSummary) {
        session.send(Request(shut_group = ShutGroup(group_id = group.group_id, shut = !group.shut)))
    }

    /**
     * Puts a tab in a Group; with no Group (0), takes a Site out of the one it is in. A tab that
     * is no Site is kept by it: the core's rule.
     */
    fun moveToGroup(tab: Long, group: Long) {
        session.send(Request(move_to_group = MoveToGroup(tab_id = tab, group_id = group)))
    }

    /** Takes a Group away. Its Sites stay: where is the core's to say. */
    fun removeGroup(id: Long) {
        session.send(Request(remove_group = RemoveGroup(group_id = id)))
    }

    /** Opens the page that makes a Group, for a tab to be put in it. */
    fun askForANewGroup(tab: Long) {
        openGroupPage(GroupAsk.New(tab))
    }

    /** Opens the page that edits a Group. */
    fun askToEdit(group: GroupSummary) {
        openGroupPage(GroupAsk.Edit(group))
    }

    /** The page is closed with nothing made or changed. */
    fun closeGroupPage() {
        groupAsked = null
    }

    private fun openGroupPage(ask: GroupAsk) {
        groupRefusal = null
        groupAsked = ask
        // The colours to choose among are the core's, which lists them with the Spaces (025).
        if (spaceColours.isEmpty()) askForSpaces()
    }

    /**
     * What the Group page asked for, when its button was pressed: a new Group with that name
     * and colour, and the tab in it; or the name and the colour of the Group it edits, each
     * when it is another, the name first. The name goes as it was typed: tidying it is the
     * core's. The page closes when the core has done it, and stays, with the core's code in
     * [groupRefusal], when it would not.
     */
    fun finishGroupPage(name: String, colour: String) {
        val asked = groupAsked ?: return
        groupRefusal = null
        session.scope.launch {
            try {
                when (asked) {
                    is GroupAsk.New -> {
                        val window = windowId ?: return@launch
                        val made = session.core.request(Request(make_group = MakeGroup(window_id = window, name = name))).group_made
                        if (made == null) {
                            groupRefusal = "group.failed"
                            return@launch
                        }
                        if (colour.isNotEmpty()) session.send(Request(colour_group = ColourGroup(group_id = made.group_id, colour = colour)))
                        moveToGroup(asked.tab, made.group_id)
                    }
                    is GroupAsk.Edit -> {
                        val id = asked.group.group_id
                        if (name != asked.group.name) session.core.request(Request(rename_group = RenameGroup(group_id = id, name = name)))
                        if (colour != asked.group.colour) session.core.request(Request(colour_group = ColourGroup(group_id = id, colour = colour)))
                    }
                }
            } catch (refused: CoreError.Refused) {
                groupRefusal = refused.code
                return@launch
            } catch (_: CoreError) {
                groupRefusal = if (asked is GroupAsk.New) "group.failed" else "group.change_failed"
                return@launch
            }
            if (groupAsked == asked) groupAsked = null
        }
    }

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

    /**
     * Goes to what was offered. A place is loaded in the tab in front. A tab that is open (038)
     * is brought in front and nothing is loaded; the core is told it was chosen while typing,
     * and closes the tab typed in when that was an empty one.
     */
    fun choose(suggestion: AddressSuggestion) {
        if (suggestion.tab_id == 0L) {
            load(suggestion.url)
            return
        }
        isEditingAddress = false
        activate(suggestion.tab_id, chosenWhileTyping = true)
    }

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

    /** The first run's list was closed with the default kept: it is not shown again while the core takes the choice down. */
    private var hasKeptDefaultSearch by mutableStateOf(false)

    /**
     * The list of engines is before the person: because nobody has chosen yet (the first run's
     * choice, offered once with the default to keep), or because they asked to change.
     */
    val asksWhereSearchesGo: Boolean get() = search.let { it != null && ((!it.chosen && !hasKeptDefaultSearch) || isChangingSearch) }

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

    /**
     * The list is closed with nothing changed. At a first run, when nobody has chosen, that is
     * keeping the default: it is chosen, so that the choice is not offered again.
     */
    fun keepSearch() {
        search?.takeIf { !it.chosen }?.let {
            hasKeptDefaultSearch = true
            chooseSearchEngine(it.chosen_id)
        }
        isChangingSearch = false
    }

    // Going on past a certificate (026 §6)

    /** The load in front failed for its site's certificate, and the core says it may be gone on with. */
    val canGoOnPastCertificate: Boolean get() = hasFailed && tab.certificate_passable

    /** The warning is open: "Advanced…" was pressed, for this failure of the tab in front. */
    var isWarnedOfCertificate by mutableStateOf(warned)
        private set

    /** Shows the warning. Nothing goes on by this. */
    fun warnOfCertificate() {
        if (canGoOnPastCertificate) isWarnedOfCertificate = true
    }

    /**
     * The person goes on, having been warned: the core is asked, and what comes of it is the
     * core's to say. Without the warning on the screen, nothing is asked.
     */
    fun goOnPastCertificate() {
        val front = frontTabId ?: return
        if (!canGoOnPastCertificate || !isWarnedOfCertificate) return
        isWarnedOfCertificate = false
        session.send(Request(proceed_past_certificate = ProceedPastCertificate(tab_id = front)))
    }

    // Several Spaces (024 §6)

    /** The Space the window shows, by the core's name for it. 0 until the core has said. */
    var spaceId by mutableStateOf(0L)
        private set

    /** That Space's name. Empty for the Space the app made, until somebody names it. */
    var spaceName by mutableStateOf("")
        private set

    /** That Space's colour, by the core's name for it (025). Empty: none. */
    var spaceColour by mutableStateOf("")
        private set

    /**
     * What an undo would undo now in the Space the window shows (033), by the core's word for
     * it. Empty: there is nothing to undo.
     */
    var undoKind by mutableStateOf("")
        private set

    /** A closed tab can be brought back in the Space the window shows: the core says so. */
    val canReopenClosedTab: Boolean get() = undoKind == UNDO_CLOSED_TAB

    /**
     * Brings back the tab that was last closed in the Space the window shows (033). The core
     * is asked to undo, and says what comes of it; nothing is asked while the window has been
     * told there is nothing to bring back.
     */
    fun reopenClosedTab() {
        val open = window ?: return
        if (!canReopenClosedTab) return
        session.send(Request(undo = Undo(window_id = open.windowId)))
    }

    /** The colours a Space can have, as the core lists them with the Spaces. */
    var spaceColours by mutableStateOf<List<String>>(emptyList())
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
     * Gives the Space the window shows another name, another colour, or both (024 §6.3, 025),
     * and closes the list: what is null stays as it is. The name goes as it was typed. What the
     * core will not take leaves the list open, with the core's code in [spaceRefusal]; the chip
     * says what is new when the core's snapshot does.
     */
    fun editSpace(name: String?, colour: String?) {
        val shown = spaceId.takeIf { it != 0L } ?: return
        spaceRefusal = null
        session.scope.launch {
            try {
                // The name first: a name the core will not take leaves the colour as it was.
                if (name != null) session.core.request(Request(rename_space = RenameSpace(space_id = shown, name = name)))
                if (colour != null) session.core.request(Request(colour_space = ColourSpace(space_id = shown, colour = colour)))
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
        askForSpaces()
    }

    /** Asks the core for the Spaces, and the colours one can have: its answer is what is then shown. */
    private fun askForSpaces() {
        session.scope.launch {
            val answer =
                try {
                    session.core.request(Request(query_spaces = QuerySpaces())).spaces_page
                } catch (_: CoreError) {
                    null
                }
            if (answer != null) {
                spaces = answer.spaces
                spaceColours = answer.colours
            }
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
    fun makeSpace(name: String, sharing: Long?, colour: String = "") {
        spaceRefusal = null
        session.scope.launch {
            val made =
                try {
                    session.core.request(Request(create_space = CreateSpace(name = name, share_logins_with = sharing ?: 0, colour = colour))).space_created
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
        // The warning before going on past a certificate (026) is one failure's: another tab,
        // another load or another site starts again from "Advanced…".
        // (The window's first snapshot is no other failure than the one a picture was made of.)
        val first = tab.tab_id == 0L
        val goesOn = front.load_state == LoadState.LOAD_STATE_FAILED && front.certificate_passable
        val sameFailure = goesOn && (first || (!switched && front.certificate_host == tab.certificate_host))
        if (!sameFailure) isWarnedOfCertificate = false
        tab = front
        tabs = snapshot.tabs
        askForIcons(snapshot.tabs)
        groups = snapshot.groups
        favouritesFull = snapshot.favourites_full
        layout = snapshot.layout
        downloadsActive = snapshot.downloads_active
        spaceId = snapshot.space_id
        spaceName = snapshot.space_name
        spaceColour = snapshot.space_colour
        undoKind = snapshot.undo
        isPrivate = asPrivate || snapshot.private_window
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

/** What the core says an undo would undo when a closed tab can be brought back (033). */
private const val UNDO_CLOSED_TAB = "tab.closed"

/** The web view a picture's link menu is said to be of: none of the core's, whose ids count up from one, and none of a shell's. */
private const val PICTURED = -1L

/** What a page may say or ask that this build has a bar for (016): a later contract's kind is not shown. */
private val SPOKEN = setOf(DialogKind.DIALOG_KIND_ALERT, DialogKind.DIALOG_KIND_CONFIRM, DialogKind.DIALOG_KIND_PROMPT)

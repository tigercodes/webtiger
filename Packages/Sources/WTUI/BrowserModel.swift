public import Observation
public import WTContract

// A window's model: what the core last said about it, and the user's intents sent back.
import CoreGraphics
import Foundation
import WTCoreClient

@MainActor
@Observable
public final class BrowserModel {
    /// The tab in front, as the core last published it. The window's controls act on it.
    public private(set) var tab = WTTabView()
    /// Every tab of the window, in the order they are shown.
    public private(set) var tabs: [WTTabSummary] = []
    /// The Groups of the row the window shows (041), in their order.
    public private(set) var groups: [WTGroupSummary] = []
    /// The row has as many Favourites as it can (042): the core says so, and no more is
    /// offered.
    public private(set) var favouritesFull = false
    /// How many tabs the row has (047): every one, those of shut Groups among them. The
    /// core's count: `tabs` has the ones that are shown.
    public private(set) var rowTabs = 0
    /// How the window lays out its tabs (043): the core's word.
    public private(set) var layout: WTLayout = .strip
    /// The window is wide enough for a sidebar beside a page (043): a Mac's always; an
    /// iPad's when its window is, which the window says by its size class, never by the
    /// device. A phone's is not.
    public var hasRoomForSidebar = BrowserModel.startsWithRoomForSidebar
    /// What the Group page is open for; nothing while it is not open.
    public internal(set) var groupAsked: GroupAsk?
    /// Why the core would not make a Group, or change one, by its code: said on that page.
    public internal(set) var groupRefusal: String?
    /// How many downloads of the window's Space are coming (013): the core's count.
    public private(set) var downloadsActive: UInt32 = 0
    /// The Space the window shows (024), by the core's name for it; and its own name, which
    /// is empty for the Space the app made until somebody names it.
    public private(set) var spaceID: UInt64 = 0
    public private(set) var spaceName = ""
    /// That Space's colour, by the core's name for it (025); empty for none.
    public private(set) var spaceColour = ""
    /// That Space's icon, an emoji of the core's (050); empty for none.
    public private(set) var spaceIcon = ""
    /// The icons a Space can have, as the core lists them with the Spaces.
    public internal(set) var spaceIcons: [String] = []
    /// The colours a Space can have, as the core lists them with the Spaces.
    public internal(set) var spaceColours: [String] = []
    /// The times a Space's tidying can have (049), in seconds, as the core lists them with
    /// the Spaces: never, which is 0, first.
    public internal(set) var spaceTidyTimes: [UInt32] = []
    /// The Spaces as the core last listed them: asked for when the list is opened. `nil`
    /// until it has answered.
    public internal(set) var spaces: [WTSpaceSummary]?
    /// What an undo would undo now in the Space the window shows (033), by the core's word
    /// for it; empty when there is nothing to undo.
    public internal(set) var undoKind = ""
    /// What undoing the row's arrangement would undo (044), by the core's name for it; empty
    /// when there is nothing.
    public private(set) var undoArrangementKind = ""
    /// The pictures of the tabs' icons (046), by the numbers the core gives them, as far as
    /// this window has them; and the numbers whose pictures the core was asked for.
    var icons: [UInt64: CGImage] = [:]
    @ObservationIgnored var iconsAsked: Set<UInt64> = []
    /// The window is a private one (037): made to be one, or said to be one by the core.
    public internal(set) var isPrivate = false
    /// The window is to be opened in the core as a private one. Set before it opens.
    var opensPrivate = false {
        didSet { isPrivate = isPrivate || opensPrivate }
    }

    /// Why the core would not make a Space, by its code: said on the page that asked.
    public internal(set) var spaceRefusal: String?
    /// The warning before going on past a certificate is open (026): for this failure of the
    /// tab in front, and no other.
    public internal(set) var isWarnedOfCertificate = false
    /// What is in the address field. It follows the core until the user types in it. Each
    /// change while it is typed in asks what to offer for it (011).
    public var addressText = "" {
        didSet {
            if addressText != oldValue {
                suggest()
            }
        }
    }

    /// The address field has the keyboard: what the user types is not overwritten.
    public var isEditingAddress = false {
        didSet {
            show()
            if !isEditingAddress {
                putSuggestionsAway()
            }
        }
    }

    /// What the model last put in the field. While the field still holds it, nothing was typed.
    @ObservationIgnored private var shownText = ""

    /// The user has changed what the field was given.
    public var hasTypedAddress: Bool {
        addressText != shownText
    }

    /// What the mark beside the address field says. It is the page's while the field shows the
    /// page's address; once something else has been typed there, the mark says nothing: a
    /// padlock must not vouch for what a person typed.
    public var fieldSecurity: WTAddressView.Security {
        hasTypedAddress ? .unspecified : tab.address.security
    }

    let session: BrowserSession
    private var window: OpenedWindow?
    /// Addresses the system handed over before the window was open (031).
    @ObservationIgnored var addressesWaiting: [String] = []

    /// What the core says of where searches go (020): the engines, the one in use, and whether
    /// anybody chose it. `nil` until the core has said. Setting it tells nobody: what the
    /// core said is taken in `told(whereSearchesGo:)`.
    public internal(set) var search: WTSearchSettings?

    /// The list is open to change where searches go: an empty tab's "Change…" was pressed.
    public internal(set) var isChangingSearch = false
    /// The first run's list was closed with the default kept: it is not shown again while the
    /// core takes the choice down.
    public internal(set) var hasKeptDefaultSearch = false

    public init(session: BrowserSession) {
        self.session = session
    }

    /// A model whose find bar is open on `finding` from the start, before anything has been
    /// asked of the core: for a picture of the bar (004), which must not depend on when the
    /// window got round to opening it.
    init(
        session: BrowserSession,
        finding: String?,
        typing: String? = nil,
        offered: [WTAddressSuggestion] = [],
        zooming: Bool = false,
        searching: WTSearchSettings? = nil
    ) {
        self.session = session
        isZooming = zooming
        search = searching
        if let finding {
            isFinding = true
            findText = finding
        }
        // The same for the address field with something typed in it and places offered for
        // it (011): a picture of the list.
        if let typing {
            isEditingAddress = true
            addressText = typing
            suggestions = offered
        }
    }

    /// Opens the window in the core. Call once, when the view appears.
    public func open() async {
        guard window == nil else { return }
        window = try? await session.openWindow(private: opensPrivate) { [weak self] snapshot in
            self?.apply(snapshot)
        }
        // Where searches go, and whether anybody chose it (020): asked once, as the window
        // opens.
        askWhereSearchesGo()
        // Links pressed in other apps before the window was open (031): never a private
        // window's to open (037).
        if !opensPrivate {
            openWhatWaited()
        }
    }

    public func close() {
        guard let window else { return }
        self.window = nil
        session.closeWindow(window)
    }

    /// The tab the window's controls act on: the one in front. Until the first snapshot says
    /// which that is, it is the tab the window opened with.
    var frontTabID: UInt64? {
        tab.tabID != 0 ? tab.tabID : window?.tabID
    }

    /// Sends the address field's text to the core, which decides what it is.
    public func submit() {
        // With a place marked by the arrow keys, Return goes there.
        if let marked = markedSuggestion, suggestions.indices.contains(marked) {
            choose(suggestions[marked])
            return
        }
        guard let frontTabID else { return }
        var submit = WTSubmitAddress()
        submit.tabID = frontTabID
        submit.text = addressText
        isEditingAddress = false
        send(.submitAddress(submit))
    }

    public func perform(_ command: WTNavigationCommand) {
        guard let frontTabID else { return }
        var tabCommand = WTTabCommand()
        tabCommand.tabID = frontTabID
        tabCommand.command = command
        send(.tabCommand(tabCommand))
    }

    /// Opens an empty tab, in front, at the end of the window's tabs.
    public func openTab() {
        guard let window else { return }
        var open = WTOpenTab()
        open.windowID = window.windowID
        send(.openTab(open))
    }

    /// Closes a tab. The core says which tab comes forward, and never leaves the window empty.
    public func closeTab(_ id: UInt64) {
        var close = WTCloseTab()
        close.tabID = id
        send(.closeTab(close))
    }

    /// Brings a tab in front. `chosenWhileTyping`: it was chosen from what was offered while
    /// typing in the tab in front (038); the core then closes that tab when it is an empty one.
    public func activate(_ id: UInt64, chosenWhileTyping: Bool = false) {
        guard id != tab.tabID else { return }
        var activate = WTActivateTab()
        activate.tabID = id
        activate.chosenWhileTyping = chosenWhileTyping
        send(.activateTab(activate))
    }

    // MARK: Places offered as the address field is typed in (011)

    /// The places the core offers for what is typed, best first. The model keeps none of its
    /// own: each is the core's answer to what is now in the field.
    public private(set) var suggestions: [WTAddressSuggestion] = []
    /// Which of them the arrow keys have marked, if any: Return then goes there.
    public private(set) var markedSuggestion: Int?
    /// Counts what was asked: an answer to anything but the last question is out of date.
    @ObservationIgnored private var suggestionsAsked = 0
    /// The commands sent with the last question (048), and what only the window can do.
    @ObservationIgnored var commandsAsked: [BarCommand] = []
    @ObservationIgnored var windowDoes = WindowDoes()

    /// Asks what to offer for what is in the field: only while it is being typed in.
    private func suggest() {
        guard isEditingAddress, hasTypedAddress, !addressText.isEmpty, let window else {
            putSuggestionsAway()
            return
        }
        suggestionsAsked += 1
        let question = suggestionsAsked
        var ask = WTSuggestAddresses()
        ask.windowID = window.windowID
        ask.text = addressText
        askOfTheCommandBar(&ask)
        let answer = session.core.send(.suggestAddresses(ask))
        Task { [weak self] in
            guard case let .addressSuggestions(offered) = try? await answer.value, let self, question == suggestionsAsked else { return }
            suggestions = offered.suggestions
            markedSuggestion = nil
        }
    }

    /// Puts the list away: nothing is offered, and an answer still on its way is for nothing.
    public func putSuggestionsAway() {
        suggestionsAsked += 1
        if !suggestions.isEmpty {
            suggestions = []
        }
        markedSuggestion = nil
    }

    /// Goes to what was offered. A place is loaded in the tab in front. A tab that is open
    /// (038) is brought in front and nothing is loaded; the core is told it was chosen while
    /// typing, and closes the tab typed in when that was an empty one.
    public func choose(_ suggestion: WTAddressSuggestion) {
        // A Space is shown, and a command done (048).
        if chooseOfTheCommandBar(suggestion) {
            return
        }
        guard suggestion.tabID != 0 else {
            load(suggestion.url)
            return
        }
        isEditingAddress = false
        activate(suggestion.tabID, chosenWhileTyping: true)
    }

    /// Marks the next place offered: from none to the first, and from the last to none, where
    /// Return sends what was typed again.
    public func markNextSuggestion() {
        guard !suggestions.isEmpty else { return }
        switch markedSuggestion {
        case nil: markedSuggestion = 0
        case let marked? where marked + 1 < suggestions.count: markedSuggestion = marked + 1
        default: markedSuggestion = nil
        }
    }

    /// Marks the place before: from none to the last, and from the first to none.
    public func markPreviousSuggestion() {
        guard !suggestions.isEmpty else { return }
        switch markedSuggestion {
        case nil: markedSuggestion = suggestions.count - 1
        case let marked? where marked > 0: markedSuggestion = marked - 1
        default: markedSuggestion = nil
        }
    }

    // MARK: Zoom (012)

    /// The zoom bar is open. A model made with `zooming` starts with it open: for a picture of
    /// the bar (004).
    public private(set) var isZooming = false

    /// What the page of the tab in front is zoomed to: the core's word for it, with whether
    /// there is a level above and one below. `nil` while there is no page that can be zoomed.
    public var zoom: WTZoomView? {
        tab.hasZoom ? tab.zoom : nil
    }

    /// Opens the zoom bar, in place of the find bar: one bar at a time. There is nothing to
    /// zoom without a page.
    public func openZoom() {
        guard zoom != nil else { return }
        closeFind()
        isZooming = true
    }

    public func closeZoom() {
        isZooming = false
    }

    /// Asks the core to make the page of the tab in front bigger or smaller, or to put it
    /// back. Which level that is, and which other tabs have it, is the core's.
    public func zoom(_ change: WTZoomChange) {
        guard let frontTabID, zoom != nil else { return }
        var ask = WTZoomPage()
        ask.tabID = frontTabID
        ask.change = change
        send(.zoomPage(ask))
    }

    // MARK: Find on the page (010)

    /// The find bar is open.
    public private(set) var isFinding = false
    /// What is typed in the find bar. Each change is looked for at once.
    public var findText = "" {
        didSet {
            if findText != oldValue, isFinding {
                look(backwards: false)
            }
        }
    }

    /// The tab the find is in: where it must be stopped when the bar closes or another tab
    /// comes in front.
    private var findingTabID: UInt64?

    /// What the find bar says of what was found.
    public enum FindOutcome: Equatable {
        /// Nothing to say: nothing looked for, nothing known yet, or found by an engine that
        /// does not count.
        case nothing
        case notFound
        /// Which match is shown, and how many there are.
        case match(Int, of: Int)
    }

    /// What the core says was found of what is now in the bar. The model keeps no count of
    /// its own.
    public var findOutcome: FindOutcome {
        guard isFinding, tab.hasFind, tab.find.answered, tab.find.text == findText else { return .nothing }
        guard tab.find.found else { return .notFound }
        return tab.find.counted ? .match(Int(tab.find.current), of: Int(tab.find.matches)) : .nothing
    }

    /// Opens the find bar, in place of the zoom bar: one bar at a time. There is nothing to
    /// find in a tab with no page.
    public func openFind() {
        guard hasPage else { return }
        isZooming = false
        isFinding = true
    }

    /// Closes the find bar: the find stops, and its marks go.
    public func closeFind() {
        guard isFinding else { return }
        isFinding = false
        findText = ""
        stopFinding()
    }

    public func findNext() {
        look(backwards: false)
    }

    public func findPrevious() {
        look(backwards: true)
    }

    private func look(backwards: Bool) {
        guard isFinding, hasPage, let frontTabID else { return }
        var find = WTFindInPage()
        find.tabID = frontTabID
        find.text = findText
        find.backwards = backwards
        // Empty text is "stop": the tab then has no find to stop later.
        findingTabID = findText.isEmpty ? nil : frontTabID
        send(.findInPage(find))
    }

    private func stopFinding() {
        guard let finding = findingTabID else { return }
        findingTabID = nil
        var find = WTFindInPage()
        find.tabID = finding
        send(.findInPage(find))
    }

    public var isLoading: Bool {
        tab.loadState == .loading
    }

    public var hasFailed: Bool {
        tab.loadState == .failed
    }

    public var hasPage: Bool {
        tab.webViewID != 0
    }

    // MARK: Sites (039)

    /// The tab in front can be kept, or is a Site that can be let go: the one command has
    /// something to do.
    public var canKeepOrLetGo: Bool {
        tab.kept || hasPage
    }

    /// Keeps a tab as a Site, or lets it go when it is one (039). What either means, and
    /// where the tab then sits, is the core's.
    public func keepOrLetGo(_ id: UInt64) {
        let kept = id == tab.tabID ? tab.kept : tabs.first { $0.tabID == id }?.kept ?? false
        if kept {
            var letGo = WTLetGoSite()
            letGo.tabID = id
            send(.letGoSite(letGo))
        } else {
            var keep = WTKeepTab()
            keep.tabID = id
            send(.keepTab(keep))
        }
    }

    /// The same for the tab in front: what the menu's item and its key ask.
    public func keepOrLetGoTheTabInFront() {
        guard let frontTabID, canKeepOrLetGo else { return }
        keepOrLetGo(frontTabID)
    }

    /// The words of the command that sends the Site in front back to its home, with its
    /// name ("Return to Mail"); the words with no name when the tab in front is not a Site
    /// that is away, for which the command is grey.
    public var returnWords: String {
        tab.away ? Self.returnWords(to: tab.siteName) : Words.sitesReturnSite.string()
    }

    /// "Return to Mail": for a Site by the name it was kept with. A Site kept with no name
    /// is not named.
    static func returnWords(to name: String, locale: Locale = .current) -> String {
        name.isEmpty ? Words.sitesReturnSite.string(locale) : Words.sitesReturn.filled(name, locale: locale)
    }

    /// Sends a Site back to its home (039). Whether the tab is one that is away is the
    /// core's to say: for any other it does nothing.
    public func sendHome(_ id: UInt64) {
        var send = WTSendSiteHome()
        send.tabID = id
        self.send(.sendSiteHome(send))
    }

    /// The same for the tab in front, when it is a Site that is away: what the menu's item
    /// and its key ask.
    public func sendHomeTheTabInFront() {
        guard let frontTabID, tab.away else { return }
        sendHome(frontTabID)
    }

    private func apply(_ snapshot: WTWindowSnapshot) {
        // Another tab came in front: what was typed was typed for the tab that was there. (The
        // window's first snapshot names its first tab, and is no switch: what was typed before
        // it arrived stays.)
        let switched = tab.tabID != 0 && snapshot.tab.tabID != tab.tabID
        if switched {
            // The find was in the tab that was in front: it stops there, and the bar closes.
            closeFind()
            // What was offered was offered for what was typed for that tab.
            putSuggestionsAway()
        }
        keepWarning(for: snapshot.tab, switched: switched)
        tab = snapshot.tab
        tabs = snapshot.tabs
        askForIcons(snapshot.tabs)
        groups = snapshot.groups
        favouritesFull = snapshot.favouritesFull
        rowTabs = Int(snapshot.rowTabs)
        layout = snapshot.layout
        downloadsActive = snapshot.downloadsActive
        spaceID = snapshot.spaceID
        spaceName = snapshot.spaceName
        spaceColour = snapshot.spaceColour
        spaceIcon = snapshot.spaceIcon
        undoKind = snapshot.undo
        undoArrangementKind = snapshot.undoArrangement
        isPrivate = opensPrivate || snapshot.privateWindow
        // The zoom bar is the window's, and says the level of the page in front: with no page
        // there that can be zoomed, it has nothing to say.
        if isZooming, !tab.hasZoom {
            isZooming = false
        }
        // A field with the keyboard still follows the page until something is typed in it: one
        // that was given the focus and left alone must not go on showing what was there then.
        if switched || !isEditingAddress || !hasTypedAddress {
            show()
        }
    }

    /// Puts the page's address in the field: whole while it is edited, the short form at rest.
    private func show() {
        shownText = isEditingAddress && !tab.address.url.isEmpty ? tab.address.url : tab.address.display
        addressText = shownText
    }

    /// The window's id in the core, once it has opened: what its history is asked by (008).
    public var windowID: UInt64? {
        window?.windowID
    }

    /// Sends the tab in front to an address the app has, as if it had been typed: a visit
    /// chosen from history.
    public func load(_ address: String) {
        guard let frontTabID else { return }
        isEditingAddress = false
        var submit = WTSubmitAddress()
        submit.tabID = frontTabID
        submit.text = address
        send(.submitAddress(submit))
    }

    func send(_ request: WTRequest.OneOf_Kind) {
        // Sent here and now, so what was done first reaches the core first. The answer is not
        // waited for: a refusal shows in the next snapshot (a failed load) or changes nothing
        // (a command for a tab with no page), and there is nothing more to tell the user here.
        _ = session.core.send(request)
    }
}

// A window's model: what the core last said about it, and the user's intents sent back.
public import Observation
public import WTContract
import WTCoreClient

@MainActor
@Observable
public final class BrowserModel {
    /// The tab in front, as the core last published it. The window's controls act on it.
    public private(set) var tab = WTTabView()
    /// Every tab of the window, in the order they are shown.
    public private(set) var tabs: [WTTabSummary] = []
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

    private let session: BrowserSession
    private var window: OpenedWindow?

    public init(session: BrowserSession) {
        self.session = session
    }

    /// A model whose find bar is open on `finding` from the start, before anything has been
    /// asked of the core: for a picture of the bar (004), which must not depend on when the
    /// window got round to opening it.
    init(session: BrowserSession, finding: String?, typing: String? = nil, offered: [WTAddressSuggestion] = []) {
        self.session = session
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
        window = try? await session.openWindow { [weak self] snapshot in
            self?.apply(snapshot)
        }
    }

    public func close() {
        guard let window else { return }
        self.window = nil
        session.closeWindow(window)
    }

    /// The tab the window's controls act on: the one in front. Until the first snapshot says
    /// which that is, it is the tab the window opened with.
    private var frontTabID: UInt64? {
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

    /// Brings a tab in front.
    public func activate(_ id: UInt64) {
        guard id != tab.tabID else { return }
        var activate = WTActivateTab()
        activate.tabID = id
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

    /// Goes to a place that was offered, in the tab in front.
    public func choose(_ suggestion: WTAddressSuggestion) {
        load(suggestion.url)
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

    /// Opens the find bar. There is nothing to find in a tab with no page.
    public func openFind() {
        guard hasPage else { return }
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
        tab = snapshot.tab
        tabs = snapshot.tabs
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

    private func send(_ request: WTRequest.OneOf_Kind) {
        // Sent here and now, so what was done first reaches the core first. The answer is not
        // waited for: a refusal shows in the next snapshot (a failed load) or changes nothing
        // (a command for a tab with no page), and there is nothing more to tell the user here.
        _ = session.core.send(request)
    }
}

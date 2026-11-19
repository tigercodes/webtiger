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
    /// What is in the address field. It follows the core until the user types in it.
    public var addressText = ""
    /// The address field has the keyboard: what the user types is not overwritten.
    public var isEditingAddress = false {
        didSet { show() }
    }

    /// What the model last put in the field. While the field still holds it, nothing was typed.
    @ObservationIgnored private var shownText = ""

    /// The user has changed what the field was given.
    public var hasTypedAddress: Bool {
        addressText != shownText
    }

    private let session: BrowserSession
    private var window: OpenedWindow?

    public init(session: BrowserSession) {
        self.session = session
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
        // Another tab came in front: what was typed was typed for the tab that was there.
        let switched = snapshot.tab.tabID != tab.tabID
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

    private func send(_ request: WTRequest.OneOf_Kind) {
        let core = session.core
        // A refusal shows in the next snapshot (a failed load) or changes nothing (a command
        // for a tab with no page); there is nothing more to tell the user here.
        Task { try? await core.perform(request) }
    }
}

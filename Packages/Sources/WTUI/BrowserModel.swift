// A window's model: what the core last said about it, and the user's intents sent back.
public import Observation
public import WTContract
import WTCoreClient

@MainActor
@Observable
public final class BrowserModel {
    /// The tab as the core last published it.
    public private(set) var tab = WTTabView()
    /// What is in the address field. While the field is not being edited it follows the core.
    public var addressText = ""
    /// The address field has the keyboard: what the user types is not overwritten.
    public var isEditingAddress = false {
        didSet {
            // Editing starts from the whole address; at rest the field shows the short form.
            addressText = isEditingAddress && !tab.address.url.isEmpty ? tab.address.url : tab.address.display
        }
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

    /// Sends the address field's text to the core, which decides what it is.
    public func submit() {
        guard let window else { return }
        var submit = WTSubmitAddress()
        submit.tabID = window.tabID
        submit.text = addressText
        isEditingAddress = false
        send(.submitAddress(submit))
    }

    public func perform(_ command: WTNavigationCommand) {
        guard let window else { return }
        var tabCommand = WTTabCommand()
        tabCommand.tabID = window.tabID
        tabCommand.command = command
        send(.tabCommand(tabCommand))
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
        tab = snapshot.tab
        if !isEditingAddress {
            addressText = tab.address.display
        }
    }

    private func send(_ request: WTRequest.OneOf_Kind) {
        let core = session.core
        // A refusal shows in the next snapshot (a failed load) or changes nothing (a command
        // for a tab with no page); there is nothing more to tell the user here.
        Task { try? await core.perform(request) }
    }
}

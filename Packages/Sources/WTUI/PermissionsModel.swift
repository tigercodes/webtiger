public import Observation
public import WTContract
public import WTCoreClient

// The Permissions list's model (014): what the sites of a window's login set were allowed to
// use, as the core says, and taking it back. The shell keeps no list of its own: it asks.
import Foundation

@MainActor
@Observable
public final class PermissionsModel {
    /// The sites that were allowed something, in the order the core gives them: by name.
    public private(set) var sites: [WTSitePermissions] = []
    /// The core has answered at least once. Until then the list says nothing at all, rather
    /// than that no site was allowed anything.
    public private(set) var loaded = false

    private let core: any CoreClient
    private let windowID: UInt64
    /// Counts what was asked: an answer to anything but the last question is out of date.
    private var asked = 0

    /// What the sites of a window's Space were allowed.
    public init(core: any CoreClient, windowID: UInt64) {
        self.core = core
        self.windowID = windowID
    }

    /// Asks for the list again.
    public func reload() {
        asked += 1
        let question = asked
        var query = WTQueryPermissions()
        query.windowID = windowID
        // Sent now, after whatever was sent before it: a question that follows a taking back
        // is answered with what is left.
        let answer = core.send(.queryPermissions(query))
        Task { [weak self] in
            guard case let .permissionsPage(page) = try? await answer.value, let self, question == asked else { return }
            sites = page.sites
            loaded = true
        }
    }

    /// Takes back what one site was allowed. It leaves the list at once, and is asked about
    /// the next time one of its pages wants what it had; nothing that is running is stopped.
    public func forget(_ site: WTSitePermissions) {
        var forget = WTForgetPermissions()
        forget.windowID = windowID
        forget.origin = site.origin
        _ = core.send(.forgetPermissions(forget))
        sites.removeAll { $0.origin == site.origin }
    }

    /// Takes back what every site was allowed.
    public func forgetEverything() {
        var forget = WTForgetPermissions()
        forget.windowID = windowID
        forget.everything = true
        _ = core.send(.forgetPermissions(forget))
        reload()
    }
}

// The states of the window the design catalogue shows (004 §2): each is the snapshot a core
// would publish, given to the real window by a scripted core. Nothing here draws.
import Foundation
import SwiftUI
import WTContract
import WTCoreClient
import WTWebKit

/// One state of the browser window.
enum CatalogueState: String, CaseIterable, Identifiable {
    case empty
    case loading
    case secure
    case insecure
    case long
    case failed
    case history
    case tabs

    var id: String {
        rawValue
    }

    /// The state's name in the catalogue.
    var title: LocalizedStringKey {
        switch self {
        case .empty: "catalogue.state.empty"
        case .loading: "catalogue.state.loading"
        case .secure: "catalogue.state.secure"
        case .insecure: "catalogue.state.insecure"
        case .long: "catalogue.state.long"
        case .failed: "catalogue.state.failed"
        case .history: "catalogue.state.history"
        case .tabs: "catalogue.state.tabs"
        }
    }

    /// What a core would publish for the state. The addresses are the documentation's own.
    var snapshot: WTWindowSnapshot {
        var window = WTWindowSnapshot()
        window.windowID = 1
        window.tab.tabID = 2
        window.tab.loadState = .idle
        switch self {
        case .empty:
            break
        case .loading:
            page(&window, url: "https://example.com/", display: "example.com", security: .secure)
            window.tab.loadState = .loading
            window.tab.progress = 0.4
        case .secure:
            page(&window, url: "https://example.com/", display: "example.com", security: .secure)
        case .insecure:
            page(&window, url: "http://example.com/", display: "http://example.com", security: .insecure)
        case .long:
            let path = "example.com/a/path/that/is/longer/than/the/field/is/wide/and/then/goes/on/for/a/while?with=a&query=too"
            page(&window, url: "https://\(path)", display: path, security: .secure)
        case .failed:
            page(&window, url: "https://nowhere.example.com/", display: "nowhere.example.com", security: .secure)
            window.tab.loadState = .failed
            window.tab.errorCode = "net.dns"
        case .history:
            page(&window, url: "https://example.com/second", display: "example.com/second", security: .secure)
            window.tab.canGoBack = true
            window.tab.canGoForward = true
        case .tabs:
            page(&window, url: "https://example.com/", display: "example.com", security: .secure)
            window.tab.title = "Example Domain"
        }
        // The strip names every tab: the one in front, and for "several tabs" its neighbours,
        // one with a title, one still loading and known only by its site, one with no page.
        var front = WTTabSummary()
        front.tabID = window.tab.tabID
        front.title = window.tab.title
        front.host = window.tab.address.url.isEmpty ? "" : window.tab.address.host
        front.loading = window.tab.loadState == .loading
        window.tabs = [front]
        if self == .tabs {
            window.tabs = [
                Self.neighbour(11, title: "A page whose title is longer than a tab is wide"),
                front,
                Self.neighbour(12, host: "example.org", loading: true),
                Self.neighbour(13),
            ]
        }
        return window
    }

    private static func neighbour(_ id: UInt64, title: String = "", host: String = "", loading: Bool = false) -> WTTabSummary {
        var summary = WTTabSummary()
        summary.tabID = id
        summary.title = title
        summary.host = host
        summary.loading = loading
        return summary
    }

    private func page(_ window: inout WTWindowSnapshot, url: String, display: String, security: WTAddressView.Security) {
        window.tab.webViewID = 4
        window.tab.address.url = url
        window.tab.address.display = display
        window.tab.address.host = URLComponents(string: url)?.host ?? ""
        window.tab.address.registrableDomain = "example.com"
        window.tab.address.security = security
    }

    /// A session whose core opens one window and publishes this state for it, and nothing else:
    /// no Rust core, and no web engine doing anything.
    @MainActor
    func session() -> BrowserSession {
        let published = snapshot
        let emitter = Emitter()
        let core = ScriptedCore { request in
            switch request {
            case .openWindow:
                var opened = WTWindowOpened()
                opened.windowID = 1
                opened.tabID = 2
                return .windowOpened(opened)
            case .subscribe:
                // As the core does: a subscription's first snapshot comes with its answer.
                emitter.core?.emit(published, subscription: 3)
                var subscribed = WTSubscribed()
                subscribed.subscriptionID = 3
                return .subscribed(subscribed)
            default:
                return .ack(WTAck())
            }
        }
        emitter.core = core
        return BrowserSession(core: core, engine: WebKitEngine())
    }
}

/// Lets a scripted core's script reach the core it is the script of.
private final class Emitter: @unchecked Sendable {
    var core: ScriptedCore?
}

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
    case find
    case findNone = "find-none"
    case suggestions
    case zoom
    case prompt
    case promptDownload = "prompt-download"
    case promptAlert = "prompt-alert"
    case promptConfirm = "prompt-confirm"
    case promptText = "prompt-text"
    case promptSignIn = "prompt-sign-in"
    case promptSignInAgain = "prompt-sign-in-again"

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
        case .find: "catalogue.state.find"
        case .findNone: "catalogue.state.find_none"
        case .suggestions: "catalogue.state.suggestions"
        case .zoom: "catalogue.state.zoom"
        case .prompt: "catalogue.state.prompt"
        case .promptDownload: "catalogue.state.prompt_download"
        case .promptAlert: "catalogue.state.prompt_alert"
        case .promptConfirm: "catalogue.state.prompt_confirm"
        case .promptText: "catalogue.state.prompt_text"
        case .promptSignIn: "catalogue.state.prompt_sign_in"
        case .promptSignInAgain: "catalogue.state.prompt_sign_in_again"
        }
    }

    /// The window's zoom bar is open in this state.
    var zooming: Bool {
        self == .zoom
    }

    /// What is typed in the address field in this state, when something is.
    var typing: String? {
        self == .suggestions ? "exa" : nil
    }

    /// The places offered for it: one with a title, one whose title is longer than the list
    /// is wide, one with none.
    var offered: [WTAddressSuggestion] {
        guard self == .suggestions else { return [] }
        let places: [(String, String)] = [
            ("Example Domain", "example.com"),
            ("A page whose title is longer than the list is wide, and then goes on", "example.org/a/long/path"),
            ("", "example.net/plain.txt"),
        ]
        return places.map { title, display in
            var place = WTAddressSuggestion()
            place.title = title
            place.display = display
            place.url = "https://\(display)"
            return place
        }
    }

    /// What the window's find bar is open on in this state, when it is.
    var finding: String? {
        switch self {
        case .find: "needle"
        case .findNone: "zebra"
        default: nil
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
            // No page arrived, and there is nothing to zoom.
            window.tab.clearZoom()
        case .history:
            page(&window, url: "https://example.com/second", display: "example.com/second", security: .secure)
            window.tab.canGoBack = true
            window.tab.canGoForward = true
        case .tabs:
            page(&window, url: "https://example.com/", display: "example.com", security: .secure)
            window.tab.title = "Example Domain"
            // A Space with a name and a colour (024, 025): every other state is the Space the
            // app made, which has neither.
            window.spaceID = 2
            window.spaceName = "Work"
            window.spaceColour = "blue"
        case .find:
            // An engine that counts: the third of twelve.
            page(&window, url: "https://example.com/", display: "example.com", security: .secure)
            window.tab.find.text = "needle"
            window.tab.find.answered = true
            window.tab.find.found = true
            window.tab.find.counted = true
            window.tab.find.matches = 12
            window.tab.find.current = 3
        case .findNone:
            page(&window, url: "https://example.com/", display: "example.com", security: .secure)
            window.tab.find.text = "zebra"
            window.tab.find.answered = true
        case .suggestions:
            page(&window, url: "https://example.org/", display: "example.org", security: .secure)
        case .zoom:
            page(&window, url: "https://example.com/", display: "example.com", security: .secure)
            window.tab.zoom.percent = 125
            window.tab.zoom.canZoomIn = true
            window.tab.zoom.canZoomOut = true
        case .prompt:
            page(&window, url: "https://meet.example.com/", display: "meet.example.com", security: .secure)
            window.tab.prompt.promptID = 9
            window.tab.prompt.permission.site = "meet.example.com"
            window.tab.prompt.permission.kinds = [.camera, .microphone]
        case .promptDownload:
            page(&window, url: "https://example.com/", display: "example.com", security: .secure)
            window.tab.prompt.promptID = 9
            window.tab.prompt.download.site = "example.com"
            window.tab.prompt.download.fileName = "Annual report 2026.pdf"
        case .promptAlert:
            saying(&window, .alert, "Your basket is empty.")
        case .promptConfirm:
            // Not the page's first: it can be stopped.
            saying(&window, .confirm, "Delete “Holiday photos”? This cannot be undone.")
            window.tab.prompt.dialog.maySilence = true
        case .promptText:
            saying(&window, .prompt, "What should we call you?")
            window.tab.prompt.dialog.defaultText = "Ada"
        case .promptSignIn:
            // The load is on its way to a server that asks first: the page before it is
            // still there.
            page(&window, url: "https://example.com/", display: "example.com", security: .secure)
            window.tab.prompt.promptID = 9
            window.tab.prompt.signIn.site = "staff.example.com"
            window.tab.prompt.signIn.confidential = true
        case .promptSignInAgain:
            page(&window, url: "http://printer.example.com/", display: "http://printer.example.com", security: .insecure)
            window.tab.prompt.promptID = 9
            window.tab.prompt.signIn.site = "http://printer.example.com"
            window.tab.prompt.signIn.again = true
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
        // What a core says of any page with a site (012): at actual size until it is zoomed.
        window.tab.zoom.percent = 100
        window.tab.zoom.canZoomIn = true
        window.tab.zoom.canZoomOut = true
    }

    /// A page of example.com that says `message`, or asks (016).
    private func saying(_ window: inout WTWindowSnapshot, _ kind: WTDialogKind, _ message: String) {
        page(&window, url: "https://example.com/", display: "example.com", security: .secure)
        window.tab.prompt.promptID = 9
        window.tab.prompt.dialog.site = "example.com"
        window.tab.prompt.dialog.kind = kind
        window.tab.prompt.dialog.message = message
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

/// One state of the History list (008).
enum CatalogueHistory: String, CaseIterable, Identifiable {
    case list
    case empty
    case noneFound = "none-found"

    var id: String {
        rawValue
    }

    /// The state's name in the catalogue.
    var title: LocalizedStringKey {
        switch self {
        case .list: "catalogue.state.history_list"
        case .empty: "catalogue.state.history_empty"
        case .noneFound: "catalogue.state.history_none_found"
        }
    }

    /// What a core would answer for the state. The addresses are the documentation's own, and
    /// the times are fixed: midday on the first of January 2026, and before.
    private var page: WTHistoryPage {
        var page = WTHistoryPage()
        guard self == .list else { return page }
        page.entries = [
            Self.visit(4, "Example Domain", "https://example.com/", minutesAgo: 0),
            Self.visit(
                3,
                "A page whose title is longer than the list is wide, and then goes on",
                "https://example.org/a/long/path",
                minutesAgo: 7
            ),
            Self.visit(2, "", "https://example.net/plain.txt", minutesAgo: 95),
            Self.visit(1, "Yesterday’s page", "http://example.com/yesterday", minutesAgo: 60 * 26),
        ]
        page.more = true
        return page
    }

    /// A visit so many minutes before midday on the first of January 2026, shown as the
    /// address field shows an address: without its scheme when that is https.
    private static func visit(_ id: UInt64, _ title: String, _ url: String, minutesAgo: Int64) -> WTHistoryEntry {
        var entry = WTHistoryEntry()
        entry.visitID = id
        entry.title = title
        entry.url = url
        entry.display = url.hasPrefix("https://") ? String(url.dropFirst(8)).trimmingCharacters(in: ["/"]) : url
        entry.visitedAtMs = 1_767_268_800_000 - minutesAgo * 60000
        return entry
    }

    /// A model over a core that answers with this state, and nothing else.
    @MainActor
    func model() -> HistoryModel {
        let answer = page
        let core = ScriptedCore { request in
            if case .queryHistory = request {
                return .historyPage(answer)
            }
            return .ack(WTAck())
        }
        let model = HistoryModel(core: core, windowID: 1)
        if self == .noneFound {
            model.text = "zebra"
        }
        return model
    }
}

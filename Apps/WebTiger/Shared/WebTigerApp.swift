// The app. Everything testable lives in Packages; this only starts the core and hosts
// WTUI's window.
import SwiftUI
import WTCoreClient
import WTUI

@main
struct WebTigerApp: App {
    @State private var launch = Launch()

    #if os(macOS)
        init() {
            BrowserCommands.useTheBrowsersOwnTabs()
        }
    #endif

    var body: some Scene {
        WindowGroup {
            switch launch.outcome {
            case let .started(session):
                BrowserWindowView(session: session)
            case let .refused(code):
                StartFailedView(code: code)
            case .catalogue:
                CatalogueView()
            }
        }
        #if os(macOS)
        .defaultSize(width: 1100, height: 760)
        // The window's commands in the menu bar, with their keys (034).
        .commands { BrowserCommands() }
        #endif
    }
}

/// The core, started once for the process. Every window shares it.
@MainActor
@Observable
final class Launch {
    enum Outcome {
        case started(BrowserSession)
        case refused(code: String)
        /// The design catalogue was asked for instead of a browser (004).
        case catalogue
    }

    let outcome: Outcome

    init() {
        // `WEBTIGER_CATALOGUE=1`: the design catalogue, which needs no core.
        guard ProcessInfo.processInfo.environment["WEBTIGER_CATALOGUE"] != "1" else {
            outcome = .catalogue
            return
        }
        do {
            outcome = try .started(BrowserSession.live())
        } catch let CoreError.startRefused(code) {
            outcome = .refused(code: code)
        } catch CoreError.contractMismatch {
            outcome = .refused(code: "start.contract_major")
        } catch {
            outcome = .refused(code: "start.failed")
        }
    }
}

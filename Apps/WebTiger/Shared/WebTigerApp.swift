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
                #if os(macOS)
                    BrowserWindowView(session: session)
                #else
                    // One window at a time: a private window (037) is shown in its place.
                    BrowserWindowsView(session: session)
                #endif
            case let .refused(code):
                StartFailedView(code: code)
            case .catalogue:
                CatalogueView()
            }
        }
        #if os(macOS)
        .defaultSize(width: 1100, height: 760)
        #endif

        #if os(macOS)
            // Private windows (037): what New Private Window opens. The system does not put
            // one back when the app starts again, and hands it nothing from other apps.
            WindowGroup(id: BrowserWindowView.privateWindows) {
                switch launch.outcome {
                case let .started(session):
                    BrowserWindowView(privateIn: session)
                case let .refused(code):
                    StartFailedView(code: code)
                case .catalogue:
                    CatalogueView()
                }
            }
            .defaultSize(width: 1100, height: 760)
            .restorationBehavior(.disabled)
            .handlesExternalEvents(matching: [])
            // The windows' commands in the menu bar, with their keys (034): one menu bar for
            // every window of the app. They are said on the last kind of window there is:
            // each kind brings the system's own File menu with it, and said on an earlier
            // one, the system's Close took the place of Close Tab and Close Window again.
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

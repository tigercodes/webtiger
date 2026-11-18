// The app. Everything testable lives in Packages; this only starts the core and hosts
// WTUI's window.
import SwiftUI
import WTCoreClient
import WTUI

@main
struct WebTigerApp: App {
    @State private var launch = Launch()

    var body: some Scene {
        WindowGroup {
            switch launch.outcome {
            case let .started(session):
                BrowserWindowView(session: session)
            case let .refused(code):
                StartFailedView(code: code)
            }
        }
        #if os(macOS)
        .defaultSize(width: 1100, height: 760)
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
    }

    let outcome: Outcome

    init() {
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

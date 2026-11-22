// From tab to tab by key (035), against a scripted core: what the window's model asks. Which
// tab comes in front is the core's to say, and is tested there.
import WTContract
import WTCoreClient
@testable import WTUI
import WTWebKit
import XCTest

@MainActor
final class TabStepModelTests: XCTestCase {
    /// A model over a core that opens window 7 and says Ack to the rest.
    private func model() -> (BrowserModel, ScriptedCore) {
        let core = ScriptedCore { request in
            switch request {
            case .openWindow:
                var opened = WTWindowOpened()
                opened.windowID = 7
                opened.tabID = 2
                return .windowOpened(opened)
            case .subscribe:
                var subscribed = WTSubscribed()
                subscribed.subscriptionID = 3
                return .subscribed(subscribed)
            default:
                return .ack(WTAck())
            }
        }
        return (BrowserModel(session: BrowserSession(core: core, engine: WebKitEngine())), core)
    }

    /// The steps the core was asked for, in order: the window, the way, the place.
    private func steps(_ core: ScriptedCore) -> [String] {
        core.requests.compactMap {
            if case let .stepTab(step) = $0 {
                "\(step.windowID) \(step.to) \(step.place)"
            } else {
                nil
            }
        }
    }

    func testTheCoreIsAskedWithTheWindowTheWayAndThePlace() async {
        let (model, core) = model()
        // Before the window is open there is no window to ask for.
        model.stepTab(.next)
        model.showTab(at: 1)
        await model.open()
        XCTAssertEqual(steps(core), [])

        model.stepTab(.next)
        model.stepTab(.previous)
        model.stepTab(.last)
        model.showTab(at: 3)
        // A place is counted from one: there is none before it, and none is asked for.
        model.showTab(at: 0)
        model.showTab(at: -2)

        XCTAssertEqual(steps(core), ["7 next 0", "7 previous 0", "7 last 0", "7 place 3"])
        // The model works nothing out by itself: no tab is named to come in front.
        XCTAssertFalse(core.requests.contains {
            if case .activateTab = $0 {
                true
            } else {
                false
            }
        })
    }
}

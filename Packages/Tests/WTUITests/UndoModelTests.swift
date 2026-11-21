// Undo (033), against a scripted core: when the window's model offers to bring a closed tab
// back, and what it then asks of the core.
import WTContract
import WTCoreClient
@testable import WTUI
import WTWebKit
import XCTest

@MainActor
final class UndoModelTests: XCTestCase {
    /// A model over a core that opens a window numbered `window` and says Ack to the rest.
    private func model(window: UInt64 = 1) -> (BrowserModel, ScriptedCore) {
        let core = ScriptedCore { request in
            switch request {
            case .openWindow:
                var opened = WTWindowOpened()
                opened.windowID = window
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

    /// The window as the core says it is, with what an undo would undo.
    private func snapshot(window: UInt64, undo: String) -> WTWindowSnapshot {
        var said = WTWindowSnapshot()
        said.windowID = window
        said.tab.tabID = 2
        said.undo = undo
        return said
    }

    /// The windows the core was asked to undo in, in order.
    private func undone(_ core: ScriptedCore) -> [UInt64] {
        core.requests.compactMap {
            if case let .undo(asked) = $0 {
                asked.windowID
            } else {
                nil
            }
        }
    }

    /// Lets the session's event task run.
    private func settle(until done: () -> Bool) async {
        for _ in 0 ..< 200 where !done() {
            await Task.yield()
            try? await Task.sleep(for: .milliseconds(2))
        }
    }

    func testAClosedTabIsOfferedBackOnlyWhileTheCoreSaysThereIsOne() async {
        let (model, core) = model(window: 7)
        // Before the window is open, and before the core has said anything: nothing is offered,
        // and asking does nothing.
        XCTAssertFalse(model.canReopenClosedTab)
        model.reopenClosedTab()
        await model.open()
        XCTAssertFalse(model.canReopenClosedTab)
        model.reopenClosedTab()
        XCTAssertEqual(undone(core), [], "nothing is asked while there is nothing to bring back")

        // The core says a closed tab can be brought back: it is offered, and asked for with
        // the window, and nothing else is asked.
        core.emit(snapshot(window: 7, undo: "tab.closed"), subscription: 3)
        await settle { model.canReopenClosedTab }
        XCTAssertTrue(model.canReopenClosedTab)
        let before = core.requests.count
        model.reopenClosedTab()
        XCTAssertEqual(undone(core), [7])
        XCTAssertEqual(core.requests.count, before + 1, "one request: the core does the rest")

        // The core says there is nothing more: it is offered no longer.
        core.emit(snapshot(window: 7, undo: ""), subscription: 3)
        await settle { !model.canReopenClosedTab }
        XCTAssertFalse(model.canReopenClosedTab)
        model.reopenClosedTab()
        XCTAssertEqual(undone(core), [7])
    }

    func testAKindOfUndoTheShellDoesNotKnowIsNotOfferedAsAClosedTab() async {
        let (model, core) = model()
        await model.open()
        // What a later core may say can be undone: not a closed tab, so not offered as one.
        core.emit(snapshot(window: 1, undo: "space.renamed"), subscription: 3)
        await settle { model.undoKind == "space.renamed" }
        XCTAssertEqual(model.undoKind, "space.renamed")
        XCTAssertFalse(model.canReopenClosedTab)
        model.reopenClosedTab()
        XCTAssertEqual(undone(core), [])
    }
}

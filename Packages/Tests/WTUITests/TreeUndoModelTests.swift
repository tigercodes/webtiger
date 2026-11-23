// Undo for what is done to the tree (044), against a scripted core: the command is there when
// the core says there is something to undo, says what that was, and asks the core to undo it.
import WTContract
import WTCoreClient
@testable import WTUI
import WTWebKit
import XCTest

@MainActor
final class TreeUndoModelTests: XCTestCase {
    /// A model over a core that opens window 1 with tab 2 and says Ack to the rest.
    private func model() -> (BrowserModel, ScriptedCore) {
        let core = ScriptedCore { request in
            switch request {
            case .openWindow:
                var opened = WTWindowOpened()
                opened.windowID = 1
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

    /// The window as the core says it is: what an undo of its row's arrangement would undo.
    private func snapshot(undoes kind: String) -> WTWindowSnapshot {
        var said = WTWindowSnapshot()
        said.windowID = 1
        said.tab.tabID = 2
        said.tab.webViewID = 4
        said.undoArrangement = kind
        return said
    }

    /// The windows whose arrangement the core was asked to undo, in order.
    private func asked(_ core: ScriptedCore) -> [UInt64] {
        core.requests.compactMap { request -> UInt64? in
            if case let .undoArrangement(undo) = request {
                undo.windowID
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

    func testTheCommandIsThereWhenTheCoreSaysSoAndAsksTheCoreToUndo() async {
        let (model, core) = model()
        // Before the window is open, and while the core says there is nothing: no command,
        // and nothing is asked.
        XCTAssertFalse(model.canUndoArrangement)
        XCTAssertNil(model.treeUndo)
        model.undoArrangement()
        await model.open()
        core.emit(snapshot(undoes: ""), subscription: 3)
        await settle { model.tab.webViewID == 4 }
        XCTAssertFalse(model.canUndoArrangement)
        model.undoArrangement()
        XCTAssertEqual(asked(core), [])

        // The core says a Group was removed: the command says so, and asks the core to undo
        // the arrangement of this window's row. What comes back is the core's.
        core.emit(snapshot(undoes: "group.removed"), subscription: 3)
        await settle { model.canUndoArrangement }
        XCTAssertEqual(model.treeUndo?.words, Words.undoGroupRemoved)
        model.treeUndo?.action()
        XCTAssertEqual(asked(core), [1])
        model.undoArrangement()
        XCTAssertEqual(asked(core), [1, 1])

        // The core says there is nothing left: the command is gone.
        core.emit(snapshot(undoes: ""), subscription: 3)
        await settle { !model.canUndoArrangement }
        XCTAssertNil(model.treeUndo)
        model.undoArrangement()
        XCTAssertEqual(asked(core).count, 2)
    }

    func testTheCommandSaysWhatItUndoes() {
        let english = Locale(identifier: "en")
        let said = [
            "site.kept": "Undo Keep as Site",
            "site.let_go": "Undo Stop Keeping",
            "group.made": "Undo New Group",
            "group.removed": "Undo Remove Group",
            "group.renamed": "Undo Rename Group",
            "group.coloured": "Undo Group Colour",
            "group.moved": "Undo Move to Group",
            "favourite.added": "Undo Add to Favourites",
            "favourite.removed": "Undo Remove from Favourites",
            "tab.moved": "Undo Move Tab",
            "group.reordered": "Undo Move Group",
        ]
        for (kind, words) in said {
            XCTAssertEqual(BrowserModel.undoWords(for: kind).string(english), words, kind)
        }
        // A kind this shell has no words for, as a later core might name: it is undone all
        // the same, and the command says only that.
        XCTAssertEqual(BrowserModel.undoWords(for: "tree.something_new").string(english), "Undo")
    }
}

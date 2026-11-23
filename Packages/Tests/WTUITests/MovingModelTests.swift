// Moving a tab in its row (045), against a scripted core: what the window asks of the core to
// move a tab a place, that it asks only when the core says the step would move the tab, and
// the two commands' words, which go by how the row is drawn.
import WTContract
import WTCoreClient
@testable import WTUI
import WTWebKit
import XCTest

@MainActor
final class MovingModelTests: XCTestCase {
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

    /// A tab as the core says it is: which way a step would move it.
    private func tab(_ id: UInt64, back: Bool = false, forward: Bool = false, favourite: Bool = false) -> WTTabSummary {
        var summary = WTTabSummary()
        summary.tabID = id
        summary.host = "example.com"
        summary.stepsBack = back
        summary.stepsForward = forward
        summary.kept = favourite
        summary.favourite = favourite
        return summary
    }

    /// The window as the core says it is: tab 2 in front, these tabs, and its layout.
    private func snapshot(tabs: [WTTabSummary], layout: WTLayout = .strip) -> WTWindowSnapshot {
        var said = WTWindowSnapshot()
        said.windowID = 1
        said.tab.tabID = 2
        said.tab.webViewID = 4
        said.tabs = tabs
        said.layout = layout
        return said
    }

    /// What the core was asked to move, in order: "2 back", "5 forward".
    private func asked(_ core: ScriptedCore) -> [String] {
        core.requests.compactMap { request -> String? in
            guard case let .moveTab(move) = request else { return nil }
            guard case let .by(step)? = move.to else { return "\(move.tabID) by no step" }
            return "\(move.tabID) \(step == .back ? "back" : step == .forward ? "forward" : "nowhere")"
        }
    }

    /// Lets the session's event task run.
    private func settle(until done: () -> Bool) async {
        for _ in 0 ..< 200 where !done() {
            await Task.yield()
            try? await Task.sleep(for: .milliseconds(2))
        }
    }

    func testATabIsMovedAPlaceByAskingTheCoreAndOnlyWhenTheCoreSaysItCanGo() async {
        let (model, core) = model()
        // Before the window is open there is no tab to move.
        XCTAssertNil(model.frontTabInTheRow)
        model.moveTheTabInFront(.back)
        await model.open()

        // Two tabs of one part, the second in front: by the core's word the first can go
        // forward, the one in front back, and neither the other way.
        core.emit(snapshot(tabs: [tab(5, forward: true), tab(2, back: true)]), subscription: 3)
        await settle { model.tabs.count == 2 }
        XCTAssertEqual(model.frontTabInTheRow?.tabID, 2)
        XCTAssertTrue(model.canMove(model.tabs[0], .forward))
        XCTAssertFalse(model.canMove(model.tabs[0], .back))
        XCTAssertTrue(model.canMove(model.tabs[1], .back))
        XCTAssertFalse(model.canMove(model.tabs[1], .forward))
        XCTAssertFalse(model.canMove(model.tabs[1], .unspecified), "no step is no way to go")

        // The command for the tab in front asks nothing the way it cannot go; the way it
        // can, it asks the core for a step. A tab's own menu asks for its tab.
        model.moveTheTabInFront(.forward)
        XCTAssertEqual(asked(core), [])
        model.moveTheTabInFront(.back)
        model.moveTab(5, .forward)
        XCTAssertEqual(asked(core), ["2 back", "5 forward"])

        // The core says where they are now, and which way each can go: the window works
        // nothing out for itself.
        core.emit(snapshot(tabs: [tab(2, forward: true), tab(5, back: true)]), subscription: 3)
        await settle { model.tabs.first?.tabID == 2 }
        model.moveTheTabInFront(.back)
        XCTAssertEqual(asked(core), ["2 back", "5 forward"], "at its part's start it is asked nowhere back")
        model.moveTheTabInFront(.forward)
        XCTAssertEqual(asked(core), ["2 back", "5 forward", "2 forward"])

        // A tab alone in its part goes nowhere.
        core.emit(snapshot(tabs: [tab(2)]), subscription: 3)
        await settle { model.tabs.count == 1 }
        model.moveTheTabInFront(.back)
        model.moveTheTabInFront(.forward)
        XCTAssertEqual(asked(core).count, 3)
    }

    func testTheMenuBarsCommandsForTheTabInFrontGoByHowThatTabIsDrawn() async {
        let english = Locale(identifier: "en")
        func said(_ model: BrowserModel) -> [String] {
            model.movesOfTheTabInFront(rightToLeft: false).map { $0.words.string(english) }
        }
        let (model, core) = model()
        await model.open()
        model.hasRoomForSidebar = true
        // Along the top: left and right.
        core.emit(snapshot(tabs: [tab(5, forward: true), tab(2, back: true)]), subscription: 3)
        await settle { model.tabs.count == 2 }
        XCTAssertEqual(said(model), ["Move Tab Left", "Move Tab Right"])
        // Down the side: up and down.
        core.emit(snapshot(tabs: [tab(5, forward: true), tab(2, back: true)], layout: .sidebar), subscription: 3)
        await settle { model.showsSidebar }
        XCTAssertEqual(said(model), ["Move Tab Up", "Move Tab Down"])
        // A Favourite in front is a tile, in a row of tiles wherever the tabs are: left and right.
        core.emit(
            snapshot(tabs: [tab(5, forward: true, favourite: true), tab(2, back: true, favourite: true)], layout: .sidebar),
            subscription: 3
        )
        await settle { model.frontTabInTheRow?.favourite == true }
        XCTAssertEqual(said(model), ["Move Tab Left", "Move Tab Right"])
        // A window with no room for a sidebar draws the strip whatever the core says.
        model.hasRoomForSidebar = false
        core.emit(snapshot(tabs: [tab(5, forward: true), tab(2, back: true)], layout: .sidebar), subscription: 3)
        await settle { model.frontTabInTheRow?.favourite == false }
        XCTAssertEqual(said(model), ["Move Tab Left", "Move Tab Right"])
    }

    func testTheCommandsWordsGoByHowTheRowIsDrawn() {
        let english = Locale(identifier: "en")
        func said(_ moves: [TabMove]) -> [String] {
            moves.map { "\($0.words.string(english)): \($0.step == .back ? "back" : "forward")" }
        }
        // Along the top, where words run from left to right: left is towards the row's start.
        XCTAssertEqual(
            said(TabMove.both(atTheSide: false, rightToLeft: false)),
            ["Move Tab Left: back", "Move Tab Right: forward"]
        )
        // Where they run from right to left the row does too: left is towards its end. The
        // commands are listed in the same order.
        XCTAssertEqual(
            said(TabMove.both(atTheSide: false, rightToLeft: true)),
            ["Move Tab Left: forward", "Move Tab Right: back"]
        )
        // Down the side, up is towards the row's start, whichever way words run.
        XCTAssertEqual(
            said(TabMove.both(atTheSide: true, rightToLeft: false)),
            ["Move Tab Up: back", "Move Tab Down: forward"]
        )
        XCTAssertEqual(
            said(TabMove.both(atTheSide: true, rightToLeft: true)),
            ["Move Tab Up: back", "Move Tab Down: forward"]
        )
        // Each of the two is told apart from the other by a menu.
        XCTAssertEqual(Set(TabMove.both(atTheSide: false, rightToLeft: true).map(\.id)).count, 2)
    }
}

// Dragging in the row (045 §5b), against a scripted core: what the window asks of the core
// when a tab or a Group is dropped at a place, and the places that take nothing.
import WTContract
import WTCoreClient
@testable import WTUI
import WTWebKit
import XCTest

@MainActor
final class DraggingModelTests: XCTestCase {
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

    /// What the core was asked to move, in order, in words.
    private func asked(_ core: ScriptedCore) -> [String] {
        core.requests.compactMap { request -> String? in
            switch request {
            case let .moveTab(move):
                switch move.to {
                case let .beforeTabID(tab): "tab \(move.tabID) before tab \(tab)"
                case let .endOfGroupID(group): "tab \(move.tabID) into Group \(group)"
                case let .endOf(part): "tab \(move.tabID) to the end of \(part)"
                case let .by(step): "tab \(move.tabID) by \(step)"
                case nil: "tab \(move.tabID) nowhere"
                }
            case let .moveGroup(move): "Group \(move.groupID) before Group \(move.beforeGroupID)"
            default: nil
            }
        }
    }

    func testADropAsksTheCoreToMoveTheThingToThePlace() async {
        let (model, core) = model()
        // Before the window is open no place takes anything, and nothing is asked.
        XCTAssertFalse(model.takes(.tab(5), at: .endOfTabs))
        model.drop(.tab(5), at: .endOfTabs)
        XCTAssertEqual(asked(core), [])
        await model.open()

        // A tab: on a tab or a tile, just before it; on a Group's chip, into the Group; past
        // the last tile, past the last Site in no Group, and on "New Tab", to the end of that
        // part. What it becomes there is the core's.
        model.drop(.tab(5), at: .tab(2))
        model.drop(.tab(5), at: .group(30))
        model.drop(.tab(5), at: .endOfFavourites)
        model.drop(.tab(5), at: .endOfSites)
        model.drop(.tab(5), at: .endOfTabs)
        XCTAssertEqual(asked(core), [
            "tab 5 before tab 2", "tab 5 into Group 30", "tab 5 to the end of favourites", "tab 5 to the end of sites",
            "tab 5 to the end of tabs",
        ])

        // A Group: on another Group's chip, before it; on the line under the Groups, after
        // the last.
        model.drop(.group(31), at: .group(30))
        model.drop(.group(30), at: .endOfSites)
        XCTAssertEqual(Array(asked(core).dropFirst(5)), ["Group 31 before Group 30", "Group 30 before Group 0"])
    }

    func testAPlaceThatWouldDoNothingTakesNothing() async {
        let (model, core) = model()
        await model.open()
        // A thing on itself.
        XCTAssertFalse(model.takes(.tab(5), at: .tab(5)))
        XCTAssertFalse(model.takes(.group(30), at: .group(30)))
        // A Group among tabs: on a tab, past the tiles, on "New Tab".
        XCTAssertFalse(model.takes(.group(30), at: .tab(5)))
        XCTAssertFalse(model.takes(.group(30), at: .endOfFavourites))
        XCTAssertFalse(model.takes(.group(30), at: .endOfTabs))
        for (item, place) in [
            (RowItem.tab(5), RowPlace.tab(5)), (.group(30), .group(30)), (.group(30), .tab(5)), (.group(30), .endOfFavourites),
            (.group(30), .endOfTabs),
        ] {
            model.drop(item, at: place)
        }
        XCTAssertEqual(asked(core), [], "nothing is asked for a place that takes nothing")
        // Every other place takes a tab; a tab and a Group whose numbers are the same are
        // not the same thing.
        XCTAssertTrue(model.takes(.tab(5), at: .tab(2)))
        XCTAssertTrue(model.takes(.tab(30), at: .group(30)))
        XCTAssertTrue(model.takes(.tab(5), at: .endOfFavourites))
        XCTAssertTrue(model.takes(.tab(5), at: .endOfSites))
        XCTAssertTrue(model.takes(.tab(5), at: .endOfTabs))
        XCTAssertTrue(model.takes(.group(31), at: .group(30)))
        XCTAssertTrue(model.takes(.group(30), at: .endOfSites))
    }
}

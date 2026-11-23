// A window's layout (043), against a scripted core: what the window asks of the core to
// switch, when it draws a sidebar, and how the sidebar takes the row apart.
import WTContract
import WTCoreClient
@testable import WTUI
import WTWebKit
import XCTest

@MainActor
final class LayoutModelTests: XCTestCase {
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

    private func snapshot(_ layout: WTLayout) -> WTWindowSnapshot {
        var said = WTWindowSnapshot()
        said.windowID = 1
        said.tab.tabID = 2
        said.tab.webViewID = 4
        said.layout = layout
        return said
    }

    /// What the core was asked about the layout, in order: "sidebar", "strip".
    private func asked(_ core: ScriptedCore) -> [String] {
        core.requests.compactMap { request -> String? in
            guard case let .setLayout(set) = request else { return nil }
            return "\(set.layout == .sidebar ? "sidebar" : set.layout == .strip ? "strip" : "another") in \(set.windowID)"
        }
    }

    /// Lets the session's event task run.
    private func settle(until done: () -> Bool) async {
        for _ in 0 ..< 200 where !done() {
            await Task.yield()
            try? await Task.sleep(for: .milliseconds(2))
        }
    }

    func testTheSwitchAsksTheCoreForTheOtherLayoutWhereThereIsASidebarToDraw() async {
        let (model, core) = model()
        // Before the window is open there is nothing to switch.
        model.switchLayout()
        XCTAssertEqual(asked(core), [])
        XCTAssertFalse(model.showsSidebar)
        await model.open()
        core.emit(snapshot(.strip), subscription: 3)
        await settle { model.tab.tabID == 2 }
        XCTAssertFalse(model.showsSidebar)

        model.switchLayout()
        #if os(macOS)
            // The Mac has a sidebar: the core is asked for it, and the window draws it when
            // the core says the window has it, not before.
            XCTAssertTrue(BrowserModel.hasSidebar)
            XCTAssertEqual(asked(core), ["sidebar in 1"])
            XCTAssertFalse(model.showsSidebar, "the core has not said so yet")
            core.emit(snapshot(.sidebar), subscription: 3)
            await settle { model.layout == .sidebar }
            XCTAssertTrue(model.showsSidebar)
            // The same command asks for the strip again.
            model.switchLayout()
            XCTAssertEqual(asked(core), ["sidebar in 1", "strip in 1"])
            // A core from before layouts names none: the strip.
            core.emit(snapshot(.unspecified), subscription: 3)
            await settle { model.layout == .unspecified }
            XCTAssertFalse(model.showsSidebar)
            model.switchLayout()
            XCTAssertEqual(asked(core).last, "sidebar in 1")
        #else
            // Where there is no sidebar to draw, none is asked for, and the strip is drawn
            // whatever the core says.
            XCTAssertFalse(BrowserModel.hasSidebar)
            XCTAssertEqual(asked(core), [])
            core.emit(snapshot(.sidebar), subscription: 3)
            await settle { model.layout == .sidebar }
            XCTAssertFalse(model.showsSidebar)
            model.switchLayout()
            XCTAssertEqual(asked(core), [])
        #endif
    }

    private func tab(_ id: UInt64, kept: Bool = false, favourite: Bool = false, group: UInt64 = 0) -> WTTabSummary {
        var summary = WTTabSummary()
        summary.tabID = id
        summary.kept = kept || favourite
        summary.favourite = favourite
        summary.groupID = group
        return summary
    }

    func testTheSidebarTakesTheRowApartAndKeepsTheCoresOrderInEachPart() {
        var work = WTGroupSummary()
        work.groupID = 30
        work.name = "Work"
        var house = WTGroupSummary()
        house.groupID = 31
        house.name = "House"
        house.shut = true
        // As a core lists a row: Favourites, a Site in no Group, Work's two, none of House's
        // (it is shut), two tabs.
        let tabs = [
            tab(1, favourite: true),
            tab(2, favourite: true),
            tab(3, kept: true),
            tab(4, kept: true, group: 30),
            tab(5, kept: true, group: 30),
            tab(6),
            tab(7),
        ]
        let parts = SidebarParts(tabs: tabs, groups: [work, house])
        XCTAssertEqual(parts.favourites.map(\.tabID), [1, 2])
        XCTAssertEqual(parts.others.map(\.tabID), [6, 7])
        XCTAssertEqual(
            parts.tree.map { item -> String in
                switch item {
                case let .tab(tab): "\(tab.tabID)"
                case let .group(group): "[\(group.name)]"
                }
            },
            ["3", "[Work]", "4", "5", "[House]"]
        )
        // Every tab is in one part, once; with nothing in the row, nothing in any part.
        XCTAssertEqual(parts.favourites.count + parts.others.count + parts.tree.count, tabs.count + 2)
        XCTAssertEqual(SidebarParts(tabs: [], groups: []), SidebarParts(tabs: [], groups: []))
        XCTAssertTrue(SidebarParts(tabs: [], groups: []).tree.isEmpty)
        // Tabs alone: they are under the line, and the tree is empty.
        let plain = SidebarParts(tabs: [tab(6), tab(7)], groups: [])
        XCTAssertEqual(plain.others.map(\.tabID), [6, 7])
        XCTAssertTrue(plain.tree.isEmpty && plain.favourites.isEmpty)
    }
}

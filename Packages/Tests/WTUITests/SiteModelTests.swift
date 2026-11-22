// Sites (039), against a scripted core: the one command with two names, what it asks of the
// core for the tab in front and for a tab of the strip, and what the strip says of a tab.
import WTContract
import WTCoreClient
@testable import WTUI
import WTWebKit
import XCTest

@MainActor
final class SiteModelTests: XCTestCase {
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

    private func tab(_ id: UInt64, kept: Bool = false, asleep: Bool = false) -> WTTabSummary {
        var summary = WTTabSummary()
        summary.tabID = id
        summary.kept = kept
        summary.asleep = asleep
        return summary
    }

    /// The window as the core says it is: tab 2 in front, with a page or without, a Site or
    /// not; and the tabs of its strip.
    private func snapshot(page: Bool, kept: Bool, others: [WTTabSummary] = []) -> WTWindowSnapshot {
        var said = WTWindowSnapshot()
        said.windowID = 1
        said.tab.tabID = 2
        said.tab.webViewID = page ? 4 : 0
        said.tab.kept = kept
        said.tabs = [tab(2, kept: kept)] + others
        return said
    }

    /// What the core was asked about Sites, in order: "keep 2", "let go 5".
    private func asked(_ core: ScriptedCore) -> [String] {
        core.requests.compactMap { request -> String? in
            switch request {
            case let .keepTab(keep): "keep \(keep.tabID)"
            case let .letGoSite(letGo): "let go \(letGo.tabID)"
            default: nil
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

    func testTheCommandKeepsATabWithAPageAndLetsASiteGo() async {
        let (model, core) = model()
        // Before the window is open, and for a tab with no page: nothing to keep.
        XCTAssertFalse(model.canKeepOrLetGo)
        model.keepOrLetGoTheTabInFront()
        await model.open()
        core.emit(snapshot(page: false, kept: false), subscription: 3)
        await settle { model.tab.tabID == 2 }
        XCTAssertFalse(model.canKeepOrLetGo)
        model.keepOrLetGoTheTabInFront()
        XCTAssertEqual(asked(core), [], "an empty tab is not asked to be kept")

        // A page: the tab can be kept, and the core is asked to keep it.
        core.emit(snapshot(page: true, kept: false), subscription: 3)
        await settle { model.hasPage }
        XCTAssertTrue(model.canKeepOrLetGo)
        model.keepOrLetGoTheTabInFront()
        XCTAssertEqual(asked(core), ["keep 2"])

        // The core says it is a Site: the same command lets it go.
        core.emit(snapshot(page: true, kept: true), subscription: 3)
        await settle { model.tab.kept }
        XCTAssertTrue(model.canKeepOrLetGo)
        model.keepOrLetGoTheTabInFront()
        XCTAssertEqual(asked(core), ["keep 2", "let go 2"])

        // A Site in front that is asleep has no page, and can be let go all the same.
        core.emit(snapshot(page: false, kept: true), subscription: 3)
        await settle { !model.hasPage }
        XCTAssertTrue(model.canKeepOrLetGo)
        model.keepOrLetGoTheTabInFront()
        XCTAssertEqual(asked(core), ["keep 2", "let go 2", "let go 2"])
    }

    func testATabOfTheStripIsKeptOrLetGoByWhatTheCoreSaysOfIt() async {
        let (model, core) = model()
        await model.open()
        core.emit(snapshot(page: true, kept: false, others: [tab(5, kept: true, asleep: true), tab(6)]), subscription: 3)
        await settle { model.tabs.count == 3 }

        // From a tab's own menu: the Site is let go, the tab that is none is kept. The model
        // decides nothing else: whether a tab has a page to keep is the core's to say.
        model.keepOrLetGo(5)
        model.keepOrLetGo(6)
        model.keepOrLetGo(2)
        XCTAssertEqual(asked(core), ["let go 5", "keep 6", "keep 2"])
        // A tab the strip does not have is asked to be kept, and the core refuses it.
        model.keepOrLetGo(99)
        XCTAssertEqual(asked(core).last, "keep 99")
    }

    func testTheStripSaysThatATabIsASiteAndThatItIsAsleep() {
        let english = Locale(identifier: "en")
        XCTAssertEqual(TabStrip.said(of: tab(1), locale: english), "")
        XCTAssertEqual(TabStrip.said(of: tab(1, kept: true), locale: english), "Site")
        XCTAssertEqual(TabStrip.said(of: tab(1, asleep: true), locale: english), "Asleep")
        XCTAssertEqual(TabStrip.said(of: tab(1, kept: true, asleep: true), locale: english), "Site, Asleep")
    }
}

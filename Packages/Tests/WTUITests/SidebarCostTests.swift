// What a long row costs the sidebar (043 §8a; 000 §7: "Sidebar with 5,000 rows"). A
// measurement, not a gate: it asserts nothing about time, and runs only when asked:
// `WEBTIGER_MEASURE=1 swift test --filter SidebarCostTests` in Packages.
import SwiftUI
import WTContract
import WTCoreClient
@testable import WTUI
import WTWebKit
import XCTest

@MainActor
final class SidebarCostTests: XCTestCase {
    /// A window whose core says the sidebar, with this many tabs: of every six, five are
    /// Sites, spread over ten Groups and none; the last is in front.
    private func window(rows: Int) -> some View {
        var said = WTWindowSnapshot()
        said.windowID = 1
        said.tab.tabID = UInt64(rows)
        said.tab.webViewID = 4
        said.layout = .sidebar
        said.groups = (1 ... 10).map { number in
            var group = WTGroupSummary()
            group.groupID = UInt64(10000 + number)
            group.name = "Group \(number)"
            return group
        }
        // As a core lists a row: the Sites in no Group, each Group's, the tabs.
        var loose: [WTTabSummary] = []
        var grouped: [[WTTabSummary]] = Array(repeating: [], count: 10)
        var others: [WTTabSummary] = []
        for number in 1 ... rows {
            var tab = WTTabSummary()
            tab.tabID = UInt64(number)
            tab.title = "The title of page number \(number), as long as titles are"
            tab.host = "site\(number).example.com"
            if number % 6 == 5 {
                others.append(tab)
            } else {
                tab.kept = true
                tab.siteName = "Site \(number)"
                let place = number % 11
                if place == 0 {
                    loose.append(tab)
                } else {
                    tab.groupID = UInt64(10000 + place)
                    grouped[place - 1].append(tab)
                }
            }
        }
        said.tabs = loose + grouped.flatMap(\.self) + others
        let published = said
        let emitter = Emitter()
        let core = ScriptedCore { request in
            switch request {
            case .openWindow:
                var opened = WTWindowOpened()
                opened.windowID = 1
                opened.tabID = 2
                return .windowOpened(opened)
            case .subscribe:
                emitter.core?.emit(published, subscription: 3)
                var subscribed = WTSubscribed()
                subscribed.subscriptionID = 3
                return .subscribed(subscribed)
            default:
                return .ack(WTAck())
            }
        }
        emitter.core = core
        let session = BrowserSession(core: core, engine: WebKitEngine())
        return BrowserWindowView(session: session, finding: nil)
            .environment(\.horizontalSizeClass, .regular)
    }

    func testMeasureDrawingASidebarOfManyRows() async throws {
        try XCTSkipUnless(
            ProcessInfo.processInfo.environment["WEBTIGER_MEASURE"] == "1",
            "a measurement: asked for with WEBTIGER_MEASURE=1"
        )
        for rows in [240, 1000, 5000] {
            let started = ContinuousClock.now
            let stage = Snapshot.Stage(window(rows: rows), size: CGSize(width: 900, height: 700), dark: false)
            let picture = await stage.settledPicture()
            XCTAssertNotNil(picture)
            print("MEASURED \(rows) rows: the window with its sidebar drawn and settled in \(ContinuousClock.now - started)")
        }
    }
}

/// Lets a scripted core's script reach the core it is the script of.
private final class Emitter: @unchecked Sendable {
    var core: ScriptedCore?
}

// The window's model and zoom (012), against a scripted core: what it asks, what its bar says,
// and when the bar is open.
import WTContract
import WTCoreClient
@testable import WTUI
import WTWebKit
import XCTest

@MainActor
final class ZoomModelTests: XCTestCase {
    /// A model over a core that opens a window and says Ack to the rest.
    private func opened() async -> (BrowserModel, ScriptedCore) {
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
        let model = BrowserModel(session: BrowserSession(core: core, engine: WebKitEngine()))
        await model.open()
        return (model, core)
    }

    /// A window's snapshot: the tab in front on a page, zoomed as the core would say, or with
    /// `percent` nil on no page at all.
    private func window(front: UInt64 = 2, percent: UInt32?, canZoomIn: Bool = true, canZoomOut: Bool = true) -> WTWindowSnapshot {
        var window = WTWindowSnapshot()
        window.windowID = 1
        window.tab.tabID = front
        if let percent {
            window.tab.webViewID = 4
            window.tab.address.url = "https://example.com/"
            window.tab.address.display = "example.com"
            window.tab.zoom.percent = percent
            window.tab.zoom.canZoomIn = canZoomIn
            window.tab.zoom.canZoomOut = canZoomOut
        }
        return window
    }

    /// Lets the session's event task and the model's request tasks run.
    private func settle(until done: () -> Bool = { false }) async {
        for _ in 0 ..< 200 where !done() {
            await Task.yield()
            try? await Task.sleep(for: .milliseconds(2))
        }
    }

    private func zooms(_ core: ScriptedCore) -> [String] {
        core.requests.compactMap { request -> String? in
            if case let .zoomPage(zoom) = request {
                "\(zoom.tabID) \(zoom.change)"
            } else {
                nil
            }
        }
    }

    func testThereIsNothingToZoomWithoutAPage() async {
        let (model, core) = await opened()
        XCTAssertNil(model.zoom)

        model.openZoom()
        model.zoom(.in)
        await settle()

        XCTAssertFalse(model.isZooming, "the bar stays shut")
        XCTAssertEqual(zooms(core), [], "and nothing is asked")
    }

    func testBiggerSmallerAndActualSizeAskTheCoreForTheTabInFront() async {
        let (model, core) = await opened()
        core.emit(window(front: 9, percent: 100), subscription: 3)
        await settle { model.zoom != nil }

        model.zoom(.in)
        model.zoom(.out)
        model.zoom(.reset)
        await settle { self.zooms(core).count == 3 }

        XCTAssertEqual(zooms(core), ["9 in", "9 out", "9 reset"], "which level each is, is the core's")
        XCTAssertFalse(model.isZooming, "the keys zoom with the bar shut")
    }

    func testTheBarSaysWhatTheCoreSays() async {
        let (model, core) = await opened()
        core.emit(window(percent: 100), subscription: 3)
        await settle { model.zoom != nil }
        model.openZoom()
        XCTAssertTrue(model.isZooming)
        XCTAssertEqual(model.zoom?.percent, 100)

        // The model keeps no level of its own: asking changes nothing until the core says.
        model.zoom(.in)
        XCTAssertEqual(model.zoom?.percent, 100)
        core.emit(window(percent: 300, canZoomIn: false), subscription: 3)
        await settle { model.zoom?.percent == 300 }

        XCTAssertEqual(model.zoom?.canZoomIn, false, "nothing above the largest: the core says so")
        XCTAssertEqual(model.zoom?.canZoomOut, true)
    }

    func testOneBarAtATime() async {
        let (model, core) = await opened()
        core.emit(window(percent: 110), subscription: 3)
        await settle { model.zoom != nil }

        model.openFind()
        model.openZoom()
        XCTAssertTrue(model.isZooming)
        XCTAssertFalse(model.isFinding, "the zoom bar opens in place of the find bar")

        model.openFind()
        XCTAssertTrue(model.isFinding)
        XCTAssertFalse(model.isZooming, "and the find bar in place of the zoom bar")

        model.openZoom()
        model.closeZoom()
        XCTAssertFalse(model.isZooming)
    }

    func testTheBarClosesWhenThePageInFrontCannotBeZoomed() async {
        let (model, core) = await opened()
        core.emit(window(percent: 125), subscription: 3)
        await settle { model.zoom != nil }
        model.openZoom()

        // Another tab's page in front: the bar is the window's, and says that page's level.
        core.emit(window(front: 9, percent: 90), subscription: 3)
        await settle { model.tab.tabID == 9 }
        XCTAssertTrue(model.isZooming)
        XCTAssertEqual(model.zoom?.percent, 90)

        // An empty tab in front: nothing to zoom, and nothing for the bar to say.
        core.emit(window(front: 11, percent: nil), subscription: 3)
        await settle { model.tab.tabID == 11 }
        XCTAssertFalse(model.isZooming)
        XCTAssertNil(model.zoom)
    }
}

// The adapter suite's WebKit run, for zoom (012): what WebKit's own page zoom does when the
// adapter asks for it. A zoomed page is laid out again in less room, or more, with its text
// the size it was in the page's own units: the measure here is how wide the page is laid out.
// The page says it is as wide as the device, as pages for phones do: on an iPhone one that
// does not is laid out 980 wide and shrunk to fit, and its measures are another matter (012 §8).
import WebKit
import WTContract
import WTCoreClient
import WTWebKit
import XCTest

@MainActor
final class WebKitZoomTests: AdapterTestCase {
    /// A web view of a known size on a page.
    private func page(_ id: UInt64 = 1, at path: String = "/sized", titled title: String = "Sized") async {
        await create(id)
        engine.webView(for: id)?.frame = CGRect(x: 0, y: 0, width: 800, height: 600)
        await load(id, site.url(path))
        await wait(id, "the page to load") { self.finished($0) && self.hasTitle(title)($0) }
    }

    @discardableResult
    private func zoom(_ percent: UInt32, in id: UInt64 = 1) async -> WTHostResponse.OneOf_Kind {
        var zoom = WTWebViewSetZoom()
        zoom.webViewID = id
        zoom.percent = percent
        return await engine.answer(.webViewSetZoom(zoom))
    }

    /// How wide the page is laid out, in the page's own units.
    private func width(_ id: UInt64 = 1) async -> Double {
        await run("document.documentElement.clientWidth", in: id) as? Double ?? -1
    }

    /// How tall the page's one line of text is, in the page's own units.
    private func lineHeight(_ id: UInt64 = 1) async -> Double {
        await run("document.getElementById('line').getBoundingClientRect().height", in: id) as? Double ?? -1
    }

    /// Waits for the page to be laid out `wide`, give or take a unit, and says what it was.
    private func settled(at wide: Double, in id: UInt64 = 1) async -> Double {
        var last = await width(id)
        for _ in 0 ..< Patience.steps where abs(last - wide) > 1 {
            try? await Task.sleep(for: .milliseconds(10))
            last = await width(id)
        }
        return last
    }

    func testAZoomedPageIsLaidOutInLessRoomAndPutBackInAllOfIt() async {
        await page()
        let actual = await width()
        XCTAssertEqual(actual, 800, accuracy: 1, "at actual size the page is laid out as wide as the web view")
        let line = await lineHeight()
        XCTAssertEqual(line, 20, accuracy: 1)

        guard case .ack = await zoom(200) else { return XCTFail("a zoom is acknowledged") }
        let doubled = await settled(at: actual / 2)
        XCTAssertEqual(doubled, actual / 2, accuracy: 1, "twice the size: half the room")
        let zoomedLine = await lineHeight()
        XCTAssertEqual(zoomedLine, line, accuracy: 1, "and the text is as tall as it was in the page's units: twice its share of the room")

        await zoom(50)
        let halved = await settled(at: actual * 2)
        XCTAssertEqual(halved, actual * 2, accuracy: 1, "half the size: twice the room")

        await zoom(100)
        let back = await settled(at: actual)
        XCTAssertEqual(back, actual, accuracy: 1)
    }

    func testTheZoomStaysForThePageTheWebViewGoesOnTo() async {
        await page()
        let actual = await width()
        await zoom(200)
        _ = await settled(at: actual / 2)
        let loads = reports(1).count(where: {
            if case .navigationFinished = $0 {
                true
            } else {
                false
            }
        })

        // Another page that says it is as wide as the device: the same one, by another address.
        await load(1, site.url("/sized?again=1"))
        await wait(1, "the next page to load") { reports in
            reports.count(where: {
                if case .navigationFinished = $0 {
                    true
                } else {
                    false
                }
            }) > loads
        }

        let next = await settled(at: actual / 2)
        XCTAssertEqual(next, actual / 2, accuracy: 1, "the web view keeps its zoom: the core asks again only for another level")
    }

    func testAZoomIsOneWebViewsOwn() async {
        await page(1)
        await page(2)
        let actual = await width(2)

        await zoom(200, in: 1)
        _ = await settled(at: actual / 2, in: 1)

        let other = await width(2)
        XCTAssertEqual(other, actual, accuracy: 1, "the other web view is as it was")
    }

    func testALevelNoPageCanBeReadAtIsHeldToOneThatCan() async {
        await page()
        let actual = await width()

        await zoom(5)
        let smallest = await settled(at: actual * 2)
        XCTAssertEqual(smallest, actual * 2, accuracy: 1, "no smaller than half")

        await zoom(50000)
        let largest = await settled(at: actual / 5)
        XCTAssertEqual(largest, actual / 5, accuracy: 1, "no larger than five times")
    }

    func testAWebViewThatIsNotThereIsRefusedWithACode() async {
        guard case let .error(error) = await zoom(200, in: 99) else { return XCTFail("expected a refusal") }
        XCTAssertEqual(error.code, "webview.unknown")
    }
}

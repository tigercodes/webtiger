// The adapter suite's WebKit run, for a link for the system (007): a letter to write, a number
// to ring, a message to send. The adapter refuses such an address in the web view, as before,
// and reports it; it opens nothing. Whether the address is handed to the system is the core's
// to say, and is tested there.
import WebKit
import WTContract
import WTCoreClient
import WTWebKit
import XCTest

@MainActor
final class WebKitOutsideLinkTests: AdapterTestCase {
    /// As the core's policy does: these are for the system.
    private func theSystems(_ urls: String...) {
        for url in urls {
            core.setVerdict(.external, for: url)
        }
    }

    /// The links for the system a web view's page has asked for, in order.
    private func outsideLinks(_ id: UInt64) -> [WTOutsideLinkRequested] {
        reports(id).compactMap {
            if case let .outsideLinkRequested(asked) = $0 {
                asked
            } else {
                nil
            }
        }
    }

    private func contact(_ id: UInt64 = 1) async {
        theSystems("mailto:ada@example.com", "tel:+15550100", "sms:+15550100")
        await create(id)
        await load(id, site.url("/contact"))
        await wait(id, "the page to load") { self.finished($0) && self.hasTitle("Contact")($0) }
    }

    func testAClickedLinkIsReportedBeforeItsRefusal() async {
        await contact()
        let before = names(reports(1)).count

        await run("document.getElementById('letter').click()", in: 1)
        await wait(1, "the link to be reported") { _ in !self.outsideLinks(1).isEmpty }

        XCTAssertEqual(outsideLinks(1).map(\.url), ["mailto:ada@example.com"])
        XCTAssertEqual(outsideLinks(1).map(\.userGesture), [true], "a link that was clicked")
        // Reported first, then refused like any navigation the policy does not allow: the core
        // must hear of the link before the refusal that may close a window (006).
        await wait(1, "the refusal") { self.names($0).contains("failed(net.cancelled)") }
        XCTAssertEqual(Array(names(reports(1)).dropFirst(before)), ["outside", "failed(net.cancelled)"])
        // The page is where it was.
        XCTAssertEqual(lastTitle(1), "Contact")
        let location = await run("location.pathname", in: 1) as? String
        XCTAssertEqual(location, "/contact")
    }

    func testAScriptsNavigationIsReportedAsNotClickedFor() async {
        await contact()

        await run("document.getElementById('script').click()", in: 1)
        await wait(1, "the link to be reported") { _ in !self.outsideLinks(1).isEmpty }

        // WebKit does not say whether a script's navigation was clicked for, so the adapter
        // does not say it was: the core opens nothing for it.
        XCTAssertEqual(outsideLinks(1).map(\.url), ["tel:+15550100"])
        XCTAssertEqual(outsideLinks(1).map(\.userGesture), [false])
    }

    func testALinkInAFrameIsReportedToo() async {
        await contact()
        let before = names(reports(1)).count

        await run("document.getElementById('frame').contentDocument.getElementById('framed').click()", in: 1)
        await wait(1, "the link to be reported") { _ in !self.outsideLinks(1).isEmpty }
        try? await Task.sleep(for: .milliseconds(300))

        XCTAssertEqual(outsideLinks(1).map(\.url), ["sms:+15550100"])
        XCTAssertEqual(outsideLinks(1).map(\.userGesture), [true])
        // Nothing of the page itself was going anywhere, and nothing is said of it.
        XCTAssertEqual(Array(names(reports(1)).dropFirst(before)), ["outside"])
    }

    func testAPageThatGoesThereByItselfIsReportedAsNotClickedFor() async {
        theSystems("tel:+15550100")
        await create(1)
        await load(1, site.url("/rings"))
        await wait(1, "the page to have tried") { _ in !self.outsideLinks(1).isEmpty }

        XCTAssertEqual(outsideLinks(1).map(\.userGesture), [false], "nobody clicked")
    }

    func testALinkThatOpensAWindowOnSuchAnAddressIsReportedAndOpensNoWindow() async {
        theSystems("mailto:ada@example.com")
        await create(1)
        await load(1, site.url("/opener"))
        await wait(1, "the page to load") { self.finished($0) && self.hasTitle("Opener")($0) }
        let before = names(reports(1)).count

        await run("document.getElementById('letter').click()", in: 1)
        await wait(1, "the link to be reported") { _ in !self.outsideLinks(1).isEmpty }
        try? await Task.sleep(for: .milliseconds(300))

        // WebKit asks about a link's address before it asks for a window: the report is the
        // asking page's, and no window is made.
        XCTAssertEqual(outsideLinks(1).map(\.url), ["mailto:ada@example.com"])
        XCTAssertEqual(outsideLinks(1).map(\.userGesture), [true])
        XCTAssertEqual(Array(names(reports(1)).dropFirst(before)), ["outside"])
        XCTAssertEqual(engine.webViewCount, 1)
    }

    func testAWindowScriptOpensOnSuchAnAddressReportsItInTheWindow() async {
        theSystems("tel:+15550100")
        await create(1)
        await load(1, site.url("/opener"))
        await wait(1, "the page to load") { self.finished($0) && self.hasTitle("Opener")($0) }

        await run("document.getElementById('ring').click()", in: 1)
        let new = await awaitWindow(askedBy: 1).newWebViewID
        await wait(new, "the refusal") { self.names($0).contains("failed(net.cancelled)") }

        // For a window script opens, WebKit makes the window first and asks about the
        // address in it. The link is reported there, before the refusal the core closes the
        // window on; the click that opened the window is what the core counts (007 §3).
        XCTAssertEqual(outsideLinks(new).map(\.url), ["tel:+15550100"])
        XCTAssertEqual(names(reports(new)).filter { $0 == "outside" || $0.hasPrefix("failed") }, ["outside", "failed(net.cancelled)"])
        XCTAssertTrue(outsideLinks(1).isEmpty)
    }

    func testTheEngineItselfOpensNothing() async {
        await create(1)
        var open = WTOpenOutside()
        open.url = "mailto:ada@example.com"
        let answer = await engine.answer(.openOutside(open))
        guard case let .error(error) = answer else { return XCTFail("the System service is not a web engine's to answer") }
        XCTAssertEqual(error.code, "host.unsupported")
    }
}

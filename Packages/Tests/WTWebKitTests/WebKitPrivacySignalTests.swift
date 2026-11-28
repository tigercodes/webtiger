// Global Privacy Control (069), against real WebKit and the suite's own pages. What WebKit
// lets a shell do, which is what the feature was built on: a page's scripts read the signal
// in every frame, from before the page's own scripts run, and cannot make it say otherwise;
// a load the shell begins carries the header; what the page fetches of its own accord does
// not, and a web view whose pages are not told the signal has neither.
import WebKit
import WTContract
@testable import WTWebKit
import XCTest

@MainActor
final class WebKitPrivacySignalTests: AdapterTestCase {
    /// The header a request for a path came with, each time it was asked for: nil for a
    /// request that had none.
    private func signal(at path: String) -> [String?] {
        site.headers(of: path).map { $0["sec-gpc"] }
    }

    func testAPagesScriptsReadTheSignalAndCannotMakeItSayOtherwise() async {
        _ = await create(1, privacySignal: true)
        _ = await load(1, site.url("/gpc"))
        // What the page read first, and what it read after trying to write it and to make
        // it again: true both times.
        await wait(1, "the page to say what it read") { self.hasTitle("gpc=true then true")($0) }
        XCTAssertEqual(lastTitle(1), "gpc=true then true")
    }

    func testAFrameOfAnotherOriginReadsTheSignalToo() async {
        _ = await create(1, privacySignal: true)
        _ = await load(1, site.url("/gpc-in-a-frame"))
        await wait(1, "the frame to say what it read") { self.hasTitle("frame: gpc=true then true")($0) }
        XCTAssertEqual(lastTitle(1), "frame: gpc=true then true")
    }

    func testALoadTheShellBeginsCarriesTheHeaderAndWhatThePageFetchesDoesNot() async {
        _ = await create(1, privacySignal: true)
        _ = await load(1, site.url("/tracked"))
        await wait(1, "the page") { self.finished($0) }
        // The page itself, which the shell asked for: with the header.
        XCTAssertEqual(signal(at: "/tracked"), ["1"])
        // Its picture, which WebKit fetched of its own accord: WebKit gives a shell no way
        // to add to it, and it goes without.
        XCTAssertEqual(signal(at: "/pixel"), [nil])

        // A second load the shell begins, in the same web view: with it again.
        _ = await load(1, site.url("/landed"))
        await wait(1, "the second page") { self.finished($0, times: 2) }
        XCTAssertEqual(signal(at: "/landed"), ["1"])
    }

    func testALinkThePageFollowsByItselfGoesWithoutTheHeader() async {
        _ = await create(1, privacySignal: true)
        _ = await load(1, site.url("/"))
        await wait(1, "the page") { self.finished($0) }
        // The page's own link, followed in the page: the engine's navigation, which a
        // shell cannot add to.
        await run("document.getElementById('next').click()", in: 1)
        await wait(1, "the link's page") { self.hasTitle("Landed")($0) }
        XCTAssertEqual(signal(at: "/"), ["1"])
        XCTAssertEqual(signal(at: "/landed"), [nil])
        // Its scripts read the signal all the same: the script is the web view's.
        _ = await load(1, site.url("/gpc"))
        await wait(1, "the page to say what it read") { self.hasTitle("gpc=true then true")($0) }
    }

    func testAWindowAPageOpensIsToldAsThePageThatOpenedItIs() async {
        // The web view WebKit makes for a page's window is made from the asking page's
        // configuration, and so with its scripts.
        _ = await create(1, privacySignal: true)
        _ = await load(1, site.url("/opener"))
        await wait(1, "the page") { self.finished($0) }
        await run("document.getElementById('open').click()", in: 1)
        let new = await awaitWindow(askedBy: 1).newWebViewID
        await wait(new, "the window's page") { self.hasTitle("Opened by a page")($0) }
        let read = await run("String(navigator.globalPrivacyControl)", in: new) as? String
        XCTAssertEqual(read, "true")
        // And a load the shell begins in it carries the header.
        _ = await load(new, site.url("/landed"))
        await wait(new, "a page loaded in the window") { self.hasTitle("Landed")($0) }
        XCTAssertEqual(signal(at: "/landed"), ["1"])

        // And one opened by a page that is not told, is not.
        _ = await create(2)
        _ = await load(2, site.url("/opener"))
        await wait(2, "the page") { self.finished($0) }
        await run("document.getElementById('open').click()", in: 2)
        let other = await awaitWindow(askedBy: 2).newWebViewID
        await wait(other, "the window's page") { self.hasTitle("Opened by a page")($0) }
        let unread = await run("String(navigator.globalPrivacyControl)", in: other) as? String
        XCTAssertEqual(unread, "undefined")
        _ = await load(other, site.url("/landed"))
        await wait(other, "a page loaded in the other window") { self.hasTitle("Landed")($0) }
        XCTAssertEqual(signal(at: "/landed"), ["1", nil])
    }

    func testAWebViewWhosePagesAreNotToldHasNeither() async {
        _ = await create(1)
        _ = await load(1, site.url("/gpc"))
        // Nothing to read at first; and what the page then writes is the page's own.
        await wait(1, "the page to say what it read") { self.hasTitle("gpc=undefined then false")($0) }
        XCTAssertEqual(signal(at: "/gpc"), [nil])
    }

    func testTheHeaderIsOnALoadAndNowhereElse() {
        let url = URL(string: "https://example.com/a?b=c")
        let told = PrivacySignal.request(for: url ?? URL(fileURLWithPath: "/"), told: true)
        XCTAssertEqual(told.value(forHTTPHeaderField: "Sec-GPC"), "1")
        XCTAssertEqual(told.url, url)
        let untold = PrivacySignal.request(for: url ?? URL(fileURLWithPath: "/"), told: false)
        XCTAssertNil(untold.value(forHTTPHeaderField: "Sec-GPC"))
        XCTAssertEqual(untold.allHTTPHeaderFields ?? [:], [:])
    }
}

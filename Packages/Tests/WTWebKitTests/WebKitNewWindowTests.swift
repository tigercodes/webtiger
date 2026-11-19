// The adapter suite's WebKit run, for a page that asks for a new window (006): the one web
// view an adapter makes unasked. What is tested is the adapter alone: that it makes the web
// view WebKit asks for, names it, reports it, and keeps the hold between the two pages. What
// becomes of the web view is the core's to say, and is tested there.
import WebKit
import WTContract
import WTCoreClient
import WTWebKit
import XCTest

@MainActor
final class WebKitNewWindowTests: AdapterTestCase {
    /// A web view on the page that asks for windows when clicked.
    private func opener(_ id: UInt64 = 1, loginSet: UUID? = nil) async {
        await create(id, loginSet: loginSet)
        await load(id, site.url("/opener"))
        await wait(id, "the opener to load") { self.finished($0) && self.hasTitle("Opener")($0) }
    }

    func testALinkWithATargetAsksForAWindow() async {
        await opener()
        let before = names(reports(1))

        await run("document.getElementById('blank').click()", in: 1)
        let asked = await awaitWindow(askedBy: 1)

        let new = asked.newWebViewID
        XCTAssertNotEqual(new & WebKitEngine.ownID, 0, "the adapter named the web view, with an id of the shell's own")
        XCTAssertEqual(asked.url, site.url("/opened"), "where the page wants it to go")
        XCTAssertTrue(asked.userGesture)
        XCTAssertNotNil(engine.webView(for: new), "the web view is there before the core has said a word")
        XCTAssertEqual(engine.webViewCount, 2)

        // The address loads in the new web view, reported like any other's.
        await wait(new, "the new window's page") { self.finished($0) && self.hasTitle("Opened")($0) }
        let order = names(reports(new)).filter { ["started", "committed", "finished"].contains($0) }
        XCTAssertEqual(order, ["started", "committed", "finished"])
        let commits = reports(new).compactMap {
            if case let .navigationCommitted(committed) = $0 {
                committed.url
            } else {
                nil
            }
        }
        XCTAssertEqual(commits, [site.url("/opened")])
        // A link gives the page it opens no hold on the page it was in: only window.open does.
        XCTAssertEqual(lastTitle(new), "Opened", "the new page found no opener")

        // The web view that asked went nowhere: the one thing it has said since is that it asked.
        XCTAssertEqual(Array(names(reports(1)).dropFirst(before.count)), ["window"])
        XCTAssertEqual(lastTitle(1), "Opener")
    }

    func testAWindowOpenedByScriptKeepsTheHoldOfThePageThatOpenedIt() async {
        await opener()

        await run("document.getElementById('open').click()", in: 1)
        let new = await awaitWindow(askedBy: 1).newWebViewID

        // The new page has an opener, and says so; and it reached the page that opened it.
        await wait(new, "the new page to find its opener") { self.hasTitle("Opened by a page")($0) }
        await wait(1, "the opener to hear from the window it opened") { self.hasTitle("heard=hello")($0) }
    }

    func testTheNewWebViewIsInTheLoginSetOfThePageThatAsked() async {
        let loginSet = newLoginSet()
        await create(1, loginSet: loginSet)
        await load(1, site.url("/set?v=ada"))
        await wait(1, "the sign-in") { self.finished($0) && self.hasTitle("Set")($0) }
        await load(1, site.url("/opener"))
        await wait(1, "the opener to load") { self.hasTitle("Opener")($0) }

        await run("window.open('/whoami')", in: 1)
        let new = await awaitWindow(askedBy: 1).newWebViewID
        await wait(new, "the new window to say who is signed in") { self.hasTitle("who=ada")($0) }

        // And it is a web view like any other: the core's requests about it are answered.
        guard case .ack = await load(new, site.url("/landed")) else { return XCTFail("a load in it is acknowledged") }
        await wait(new, "the load the core asked for") { self.hasTitle("Landed")($0) }

        // Another login set's page that asks gets a web view in its own.
        await create(2)
        await load(2, site.url("/opener"))
        await wait(2, "the other opener") { self.hasTitle("Opener")($0) }
        await run("window.open('/whoami')", in: 2)
        let other = await awaitWindow(askedBy: 2).newWebViewID
        await wait(other, "the other window to say nobody is signed in") { self.hasTitle("who=none")($0) }
        XCTAssertNotEqual(other, new, "no id is used twice")
    }

    func testAPageAskingForItsWindowToBeClosedIsReported() async {
        await opener()
        await run("document.getElementById('open').click()", in: 1)
        let new = await awaitWindow(askedBy: 1).newWebViewID
        await wait(new, "the new page") { self.finished($0) && self.hasTitle("Opened by a page")($0) }

        await run("document.getElementById('close').click()", in: new)

        await wait(new, "the page to ask for its window to be closed") { self.names($0).contains("close") }
        // Asking is all a page can do: the web view is the core's to destroy.
        XCTAssertNotNil(engine.webView(for: new))
    }

    func testAWindowAskedForWithNobodyClickingIsNotMade() async {
        await create(1)
        await load(1, site.url("/pops"))
        // The page says what window.open gave it: nothing.
        await wait(1, "the page to have asked") { self.finished($0) && self.hasTitle("blocked")($0) }

        XCTAssertEqual(windowsAsked(by: 1).count, 0, "nothing was asked of the core")
        XCTAssertEqual(engine.webViewCount, 1, "and no web view was made")
    }

    func testAPageThatClicksItsOwnLinkGetsNoWindow() async {
        await create(1)
        await load(1, site.url("/clicks"))
        await wait(1, "the page to have clicked") { self.finished($0) && self.hasTitle("clicked")($0) }
        try? await Task.sleep(for: .milliseconds(500))

        XCTAssertEqual(windowsAsked(by: 1).count, 0, "nothing was asked of the core")
        XCTAssertEqual(engine.webViewCount, 1, "and no web view was made")
    }

    func testAWindowTheCoreRefusesIsDestroyed() async {
        await opener()
        await run("document.getElementById('open').click()", in: 1)
        let new = await awaitWindow(askedBy: 1).newWebViewID
        weak let webView = engine.webView(for: new)
        XCTAssertNotNil(webView)

        // What the core sends for a window it refuses.
        await destroy(new)

        XCTAssertNil(engine.webView(for: new))
        XCTAssertEqual(engine.webViewCount, 1)
        for _ in 0 ..< 200 where webView != nil {
            try? await Task.sleep(for: .milliseconds(10))
        }
        XCTAssertNil(webView, "released, not just forgotten")
        // The page that asked is as it was, and can ask again; the next window has a new id.
        XCTAssertEqual(lastTitle(1), "Opener")
        await run("document.getElementById('open').click()", in: 1)
        let next = await awaitWindow(askedBy: 1, count: 2).newWebViewID
        XCTAssertNotEqual(next, new)
        XCTAssertNotEqual(next & WebKitEngine.ownID, 0)
    }

    /// The failures a web view has reported, in order.
    private func failures(_ id: UInt64) -> [WTNavigationFailed] {
        reports(id).compactMap {
            if case let .navigationFailed(failed) = $0 {
                failed
            } else {
                nil
            }
        }
    }

    func testAWindowOpenedByScriptOnARefusedAddressSaysItWasRefused() async {
        await opener()
        // The core's policy refuses where the script sends the window.
        core.setVerdict(.cancel, for: site.url("/landed"))

        await run("window.open('/landed')", in: 1)
        let new = await awaitWindow(askedBy: 1).newWebViewID

        // For a window script opens, WebKit asks for the web view first and about the address
        // after, in the new web view. The refusal is reported there, as one: the core closes
        // the tab of a window that was refused the only place it was going.
        await wait(new, "the refusal") { _ in !self.failures(new).isEmpty }
        XCTAssertEqual(failures(new).map(\.code), ["net.cancelled"])
        XCTAssertEqual(failures(new).map(\.refused), [true])
        XCTAssertEqual(names(reports(new)).filter { $0 == "committed" }, [], "nothing committed in it")
    }

    func testAWindowRedirectedToARefusedAddressSaysItWasRefused() async {
        await opener()
        // The window's address is allowed; where the server sends it on to is not.
        core.setVerdict(.cancel, for: site.url("/landed"))

        await run("window.open('/redirect')", in: 1)
        let new = await awaitWindow(askedBy: 1).newWebViewID

        // The load began, so its end is what says it was refused.
        await wait(new, "the load's end") { _ in self.failures(new).contains(where: \.refused) }
        XCTAssertTrue(failures(new).allSatisfy { $0.code == "net.cancelled" })
        XCTAssertEqual(failures(new).last?.refused, true, "the last word on the load is that it was refused")
        XCTAssertEqual(names(reports(new)).filter { $0 == "committed" }, [])
    }

    func testALoadThatIsStoppedIsNotARefusal() async {
        await create(1)
        await load(1, site.url("/slow"))
        await wait(1, "the load to start") { self.names($0).contains("started") }

        await command(1, .stop)

        await wait(1, "the load's end") { _ in !self.failures(1).isEmpty }
        XCTAssertEqual(failures(1).map(\.code), ["net.cancelled"])
        XCTAssertEqual(failures(1).map(\.refused), [false], "stopped, not refused")
    }

    func testANewWindowTheCoreRefusesToNavigateSaysNothingOfThePageThatAsked() async {
        await opener()
        let before = names(reports(1))
        // The core's policy refuses where the link leads.
        core.setVerdict(.cancel, for: site.url("/opened"))

        await run("document.getElementById('blank').click()", in: 1)
        try? await Task.sleep(for: .milliseconds(500))

        XCTAssertEqual(windowsAsked(by: 1).count, 0, "a navigation the policy refuses opens no window")
        XCTAssertEqual(Array(names(reports(1)).dropFirst(before.count)), [], "and the page that asked did not fail: it went nowhere")
        XCTAssertEqual(engine.webViewCount, 1)
    }
}

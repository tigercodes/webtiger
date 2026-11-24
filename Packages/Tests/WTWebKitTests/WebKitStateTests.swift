// A web view's state through the adapter (051): what the core is handed when it asks for
// one, and what a web view made from it is. The core is scripted, and keeps nothing: the
// state goes from one web view to the next in the test's own hands, as the core's store would
// carry it. What WebKit's state is in itself (data, small, without a form's text) is held by
// `WebKitRestoreTests`; here it is the adapter's part: asking, handing back, falling back to
// the address, and what is reported on the way.
import WebKit
import WTContract
import WTCoreClient
import WTWebKit
import XCTest

@MainActor
final class WebKitStateTests: AdapterTestCase {
    /// The state of a web view, as the core asks for it. Nil when the adapter answers with
    /// anything but a state.
    private func state(of id: UInt64) async -> Data? {
        var save = WTWebViewSaveState()
        save.webViewID = id
        guard case let .webViewState(state) = await engine.answer(.webViewSaveState(save)) else { return nil }
        return state.state
    }

    /// Has a web view made from a state, with an address to fall back on, as the core asks.
    @discardableResult
    private func restore(_ id: UInt64, from state: Data, orLoad url: String) async -> WTHostResponse.OneOf_Kind {
        var restore = WTWebViewRestore()
        restore.webViewID = id
        restore.state = state
        restore.url = url
        return await engine.answer(.webViewRestore(restore))
    }

    /// A web view of a size a page can be scrolled in: the adapter makes them with none, and
    /// the window gives them theirs.
    private func createSized(_ id: UInt64, loginSet: UUID) async {
        await create(id, loginSet: loginSet)
        engine.webView(for: id)?.frame = CGRect(x: 0, y: 0, width: 800, height: 600)
    }

    /// Goes through Home and Landed to the tall page in web view 1, each by a load of its own.
    private func goToTheTallPage(in loginSet: UUID) async {
        await createSized(1, loginSet: loginSet)
        for (number, path) in ["/", "/landed", "/tall"].enumerated() {
            await load(1, site.url(path))
            await wait(1, "\(path) to finish") { self.finished($0, times: number + 1) }
        }
    }

    private func commits(_ id: UInt64) -> [String] {
        reports(id).compactMap {
            if case let .navigationCommitted(committed) = $0 {
                committed.url
            } else {
                nil
            }
        }
    }

    /// What the adapter last said of a web view's Back and Forward.
    private func history(_ id: UInt64) -> (back: Bool, forward: Bool)? {
        reports(id).reversed().compactMap {
            if case let .historyChanged(changed) = $0 {
                (back: changed.canGoBack, forward: changed.canGoForward)
            } else {
                nil
            }
        }.first
    }

    private func scrolled(_ id: UInt64) async -> Double {
        await run("window.scrollY", in: id) as? Double ?? -1
    }

    // MARK: A state given and taken

    func testAWebViewMadeFromAStateIsWhereTheOtherWasWithItsListAndItsScroll() async throws {
        let loginSet = newLoginSet()
        await goToTheTallPage(in: loginSet)
        await run("window.scrollTo(0, 2400)", in: 1)
        // WebKit writes where a page is scrolled to into its list a moment after the scroll.
        try await Task.sleep(for: .milliseconds(1200))
        let given = await state(of: 1)
        let kept = try XCTUnwrap(given, "a state is answered with a state")
        XCTAssertFalse(kept.isEmpty, "WebKit has a state to give")
        XCTAssertLessThan(kept.count, 512 * 1024, "under the most the core keeps (051 D3)")
        // The tab sleeps: its web view is gone before another is made.
        await destroy(1)

        await createSized(2, loginSet: loginSet)
        let answer = await restore(2, from: kept, orLoad: site.url("/tall"))
        guard case .ack = answer else { return XCTFail("a restore is acknowledged, not \(answer)") }
        await wait(2, "the page it was at") { self.finished($0) && self.hasTitle("Tall")($0) }

        // It arrives as any page does: a load that starts and commits, once, at the page the
        // other web view was at. Nothing said its address before it had committed.
        let order = names(reports(2)).filter { ["started", "committed", "finished", "address"].contains($0) }
        XCTAssertEqual(order, ["started", "committed", "finished"])
        XCTAssertEqual(commits(2), [site.url("/tall")])
        // The list came with it: Back goes back, and there is nowhere forward.
        let said = try XCTUnwrap(history(2), "its Back and Forward were said")
        XCTAssertTrue(said.back)
        XCTAssertFalse(said.forward)
        // And it is where it was scrolled to.
        var down = 0.0
        for _ in 0 ..< 300 {
            down = await scrolled(2)
            if down > 0 {
                break
            }
            try await Task.sleep(for: .milliseconds(10))
        }
        XCTAssertEqual(down, 2400, accuracy: 1)

        // Back goes to the page before it, which a web view that was only loaded could not.
        await command(2, .back)
        await wait(2, "the page before") { self.hasTitle("Landed")($0) && self.finished($0, times: 2) }
        XCTAssertEqual(commits(2), [site.url("/tall"), site.url("/landed")])
        XCTAssertEqual(history(2)?.forward, true)
    }

    func testAStateTakenInTheMiddleOfTheListHasSomewhereToGoForward() async throws {
        let loginSet = newLoginSet()
        await goToTheTallPage(in: loginSet)
        await command(1, .back)
        // Its fourth page: the one before, again. (Its title was said once already, on the
        // way to the tall page: the commit is what says it is back.)
        await wait(1, "the page before") { _ in self.commits(1).count == 4 && self.finished(self.reports(1), times: 4) }
        XCTAssertEqual(commits(1).last, site.url("/landed"))
        let given = await state(of: 1)
        let kept = try XCTUnwrap(given, "a state is answered with a state")
        await destroy(1)

        await createSized(2, loginSet: loginSet)
        await restore(2, from: kept, orLoad: site.url("/landed"))
        await wait(2, "the page it was at") { self.finished($0) && self.hasTitle("Landed")($0) }
        XCTAssertEqual(commits(2), [site.url("/landed")])
        let said = try XCTUnwrap(history(2))
        XCTAssertTrue(said.back)
        XCTAssertTrue(said.forward)
    }

    func testAPageOfAWebViewMadeFromAStateStillSaysWhenItChangesItsOwnAddress() async throws {
        let loginSet = newLoginSet()
        await goToTheTallPage(in: loginSet)
        let given = await state(of: 1)
        let kept = try XCTUnwrap(given, "a state is answered with a state")
        await destroy(1)
        await createSized(2, loginSet: loginSet)
        await restore(2, from: kept, orLoad: site.url("/tall"))
        await wait(2, "the page it was at") { self.finished($0) && self.hasTitle("Tall")($0) }
        XCTAssertFalse(names(reports(2)).contains("address"), "nothing but the commit said the page's address")

        // The page writes its own address, as a page may: that is said, as of any page.
        await run("history.pushState({}, '', '/elsewhere')", in: 2)
        await wait(2, "the page's own address") { reports in
            reports.contains {
                if case let .addressChanged(changed) = $0 {
                    changed.url == self.site.url("/elsewhere")
                } else {
                    false
                }
            }
        }
    }

    // MARK: A state that is not taken

    func testAStateWebKitWillNotTakeLoadsTheAddress() async throws {
        let loginSet = newLoginSet()
        // Bytes that are no state of anything; a state cut short; and none at all.
        await goToTheTallPage(in: loginSet)
        let given = await state(of: 1)
        let kept = try XCTUnwrap(given, "a state is answered with a state")
        await destroy(1)
        let cut = kept.prefix(kept.count / 2)
        for (number, bad) in [Data("no state of WebKit's".utf8), Data(cut), Data()].enumerated() {
            let id = UInt64(10 + number)
            await createSized(id, loginSet: loginSet)
            let answer = await restore(id, from: bad, orLoad: site.url("/landed"))
            guard case .ack = answer else { return XCTFail("the tab wakes all the same, not \(answer)") }
            await wait(id, "the address to load") { self.finished($0) && self.hasTitle("Landed")($0) }
            XCTAssertEqual(commits(id), [site.url("/landed")], "state \(number)")
            let said = try XCTUnwrap(history(id))
            XCTAssertFalse(said.back, "a web view that was loaded has been nowhere else")
            XCTAssertFalse(said.forward)
        }
    }

    func testAStateThatIsNotTakenByAWebViewThatHasAPageLoadsTheAddressToo() async throws {
        // The contract has no rule that a web view made from a state is a new one. One that
        // has a page, and a list of its own, is as it was when the state is not taken, and is
        // then sent to the address like any other.
        await create(1)
        await load(1, site.url("/"))
        await wait(1, "the page") { self.finished($0) && self.hasTitle("Home")($0) }
        let answer = await restore(1, from: Data("no state of WebKit's".utf8), orLoad: site.url("/landed"))
        guard case .ack = answer else { return XCTFail("the tab wakes all the same, not \(answer)") }
        await wait(1, "the address to load") { self.finished($0, times: 2) && self.hasTitle("Landed")($0) }
        XCTAssertEqual(commits(1), [site.url("/"), site.url("/landed")])
        let said = try XCTUnwrap(history(1))
        XCTAssertTrue(said.back, "its own list is as it was, with the address after it")
    }

    func testAWebViewThereIsNotHasNoStateAndTakesNone() async {
        var save = WTWebViewSaveState()
        save.webViewID = 99
        guard case let .error(none) = await engine.answer(.webViewSaveState(save)) else {
            return XCTFail("a web view there is not has no state")
        }
        XCTAssertEqual(none.code, "webview.unknown")
        guard case let .error(refused) = await restore(99, from: Data("anything".utf8), orLoad: site.url("/")) else {
            return XCTFail("a web view there is not takes no state")
        }
        XCTAssertEqual(refused.code, "webview.unknown")
    }

    func testAWebViewThatHasBeenNowhereHasAStateThatLoadsTheAddress() async throws {
        // A web view that has loaded nothing: whatever WebKit gives for it, a web view made
        // from it ends at the address it was to fall back on.
        let loginSet = newLoginSet()
        await createSized(1, loginSet: loginSet)
        let given = await state(of: 1)
        let kept = try XCTUnwrap(given, "a state is answered with a state")
        await destroy(1)
        await createSized(2, loginSet: loginSet)
        await restore(2, from: kept, orLoad: site.url("/"))
        await wait(2, "the address to load") { self.finished($0) && self.hasTitle("Home")($0) }
        XCTAssertEqual(commits(2), [site.url("/")])
    }

    // MARK: The navigation policy

    func testAPageThatAStateSaysIsAskedOfTheCoreLikeAnyOther() async throws {
        let loginSet = newLoginSet()
        await goToTheTallPage(in: loginSet)
        let given = await state(of: 1)
        let kept = try XCTUnwrap(given, "a state is answered with a state")
        await destroy(1)

        // The core refuses the page the state is at: it does not load, and is never said to
        // be the page's address.
        core.setVerdict(.cancel, for: site.url("/tall"))
        await createSized(2, loginSet: loginSet)
        await restore(2, from: kept, orLoad: site.url("/tall"))
        await wait(2, "the refusal") { reports in
            reports.contains {
                if case let .navigationFailed(failed) = $0 {
                    failed.code == "net.cancelled"
                } else {
                    false
                }
            }
        }
        XCTAssertEqual(commits(2), [])
        XCTAssertFalse(names(reports(2)).contains("address"), "an address that never committed is never reported as the page's")
    }

    // MARK: Kept apart

    func testAStateDoesNotCarryItsLoginSetWithIt() async throws {
        // A state is a list of pages, not who is signed in: made into a web view of another
        // login set, it is that login set's page.
        let ada = newLoginSet()
        let grace = newLoginSet()
        await createSized(1, loginSet: ada)
        await load(1, site.url("/set?v=ada"))
        await wait(1, "the cookie to be set") { self.finished($0) }
        await load(1, site.url("/whoami"))
        await wait(1, "who is asked") { self.hasTitle("who=ada")($0) }
        let given = await state(of: 1)
        let kept = try XCTUnwrap(given, "a state is answered with a state")
        await destroy(1)

        await createSized(2, loginSet: grace)
        await restore(2, from: kept, orLoad: site.url("/whoami"))
        await wait(2, "who is asked, in the other login set") { self.finished($0) }
        XCTAssertEqual(lastTitle(2), "who=none", "the other login set is signed in as nobody")
        // And in its own, it is still signed in.
        await createSized(3, loginSet: ada)
        await restore(3, from: kept, orLoad: site.url("/whoami"))
        await wait(3, "who is asked, in its own") { self.hasTitle("who=ada")($0) }
    }
}

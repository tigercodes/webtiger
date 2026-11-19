// The adapter suite's WebKit run, for find on the page (010): what WebKit's own find does when
// the adapter asks it, and what the adapter reports of it. WebKit shows a match by selecting
// it, says whether there was one, and does not count.
import WebKit
import WTContract
import WTCoreClient
import WTWebKit
import XCTest

@MainActor
final class WebKitFindTests: AdapterTestCase {
    /// A web view on the page with one word three times.
    private func page(_ id: UInt64 = 1) async {
        await create(id)
        await load(id, site.url("/find"))
        await wait(id, "the page to load") { self.finished($0) && self.hasTitle("Find")($0) }
    }

    @discardableResult
    private func find(_ text: String, backwards: Bool = false, in id: UInt64 = 1) async -> WTHostResponse.OneOf_Kind {
        var find = WTWebViewFind()
        find.webViewID = id
        find.text = text
        find.backwards = backwards
        return await engine.answer(.webViewFind(find))
    }

    /// What the adapter has reported of finds in a web view, in order.
    private func results(_ id: UInt64 = 1) -> [WTFindResult] {
        reports(id).compactMap {
            if case let .findResult(result) = $0 {
                result
            } else {
                nil
            }
        }
    }

    /// Finds, and waits for what was found.
    private func found(_ text: String, backwards: Bool = false) async -> WTFindResult? {
        let before = results().count
        await find(text, backwards: backwards)
        await wait(1, "the find's result") { _ in self.results().count > before }
        return results().last
    }

    /// The page's selection: its text, and the paragraph it is in.
    private func selection() async -> String {
        let script = """
        (() => {
            const chosen = getSelection();
            if (!chosen || chosen.rangeCount === 0 || chosen.isCollapsed) return '';
            return chosen.toString() + ' in ' + (chosen.anchorNode.parentElement.id || '?');
        })()
        """
        return await run(script, in: 1) as? String ?? "nothing came back"
    }

    func testTextThatIsThereIsFoundAndShown() async {
        await page()

        let result = await found("needle")

        XCTAssertEqual(result?.found, true)
        XCTAssertEqual(result?.counted, false, "WebKit does not count")
        let shown = await selection()
        XCTAssertEqual(shown, "needle in one", "the first match is the page's selection")
    }

    func testTheSameTextAgainGoesToTheNextAndRoundToTheFirst() async {
        await page()

        var places: [String] = []
        for _ in 0 ..< 4 {
            let result = await found("needle")
            XCTAssertEqual(result?.found, true)
            await places.append(selection())
        }

        // Whatever its case: the second is written with a capital.
        XCTAssertEqual(places, ["needle in one", "Needle in two", "needle in three", "needle in one"])
    }

    func testBackwardsGoesToTheOneBefore() async {
        await page()
        _ = await found("needle")
        _ = await found("needle")
        let second = await selection()
        XCTAssertEqual(second, "Needle in two")

        _ = await found("needle", backwards: true)
        let first = await selection()
        XCTAssertEqual(first, "needle in one")
        // And round, from the first to the last.
        _ = await found("needle", backwards: true)
        let last = await selection()
        XCTAssertEqual(last, "needle in three")
    }

    func testTextThatIsNotThereIsNotFound() async {
        await page()

        let result = await found("zebra")

        XCTAssertEqual(result?.found, false)
        let shown = await selection()
        XCTAssertEqual(shown, "", "and nothing is shown")
    }

    func testEmptyTextClearsWhatWasShown() async {
        await page()
        _ = await found("needle")
        let shown = await selection()
        XCTAssertEqual(shown, "needle in one")
        let before = results().count

        guard case .ack = await find("") else { return XCTFail("stopping is acknowledged") }

        for _ in 0 ..< 200 {
            if await selection().isEmpty {
                break
            }
            try? await Task.sleep(for: .milliseconds(10))
        }
        let after = await selection()
        XCTAssertEqual(after, "", "the marks are gone")
        XCTAssertEqual(results().count, before, "and nothing is reported of nothing")
    }

    func testAWebViewThatIsNotThereIsRefusedWithACode() async {
        guard case let .error(error) = await find("needle", in: 99) else { return XCTFail("expected a refusal") }
        XCTAssertEqual(error.code, "webview.unknown")
    }
}

// The History list's model (008) against a scripted core: what it asks, what it shows of the
// answers, and what it sends to have visits forgotten.
import Foundation
import WTContract
import WTCoreClient
@testable import WTUI
import XCTest

@MainActor
final class HistoryModelTests: XCTestCase {
    private func entry(_ id: UInt64, _ title: String) -> WTHistoryEntry {
        var entry = WTHistoryEntry()
        entry.visitID = id
        entry.title = title
        entry.url = "https://example.com/\(id)"
        entry.display = "example.com/\(id)"
        return entry
    }

    /// A core with six visits, which answers two at a time, and for words with one visit
    /// named after them.
    private func core() -> ScriptedCore {
        let visits = (1 ... 6).reversed().map { entry(UInt64($0), "Page \($0)") }
        return ScriptedCore { request in
            guard case let .queryHistory(query) = request else { return .ack(WTAck()) }
            var page = WTHistoryPage()
            if query.text.isEmpty {
                let older = visits.filter { query.beforeVisitID == 0 || $0.visitID < query.beforeVisitID }
                page.entries = Array(older.prefix(2))
                page.more = older.count > 2
            } else {
                var found = WTHistoryEntry()
                found.visitID = 99
                found.title = "Found by \(query.text)"
                page.entries = [found]
            }
            return .historyPage(page)
        }
    }

    private func settle(until done: () -> Bool) async {
        for _ in 0 ..< 200 where !done() {
            await Task.yield()
            try? await Task.sleep(for: .milliseconds(2))
        }
    }

    private func titles(_ model: HistoryModel) -> [String] {
        model.entries.map(\.title)
    }

    func testItAsksForTheWindowsNewestVisitsAndShowsThem() async {
        let core = core()
        let model = HistoryModel(core: core, windowID: 7)
        XCTAssertFalse(model.loaded, "nothing is said before the core has answered")

        model.reload()
        await settle { model.loaded }

        guard case let .queryHistory(query) = core.requests.last else { return XCTFail("expected QueryHistory") }
        XCTAssertEqual(query.windowID, 7)
        XCTAssertEqual(query.text, "")
        XCTAssertEqual(query.beforeVisitID, 0, "from the newest")
        XCTAssertEqual(titles(model), ["Page 6", "Page 5"])
        XCTAssertTrue(model.more)
    }

    func testOlderVisitsAreAskedForAfterTheLastOneShown() async {
        let core = core()
        let model = HistoryModel(core: core, windowID: 7)
        model.reload()
        await settle { model.loaded }

        model.showOlder()
        await settle { model.entries.count == 4 }

        guard case let .queryHistory(query) = core.requests.last else { return XCTFail("expected QueryHistory") }
        XCTAssertEqual(query.beforeVisitID, 5, "older than the last one shown")
        XCTAssertEqual(titles(model), ["Page 6", "Page 5", "Page 4", "Page 3"])
        XCTAssertTrue(model.more)

        model.showOlder()
        await settle { model.entries.count == 6 }
        XCTAssertFalse(model.more)
        // With no more, nothing is asked.
        let asked = core.requests.count
        model.showOlder()
        XCTAssertEqual(core.requests.count, asked)
    }

    func testTypingWordsAsksAgainAndOnlyTheLastAnswerIsShown() async {
        let core = core()
        let model = HistoryModel(core: core, windowID: 7)
        model.reload()

        // Typed one letter after the other, each asked for at once.
        model.text = "r"
        model.text = "ru"
        model.text = "rust"
        await settle { self.titles(model) == ["Found by rust"] }

        XCTAssertEqual(titles(model), ["Found by rust"], "not an answer to what was typed on the way")
        let texts = core.requests.compactMap { request -> String? in
            if case let .queryHistory(query) = request {
                query.text
            } else {
                nil
            }
        }
        XCTAssertEqual(texts, ["", "r", "ru", "rust"])
        // The same words again ask nothing.
        model.text = "rust"
        XCTAssertEqual(core.requests.count, 4)
    }

    func testForgettingAVisitSendsItAndTakesItOffTheList() async {
        let core = core()
        let model = HistoryModel(core: core, windowID: 7)
        model.reload()
        await settle { model.loaded }

        model.forget(model.entries[0])

        XCTAssertEqual(titles(model), ["Page 5"], "gone from the list at once")
        guard case let .forgetHistory(forget) = core.requests.last else { return XCTFail("expected ForgetHistory") }
        XCTAssertEqual(forget.windowID, 7)
        XCTAssertEqual(forget.what, .visitID(6))
    }

    func testForgettingManyAsksAgainForWhatIsLeft() async {
        let core = core()
        let model = HistoryModel(core: core, windowID: 7)
        model.reload()
        await settle { model.loaded }

        model.forget(since: Date(timeIntervalSince1970: 1_767_268_800.5))
        model.forgetEverything()

        // Sent in the order made: each forgetting, then the question that follows it.
        let kinds = core.requests.dropFirst().map { request -> String in
            switch request {
            case let .forgetHistory(forget):
                switch forget.what {
                case let .sinceMs(since): "forget since \(since)"
                case .everything(true): "forget everything"
                default: "forget something else"
                }
            case .queryHistory: "ask"
            default: "other"
            }
        }
        XCTAssertEqual(kinds, ["forget since 1767268800500", "ask", "forget everything", "ask"])
    }
}

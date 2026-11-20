// The Downloads list's model (013) against a scripted core: what it asks, what it shows, and
// what its words are for how a download stands.
import WTContract
import WTCoreClient
@testable import WTUI
import XCTest

@MainActor
final class DownloadsModelTests: XCTestCase {
    private func entry(
        _ id: UInt64,
        _ name: String,
        _ state: WTDownloadState = .finished,
        received: UInt64 = 0,
        total: UInt64 = 0,
        code: String = ""
    ) -> WTDownloadEntry {
        var entry = WTDownloadEntry()
        entry.id = id
        entry.fileName = name
        entry.state = state
        entry.receivedBytes = received
        entry.totalBytes = total
        entry.errorCode = code
        return entry
    }

    /// A core that answers a question about downloads with `pages`: the first for the newest,
    /// the second for those before it.
    private func core(_ newest: [WTDownloadEntry], older: [WTDownloadEntry] = []) -> ScriptedCore {
        ScriptedCore { request in
            guard case let .queryDownloads(query) = request else { return .ack(WTAck()) }
            var page = WTDownloadsPage()
            page.downloads = query.beforeID == 0 ? newest : older
            page.more = query.beforeID == 0 && !older.isEmpty
            return .downloadsPage(page)
        }
    }

    /// Lets the model's request tasks run.
    private func settle(until done: () -> Bool = { false }) async {
        for _ in 0 ..< 200 where !done() {
            await Task.yield()
            try? await Task.sleep(for: .milliseconds(2))
        }
    }

    private func queries(_ core: ScriptedCore) -> [WTQueryDownloads] {
        core.requests.compactMap {
            if case let .queryDownloads(query) = $0 {
                query
            } else {
                nil
            }
        }
    }

    func testTheListIsWhatTheCoreSaysNewestFirstAndOlderWhenAskedFor() async {
        let core = core([entry(3, "c.pdf"), entry(2, "b.pdf")], older: [entry(1, "a.pdf")])
        let model = DownloadsModel(core: core, windowID: 7)
        XCTAssertFalse(model.loaded, "nothing is said until the core has answered")

        model.reload()
        await settle { model.loaded }

        XCTAssertEqual(model.entries.map(\.fileName), ["c.pdf", "b.pdf"])
        XCTAssertTrue(model.more)
        XCTAssertEqual(queries(core).map(\.windowID), [7], "the window's own")

        model.showOlder()
        await settle { model.entries.count == 3 }

        XCTAssertEqual(model.entries.map(\.fileName), ["c.pdf", "b.pdf", "a.pdf"])
        XCTAssertFalse(model.more)
        XCTAssertEqual(queries(core).last?.beforeID, 2, "older than the last one shown")
    }

    func testStoppingAsksTheCoreAndTheListIsAskedForAgain() async {
        let coming = entry(5, "big.zip", .coming, received: 10, total: 100)
        let core = core([coming])
        let model = DownloadsModel(core: core, windowID: 7)
        model.reload()
        await settle { model.loaded }
        XCTAssertTrue(model.isComing)

        model.stop(coming)
        await settle { self.queries(core).count == 2 }

        guard case let .cancelDownload(cancel) = core.requests[1]
        else { return XCTFail("expected CancelDownload, then the question again") }
        XCTAssertEqual(cancel.downloadID, 5)
        XCTAssertEqual(model.entries.map(\.id), [5], "it stays in the list: how it ended is the core's to say")
    }

    func testForgettingOneLeavesTheListAtOnceAndForgettingAllAsksAgain() async {
        let core = core([entry(3, "c.pdf"), entry(2, "b.pdf")])
        let model = DownloadsModel(core: core, windowID: 7)
        model.reload()
        await settle { model.loaded }

        model.forget(model.entries[0])
        XCTAssertEqual(model.entries.map(\.fileName), ["b.pdf"], "at once, before any answer")
        guard case let .forgetDownloads(one) = core.requests.last else { return XCTFail("expected ForgetDownloads") }
        XCTAssertEqual(one.windowID, 7)
        XCTAssertEqual(one.downloadID, 3)

        model.forgetEverything()
        await settle { self.queries(core).count == 2 }
        let all = core.requests.compactMap { request -> WTForgetDownloads? in
            if case let .forgetDownloads(forget) = request {
                forget
            } else {
                nil
            }
        }
        XCTAssertEqual(all.last?.everything, true)
    }

    func testWhileTheListIsShownItIsAskedForAgainEverySecond() async {
        let core = core([entry(5, "big.zip", .coming, received: 10, total: 100)])
        let model = DownloadsModel(core: core, windowID: 7)

        let watching = Task { await model.watch() }
        try? await Task.sleep(for: .milliseconds(2300))
        watching.cancel()
        await watching.value
        let asked = queries(core).count
        try? await Task.sleep(for: .milliseconds(1200))

        XCTAssertGreaterThanOrEqual(asked, 3, "at once, and about every second after")
        XCTAssertLessThanOrEqual(asked, 4)
        XCTAssertEqual(queries(core).count, asked, "and no more once the list is gone")
    }

    func testHowADownloadStandsIsSaidInWords() {
        let megabyte: UInt64 = 1_000_000
        XCTAssertEqual(
            DownloadsView.standing(entry(1, "a", .coming, received: megabyte, total: 4 * megabyte)),
            "\(DownloadsView.size(megabyte)) of \(DownloadsView.size(4 * megabyte))"
        )
        XCTAssertEqual(DownloadsView.standing(entry(1, "a", .coming, received: megabyte)), "\(DownloadsView.size(megabyte)) so far")
        XCTAssertEqual(DownloadsView.standing(entry(1, "a", .finished, total: megabyte)), DownloadsView.size(megabyte))
        XCTAssertEqual(DownloadsView.standing(entry(1, "a", .finished)), "", "a size nobody said is not made up")
        XCTAssertEqual(DownloadsView.standing(entry(1, "a", .failed, code: "download.cancelled")), "Stopped")
        XCTAssertEqual(DownloadsView.standing(entry(1, "a", .failed, code: "download.network")), "The connection was lost.")
        XCTAssertEqual(DownloadsView.standing(entry(1, "a", .failed, code: "download.disk")), "The file couldn’t be saved.")
        XCTAssertEqual(
            DownloadsView.standing(entry(1, "a", .failed, code: "download.interrupted")),
            "WebTiger was closed before it finished."
        )
        // A code this build has no words for still says something, and never the code's own text.
        XCTAssertEqual(DownloadsView.standing(entry(1, "a", .failed, code: "download.something_new")), "The download failed.")
        XCTAssertEqual(DownloadsView.standing(entry(1, "a", .failed, code: "../../etc")), "The download failed.")
    }
}

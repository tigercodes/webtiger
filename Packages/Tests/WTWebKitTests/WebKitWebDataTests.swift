// What pages keep, kept in memory (001 §7): an engine made so, as the app's is in fake mode,
// gives each login set a store of its own and writes none of it down. What a page kept is
// there while the engine lives, apart from every other login set's, and is gone with it: so
// that one run of a suite leaves nothing for the next.
import WebKit
import WTContract
import WTCoreClient
import WTWebKit
import XCTest

@MainActor
final class WebKitWebDataTests: AdapterTestCase {
    /// Makes a web view of a login set in an engine that is not the case's own.
    private func create(_ id: UInt64, _ loginSet: UUID, in other: WebKitEngine, line: UInt = #line) async {
        var create = WTWebViewCreate()
        create.webViewID = id
        create.engine = .webkit
        create.loginSet.id = loginSet.uuidString.lowercased()
        guard case .ack = await other.answer(.webViewCreate(create)) else { return XCTFail("web view \(id) was not made", line: line) }
    }

    private func destroy(_ id: UInt64, in other: WebKitEngine) async {
        var destroy = WTWebViewDestroy()
        destroy.webViewID = id
        _ = await other.answer(.webViewDestroy(destroy))
    }

    /// Loads a page of the site in a web view of any engine, and waits for its title.
    private func open(_ id: UInt64, _ path: String, titled title: String, in other: WebKitEngine, line: UInt = #line) async {
        var load = WTWebViewLoad()
        load.webViewID = id
        load.url = site.url(path)
        _ = await other.answer(.webViewLoad(load))
        await wait(id, "\(path) to say \(title)", line: line) { self.finished($0) && self.hasTitle(title)($0) }
    }

    /// Who the site says is signed in, asked from a web view of any engine.
    private func who(_ id: UInt64, in other: WebKitEngine, line: UInt = #line) async -> String {
        var load = WTWebViewLoad()
        load.webViewID = id
        load.url = site.url("/whoami")
        _ = await other.answer(.webViewLoad(load))
        await wait(id, "who is signed in", line: line) { self.finished($0) && self.lastTitle(id).hasPrefix("who=") }
        return lastTitle(id)
    }

    /// An engine that keeps what pages keep in memory, reporting to the case's core.
    private func inMemory() -> WebKitEngine {
        let made = WebKitEngine(webData: .inMemory)
        made.attach(core)
        return made
    }

    func testWhatAPageKeepsInMemoryIsThereWhileTheEngineLivesAndGoneWithIt() async {
        // An engine keeps, unless it is made otherwise.
        XCTAssertEqual(engine.webData, .kept)
        let loginSet = newLoginSet()
        do {
            // Signed in with a cookie that is to be kept for an hour.
            let first = inMemory()
            await create(11, loginSet, in: first)
            await open(11, "/set?v=ada", titled: "Set", in: first)
            let signedIn = await who(11, in: first)
            XCTAssertEqual(signedIn, "who=ada")
            // A tab that sleeps wakes with what it had: its web view goes, and the login set's
            // next one is signed in still. The engine holds the store.
            await destroy(11, in: first)
            await create(12, loginSet, in: first)
            let woken = await who(12, in: first)
            XCTAssertEqual(woken, "who=ada", "the login set's store is held while the engine lives")
            await destroy(12, in: first)
        }
        // The app starts again: another engine, the same login set. Nobody is signed in.
        let second = inMemory()
        await create(13, loginSet, in: second)
        let again = await who(13, in: second)
        XCTAssertEqual(again, "who=none", "what was kept in memory went with the engine")
        await destroy(13, in: second)
        // And nothing was written down: an engine that keeps finds nothing of the login set's.
        await create(14, loginSet: loginSet)
        let kept = await who(14, in: engine)
        XCTAssertEqual(kept, "who=none", "nothing of it is on disk")

        // The same done with engines that keep: the second finds who the first signed in. (So
        // this test can tell the two apart.)
        let other = newLoginSet()
        await create(15, loginSet: other)
        await open(15, "/set?v=grace", titled: "Set", in: engine)
        await destroy(15)
        let next = WebKitEngine()
        next.attach(core)
        await create(16, other, in: next)
        let found = await who(16, in: next)
        XCTAssertEqual(found, "who=grace", "an engine that keeps, keeps")
        await destroy(16, in: next)
    }

    func testLoginSetsShareNothingInMemoryEither() async {
        let memory = inMemory()
        let personal = newLoginSet()
        let work = newLoginSet()
        await create(21, personal, in: memory)
        await create(22, work, in: memory)
        await create(23, personal, in: memory)
        await open(21, "/set?v=ada", titled: "Set", in: memory)
        let elsewhere = await who(22, in: memory)
        XCTAssertEqual(elsewhere, "who=none", "a login set kept in memory is its own, as one on disk is")
        let same = await who(23, in: memory)
        XCTAssertEqual(same, "who=ada")
        // Each signs in as somebody else, and stays who it is.
        await open(22, "/set?v=grace", titled: "Set", in: memory)
        let first = await who(21, in: memory)
        let second = await who(22, in: memory)
        XCTAssertEqual([first, second], ["who=ada", "who=grace"])
        for id: UInt64 in [21, 22, 23] {
            await destroy(id, in: memory)
        }
    }

    func testDeletingALoginSetKeptInMemoryLeavesNothing() async {
        let memory = inMemory()
        let loginSet = newLoginSet()
        await create(31, loginSet, in: memory)
        await open(31, "/set?v=ada", titled: "Set", in: memory)
        // Not while a web view uses it.
        let whileUsed = await memory.deleteLoginSet(loginSet)
        XCTAssertFalse(whileUsed)
        let still = await who(31, in: memory)
        XCTAssertEqual(still, "who=ada", "and nothing was deleted")
        await destroy(31, in: memory)
        let deleted = await memory.deleteLoginSet(loginSet)
        XCTAssertTrue(deleted)
        await create(32, loginSet, in: memory)
        let after = await who(32, in: memory)
        XCTAssertEqual(after, "who=none")
        await destroy(32, in: memory)
        // One that was never used has nothing to delete.
        let never = await memory.deleteLoginSet(UUID())
        XCTAssertTrue(never)
    }
}

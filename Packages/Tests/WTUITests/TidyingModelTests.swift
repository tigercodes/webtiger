// Tidying in the windows (049), against a scripted core: what the Archive's list asks and shows,
// what bringing a tab back and forgetting one send; and a Space's time, which the page that
// edits a Space asks for. The shell times nothing and keeps nothing: every test here is of
// what is asked of the core, and of what is shown of its answers.
import WTContract
import WTCoreClient
import WTDesign
@testable import WTUI
import WTWebKit
import XCTest

@MainActor
final class TidyingModelTests: XCTestCase {
    /// What the scripted core has in its Archive, and what it refuses.
    private final class Told: @unchecked Sendable {
        private let lock = NSLock()
        private var kept: [WTArchiveEntry] = (1 ... 5).reversed().map { Told.tidied(Int64($0), "Page \($0)") }
        private var refusesTimes: String?
        private var tidyAfter: UInt32 = 0

        static func tidied(_ id: Int64, _ title: String) -> WTArchiveEntry {
            var entry = WTArchiveEntry()
            entry.id = id
            entry.title = title
            entry.url = "https://example.com/\(id)"
            entry.display = "example.com/\(id)"
            entry.tidiedAt = 1_767_268_800_000 + id * 60000
            return entry
        }

        func refuseTimes(with code: String?) {
            lock.withLock { refusesTimes = code }
        }

        /// Another window took this one out of the Archive.
        func lose(_ id: Int64) {
            lock.withLock { kept.removeAll { $0.id == id } }
        }

        func answer(_ request: WTRequest.OneOf_Kind) throws -> WTResponse.OneOf_Kind? {
            try lock.withLock {
                switch request {
                case let .queryArchive(query):
                    // Two at a time, newest first.
                    let older = kept.filter { query.before == 0 || $0.id < query.before }
                    var page = WTArchivePage()
                    page.entries = Array(older.prefix(2))
                    page.more = older.count > 2
                    return .archivePage(page)
                case let .restoreArchived(restore):
                    guard kept.contains(where: { $0.id == restore.entryID }) else {
                        throw CoreError.refused(code: "archive.unknown", field: "entry_id")
                    }
                    kept.removeAll { $0.id == restore.entryID }
                    var opened = WTTabOpened()
                    opened.tabID = 9
                    return .tabOpened(opened)
                case let .forgetArchived(forget):
                    if forget.all {
                        kept = []
                    } else {
                        kept.removeAll { $0.id == forget.entryID }
                    }
                    return .ack(WTAck())
                case .querySpaces:
                    var page = WTSpacesPage()
                    var home = WTSpaceSummary()
                    home.spaceID = 1
                    var work = WTSpaceSummary()
                    work.spaceID = 2
                    work.name = "Work"
                    work.tidyAfterSeconds = tidyAfter
                    page.spaces = [home, work]
                    page.tidyTimes = [0, 43200, 86400, 604_800, 2_592_000]
                    return .spacesPage(page)
                case let .setSpaceTidying(set):
                    if let refusesTimes {
                        var refused = WTError()
                        refused.code = refusesTimes
                        return .error(refused)
                    }
                    tidyAfter = set.afterSeconds
                    return .ack(WTAck())
                default:
                    return nil
                }
            }
        }
    }

    private let told = Told()

    private func core() -> ScriptedCore {
        let told = told
        return ScriptedCore { request in
            if let answer = try told.answer(request) {
                return answer
            }
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
    }

    private func settle(until done: () -> Bool = { false }) async {
        for _ in 0 ..< 200 where !done() {
            await Task.yield()
            try? await Task.sleep(for: .milliseconds(2))
        }
    }

    private func titles(_ model: ArchiveModel) -> [String] {
        model.entries.map(\.title)
    }

    /// What was asked of the core about tidying, in order, in a few words each.
    private func asked(_ core: ScriptedCore) -> [String] {
        core.requests.compactMap {
            switch $0 {
            case let .queryArchive(ask): "list \(ask.windowID) before \(ask.before)"
            case let .restoreArchived(ask): "bring back \(ask.entryID) in \(ask.windowID)"
            case let .forgetArchived(ask): ask.all ? "forget all in \(ask.windowID)" : "forget \(ask.entryID) in \(ask.windowID)"
            case let .setSpaceTidying(ask): "tidy \(ask.spaceID) after \(ask.afterSeconds)"
            case let .renameSpace(ask): "name \(ask.spaceID) '\(ask.name)'"
            case let .colourSpace(ask): "colour \(ask.spaceID) '\(ask.colour)'"
            default: nil
            }
        }
    }

    // MARK: The Archive

    func testTheArchiveIsAskedOfTheWindowsSpaceNewestFirstAndOlderOnesAfterTheLastShown() async {
        let core = core()
        let model = ArchiveModel(core: core, windowID: 7)
        XCTAssertFalse(model.loaded, "nothing is said before the core has answered")

        model.reload()
        await settle { model.loaded }
        XCTAssertEqual(titles(model), ["Page 5", "Page 4"])
        XCTAssertTrue(model.more)

        model.showOlder()
        await settle { model.entries.count == 4 }
        XCTAssertEqual(titles(model), ["Page 5", "Page 4", "Page 3", "Page 2"])
        model.showOlder()
        await settle { model.entries.count == 5 }
        XCTAssertFalse(model.more)
        XCTAssertEqual(asked(core), ["list 7 before 0", "list 7 before 4", "list 7 before 2"])
        // With no more, nothing is asked.
        model.showOlder()
        XCTAssertEqual(asked(core).count, 3)
    }

    func testATabBroughtBackIsAskedOfTheCoreAndLeavesTheListAtOnce() async {
        let core = core()
        let model = ArchiveModel(core: core, windowID: 7)
        model.reload()
        await settle { model.loaded }

        let entry = model.entries[1]
        let back = await model.bringBack(entry)

        XCTAssertTrue(back, "the core made it a tab again")
        XCTAssertEqual(titles(model), ["Page 5"])
        XCTAssertEqual(asked(core).last, "bring back 4 in 7")
        // The model opens nothing by itself: the tab is the core's, and the window's snapshot
        // says so. Nothing else was asked.
        XCTAssertEqual(asked(core).count, 2)
    }

    func testATabTheArchiveHasNoLongerIsNotBroughtBackAndTheListIsAskedForAgain() async {
        let core = core()
        let model = ArchiveModel(core: core, windowID: 7)
        model.reload()
        await settle { model.loaded }
        let entry = model.entries[0]
        // Another window forgot it meanwhile.
        told.lose(entry.id)

        let back = await model.bringBack(entry)

        XCTAssertFalse(back)
        await settle { self.titles(model) == ["Page 4", "Page 3"] }
        XCTAssertEqual(titles(model), ["Page 4", "Page 3"], "what the Archive has now")
        XCTAssertEqual(asked(core), ["list 7 before 0", "bring back 5 in 7", "list 7 before 0"])
    }

    func testOneIsForgottenAndThenAllOfThem() async {
        let core = core()
        let model = ArchiveModel(core: core, windowID: 7)
        model.reload()
        await settle { model.loaded }

        model.forget(model.entries[0])
        XCTAssertEqual(titles(model), ["Page 4"], "it leaves the list at once")
        XCTAssertEqual(asked(core).last, "forget 5 in 7")

        model.forgetEverything()
        await settle { model.entries.isEmpty }
        XCTAssertEqual(titles(model), [])
        XCTAssertFalse(model.more)
        XCTAssertEqual(Array(asked(core).suffix(2)), ["forget all in 7", "list 7 before 0"], "and what is left is asked for")
    }

    // MARK: A Space's time

    /// A model whose window the core says shows the Space "Work".
    private func showingWork() async -> (BrowserModel, ScriptedCore) {
        let core = core()
        let model = BrowserModel(session: BrowserSession(core: core, engine: WebKitEngine()))
        await model.open()
        var window = WTWindowSnapshot()
        window.windowID = 1
        window.tab.tabID = 2
        window.spaceID = 2
        window.spaceName = "Work"
        core.emit(window, subscription: 3)
        await settle { model.spaceID == 2 }
        return (model, core)
    }

    func testTheTimesASpaceCanHaveAreTheCoresAndASpaceSaysItsOwn() async {
        let (model, _) = await showingWork()
        XCTAssertEqual(model.spaceTidyTimes, [], "nothing until the core has listed the Spaces")

        model.openSpaces()
        await settle { model.spaces != nil }
        XCTAssertEqual(model.spaceTidyTimes, [0, 43200, 86400, 604_800, 2_592_000])
        XCTAssertEqual(model.spaces?.map(\.tidyAfterSeconds), [0, 0], "a Space has no time until it is given one")

        let given = await model.tidySpace(after: 86400)
        XCTAssertTrue(given)
        model.openSpaces()
        await settle { model.spaces?.last?.tidyAfterSeconds == 86400 }
        XCTAssertEqual(model.spaces?.map(\.tidyAfterSeconds), [0, 86400], "what the list says is the core's word")
    }

    func testEditingASpaceAsksForItsTimeAfterItsNameAndItsColourAndOnlyWhenItIsAnother() async {
        let (model, core) = await showingWork()

        let timed = await model.editSpace(SpaceEdit(tidyAfter: 43200))
        let never = await model.editSpace(SpaceEdit(tidyAfter: 0))
        let all = await model.editSpace(SpaceEdit(name: "Desk", colour: "red", tidyAfter: 604_800))
        let nothing = await model.editSpace(SpaceEdit())

        XCTAssertTrue(timed && never && all && nothing)
        XCTAssertEqual(
            asked(core),
            ["tidy 2 after 43200", "tidy 2 after 0", "name 2 'Desk'", "colour 2 'red'", "tidy 2 after 604800"]
        )
    }

    func testATimeTheCoreWillNotTakeSaysWhyInWords() async {
        let (model, core) = await showingWork()
        told.refuseTimes(with: "tidy.time_unknown")

        let refused = await model.editSpace(SpaceEdit(tidyAfter: 3600))

        XCTAssertFalse(refused)
        XCTAssertEqual(model.spaceRefusal, "tidy.time_unknown")
        XCTAssertEqual(asked(core), ["tidy 2 after 3600"])
        // In words, and never by its code.
        let words = SpacesView.words(for: "tidy.time_unknown", unknown: Words.errorSpaceRenameFailed)
        XCTAssertEqual(words, "That isn’t a time tabs can be tidied after.")
        // Taken, it says nothing of before.
        told.refuseTimes(with: nil)
        let taken = await model.editSpace(SpaceEdit(tidyAfter: 43200))
        XCTAssertTrue(taken)
        XCTAssertNil(model.spaceRefusal)
    }

    func testBeforeTheCoreHasSaidWhichSpaceNoTimeIsAsked() async {
        let core = core()
        let model = BrowserModel(session: BrowserSession(core: core, engine: WebKitEngine()))
        await model.open()
        let asked = await model.tidySpace(after: 43200)
        XCTAssertFalse(asked)
        XCTAssertEqual(self.asked(core), [], "no Space is shown yet: there is none to give a time")
    }

    func testEveryTimeHasWordsAndAnotherNumberHasNone() {
        XCTAssertEqual(
            [0, 43200, 86400, 604_800, 2_592_000].map { TidyTime.words($0, locale: Locale(identifier: "en")) },
            ["Never", "After 12 hours", "After a day", "After a week", "After 30 days"]
        )
        // A time this build has no words for is not shown as a number.
        XCTAssertEqual(TidyTime.words(3600), "")
        XCTAssertEqual(TidyTime.words(1), "")
    }

    #if WEBTIGER_CORE
        /// The times the real core lists are the ones this shell has words for; and the real
        /// core takes each of them for a Space, and says so in its list.
        func testTheShellHasWordsForEveryTimeTheCoreListsAndTheCoreTakesEach() async throws {
            let session = try BrowserSession.live(environment: ["WEBTIGER_FAKE": "1"])
            defer { session.core.stop() }
            guard case let .spacesPage(page) = try await session.core.send(.querySpaces(WTQuerySpaces())).value else {
                return XCTFail("the core lists the Spaces")
            }
            XCTAssertEqual(page.tidyTimes.first, 0, "never is the first")
            XCTAssertGreaterThan(page.tidyTimes.count, 1)
            let space = try XCTUnwrap(page.spaces.first)
            XCTAssertEqual(space.tidyAfterSeconds, 0, "a Space is made with no time")
            for time in page.tidyTimes {
                XCTAssertFalse(TidyTime.words(time).isEmpty, "\(time)")
                var set = WTSetSpaceTidying()
                set.spaceID = space.spaceID
                set.afterSeconds = time
                guard case .ack = try await session.core.send(.setSpaceTidying(set)).value else {
                    return XCTFail("the core takes \(time)")
                }
                guard case let .spacesPage(after) = try await session.core.send(.querySpaces(WTQuerySpaces())).value else {
                    return XCTFail("the core lists the Spaces")
                }
                XCTAssertEqual(after.spaces.first?.tidyAfterSeconds, time)
            }
        }
    #endif
}

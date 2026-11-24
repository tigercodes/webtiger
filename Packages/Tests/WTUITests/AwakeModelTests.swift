// What is awake (056) in the windows, against a scripted core: what the list asks and shows,
// what "Sleep All Others" sends, and the words for why a tab is awake and where it is. The
// shell decides nothing about which tabs sleep: every test here is of what is asked of the
// core, and of what is shown of its answers. With the real core, that the core's answer is one
// this shell reads.
import WTContract
import WTCoreClient
@testable import WTUI
import XCTest

@MainActor
final class AwakeModelTests: XCTestCase {
    /// What the scripted core says is awake, and what it was asked.
    private final class Told: @unchecked Sendable {
        private let lock = NSLock()
        private var page = WTAwakePage()

        func say(_ tabs: [WTAwakeTab], privateTabs: UInt32 = 0, asleep: UInt32 = 0) {
            lock.withLock {
                page.tabs = tabs
                page.privateTabs = privateTabs
                page.asleep = asleep
            }
        }

        func answer(_ request: WTRequest.OneOf_Kind) -> WTResponse.OneOf_Kind {
            lock.withLock {
                switch request {
                case .queryAwake:
                    return .awakePage(page)
                case .sleepOthers:
                    // The core puts to sleep what can be: the tabs out of sight with no reason.
                    let slept = page.tabs.filter { !$0.inSight && $0.keepsAwake.isEmpty }
                    page.tabs.removeAll { !$0.inSight && $0.keepsAwake.isEmpty }
                    page.asleep += UInt32(slept.count)
                    return .ack(WTAck())
                default:
                    return .ack(WTAck())
                }
            }
        }
    }

    private let told = Told()

    private func core() -> ScriptedCore {
        let told = told
        return ScriptedCore { told.answer($0) }
    }

    private static func tab(
        _ id: UInt64, _ title: String, host: String = "example.com", space: String = "", inSight: Bool = false,
        privateWindow: Bool = false, _ reasons: [WTKeepAwake] = []
    ) -> WTAwakeTab {
        var tab = WTAwakeTab()
        tab.tabID = id
        tab.windowID = 7
        tab.title = title
        tab.host = host
        tab.spaceName = space
        tab.inSight = inSight
        tab.privateWindow = privateWindow
        tab.keepsAwake = reasons
        return tab
    }

    private func settle(until done: () -> Bool = { false }) async {
        for _ in 0 ..< 200 where !done() {
            await Task.yield()
            try? await Task.sleep(for: .milliseconds(2))
        }
    }

    private func asked(_ core: ScriptedCore) -> [String] {
        core.requests.compactMap {
            switch $0 {
            case let .queryAwake(ask): "what is awake, for \(ask.windowID)"
            case let .sleepOthers(ask): "sleep others, from \(ask.windowID)"
            default: nil
            }
        }
    }

    func testTheListIsAskedOfTheWindowAndShownInTheCoresOrderWithItsCounts() async {
        told.say(
            [Self.tab(3, "Third", inSight: true), Self.tab(1, "First"), Self.tab(2, "Second", [.sound])],
            privateTabs: 2, asleep: 9
        )
        let core = core()
        let model = AwakeModel(core: core, windowID: 7)
        XCTAssertFalse(model.loaded, "nothing is said before the core has answered")
        XCTAssertFalse(model.othersCouldSleep)

        model.reload()
        await settle { model.loaded }
        XCTAssertEqual(model.tabs.map(\.title), ["Third", "First", "Second"], "in the core's order, and no other")
        XCTAssertEqual(model.privateTabs, 2)
        XCTAssertEqual(model.asleep, 9)
        XCTAssertEqual(asked(core), ["what is awake, for 7"])
    }

    func testSleepAllOthersIsAskedOfTheCoreAndTheListIsAskedForAgain() async {
        told.say([Self.tab(3, "Third", inSight: true), Self.tab(1, "First"), Self.tab(2, "Second", [.sound])], asleep: 4)
        let core = core()
        let model = AwakeModel(core: core, windowID: 7)
        model.reload()
        await settle { model.loaded }
        XCTAssertTrue(model.othersCouldSleep, "one is out of sight with nothing to keep it awake")

        model.sleepOthers()
        await settle { model.tabs.count == 2 }
        // The command, and then the question: in that order, so the answer is of what is left.
        XCTAssertEqual(asked(core), ["what is awake, for 7", "sleep others, from 7", "what is awake, for 7"])
        XCTAssertEqual(model.tabs.map(\.title), ["Third", "Second"], "what the core left awake: the shell took none off itself")
        XCTAssertEqual(model.asleep, 5)
        XCTAssertFalse(model.othersCouldSleep, "the one in sight and the one with a reason are left: there are no others")
    }

    func testTheButtonIsWorthPressingOnlyWhenSomeTabCouldSleep() async {
        let core = core()
        let model = AwakeModel(core: core, windowID: 7)
        let cases: [([WTAwakeTab], Bool)] = [
            ([], false),
            ([Self.tab(1, "In sight", inSight: true)], false),
            ([Self.tab(1, "With a reason", [.typed])], false),
            // A private window's tab does not sleep (037), whatever else is so of it.
            ([Self.tab(1, "Private", privateWindow: true)], false),
            ([Self.tab(1, "In sight", inSight: true), Self.tab(2, "Behind")], true),
        ]
        for (index, (tabs, could)) in cases.enumerated() {
            // Each case's answer says a number of its own: the list is this case's, and not
            // the one before's still.
            told.say(tabs, asleep: UInt32(100 + index))
            model.reload()
            await settle { model.loaded && model.asleep == 100 + index }
            XCTAssertEqual(model.asleep, 100 + index, "the list is this case's")
            XCTAssertEqual(model.othersCouldSleep, could, "\(tabs.map(\.title))")
        }
    }

    /// A core whose answers wait until the test lets each go, in whatever order it likes.
    private final class HeldCore: CoreClient, @unchecked Sendable {
        private let lock = NSLock()
        private var asked = 0
        private var waiting: [Int: CheckedContinuation<WTResponse.OneOf_Kind, any Error>] = [:]
        let events = AsyncStream<WTEvent.OneOf_Kind> { _ in }

        /// How many questions are waiting for their answers.
        var held: Int {
            lock.withLock { waiting.count }
        }

        func send(_: WTRequest.OneOf_Kind) -> CoreAnswer {
            let number = lock.withLock { () -> Int in
                asked += 1
                return asked
            }
            return Task {
                try await withCheckedThrowingContinuation { continuation in
                    lock.withLock { waiting[number] = continuation }
                }
            }
        }

        /// Answers the question that was asked `number`th, from one.
        func answer(_ number: Int, with page: WTAwakePage) {
            lock.withLock { waiting.removeValue(forKey: number) }?.resume(returning: .awakePage(page))
        }

        func report(_: WTEngineEvent) {}

        func navigationVerdict(for _: String, webViewID _: UInt64, mainFrame _: Bool, userGesture _: Bool) -> NavigationVerdict {
            .allow
        }

        func stop() {}
    }

    func testAnAnswerThatArrivesAfterALaterQuestionsIsNotShown() async {
        let core = HeldCore()
        let model = AwakeModel(core: core, windowID: 7)
        // Asked twice, and the second question is answered first.
        model.reload()
        model.reload()
        await settle { core.held == 2 }
        var now = WTAwakePage()
        now.tabs = [Self.tab(2, "Now")]
        now.asleep = 2
        core.answer(2, with: now)
        await settle { model.loaded }
        XCTAssertEqual(model.tabs.map(\.title), ["Now"])

        // The first question's answer arrives late: it is of what was so before, and is not shown.
        var then = WTAwakePage()
        then.tabs = [Self.tab(1, "Then")]
        then.asleep = 1
        core.answer(1, with: then)
        await settle { core.held == 0 }
        await settle()
        XCTAssertEqual(model.tabs.map(\.title), ["Now"], "an answer to an older question is out of date")
        XCTAssertEqual(model.asleep, 2)
    }

    func testWhyATabIsAwakeIsSaidInTheCoresOrderAndOnScreenFirst() {
        let english = Locale(identifier: "en")
        let why = { (tab: WTAwakeTab) in AwakeView.why(tab, locale: english) }
        XCTAssertEqual(why(Self.tab(1, "A", inSight: true)), "On screen")
        XCTAssertEqual(why(Self.tab(1, "A")), "Will sleep by itself")
        XCTAssertEqual(why(Self.tab(1, "A", [.sound])), "Playing sound")
        XCTAssertEqual(why(Self.tab(1, "A", [.capture])), "Using the camera or microphone")
        XCTAssertEqual(why(Self.tab(1, "A", [.typed])), "Has text you typed")
        XCTAssertEqual(why(Self.tab(1, "A", [.download])), "Downloading")
        XCTAssertEqual(why(Self.tab(1, "A", [.question])), "Asking you something")
        XCTAssertEqual(why(Self.tab(1, "A", [.loading])), "Loading")
        XCTAssertEqual(
            why(Self.tab(1, "A", inSight: true, [.sound, .typed, .loading])),
            "On screen, Playing sound, Has text you typed, Loading"
        )
        // A reason this shell has no words for is not said, and is not taken for none:
        // nothing is said of a tab whose only reason is one, rather than that it will sleep.
        XCTAssertEqual(why(Self.tab(1, "A", [.sound, .UNRECOGNIZED(99)])), "Playing sound")
        XCTAssertEqual(why(Self.tab(1, "A", [.UNRECOGNIZED(99)])), "")
        XCTAssertEqual(why(Self.tab(1, "A", inSight: true, [.UNRECOGNIZED(99)])), "On screen")
        XCTAssertNil(AwakeView.word(for: .unspecified))
        XCTAssertNil(AwakeView.word(for: .UNRECOGNIZED(99)))
    }

    func testEveryReasonTheContractHasHasWords() {
        for reason in WTKeepAwake.allCases where reason != .unspecified {
            XCTAssertNotNil(AwakeView.word(for: reason), "\(reason)")
        }
    }

    func testATabIsNamedByItsPageOrItsSiteAndPlacedByItsSpace() {
        let english = Locale(identifier: "en")
        XCTAssertEqual(AwakeView.name(Self.tab(1, "Example Domain"), locale: english), "Example Domain")
        XCTAssertEqual(AwakeView.name(Self.tab(1, "", host: "example.org"), locale: english), "example.org")
        XCTAssertEqual(AwakeView.name(Self.tab(1, "", host: ""), locale: english), "New Tab")
        // Where it is: the Space by its name, the app's own Space by the shell's word, a
        // private window as that; and the site, where the name is not the site already.
        XCTAssertEqual(AwakeView.place(Self.tab(1, "A", host: "example.com", space: "Work"), locale: english), "Work · example.com")
        XCTAssertEqual(AwakeView.place(Self.tab(1, "A", host: "example.com"), locale: english), "Home · example.com")
        XCTAssertEqual(
            AwakeView.place(Self.tab(1, "A", host: "example.com", privateWindow: true), locale: english), "Private Window · example.com"
        )
        XCTAssertEqual(AwakeView.place(Self.tab(1, "", host: "example.com", space: "Work"), locale: english), "Work")
        XCTAssertEqual(AwakeView.place(Self.tab(1, "A", host: "", space: "Work"), locale: english), "Work")
    }

    #if WEBTIGER_CORE
        /// The real core's answer is one this shell reads: a session in fake mode, a window
        /// with a page-less tab, and nothing awake.
        func testTheRealCoreAnswersWhatIsAwakeAndTakesTheCommand() async throws {
            let session = try BrowserSession.live(environment: ["WEBTIGER_FAKE": "1"])
            defer { session.core.stop() }
            let opened = try await session.openWindow { _ in }
            let model = AwakeModel(core: session.core, windowID: opened.windowID)
            model.reload()
            await settle { model.loaded }
            XCTAssertTrue(model.loaded, "the core answered with a list")
            XCTAssertEqual(model.tabs.count, 0, "an empty tab has no page, and is not awake")
            XCTAssertEqual(model.asleep, 0)
            model.sleepOthers()
            await settle()
            XCTAssertTrue(model.loaded)
            // A window that is not there is told nothing, and the list stays as it was.
            let nowhere = AwakeModel(core: session.core, windowID: opened.windowID + 1000)
            nowhere.reload()
            await settle()
            XCTAssertFalse(nowhere.loaded)
        }
    #endif
}

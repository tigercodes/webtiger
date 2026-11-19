// A window's model against a scripted core: what it sends, and what it shows.
import WTContract
import WTCoreClient
@testable import WTUI
import WTWebKit
import XCTest

@MainActor
final class BrowserModelTests: XCTestCase {
    private func session() -> (BrowserSession, ScriptedCore) {
        let core = ScriptedCore { request in
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
        return (BrowserSession(core: core, engine: WebKitEngine()), core)
    }

    private func snapshot(
        url: String,
        display: String,
        title: String = "",
        state: WTLoadState = .idle,
        webView: UInt64 = 0
    ) -> WTWindowSnapshot {
        var window = WTWindowSnapshot()
        window.windowID = 1
        window.tab.tabID = 2
        window.tab.webViewID = webView
        window.tab.title = title
        window.tab.loadState = state
        window.tab.address.url = url
        window.tab.address.display = display
        return window
    }

    /// Lets the session's event task and the model's request tasks run.
    private func settle(until done: () -> Bool = { false }) async {
        for _ in 0 ..< 200 where !done() {
            await Task.yield()
            try? await Task.sleep(for: .milliseconds(2))
        }
    }

    func testOpeningAsksTheCoreForAWindowAndSubscribes() async {
        let (session, core) = session()
        let model = BrowserModel(session: session)
        await model.open()
        await model.open() // once only
        XCTAssertEqual(core.requests.count, 2)
        guard case .openWindow = core.requests[0], case let .subscribe(subscribe) = core.requests[1] else {
            return XCTFail("expected OpenWindow then Subscribe")
        }
        XCTAssertEqual(subscribe.window.windowID, 1)
    }

    func testSubmittingSendsWhatWasTypedAsTyped() async {
        let (session, core) = session()
        let model = BrowserModel(session: session)
        await model.open()
        model.isEditingAddress = true
        model.addressText = "  rust borrow checker "
        model.submit()
        await settle { core.requests.count == 3 }
        guard case let .submitAddress(submit) = core.requests.last else { return XCTFail("expected SubmitAddress") }
        XCTAssertEqual(submit.tabID, 2)
        XCTAssertEqual(submit.text, "  rust borrow checker ", "the core decides what typed text is; the shell does not tidy it")
        XCTAssertFalse(model.isEditingAddress)
    }

    func testTheFieldFollowsTheCoreUnlessTheUserIsTyping() async {
        let (session, core) = session()
        let model = BrowserModel(session: session)
        await model.open()

        core.emit(snapshot(url: "https://example.com/", display: "example.com", title: "Example"), subscription: 3)
        await settle { model.tab.title == "Example" }
        XCTAssertEqual(model.addressText, "example.com")

        model.isEditingAddress = true
        XCTAssertEqual(model.addressText, "https://example.com/", "editing starts from the whole address")
        model.addressText = "exam"
        core.emit(snapshot(url: "https://example.com/inbox", display: "example.com/inbox", title: "Inbox"), subscription: 3)
        await settle { model.tab.title == "Inbox" }
        XCTAssertEqual(model.addressText, "exam", "typing is never overwritten")

        model.isEditingAddress = false
        XCTAssertEqual(model.addressText, "example.com/inbox", "leaving the field shows the page again")
    }

    func testAFieldWithTheKeyboardFollowsThePageUntilSomethingIsTyped() async {
        let (session, core) = session()
        let model = BrowserModel(session: session)
        await model.open()

        // The field has the keyboard before any page is there, and nothing is typed.
        model.isEditingAddress = true
        XCTAssertEqual(model.addressText, "")
        core.emit(snapshot(url: "https://example.com/", display: "example.com", title: "Example"), subscription: 3)
        await settle { model.tab.title == "Example" }
        XCTAssertEqual(model.addressText, "https://example.com/", "an untouched field shows the page that arrived, whole")

        core.emit(snapshot(url: "https://example.com/inbox", display: "example.com/inbox", title: "Inbox"), subscription: 3)
        await settle { model.tab.title == "Inbox" }
        XCTAssertEqual(model.addressText, "https://example.com/inbox", "and goes on following it")

        model.addressText = "exam"
        core.emit(snapshot(url: "https://example.com/sent", display: "example.com/sent", title: "Sent"), subscription: 3)
        await settle { model.tab.title == "Sent" }
        XCTAssertEqual(model.addressText, "exam", "until something is typed")
    }

    /// Lets a core's script reach the core it is the script of.
    private final class Emitter: @unchecked Sendable {
        var core: ScriptedCore?
    }

    func testTheFirstSnapshotIsDrawnEvenWhenItComesBeforeItsSubscriptionIsKnown() async {
        let first = snapshot(url: "https://example.com/", display: "example.com", title: "First")
        let emitter = Emitter()
        let core = ScriptedCore { request in
            switch request {
            case .openWindow:
                var opened = WTWindowOpened()
                opened.windowID = 1
                opened.tabID = 2
                return .windowOpened(opened)
            case .subscribe:
                // The core publishes a subscription's first snapshot at once, with its answer.
                // Here the snapshot is well ahead of the answer: the session has it before the
                // window knows which subscription is its own.
                emitter.core?.emit(first, subscription: 3)
                Thread.sleep(forTimeInterval: 0.05)
                var subscribed = WTSubscribed()
                subscribed.subscriptionID = 3
                return .subscribed(subscribed)
            default:
                return .ack(WTAck())
            }
        }
        emitter.core = core
        let model = BrowserModel(session: BrowserSession(core: core, engine: WebKitEngine()))
        await model.open()
        await settle { model.tab.title == "First" }
        XCTAssertEqual(model.tab.title, "First", "a window's first snapshot is never lost")
        XCTAssertEqual(model.addressText, "example.com")
    }

    // MARK: Tabs (005)

    /// A snapshot whose tab in front is `front`, with `others` beside it in the strip.
    private func tabs(front: UInt64, url: String, display: String, others: [UInt64] = []) -> WTWindowSnapshot {
        var window = snapshot(url: url, display: display)
        window.tab.tabID = front
        window.tabs = ([front] + others).map { id in
            var summary = WTTabSummary()
            summary.tabID = id
            return summary
        }
        return window
    }

    func testTheWindowsControlsActOnTheTabInFront() async {
        let (session, core) = session()
        let model = BrowserModel(session: session)
        await model.open()
        // Before any snapshot, the tab the window opened with.
        model.perform(.stop)
        await settle { core.requests.count == 3 }
        guard case let .tabCommand(first) = core.requests.last else { return XCTFail("expected TabCommand") }
        XCTAssertEqual(first.tabID, 2)

        // Another tab is in front now: the controls are its.
        core.emit(tabs(front: 9, url: "https://example.org/", display: "example.org", others: [2]), subscription: 3)
        await settle { model.tab.tabID == 9 }
        XCTAssertEqual(model.tabs.map(\.tabID), [9, 2], "the strip lists the window's tabs in their order")
        model.perform(.reload)
        model.isEditingAddress = true
        model.addressText = "example.com"
        model.submit()
        await settle { core.requests.count == 5 }
        guard case let .tabCommand(command) = core.requests[3], case let .submitAddress(submit) = core.requests[4] else {
            return XCTFail("expected a TabCommand and a SubmitAddress")
        }
        XCTAssertEqual(command.tabID, 9)
        XCTAssertEqual(submit.tabID, 9)
    }

    func testOpeningClosingAndActivatingTabsAskTheCore() async {
        let (session, core) = session()
        let model = BrowserModel(session: session)
        await model.open()
        core.emit(tabs(front: 2, url: "", display: "", others: [5]), subscription: 3)
        await settle { model.tabs.count == 2 }

        model.openTab()
        model.activate(5)
        model.closeTab(5)
        // Each has been sent by the time its call returns: nothing is waited for here.
        XCTAssertEqual(core.requests.count, 5)
        guard case let .openTab(open) = core.requests[2], case let .activateTab(activate) = core.requests[3],
              case let .closeTab(close) = core.requests[4]
        else { return XCTFail("expected OpenTab, ActivateTab and CloseTab, in that order") }
        XCTAssertEqual(open.windowID, 1)
        XCTAssertEqual(open.text, "", "a new tab is empty")
        XCTAssertFalse(open.background, "and in front")
        XCTAssertEqual(activate.tabID, 5)
        XCTAssertEqual(close.tabID, 5)

        // The tab in front is in front already: nothing is asked.
        model.activate(2)
        await settle()
        XCTAssertEqual(core.requests.count, 5)
    }

    /// What a person does first, the core hears first. (Each gesture used to start a task of
    /// its own to send its request, and two tasks may run in either order.)
    func testGesturesReachTheCoreInTheOrderTheyWereMade() async {
        let (session, core) = session()
        let model = BrowserModel(session: session)
        await model.open()
        let commands: [WTNavigationCommand] = [.reload, .stop, .back, .forward]

        let made = (0 ..< 200).map { commands[$0 % commands.count] }
        for command in made {
            model.perform(command)
        }

        let heard = core.requests.dropFirst(2).map { request -> WTNavigationCommand in
            if case let .tabCommand(command) = request {
                command.command
            } else {
                .unspecified
            }
        }
        XCTAssertEqual(Array(heard), made)
    }

    // MARK: Find on the page (010)

    /// A window whose tab in front has a page, in which things can be found.
    private func finding() async -> (BrowserModel, ScriptedCore) {
        let (session, core) = session()
        let model = BrowserModel(session: session)
        await model.open()
        core.emit(snapshot(url: "https://example.com/", display: "example.com", webView: 4), subscription: 3)
        await settle { model.hasPage }
        return (model, core)
    }

    private func finds(_ core: ScriptedCore) -> [String] {
        core.requests.compactMap { request -> String? in
            if case let .findInPage(find) = request {
                "\(find.tabID) \(find.text.isEmpty ? "(stop)" : find.text)\(find.backwards ? " back" : "")"
            } else {
                nil
            }
        }
    }

    func testWhatIsTypedInTheFindBarIsLookedForAtOnce() async {
        let (model, core) = await finding()
        XCTAssertFalse(model.isFinding)
        // Nothing is looked for while the bar is shut.
        model.findText = "stray"
        model.findText = ""

        model.openFind()
        model.findText = "n"
        model.findText = "ne"
        model.findNext()
        model.findPrevious()

        XCTAssertTrue(model.isFinding)
        XCTAssertEqual(finds(core), ["2 n", "2 ne", "2 ne", "2 ne back"], "each letter, then the next match, then the one before")
    }

    func testClosingTheFindBarStopsTheFind() async {
        let (model, core) = await finding()
        model.openFind()
        model.findText = "needle"

        model.closeFind()

        XCTAssertFalse(model.isFinding)
        XCTAssertEqual(model.findText, "", "the bar opens empty the next time")
        XCTAssertEqual(finds(core), ["2 needle", "2 (stop)"])
        // Closed already: nothing more is sent.
        model.closeFind()
        XCTAssertEqual(finds(core).count, 2)
    }

    func testEmptyingTheFieldStopsTheFindAndTheBarStaysOpen() async {
        let (model, core) = await finding()
        model.openFind()
        model.findText = "needle"

        model.findText = ""

        XCTAssertTrue(model.isFinding)
        XCTAssertEqual(finds(core), ["2 needle", "2 (stop)"])
        // There is no find left to stop when the bar is closed.
        model.closeFind()
        XCTAssertEqual(finds(core).count, 2)
    }

    func testAnotherTabInFrontStopsTheFindInTheTabThatHadIt() async {
        let (model, core) = await finding()
        model.openFind()
        model.findText = "needle"

        core.emit(tabs(front: 9, url: "https://example.org/", display: "example.org", others: [2]), subscription: 3)
        await settle { model.tab.tabID == 9 }

        XCTAssertFalse(model.isFinding, "the bar closes")
        XCTAssertEqual(finds(core), ["2 needle", "2 (stop)"], "and the find stops in the tab it was in, not the one in front")
    }

    func testThereIsNothingToFindInATabWithNoPage() async {
        let (session, core) = session()
        let model = BrowserModel(session: session)
        await model.open()

        model.openFind()
        model.findText = "needle"

        XCTAssertFalse(model.isFinding)
        XCTAssertEqual(finds(core), [])
    }

    func testTheFindBarSaysWhatTheCoreSaysWasFound() async {
        let (model, core) = await finding()
        model.openFind()
        model.findText = "needle"
        XCTAssertEqual(model.findOutcome, .nothing, "nothing is known yet: that is not \"not found\"")

        func publish(_ change: (inout WTFindView) -> Void) async {
            var window = snapshot(url: "https://example.com/", display: "example.com", webView: 4)
            window.tab.find.text = "needle"
            change(&window.tab.find)
            let before = model.tab
            core.emit(window, subscription: 3)
            await settle { model.tab != before }
        }

        await publish { find in
            find.answered = true
            find.found = true
            find.counted = true
            find.matches = 12
            find.current = 3
        }
        XCTAssertEqual(model.findOutcome, .match(3, of: 12))

        // An engine that does not count: found, and nothing more to say.
        await publish { find in
            find.answered = true
            find.found = true
        }
        XCTAssertEqual(model.findOutcome, .nothing)

        await publish { find in
            find.answered = true
        }
        XCTAssertEqual(model.findOutcome, .notFound)

        // Another letter typed: what the core said was of the text before.
        model.findText = "needles"
        XCTAssertEqual(model.findOutcome, .nothing)
    }

    func testWhatWasTypedIsDroppedWhenAnotherTabComesInFront() async {
        let (session, core) = session()
        let model = BrowserModel(session: session)
        await model.open()
        core.emit(tabs(front: 2, url: "https://example.com/", display: "example.com"), subscription: 3)
        await settle { model.tab.tabID == 2 }
        model.isEditingAddress = true
        model.addressText = "half an addr"

        // The same tab, changed: what is being typed stays.
        core.emit(tabs(front: 2, url: "https://example.com/next", display: "example.com/next"), subscription: 3)
        await settle { model.tab.address.url == "https://example.com/next" }
        XCTAssertEqual(model.addressText, "half an addr")

        // Another tab in front: it was typed for the tab that was there.
        core.emit(tabs(front: 9, url: "https://example.org/", display: "example.org", others: [2]), subscription: 3)
        await settle { model.tab.tabID == 9 }
        XCTAssertEqual(model.addressText, "https://example.org/", "the field shows the tab that is in front")
        XCTAssertFalse(model.hasTypedAddress)
    }

    func testSnapshotsForAnotherSubscriptionAreIgnored() async {
        let (session, core) = session()
        let model = BrowserModel(session: session)
        await model.open()
        core.emit(snapshot(url: "https://other.example/", display: "other.example", title: "Other"), subscription: 99)
        core.emit(snapshot(url: "https://example.com/", display: "example.com", title: "Mine"), subscription: 3)
        await settle { model.tab.title == "Mine" }
        XCTAssertEqual(model.tab.title, "Mine")
    }

    func testCommandsAndClosingReachTheCore() async {
        let (session, core) = session()
        let model = BrowserModel(session: session)
        await model.open()
        model.perform(.reload)
        await settle { core.requests.count == 3 }
        guard case let .tabCommand(command) = core.requests.last else { return XCTFail("expected TabCommand") }
        XCTAssertEqual(command.command, .reload)

        model.close()
        await settle { core.requests.count == 4 }
        guard case let .closeWindow(close) = core.requests.last else { return XCTFail("expected CloseWindow") }
        XCTAssertEqual(close.windowID, 1)
    }

    func testStateFlagsFollowTheSnapshot() async {
        let (session, core) = session()
        let model = BrowserModel(session: session)
        await model.open()
        XCTAssertFalse(model.hasPage)
        var loading = snapshot(url: "https://example.com/", display: "example.com", state: .loading)
        loading.tab.webViewID = 4
        core.emit(loading, subscription: 3)
        await settle { model.isLoading }
        XCTAssertTrue(model.isLoading)
        XCTAssertTrue(model.hasPage)
        XCTAssertFalse(model.hasFailed)
    }

    func testEveryErrorCodeHasWords() {
        XCTAssertEqual(FailedPageView.words(for: "net.dns"), "The site’s address couldn’t be found.")
        XCTAssertEqual(FailedPageView.words(for: "page.renderer_gone"), "The page stopped unexpectedly.")
        // A code this build has no words for still says something, and never shows the code's text.
        XCTAssertEqual(FailedPageView.words(for: "net.something_new"), "Something went wrong while loading the page.")
    }
}

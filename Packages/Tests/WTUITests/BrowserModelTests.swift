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

    private func snapshot(url: String, display: String, title: String = "", state: WTLoadState = .idle) -> WTWindowSnapshot {
        var window = WTWindowSnapshot()
        window.windowID = 1
        window.tab.tabID = 2
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

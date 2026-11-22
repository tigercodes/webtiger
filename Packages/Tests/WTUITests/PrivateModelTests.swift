// A private window's model (037), against a scripted core: how the window is asked for, what
// the model says it is, and that nothing from outside the browser is opened in it.
import WTContract
import WTCoreClient
@testable import WTUI
import WTWebKit
import XCTest

@MainActor
final class PrivateModelTests: XCTestCase {
    private func core() -> ScriptedCore {
        ScriptedCore { request in
            switch request {
            case .openWindow:
                var opened = WTWindowOpened()
                opened.windowID = 7
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

    /// Whether each window the core was asked to open was asked for as a private one.
    private func askedFor(_ core: ScriptedCore) -> [Bool] {
        core.requests.compactMap {
            if case let .openWindow(open) = $0 {
                open.privateWindow
            } else {
                nil
            }
        }
    }

    /// Lets the session's event task run.
    private func settle(until done: () -> Bool) async {
        for _ in 0 ..< 200 where !done() {
            await Task.yield()
            try? await Task.sleep(for: .milliseconds(2))
        }
    }

    func testAPrivateWindowIsAskedForAsOneAndIsOneFromTheStart() async {
        let core = core()
        let session = BrowserSession(core: core, engine: WebKitEngine())
        let ordinary = BrowserModel(session: session)
        let hidden = BrowserModel(session: session)
        hidden.opensPrivate = true
        // Before the core has said anything: the window knows what it was made to be.
        XCTAssertFalse(ordinary.isPrivate)
        XCTAssertTrue(hidden.isPrivate)

        await ordinary.open()
        await hidden.open()

        XCTAssertEqual(askedFor(core), [false, true])
    }

    func testAWindowTheCoreSaysIsPrivateIsOne() async {
        let core = core()
        let model = BrowserModel(session: BrowserSession(core: core, engine: WebKitEngine()))
        await model.open()
        var said = WTWindowSnapshot()
        said.windowID = 7
        said.tab.tabID = 2
        core.emit(said, subscription: 3)
        await settle { model.tab.tabID == 2 }
        XCTAssertFalse(model.isPrivate)

        said.privateWindow = true
        core.emit(said, subscription: 3)
        await settle { model.isPrivate }
        XCTAssertTrue(model.isPrivate)
    }

    func testALinkFromAnotherAppThatWaitedIsNotOpenedInAPrivateWindow() async {
        // A link pressed in another app before any window was open waits for one (031). A
        // private window that opens does not take it; an ordinary one does.
        let core = core()
        let session = BrowserSession(core: core, engine: WebKitEngine())
        let hidden = BrowserModel(session: session)
        hidden.opensPrivate = true
        hidden.openFromOutside("https://example.com/from-outside")
        await hidden.open()
        let opened: () -> Int = { core.requests.count(where: {
            if case .openAddress = $0 {
                true
            } else {
                false
            }
        }) }
        XCTAssertEqual(opened(), 0, "nothing from outside is asked for in a private window")

        let ordinary = BrowserModel(session: session)
        ordinary.openFromOutside("https://example.com/from-outside")
        await ordinary.open()
        XCTAssertEqual(opened(), 1)
    }
}

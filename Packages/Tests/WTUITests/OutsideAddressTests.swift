// A link pressed in another app (031), against a scripted core: what the window's model asks
// of the core for an address the system handed the app, and what becomes of addresses that
// come before the window is open. Nothing is loaded.
import WTContract
import WTCoreClient
@testable import WTUI
import WTWebKit
import XCTest

@MainActor
final class OutsideAddressTests: XCTestCase {
    /// A model over a core that opens a window numbered `window` and says Ack to the rest.
    private func model(window: UInt64 = 1) -> (BrowserModel, ScriptedCore) {
        let core = ScriptedCore { request in
            switch request {
            case .openWindow:
                var opened = WTWindowOpened()
                opened.windowID = window
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
        return (BrowserModel(session: BrowserSession(core: core, engine: WebKitEngine())), core)
    }

    /// The addresses the core was asked to open, each with the window it was for, in order.
    private func opened(_ core: ScriptedCore) -> [String] {
        core.requests.compactMap {
            if case let .openAddress(asked) = $0 {
                "\(asked.windowID) \(asked.url)"
            } else {
                nil
            }
        }
    }

    func testAnAddressFromOutsideIsHandedToTheCoreWithItsWindowAsItWasGiven() async {
        let (model, core) = model(window: 7)
        await model.open()

        // An address of the web, and what is none: each handed on as it is. Which of them
        // is opened is the core's rule (031 §3), not the shell's.
        let addresses = [
            "https://example.com/story?id=7#top",
            "HTTP://Example.COM",
            "example.com",
            "javascript:alert(1)",
            "file:///etc/hosts",
        ]
        for address in addresses {
            model.openFromOutside(address)
        }

        XCTAssertEqual(opened(core), addresses.map { "7 \($0)" })
        // Nothing else was asked for them: no tab opened by the shell, nothing typed for the
        // core to mend, nothing searched for.
        let others = core.requests.filter {
            switch $0 {
            case .openTab, .submitAddress, .searchWords: true
            default: false
            }
        }
        XCTAssertTrue(others.isEmpty, "\(others.count) other requests")
    }

    func testAddressesThatComeBeforeTheWindowIsOpenWaitForItEachOfThemInOrder() async {
        let (model, core) = model()

        // Several links at once, as an app that opens a list of them sends them; or the one
        // the app was started by, before its window has opened.
        for path in ["first", "second", "first", "third"] {
            model.openFromOutside("https://example.com/\(path)")
        }
        XCTAssertEqual(opened(core), [], "there is no window to open them in yet")

        await model.open()

        // Each of them, once, in the order they came: one that came twice is opened twice.
        let came = ["first", "second", "first", "third"].map { "1 https://example.com/\($0)" }
        XCTAssertEqual(opened(core), came)
        model.openFromOutside("https://example.org/")
        XCTAssertEqual(opened(core), came + ["1 https://example.org/"])

        // A window that is closed and opened again does not open them a second time.
        model.close()
        await model.open()
        XCTAssertEqual(opened(core), came + ["1 https://example.org/"])
    }
}

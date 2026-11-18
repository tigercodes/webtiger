import WTContract
import WTCoreClient
import XCTest

final class ScriptedCoreTests: XCTestCase {
    func testRequestsAreRecordedAndAnsweredByTheScript() async throws {
        let core = ScriptedCore { request in
            guard case .openWindow = request else { return .ack(WTAck()) }
            var opened = WTWindowOpened()
            opened.windowID = 7
            opened.tabID = 8
            return .windowOpened(opened)
        }
        let answer = try await core.request(.openWindow(WTOpenWindow()))
        guard case let .windowOpened(opened) = answer else { return XCTFail("expected WindowOpened") }
        XCTAssertEqual(opened.windowID, 7)
        try await core.perform(.closeWindow(WTCloseWindow()))
        XCTAssertEqual(core.requests.count, 2)
    }

    func testAScriptCanRefuse() async {
        let core = ScriptedCore { _ in throw CoreError.refused(code: "tab.unknown", field: "tab_id") }
        do {
            try await core.perform(.submitAddress(WTSubmitAddress()))
            XCTFail("expected a refusal")
        } catch {
            XCTAssertEqual(error as? CoreError, .refused(code: "tab.unknown", field: "tab_id"))
        }
    }

    func testEventsArriveInOrderAndEndWhenStopped() async {
        let core = ScriptedCore()
        var first = WTWindowSnapshot()
        first.windowID = 1
        var second = WTWindowSnapshot()
        second.windowID = 2
        core.emit(first, subscription: 5)
        core.emit(second, subscription: 5)
        core.stop()
        var seen: [UInt64] = []
        for await event in core.events {
            if case let .snapshot(snapshot) = event {
                seen.append(snapshot.window.windowID)
            }
        }
        XCTAssertEqual(seen, [1, 2])
        XCTAssertTrue(core.isStopped)
    }

    func testVerdictsDefaultToAllow() {
        let core = ScriptedCore()
        core.setVerdict(.cancel, for: "javascript:alert(1)")
        XCTAssertEqual(core.navigationVerdict(for: "https://example.com/", webViewID: 1, mainFrame: true, userGesture: false), .allow)
        XCTAssertEqual(core.navigationVerdict(for: "javascript:alert(1)", webViewID: 1, mainFrame: true, userGesture: false), .cancel)
    }
}

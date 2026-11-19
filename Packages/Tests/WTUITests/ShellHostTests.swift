// The shell's own host (007): the System service is answered here, by an opener a test
// replaces with one that only writes down what it was asked; everything else is the engine's.
import Foundation
import WTContract
import WTCoreClient
@testable import WTUI
import XCTest

@MainActor
final class ShellHostTests: XCTestCase {
    /// An engine that writes down what it was asked and acknowledges it.
    @MainActor
    private final class Engine: HostServices {
        var asked: [WTHostRequest.OneOf_Kind] = []

        func answer(_ request: WTHostRequest.OneOf_Kind) async -> WTHostResponse.OneOf_Kind {
            asked.append(request)
            return .ack(WTAck())
        }
    }

    /// What an opener was asked to open.
    @MainActor
    private final class Opened {
        var urls: [URL] = []
    }

    /// A host over an engine and an opener that only write down what they were asked.
    private struct Subject {
        let host: ShellHost
        let engine: Engine
        let opened: Opened
    }

    private func subject(opens: Bool = true) -> Subject {
        let engine = Engine()
        let opened = Opened()
        let opener = OutsideOpener { url in
            opened.urls.append(url)
            return opens
        }
        return Subject(host: ShellHost(engine: engine, opener: opener), engine: engine, opened: opened)
    }

    private func openOutside(_ url: String) -> WTHostRequest.OneOf_Kind {
        var open = WTOpenOutside()
        open.url = url
        return .openOutside(open)
    }

    private func code(_ answer: WTHostResponse.OneOf_Kind) -> String? {
        if case let .error(error) = answer {
            error.code
        } else {
            nil
        }
    }

    func testAnOutsideLinkGoesToTheOpenerAsItWasGiven() async {
        let subject = subject()

        let answer = await subject.host.answer(openOutside("mailto:ada@example.com?subject=Hello"))

        guard case .ack = answer else { return XCTFail("the system took it") }
        XCTAssertEqual(subject.opened.urls.map(\.absoluteString), ["mailto:ada@example.com?subject=Hello"])
        XCTAssertTrue(subject.engine.asked.isEmpty, "an engine is asked nothing about it")
    }

    func testNothingOnTheDeviceOpensItIsACode() async {
        let subject = subject(opens: false)

        let answer = await subject.host.answer(openOutside("tel:+15550100"))

        XCTAssertEqual(code(answer), "system.no_handler")
        XCTAssertEqual(subject.opened.urls.count, 1, "the system was asked")
    }

    func testWhatIsNotAnAddressIsNotGivenToTheSystem() async {
        let subject = subject()

        for text in ["", "not an address", "ada@example.com"] {
            let answer = await subject.host.answer(openOutside(text))
            XCTAssertEqual(code(answer), "system.no_handler", text)
        }

        XCTAssertTrue(subject.opened.urls.isEmpty)
    }

    func testEveryOtherRequestIsTheEngines() async {
        let subject = subject()
        var create = WTWebViewCreate()
        create.webViewID = 4
        var load = WTWebViewLoad()
        load.webViewID = 4
        var command = WTWebViewCommand()
        command.webViewID = 4
        var destroy = WTWebViewDestroy()
        destroy.webViewID = 4
        let requests: [WTHostRequest.OneOf_Kind] = [
            .ping(WTHostPing()), .webViewCreate(create), .webViewLoad(load), .webViewCommand(command), .webViewDestroy(destroy),
        ]

        for request in requests {
            guard case .ack = await subject.host.answer(request) else { return XCTFail("the engine's answer comes back") }
        }

        XCTAssertEqual(subject.engine.asked.count, requests.count)
        XCTAssertTrue(subject.opened.urls.isEmpty)
    }

    #if os(macOS)
        /// The system's own opener, asked for real, about a scheme nothing is registered for:
        /// it says so, and shows nothing. (What it does with a letter to write is to start the
        /// mail app, which no test does.)
        func testTheSystemsOpenerSaysWhenNothingOpensAnAddress() async {
            let host = ShellHost(engine: Engine(), opener: .system)
            let answer = await host.answer(openOutside("x-webtiger-test-no-such-scheme:1"))
            XCTAssertEqual(code(answer), "system.no_handler")
        }
    #endif

    func testFakeModesOpenerOpensNothingAndSaysItDid() async {
        let host = ShellHost(engine: Engine(), opener: .nothing)
        guard case .ack = await host.answer(openOutside("mailto:ada@example.com")) else {
            return XCTFail("fake mode answers as if the system had taken it")
        }
    }
}

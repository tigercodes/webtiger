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
        /// it says so. It is asked without the system's prompt: asked as the app asks, the
        /// system puts up a dialog of its own each time ("There is no application set to open
        /// the URL"), which stays on the screen of whoever's machine this runs on. (What it
        /// does with a letter to write is to start the mail app, which no test does.)
        func testTheSystemsOpenerSaysWhenNothingOpensAnAddress() async {
            let host = ShellHost(engine: Engine(), opener: .systemWithoutItsPrompt)
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

    // Opening a downloaded file (030).

    private func openFile(_ name: String, kind: String = "") -> WTHostRequest.OneOf_Kind {
        var open = WTOpenFile()
        open.fileName = name
        open.mimeType = kind
        return .openFile(open)
    }

    /// What a file opener was asked, and what it is to say came of it.
    @MainActor
    private final class Files {
        var asked: [[String]] = []
        var comes = FileOpened.taken
    }

    func testAFileGoesToTheOpenerByItsNameAndKindAndWhatCameOfItIsTheAnswer() async {
        let engine = Engine()
        let files = Files()
        let opener = FileOpener { name, kind in
            files.asked.append([name, kind])
            return files.comes
        }
        let host = ShellHost(engine: engine, opener: .nothing, files: opener)

        guard case .ack = await host.answer(openFile("report (2).pdf", kind: "application/pdf")) else {
            return XCTFail("the system took it")
        }
        files.comes = .noHandler
        let nothingOpens = await host.answer(openFile("data.bin"))
        XCTAssertEqual(code(nothingOpens), "system.no_handler")
        files.comes = .missing
        let gone = await host.answer(openFile("gone.txt", kind: "text/plain"))
        XCTAssertEqual(code(gone), "download.missing")

        XCTAssertEqual(files.asked, [["report (2).pdf", "application/pdf"], ["data.bin", ""], ["gone.txt", "text/plain"]])
        XCTAssertTrue(engine.asked.isEmpty, "an engine is asked nothing about it")
    }

    func testInFakeModeNoFileIsOpenedAndTheCoreIsToldItWas() async {
        // The host a session is given in fake mode has no opener of files but the one that
        // opens nothing.
        let host = ShellHost(engine: Engine(), opener: .nothing)
        guard case .ack = await host.answer(openFile("report.pdf", kind: "application/pdf")) else {
            return XCTFail("fake mode answers as if the system had taken it")
        }
    }

    /// The system's own opener of files, about files that are not there: it says so, and looks
    /// nowhere but in the folder downloads are saved in. (A file that is there is not opened
    /// by a test: that starts another app.)
    func testAFileThatIsNotInTheFolderIsMissingAndANameIsNeverAPath() async throws {
        let folder = FileManager.default.temporaryDirectory.appendingPathComponent("webtiger-open-\(UUID().uuidString)", isDirectory: true)
        let inside = folder.appendingPathComponent("saved", isDirectory: true)
        try FileManager.default.createDirectory(at: inside, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: folder) }
        try Data("outside".utf8).write(to: folder.appendingPathComponent("secret.txt"))
        try Data("inside".utf8).write(to: inside.appendingPathComponent("notes.txt"))
        try FileManager.default.createDirectory(at: inside.appendingPathComponent("a folder"), withIntermediateDirectories: true)

        // The file that is there is found, as itself.
        XCTAssertEqual(FileOpener.file(named: "notes.txt", in: inside)?.lastPathComponent, "notes.txt")
        // What is not there, what is a folder, and a name that is a path or leads out.
        for name in ["gone.txt", "a folder", "", ".", "..", "../secret.txt", "/etc/hosts", "sub/notes.txt", "..\u{2F}secret.txt"] {
            XCTAssertNil(FileOpener.file(named: name, in: inside), name)
        }
        let host = ShellHost(engine: Engine(), opener: .nothing, files: .systemWithoutItsPrompt(in: inside))
        for name in ["gone.txt", "../secret.txt"] {
            let answer = await host.answer(openFile(name, kind: "text/plain"))
            XCTAssertEqual(code(answer), "download.missing", name)
        }
    }
}

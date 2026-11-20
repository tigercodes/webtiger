// The adapter suite's WebKit run, for a page that asks for a file (019): the test site's page
// of file inputs in a real web view on a Mac, pressed with the mouse, and a picker of the
// test's own that picks files the test wrote. No picker of the system's is ever shown.
import WebKit
import WTContract
import WTCoreClient
@testable import WTWebKit
import XCTest

@MainActor
final class WebKitFileTests: AdapterTestCase {
    /// What a picker was asked, kept where its closure can put it.
    private final class Asked: @unchecked Sendable {
        private let lock = NSLock()
        private var all: [FilesAsked] = []

        func add(_ asked: FilesAsked) {
            lock.withLock { all.append(asked) }
        }

        var each: [FilesAsked] {
            lock.withLock { all }
        }
    }

    private let asked = Asked()
    private var folder: URL?

    override func tearDown() async throws {
        if let folder {
            try? FileManager.default.removeItem(at: folder)
        }
        try await super.tearDown()
    }

    /// A file of the test's own, with these words in it.
    private func file(_ name: String, _ words: String) throws -> URL {
        let made = folder ?? FileManager.default.temporaryDirectory.appendingPathComponent("webtiger-upload-\(UUID().uuidString)")
        folder = made
        try FileManager.default.createDirectory(at: made, withIntermediateDirectories: true)
        let file = made.appendingPathComponent(name)
        try Data(words.utf8).write(to: file)
        return file
    }

    /// A picker that notes what it was asked and picks these.
    private func picking(_ files: [URL]) -> FilePicker {
        let asked = asked
        return FilePicker { wanted in
            asked.add(wanted)
            return files
        }
    }

    func testOneAtATimeAndAWebViewThatGoesIsGivenNothing() async throws {
        // At the seam, where a picker can be held open: two web views' pages ask.
        let notes = try file("notes.txt", "Twelve notes.")
        var release: [CheckedContinuation<Void, Never>] = []
        let held = FilePicker { _ in
            await withCheckedContinuation { release.append($0) }
            return [notes]
        }
        // The engine's own, so that what a destroyed web view is given can be seen.
        let files = engine.files
        var given: [String: [URL]?] = [:]
        for (name, webView) in [("first", UInt64(4)), ("going", 5)] {
            Task { given[name] = await files.choose(for: webView, asked: FilesAsked(several: false), with: held) }
        }
        while release.count < 2 {
            await Task.yield()
        }

        // While a picker is open for a web view, its page is given nothing more, at once.
        var second: [URL]??
        Task { second = await .some(files.choose(for: 4, asked: FilesAsked(several: false), with: held)) }
        for _ in 0 ..< 200 where second == nil {
            await Task.yield()
        }
        XCTAssertEqual(second, .some(nil), "told nothing, without waiting for anybody")
        XCTAssertEqual(release.count, 2, "and no other picker was opened for it")

        // One of the web views goes; then the person picks in both pickers.
        var destroy = WTWebViewDestroy()
        destroy.webViewID = 5
        _ = await engine.answer(.webViewDestroy(destroy))
        for waiting in release {
            waiting.resume()
        }
        while given.count < 2 {
            await Task.yield()
        }
        XCTAssertEqual(given["first"], [notes])
        XCTAssertEqual(given["going"], .some(nil), "what was picked for a web view that went goes nowhere")

        // With that picker closed, the page may ask again.
        let again = await files.choose(for: 4, asked: FilesAsked(several: false), with: FilePicker { _ in [notes] })
        XCTAssertEqual(again, [notes])
    }

    func testAPageThatMayHaveOneFileIsGivenNoMoreAndNothingPickedIsNothing() async throws {
        let one = try file("a.txt", "a")
        let two = try file("b.txt", "bb")
        let files = Files()

        let single = await files.choose(for: 4, asked: FilesAsked(several: false), with: FilePicker { _ in [one, two] })
        XCTAssertEqual(single, [one])
        let several = await files.choose(for: 4, asked: FilesAsked(several: true), with: FilePicker { _ in [one, two] })
        XCTAssertEqual(several, [one, two])
        let none = await files.choose(for: 4, asked: FilesAsked(several: true), with: .none)
        XCTAssertNil(none)
    }

    #if os(macOS)
        /// A web view on the page of file inputs, in a window that is not on the screen.
        private func page() async throws -> NSWindow {
            await create(1)
            await load(1, site.url("/upload"))
            await wait(1, "the page to load") { self.finished($0) && self.hasTitle("Upload")($0) }
            return try window(for: 1)
        }

        private func awaitTitle(_ title: String) async {
            await wait(1, "the page to say \"\(title)\"; it last said \"\(lastTitle(1))\"") { self.hasTitle(title)($0) }
        }

        func testAPressOnAFileInputAsksThePickerAndThePageHasTheFile() async throws {
            let window = try await page()
            engine.filePicker = try picking([file("notes.txt", "Twelve notes, to be sent.")])

            try await click("one", in: 1, of: window, holding: [])

            await awaitTitle("chosen: notes.txt (25): Twelve notes, to be sent.")
            XCTAssertEqual(asked.each, [FilesAsked(several: false)], "asked once, for one file")
            window.close()
        }

        func testSeveralFilesWhereSeveralMayBeChosen() async throws {
            let window = try await page()
            engine.filePicker = try picking([file("a.txt", "first"), file("b.txt", "second file")])

            try await click("many", in: 1, of: window, holding: [])

            await awaitTitle("chosen: a.txt (5), b.txt (11): first")
            XCTAssertEqual(asked.each, [FilesAsked(several: true)])
            window.close()
        }

        func testNothingPickedAndThePageHasNothingAndMayAskAgain() async throws {
            let window = try await page()
            engine.filePicker = picking([])

            try await click("one", in: 1, of: window, holding: [])
            await awaitTitle("cancelled one")

            // The page may ask again, and is then given what is picked.
            engine.filePicker = try picking([file("notes.txt", "Now.")])
            try await click("one", in: 1, of: window, holding: [])
            await awaitTitle("chosen: notes.txt (4): Now.")
            XCTAssertEqual(asked.each.count, 2)
            window.close()
        }

        func testAnEngineGivenNoPickerGivesThePageNothing() async throws {
            let window = try await page()

            // The engine as it is made: no picker of the system's, and none of a test's.
            try await click("many", in: 1, of: window, holding: [])

            await awaitTitle("cancelled many")
            window.close()
        }
    #endif
}

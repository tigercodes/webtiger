// A profile's name (009): the folder its data is kept in is named by it, so a name is letters,
// digits and hyphens and nothing else. Anything else is no profile, and never a path.
#if WEBTIGER_CORE
    @testable import WTUI
    import WTWebKit
    import XCTest

    @MainActor
    final class ProfileTests: XCTestCase {
        func testANameIsLettersDigitsAndHyphens() {
            for name in ["work", "ui-test-session", "A1", String(repeating: "a", count: 40)] {
                XCTAssertEqual(BrowserSession.profileName(name), name, name)
            }
        }

        func testAProfilesDownloadsAreInItsOwnFolder() {
            let data = URL(fileURLWithPath: "/tmp/WebTiger Profiles/work", isDirectory: true)
            // The usual app saves where the system keeps downloads.
            XCTAssertEqual(BrowserSession.downloadsFolder(profile: nil, data: data), WebKitEngine.systemDownloads)
            XCTAssertNotNil(WebKitEngine.systemDownloads)
            // A profile's are with the rest of its data, and nowhere near the person's own.
            let profiled = BrowserSession.downloadsFolder(profile: "work", data: data)
            XCTAssertEqual(profiled?.path, "/tmp/WebTiger Profiles/work/Downloads")
            XCTAssertNotEqual(profiled, WebKitEngine.systemDownloads)
        }

        func testOnlyTheAppStartedForRealLetsAPageHaveWhatTheCoreGrants() {
            // Fake mode, where the UI suites run, can put no dialog of the system's on the screen (014).
            XCTAssertEqual(BrowserSession.deviceAccess(fake: true), .never)
            XCTAssertEqual(BrowserSession.deviceAccess(fake: false), .asTheCoreSays)
            XCTAssertEqual(WebKitEngine().deviceAccess, .never, "and an engine nobody made otherwise is a test's")
        }

        func testInFakeModeWhatPagesKeepIsKeptInMemory() {
            // So that nothing of one run of a suite is there in the next (WT-D35).
            XCTAssertEqual(BrowserSession.webData(fake: true), .inMemory)
            XCTAssertEqual(BrowserSession.webData(fake: false), .kept)
            XCTAssertEqual(WebKitEngine().webData, .kept, "an engine nobody made otherwise keeps")
        }

        func testOnlyTheAppStartedForRealFetchesIconsFromTheWeb() {
            // In fake mode nothing is fetched (046 §5a), unless a suite says so: and then
            // from this device only, where its site is.
            XCTAssertEqual(BrowserSession.iconFetching(fake: false, environment: [:]), .web)
            XCTAssertEqual(BrowserSession.iconFetching(fake: false, environment: ["WEBTIGER_FAKE_ICONS": "1"]), .web)
            XCTAssertEqual(BrowserSession.iconFetching(fake: true, environment: [:]), .never)
            XCTAssertEqual(BrowserSession.iconFetching(fake: true, environment: ["WEBTIGER_FAKE_ICONS": "0"]), .never)
            XCTAssertEqual(BrowserSession.iconFetching(fake: true, environment: ["WEBTIGER_FAKE_ICONS": "1"]), .thisDevice)
            XCTAssertEqual(WebKitEngine().iconFetching, .never, "and an engine nobody made otherwise fetches none")
        }

        /// What opens a downloaded file (030), by how the app was started. In fake mode nothing
        /// does, and nothing of the system's is shown, though the file is there; started for
        /// real, an iPhone's preview is given the file. (A Mac started for real hands the file
        /// to another app, which no test does.)
        func testInFakeModeNoFileIsOpenedOrShownThoughItIsThere() async throws {
            let folder = FileManager.default.temporaryDirectory.appendingPathComponent(
                "webtiger-opener-\(UUID().uuidString)",
                isDirectory: true
            )
            try FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
            defer { try? FileManager.default.removeItem(at: folder) }
            try Data("inside".utf8).write(to: folder.appendingPathComponent("notes.txt"))
            let preview = FilePreview()

            let fake = BrowserSession.filesOpener(fake: true, folder: folder, preview: preview)
            let opened = await fake.open("notes.txt", "text/plain")
            XCTAssertEqual(opened, .taken, "fake mode answers as if the system had taken it")
            XCTAssertNil(preview.file, "and shows nothing")
            // A file that is not there is "taken" too: fake mode looks at no file at all.
            let gone = await fake.open("gone.txt", "text/plain")
            XCTAssertEqual(gone, .taken)

            #if !os(macOS)
                let real = BrowserSession.filesOpener(fake: false, folder: folder, preview: preview)
                let shown = await real.open("notes.txt", "text/plain")
                XCTAssertEqual(shown, .taken)
                XCTAssertEqual(preview.file?.lastPathComponent, "notes.txt", "started for real, the preview has the file")
                preview.file = nil
                let missing = await real.open("gone.txt", "text/plain")
                XCTAssertEqual(missing, .missing)
                XCTAssertNil(preview.file)
            #endif
        }

        func testAnEngineNobodyGaveAFolderSavesAmongTheTemporaryFiles() {
            // What every test's engine is, and fake mode's: it cannot write into anybody's Downloads.
            let folder = WebKitEngine().downloadsFolder
            XCTAssertTrue(folder.path.hasPrefix(FileManager.default.temporaryDirectory.path), folder.path)
            XCTAssertNotEqual(folder, WebKitEngine().downloadsFolder, "and each engine has its own")
        }

        func testAnythingElseIsNoProfileAtAll() {
            let refused: [String?] = [
                nil, "", "..", "../other", "a/b", "a\\b", "~", "a b", "a.b", "naïve", "名前", String(repeating: "a", count: 41), "a\u{0}b",
                "-/",
            ]
            for name in refused {
                XCTAssertNil(BrowserSession.profileName(name), name ?? "nil")
            }
        }
    }
#endif

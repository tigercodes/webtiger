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

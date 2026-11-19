// A profile's name (009): the folder its data is kept in is named by it, so a name is letters,
// digits and hyphens and nothing else. Anything else is no profile, and never a path.
#if WEBTIGER_CORE
    @testable import WTUI
    import XCTest

    @MainActor
    final class ProfileTests: XCTestCase {
        func testANameIsLettersDigitsAndHyphens() {
            for name in ["work", "ui-test-session", "A1", String(repeating: "a", count: 40)] {
                XCTAssertEqual(BrowserSession.profileName(name), name, name)
            }
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

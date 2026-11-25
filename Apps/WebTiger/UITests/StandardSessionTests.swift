// The standard session of the plan's budgets (060), in the app: fake mode can start on it, for
// an app that is to be measured (`Tools/perf`). Five Spaces, two hundred Sites and forty tabs,
// and one page alive: the tab in front, at the start address. What `Tools/perf` measures is
// an app started this way; this is the test that such an app is on that session.
import XCTest

@MainActor
final class StandardSessionTests: AppTestCase {
    func testTheAppStartsOnTheStandardSessionWithOnePageAlive() {
        let app = launch(start: site.url("/"), standardSession: true)
        XCTAssertTrue(pageShows("Home", in: app), "the tab in front is at the start address")

        // What is awake is that one tab, and every other tab of the five Spaces is asleep:
        // two hundred Sites and forty tabs, but for the one.
        press(app.buttons["history"])
        let awake = app.buttons["library-awake"]
        XCTAssertTrue(awake.waitForExistence(timeout: 30), "the Library has the list of what is awake")
        press(awake)
        XCTAssertTrue(app.buttons["awake-done"].waitForExistence(timeout: 30), "what is awake")
        expect(app.descendants(matching: .any).matching(identifier: "awake-tab"), toCount: 1, "one tab is awake")
        let asleep = app.staticTexts["awake-asleep"]
        XCTAssertEqual(asleep.label.isEmpty ? asleep.value as? String ?? "" : asleep.label, "Asleep: 239")
        press(app.buttons["awake-done"])

        // Five Spaces, and no sixth.
        press(app.buttons["space-chip"])
        XCTAssertTrue(app.buttons["space-5"].waitForExistence(timeout: 30), "the list of Spaces has a fifth")
        XCTAssertFalse(app.buttons["space-6"].exists, "and no sixth")

        // One page was fetched, once: the start address. Nothing else of the session was.
        XCTAssertEqual(site.requests.filter { $0 == "/" }.count, 1, "\(site.requests)")
        XCTAssertFalse(site.requests.contains { $0.contains("space-") }, "\(site.requests)")
    }
}

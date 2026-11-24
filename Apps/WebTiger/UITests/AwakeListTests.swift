// What is awake (056), in the app, in fake mode: the real core and the real engine. The
// Library's fifth list (057) names every tab that has a page loaded, with where it is and why
// it is awake; "Sleep All Others" puts to sleep the ones that can be, which then leave the
// list and say they are asleep in their row; the tab on screen stays.
import XCTest

@MainActor
final class AwakeListTests: AppTestCase {
    /// Sends the tab in front to a page of the test's site.
    private func go(to path: String, titled title: String, in app: XCUIApplication) {
        let field = app.textFields["address-field"]
        openLocation(field, in: app)
        field.typeText("127.0.0.1:\(site.port)\(path)\n")
        XCTAssertTrue(pageShows(title, in: app), "the page at \(path) loaded")
    }

    /// Opens a tab in front of the others, at a page of the test's site.
    private func openTab(at path: String, titled title: String, in app: XCUIApplication) {
        press(app.buttons["new-tab"])
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "an empty tab")
        go(to: path, titled: title, in: app)
    }

    /// Opens the Library at what is awake.
    private func openAwake(in app: XCUIApplication, line: UInt = #line) {
        press(app.buttons["history"])
        let awake = app.buttons["library-awake"]
        XCTAssertTrue(awake.waitForExistence(timeout: 30), "the Library has the list of what is awake", line: line)
        press(awake)
        XCTAssertTrue(app.buttons["awake-done"].waitForExistence(timeout: 30), "what is awake", line: line)
    }

    func testTheLibraryListsWhatIsAwakeAndSleepAllOthersPutsTheRestToSleep() {
        let app = launch(start: site.url("/"))
        XCTAssertTrue(pageShows("Home", in: app), "the first tab's page loaded")
        go(to: "/landed", titled: "Landed", in: app)
        openTab(at: "/whoami", titled: "who=none", in: app)
        openTab(at: "/tall", titled: "Tall", in: app)

        // The three are listed in their row's order, each with why it is awake: the two
        // behind are only waiting for their time, and the third is on screen.
        openAwake(in: app)
        let rows = app.descendants(matching: .any).matching(identifier: "awake-tab")
        expect(rows, toCount: 3, "three tabs are awake")
        let first = rows.element(boundBy: 0).label
        let third = rows.element(boundBy: 2).label
        XCTAssertTrue(first.contains("Landed") && first.contains("Will sleep by itself"), "the first: \(first)")
        XCTAssertTrue(rows.element(boundBy: 1).label.contains("who=none"), "the second is the second of the row")
        XCTAssertTrue(third.contains("Tall") && third.contains("On screen"), "the third: \(third)")
        XCTAssertEqual(app.staticTexts["awake-asleep"].label, "Asleep: 0")
        let others = app.buttons["awake-sleep-others"]
        XCTAssertTrue(others.isEnabled, "there are others to put to sleep")
        #if os(macOS)
            // On a Mac, where the switch's five names are on one line. On an iPhone the row of
            // names is moved to its end to show this list's, and the audit changes the size
            // of the text as it looks: the row moves under it, and it has faulted the contrast
            // of a name it caught moving (057 §8). The lists whose names are at the row's
            // start are audited on an iPhone, and their names are drawn as these are.
            audit(app, "what is awake")
        #endif

        // Sleep All Others: the two behind sleep, and leave the list; the one on screen stays.
        press(others)
        expect(rows, toCount: 1, "one tab is left awake")
        let left = rows.element(boundBy: 0).label
        XCTAssertTrue(left.contains("Tall") && left.contains("On screen"), "the one left: \(left)")
        let asleep = XCTNSPredicateExpectation(
            predicate: NSPredicate(format: "label == %@", "Asleep: 2"),
            object: app.staticTexts["awake-asleep"]
        )
        XCTAssertEqual(XCTWaiter.wait(for: [asleep], timeout: 30), .completed, "two are counted asleep")
        XCTAssertFalse(others.isEnabled, "there are no others now")
        press(app.buttons["awake-done"])

        // The row says the same: the two behind are asleep where they were, the third is not.
        let tabs = app.windows.element(boundBy: 0).buttons.matching(identifier: "tab")
        expect(tabs.element(boundBy: 0), toSay: "Asleep", "the first is asleep in its row")
        expect(tabs.element(boundBy: 1), toSay: "Asleep", "and the second")
        expect(tabs.element(boundBy: 2), toSay: "", "the tab on screen is awake")
        XCTAssertTrue(pageShows("Tall", in: app))
    }
}

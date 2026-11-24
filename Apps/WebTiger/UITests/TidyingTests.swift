// Tidying (049) in the app, in fake mode, with the real core and the real engine: a Space is
// given its time on the page that edits it; the tabs that are not in front are tidied when
// their time is past, and are listed in the Library's Archive; one is forgotten, and one is
// brought back, a tab again.
//
// The core's clock moves a day each time it is read here (`WEBTIGER_FAKE_CLOCK_STEP`). The
// rule that runs is the core's own, at its own look once a minute, and only the clock is
// quicker: so the test waits for a look, a minute and more.
import XCTest

@MainActor
final class TidyingTests: AppTestCase {
    /// Opens a tab and loads a page of the suite's site in it. (Not called `open`: the
    /// formatter takes a line that begins with that word for one that begins with a modifier,
    /// and where two such lines follow each other it deletes the second.)
    private func loadInANewTab(_ path: String, titled title: String, in app: XCUIApplication, line: UInt = #line) {
        press(app.buttons["new-tab"])
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "an empty tab", line: line)
        let field = app.textFields["address-field"]
        openLocation(field, in: app)
        field.typeText("127.0.0.1:\(site.port)\(path)\n")
        XCTAssertTrue(pageShows(title, in: app), "\(title) loaded", line: line)
    }

    /// Opens the page that edits the Space the window shows.
    private func editSpace(in app: XCUIApplication, line: UInt = #line) {
        press(app.buttons["space-chip"])
        XCTAssertTrue(app.buttons["spaces-rename"].waitForExistence(timeout: 30), "the list of Spaces", line: line)
        press(app.buttons["spaces-rename"])
        XCTAssertTrue(app.buttons["space-save"].waitForExistence(timeout: 30), "the page that edits a Space", line: line)
    }

    /// Saves the page that edits a Space, and waits for it to go.
    private func save(in app: XCUIApplication, line: UInt = #line) {
        let button = app.buttons["space-save"]
        press(button)
        let gone = NSPredicate(format: "exists == NO")
        let waited = XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: gone, object: button)], timeout: 30)
        XCTAssertEqual(waited, .completed, "saved, the page is gone", line: line)
    }

    /// Opens the Library at its Archive.
    private func openArchive(in app: XCUIApplication, line: UInt = #line) {
        press(app.buttons["history"])
        let archive = app.buttons["library-archive"]
        XCTAssertTrue(archive.waitForExistence(timeout: 30), "the Library has an Archive", line: line)
        press(archive)
        XCTAssertTrue(app.buttons["archive-done"].waitForExistence(timeout: 30), "the Archive", line: line)
    }

    func testTabsOutOfSightForTheirSpacesTimeAreTidiedIntoTheArchiveAndBroughtBackFromIt() {
        let app = launch(start: site.url("/landed"), clockStep: 86400)
        XCTAssertTrue(pageShows("Landed", in: app), "the first page loaded")
        loadInANewTab("/", titled: "Home", in: app)
        loadInANewTab("/whoami", titled: "who=none", in: app)
        let tabs = app.windows.element(boundBy: 0).buttons.matching(identifier: "tab")
        expect(tabs, toBe: ["Landed", "Home", "who=none"], "three tabs, the last in front")

        // Nothing has been tidied, and nothing would be: the Archive says so.
        openArchive(in: app)
        XCTAssertTrue(app.staticTexts["archive-nothing"].waitForExistence(timeout: 30), "an Archive with nothing in it says so")
        XCTAssertFalse(app.buttons["archive-clear"].isEnabled, "and has nothing to forget")
        audit(app, "the Archive with nothing in it")
        press(app.buttons["archive-done"])

        // The page that edits the Space: never, as a Space is made; and now twelve hours.
        editSpace(in: app)
        let never = app.buttons["space-tidy-0"]
        let halfADay = app.buttons["space-tidy-43200"]
        XCTAssertTrue(never.waitForExistence(timeout: 30), "the page has the times")
        XCTAssertTrue(never.isSelected, "a Space is made with no time")
        let times = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@", "space-tidy-"))
        XCTAssertEqual(times.count, 5, "the times there are")
        audit(app, "a Space's time")
        press(halfADay)
        XCTAssertTrue(halfADay.isSelected, "the one pressed is the one chosen")
        XCTAssertFalse(never.isSelected)
        save(in: app)
        expect(tabs, toCount: 3, "nothing goes when the time is given: the tabs' time starts then")

        // The core looks once a minute. By its clock a day has gone at the next look: the two
        // tabs that are not in front are tidied, and the one in front stays, with its page.
        let one = NSPredicate(format: "count == 1")
        let waited = XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: one, object: tabs)], timeout: 150)
        XCTAssertEqual(waited, .completed, "the tabs out of sight were tidied: there are \(tabs.count)")
        XCTAssertEqual(tabs.element(boundBy: 0).label, "who=none", "the one in front stays")
        XCTAssertTrue(pageShows("who=none", in: app), "and its page is as it was")

        // The Space says its time. Never again, so that nothing more goes while the test looks.
        editSpace(in: app)
        XCTAssertTrue(halfADay.waitForExistence(timeout: 30))
        XCTAssertTrue(halfADay.isSelected, "the Space says the time it was given")
        press(never)
        save(in: app)

        // The Archive lists the two, newest first, each by its name.
        openArchive(in: app)
        let entries = app.buttons.matching(identifier: "archive-entry")
        expect(entries, toCount: 2, "the two tidied tabs")
        XCTAssertTrue(entries.element(boundBy: 0).label.contains("Home"), "newest first: \(entries.element(boundBy: 0).label)")
        XCTAssertTrue(entries.element(boundBy: 1).label.contains("Landed"), entries.element(boundBy: 1).label)
        audit(app, "the Archive")

        // One is forgotten.
        press(app.buttons.matching(identifier: "archive-forget").firstMatch)
        expect(entries, toCount: 1, "one is forgotten")
        XCTAssertTrue(entries.firstMatch.label.contains("Landed"), "the other is still listed")

        // The other is brought back: a tab again, the last of the row, in front, loading its
        // address; and the list is gone.
        press(entries.firstMatch)
        XCTAssertTrue(pageShows("Landed", in: app), "brought back, it is the page on screen")
        expect(tabs, toBe: ["who=none", "Landed"], "a tab again, the last of the row")
        XCTAssertTrue(tabs.element(boundBy: 1).isSelected, "and in front")
        XCTAssertFalse(app.buttons["archive-done"].exists, "the list is gone")

        // The Archive has it no longer; and with no time, the tab that left the front stays.
        openArchive(in: app)
        XCTAssertTrue(app.staticTexts["archive-nothing"].waitForExistence(timeout: 30), "the Archive lists nothing now")
        press(app.buttons["archive-done"])
        expect(tabs, toCount: 2, "both tabs are there")
    }
}

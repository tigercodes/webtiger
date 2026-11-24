// A Space's icon (050) in the app, in fake mode, with the real core and the real engine: the
// page that makes a Space offers the icons the core lists, and a Space is made with the one
// chosen; the page that edits it says which it has, and gives it another, or none. The chip is
// said by the Space's name throughout: the icon is drawn, and is not its name.
import XCTest

@MainActor
final class SpaceIconTests: AppTestCase {
    /// Waits for the chip to say it shows the Space with this name.
    private func chipSays(_ name: String, in app: XCUIApplication) -> Bool {
        let chip = app.buttons["space-chip"]
        let says = NSPredicate(format: "label == %@", "Space: \(name)")
        return XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: says, object: chip)], timeout: 30) == .completed
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

    func testASpaceIsMadeWithAnIconAndGivenAnotherOrNoneWhereItIsEdited() {
        let app = launch(start: site.url("/"))
        XCTAssertTrue(pageShows("Home", in: app), "the window's page loaded")

        // The page that makes a Space: none, and the icons the core lists; none is the one
        // chosen until another is.
        press(app.buttons["space-chip"])
        XCTAssertTrue(app.buttons["spaces-new"].waitForExistence(timeout: 30), "the list of Spaces")
        press(app.buttons["spaces-new"])
        let name = app.textFields["space-name"]
        XCTAssertTrue(name.waitForExistence(timeout: 30), "the page that makes a Space")
        press(name)
        name.typeText("Work")
        let none = app.buttons["space-icon-none"]
        let house = app.buttons["space-icon-1"]
        let briefcase = app.buttons["space-icon-2"]
        XCTAssertTrue(none.waitForExistence(timeout: 30), "the page has the icons")
        XCTAssertTrue(none.isSelected, "a Space is given no icon it was not asked for")
        let icons = app.buttons.matching(NSPredicate(format: "identifier BEGINSWITH %@", "space-icon-"))
        XCTAssertEqual(icons.count, 25, "none, and the core's twenty-four")
        audit(app, "the icons a Space can have")
        press(briefcase)
        XCTAssertTrue(briefcase.isSelected, "the one pressed is the one chosen")
        XCTAssertFalse(none.isSelected)
        press(app.buttons["space-make"])
        XCTAssertTrue(chipSays("Work", in: app), "the window shows the Space that was made, and says its name: not its icon")

        // The page that edits it says the icon it was made with. Another is given.
        editSpace(in: app)
        XCTAssertTrue(briefcase.waitForExistence(timeout: 30))
        XCTAssertTrue(briefcase.isSelected, "the Space has the icon it was made with")
        XCTAssertFalse(house.isSelected)
        press(house)
        save(in: app)
        XCTAssertTrue(chipSays("Work", in: app))

        // And it is the one it has when the page opens again; none is an icon to give too.
        editSpace(in: app)
        XCTAssertTrue(house.waitForExistence(timeout: 30))
        XCTAssertTrue(house.isSelected, "the icon it was given")
        XCTAssertFalse(briefcase.isSelected)
        press(none)
        save(in: app)
        editSpace(in: app)
        XCTAssertTrue(none.waitForExistence(timeout: 30))
        XCTAssertTrue(none.isSelected, "given none, it has none")
        press(app.buttons["space-back"])

        // The Space the app made itself has none, and its name is what the list says of it.
        XCTAssertTrue(app.buttons["space-1"].waitForExistence(timeout: 30), "the list again")
        XCTAssertTrue(app.buttons["space-1"].label.contains("Home"), app.buttons["space-1"].label)
        XCTAssertTrue(app.buttons["space-2"].label.contains("Work"), app.buttons["space-2"].label)
    }
}

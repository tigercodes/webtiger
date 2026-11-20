// Several Spaces (024) in the app, in fake mode, with the real core and the real engine: a
// Space is made from the chip in the strip of tabs, the window shows it, and its tabs and its
// logins are its own.
import XCTest

@MainActor
final class SpaceTests: AppTestCase {
    /// What the chip says: "Space: " and the name of the Space the window shows.
    private func chipSays(_ name: String, in app: XCUIApplication) -> Bool {
        let chip = app.buttons["space-chip"]
        let says = NSPredicate(format: "label == %@", "Space: \(name)")
        return XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: says, object: chip)], timeout: 30) == .completed
    }

    /// Sends the tab in front to a page of the test site by typing its address.
    private func go(to path: String, in app: XCUIApplication) {
        let field = app.textFields["address-field"]
        openLocation(field, in: app)
        field.typeText("127.0.0.1:\(site.port)\(path)\n")
    }

    /// Opens the list of Spaces and the page that makes one, types a name, chooses whose
    /// logins, and makes it.
    private func makeSpace(_ name: String, sharing: Bool, in app: XCUIApplication) {
        press(app.buttons["space-chip"])
        XCTAssertTrue(app.buttons["spaces-new"].waitForExistence(timeout: 30), "the list of Spaces")
        press(app.buttons["spaces-new"])
        let field = app.textFields["space-name"]
        XCTAssertTrue(field.waitForExistence(timeout: 30), "the page that makes a Space")
        press(field)
        field.typeText(name)
        if sharing {
            press(app.buttons["space-logins-shared"])
        }
        press(app.buttons["space-make"])
    }

    /// A Space is made with logins of its own: the window shows it, with one empty tab, and a
    /// site that knows who is signed in in the first Space does not know them here. Back in
    /// the first Space its tab is as it was left. A Space made to share the first one's logins
    /// is signed in as the first is.
    ///
    /// The cookie is one for the session only, which an engine keeps in memory: nothing of it
    /// is there for the next run, in any Space.
    func testASpaceIsMadeAndKeepsItsTabsAndItsLoginsApart() {
        let app = launch(start: site.url("/session?v=ada"))
        XCTAssertTrue(pageShows("Session", in: app), "the first Space's page signed in")
        XCTAssertTrue(chipSays("Home", in: app), "the Space the app made has the shell's word for a name")
        let tabs = app.buttons.matching(identifier: "tab")

        // The list: the one Space there is, which the window shows.
        press(app.buttons["space-chip"])
        let home = app.buttons["space-1"]
        XCTAssertTrue(home.waitForExistence(timeout: 30), "the list of Spaces")
        XCTAssertTrue(home.label.contains("Home"), home.label)
        XCTAssertTrue(home.label.contains("Shown"), home.label)
        audit(app, "the list of Spaces")

        // The page that makes one: nothing is made of no name.
        press(app.buttons["spaces-new"])
        let name = app.textFields["space-name"]
        XCTAssertTrue(name.waitForExistence(timeout: 30), "the page that makes a Space")
        XCTAssertFalse(app.buttons["space-make"].isEnabled, "no name yet")
        press(name)
        name.typeText("Work")
        XCTAssertTrue(app.buttons["space-make"].isEnabled)
        audit(app, "making a Space")
        press(app.buttons["space-make"])

        // The window shows it: one empty tab, and logins of its own.
        XCTAssertTrue(chipSays("Work", in: app), "the window shows the Space that was made")
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "a new Space has one empty tab")
        XCTAssertEqual(tabs.count, 1)
        XCTAssertFalse(name.exists, "the page that made it is gone")
        go(to: "/whoami", in: app)
        XCTAssertTrue(pageShows("who=none", in: app), "nobody is signed in in a Space with logins of its own")

        // Back in the first Space, its tab is as it was left, and its login is its own still.
        press(app.buttons["space-chip"])
        XCTAssertTrue(home.waitForExistence(timeout: 30))
        XCTAssertTrue(app.buttons["space-2"].label.contains("Work"), app.buttons["space-2"].label)
        XCTAssertTrue(app.buttons["space-2"].label.contains("Shown"), app.buttons["space-2"].label)
        press(home)
        XCTAssertTrue(chipSays("Home", in: app))
        XCTAssertTrue(pageShows("Session", in: app), "the tab that was left is as it was")
        XCTAssertEqual(tabs.count, 1)
        go(to: "/whoami", in: app)
        XCTAssertTrue(pageShows("who=ada", in: app), "the first Space is signed in as it was")

        // A Space that shares the first one's logins is signed in as the first is.
        makeSpace("Project", sharing: true, in: app)
        XCTAssertTrue(chipSays("Project", in: app))
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30))
        go(to: "/whoami", in: app)
        XCTAssertTrue(pageShows("who=ada", in: app), "a Space that shares logins is signed in where the other is")

        // The list says who shares; and Work is as it was left, signed in as nobody.
        press(app.buttons["space-chip"])
        let work = app.buttons["space-2"]
        XCTAssertTrue(work.waitForExistence(timeout: 30))
        XCTAssertTrue(app.buttons["space-3"].label.contains("Shares logins with Home"), app.buttons["space-3"].label)
        XCTAssertTrue(home.label.contains("Shares logins with Project"), home.label)
        XCTAssertFalse(work.label.contains("Shares"), work.label)
        press(work)
        XCTAssertTrue(chipSays("Work", in: app))
        XCTAssertTrue(pageShows("who=none", in: app), "Work's tab is as it was left")
    }
}

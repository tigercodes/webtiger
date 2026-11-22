// Private windows (037) in the app, in fake mode, with the real core and the real engine: one
// is opened, signs in to the test site, and is closed. What it is signed in as is its own
// while it is open, is not the other window's, and is gone with it.
import XCTest

@MainActor
final class PrivateTests: AppTestCase {
    /// Sends the tab in front, of the window in front, to a page of the test site.
    private func go(to path: String, in app: XCUIApplication) {
        #if os(macOS)
            // Two windows, and an address field in each: the keys go to the one in front.
            app.typeKey("l", modifierFlags: .command)
            app.typeText("127.0.0.1:\(site.port)\(path)\n")
        #else
            let field = app.textFields["address-field"]
            openLocation(field, in: app)
            field.typeText("127.0.0.1:\(site.port)\(path)\n")
        #endif
    }

    /// Waits for something of the app's to be there, and presses it.
    private func pressWhenThere(_ element: XCUIElement, _ what: String, line: UInt = #line) {
        XCTAssertTrue(element.waitForExistence(timeout: 30), what, line: line)
        press(element)
    }

    #if os(macOS)
        /// How many of the app's windows show a page with this title.
        private func windows(titled title: String, in app: XCUIApplication) -> Int {
            app.windows.matching(NSPredicate(format: "title == %@", title)).count
        }

        /// Brings the window with this title in front, by its item in the Window menu.
        private func bringForward(_ title: String, in app: XCUIApplication, line: UInt = #line) {
            let menu = app.menuBars.menuBarItems["Window"]
            menu.click()
            pressWhenThere(menu.menuItems[title], "the Window menu lists \(title)", line: line)
        }

        /// A private window is a window of its own on a Mac: New Private Window, ⇧⌘N.
        func testAPrivateWindowsLoginsAreItsOwnAndGoWithIt() {
            let app = launch(start: site.url("/"))
            XCTAssertTrue(pageShows("Home", in: app), "the window's page loaded")
            XCTAssertFalse(app.buttons["private-chip"].exists, "an ordinary window says nothing of private")

            // From the File menu. It opens in front, where a new window opens in fake mode,
            // says what it is, and signs in to the site.
            let file = app.menuBars.menuBarItems["File"]
            file.click()
            pressWhenThere(file.menuItems["New Private Window"], "the File menu has New Private Window")
            XCTAssertTrue(app.buttons["private-chip"].waitForExistence(timeout: 30), "the new window says it is private")
            let two = NSPredicate(format: "count == 2")
            XCTAssertEqual(XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: two, object: app.windows)], timeout: 30), .completed)
            go(to: "/set?v=ada", in: app)
            XCTAssertTrue(pageShows("Set", in: app))
            go(to: "/whoami", in: app)
            XCTAssertTrue(pageShows("who=ada", in: app), "signed in, in the private window")

            // The other window is not signed in.
            bringForward("Home", in: app)
            go(to: "/whoami", in: app)
            XCTAssertTrue(pageShows("who=none", in: app), "the ordinary window is not signed in")
            XCTAssertEqual(windows(titled: "who=ada", in: app), 1, "and the private one is as it was")

            // The page behind the chip says what private means, and closes the window.
            bringForward("who=ada", in: app)
            pressWhenThere(app.buttons["private-chip"], "the private window's chip")
            XCTAssertTrue(app.staticTexts["private-keeps"].waitForExistence(timeout: 30), "what private means, in words")
            XCTAssertFalse(app.buttons["private-leave"].exists, "on a Mac both windows are on the screen: there is no way back to offer")
            press(app.buttons["private-close"])
            let one = NSPredicate(format: "count == 1")
            XCTAssertEqual(XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: one, object: app.windows)], timeout: 30), .completed)
            XCTAssertEqual(windows(titled: "who=none", in: app), 1)

            // Another private window, by its key, knows nothing of the first.
            app.typeKey("n", modifierFlags: [.command, .shift])
            XCTAssertTrue(app.buttons["private-chip"].waitForExistence(timeout: 30), "⇧⌘N opened a private window")
            go(to: "/whoami", in: app)
            let both = NSPredicate(format: "count == 2")
            let unsigned = app.windows.matching(NSPredicate(format: "title == %@", "who=none"))
            XCTAssertEqual(
                XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: both, object: unsigned)], timeout: 30), .completed,
                "neither window is signed in"
            )
            XCTAssertEqual(windows(titled: "who=ada", in: app), 0)
        }
    #else
        /// A phone's app shows one window at a time (037 §5.3): a private window is opened
        /// from the list of Spaces, and is shown in the place of the first.
        func testAPrivateWindowsLoginsAreItsOwnAndGoWithIt() {
            let app = launch(start: site.url("/"))
            XCTAssertTrue(pageShows("Home", in: app), "the window's page loaded")
            XCTAssertFalse(app.buttons["private-chip"].exists, "an ordinary window says nothing of private")

            // The list of Spaces, and its last row. The private window opens where a new
            // window opens in fake mode, says what it is, and signs in to the site.
            press(app.buttons["space-chip"])
            pressWhenThere(app.buttons["space-private"], "the list of Spaces ends with the private window")
            XCTAssertTrue(app.buttons["private-chip"].waitForExistence(timeout: 30), "the window says it is private")
            XCTAssertFalse(app.buttons["space-chip"].exists, "and names no Space")
            go(to: "/set?v=ada", in: app)
            XCTAssertTrue(pageShows("Set", in: app))
            go(to: "/whoami", in: app)
            XCTAssertTrue(pageShows("who=ada", in: app), "signed in, in the private window")

            // Left open, and the other window shown: it is not signed in.
            press(app.buttons["private-chip"])
            XCTAssertTrue(app.staticTexts["private-keeps"].waitForExistence(timeout: 30), "what private means, in words")
            pressWhenThere(app.buttons["private-leave"], "the way back to the other window")
            XCTAssertTrue(app.buttons["space-chip"].waitForExistence(timeout: 30), "the other window is shown")
            go(to: "/whoami", in: app)
            XCTAssertTrue(pageShows("who=none", in: app), "the ordinary window is not signed in")

            // The list says a private window is open; its row shows it, as it was left.
            press(app.buttons["space-chip"])
            XCTAssertTrue(app.staticTexts["space-private-open"].waitForExistence(timeout: 30), "the list says one is open")
            press(app.buttons["space-private"])
            XCTAssertTrue(app.buttons["private-chip"].waitForExistence(timeout: 30))
            XCTAssertTrue(pageShows("who=ada", in: app), "as it was left")

            // Closed: the other window is shown, and no private window is open.
            press(app.buttons["private-chip"])
            pressWhenThere(app.buttons["private-close"], "Close Private Window")
            XCTAssertTrue(app.buttons["space-chip"].waitForExistence(timeout: 30))
            XCTAssertTrue(pageShows("who=none", in: app))
            press(app.buttons["space-chip"])
            XCTAssertTrue(app.buttons["space-private"].waitForExistence(timeout: 30))
            XCTAssertFalse(app.staticTexts["space-private-open"].exists, "the list no longer says one is open")

            // Another private window knows nothing of the first.
            press(app.buttons["space-private"])
            XCTAssertTrue(app.buttons["private-chip"].waitForExistence(timeout: 30))
            go(to: "/whoami", in: app)
            XCTAssertTrue(pageShows("who=none", in: app), "another private window is not signed in")
        }
    #endif
}

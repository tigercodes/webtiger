// The phone's switcher (047) in the app, in fake mode, with the real core and the real engine:
// in a window with no room for a sidebar, a button says how many tabs the row has, those of a
// shut Group among them, and opens a sheet with the whole row; a tab chosen there comes in
// front, and the sheet goes; nothing is loaded for any of it. A window with room for a
// sidebar has no switcher: a Mac's, and an iPad's at its full width.
import XCTest

@MainActor
final class SwitcherTests: AppTestCase {
    /// Waits for the switcher's button to say this many tabs.
    private func expect(_ button: XCUIElement, toCount count: Int, _ what: String, line: UInt = #line) {
        let says = NSPredicate(format: "label == %@", "Tabs: \(count)")
        let waited = XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: says, object: button)], timeout: 30)
        XCTAssertEqual(waited, .completed, "\(what): the button says '\(button.label)'", line: line)
    }

    /// The one of these that can be pressed now. With the switcher open the strip is behind
    /// it: its tabs and its Group's chip are still there, by the same names, and cannot be
    /// pressed; the switcher's can.
    private func pressable(_ elements: XCUIElementQuery, _ what: String, line: UInt = #line) -> XCUIElement {
        let deadline = Date().addingTimeInterval(30)
        var found: XCUIElement?
        repeat {
            found = (0 ..< elements.count).map { elements.element(boundBy: $0) }.first { $0.exists && $0.isHittable }
            if found == nil {
                usleep(200_000)
            }
        } while found == nil && Date() < deadline
        XCTAssertNotNil(found, "\(what) can be pressed", line: line)
        return found ?? elements.firstMatch
    }

    func testTheWholeRowIsInTheSwitcherAndATabChosenThereComesInFront() {
        let app = launch(start: site.url("/landed"))
        XCTAssertTrue(pageShows("Landed", in: app), "the first tab's page loaded")
        let button = app.buttons["switcher"]
        #if os(macOS)
            // A Mac's window has room for a sidebar: it has no switcher.
            XCTAssertFalse(button.exists, "a Mac's window has no switcher")
        #else
            // An iPad's window at its full width has the layout's button, and no switcher.
            if app.buttons["layout"].exists {
                XCTAssertFalse(button.exists, "a window with room for a sidebar has no switcher")
                return
            }
            theSwitcherOfAPhone(app, button)
        #endif
    }

    #if !os(macOS)
        private func theSwitcherOfAPhone(_ app: XCUIApplication, _ button: XCUIElement) {
            let field = app.textFields["address-field"]
            let tabs = app.windows.element(boundBy: 0).buttons.matching(identifier: "tab")
            let chips = app.descendants(matching: .any).matching(identifier: "group")
            let chip = chips.firstMatch
            XCTAssertTrue(button.waitForExistence(timeout: 30), "a phone's toolbar has the switcher's button")
            expect(button, toCount: 1, "one tab")

            // A second tab, somewhere else; the first put in a Group, and the Group shut: the
            // strip shows one tab, and the button says the row has two.
            press(app.buttons["new-tab"])
            XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "a second, empty tab")
            openLocation(field, in: app)
            field.typeText("127.0.0.1:\(site.port)/whoami\n")
            XCTAssertTrue(pageShows("who=none", in: app), "the second tab's page loaded")
            expect(button, toCount: 2, "two tabs")
            makeGroup("Work", for: tabs.element(boundBy: 0), in: app)
            XCTAssertTrue(chip.waitForExistence(timeout: 30), "the Group's chip is in the row")
            press(chip)
            expect(chip, toSay: "Group, shut, Sites: 1", "the Group is shut")
            expect(tabs, toCount: 1, "the strip shows the one tab that is out of no way")
            expect(button, toCount: 2, "and the row has two still")
            let asked = site.requests.count

            // The switcher: a sheet with the row. Its Group is shut there too; opened, it has
            // its Site, by its name.
            press(button)
            let done = app.buttons["switcher-done"]
            XCTAssertTrue(done.waitForExistence(timeout: 30), "the switcher is open")
            audit(app, "the switcher")
            let listed = pressable(chips, "the Group's chip in the switcher")
            expect(listed, toSay: "Group, shut, Sites: 1", "shut, in the switcher as in the strip")
            press(listed)
            expect(listed, toSay: "Group, open", "the Group is open, in the switcher")
            let kept = pressable(tabs.matching(NSPredicate(format: "label == %@", "Landed")), "the Group's Site, a row of the switcher")

            // Chosen, it comes in front, and the switcher goes. Nothing was loaded.
            press(kept)
            XCTAssertTrue(pageShows("Landed", in: app), "the tab chosen in the switcher is in front")
            let gone = NSPredicate(format: "exists == false")
            XCTAssertEqual(
                XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: gone, object: done)], timeout: 30), .completed,
                "the switcher put itself away"
            )
            XCTAssertEqual(site.requests.count, asked, "the switcher loaded nothing")

            // A row's own menu works in the switcher as in the strip. What it opens is a page
            // of the window's own: the switcher makes way for it. A second Group is made for
            // the other tab, from its row.
            press(button)
            XCTAssertTrue(done.waitForExistence(timeout: 30), "the switcher is open")
            let other = pressable(tabs.matching(NSPredicate(format: "label == %@", "who=none")), "the other tab, a row of the switcher")
            makeGroup("Play", for: other, in: app)
            XCTAssertEqual(
                XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: gone, object: done)], timeout: 30), .completed,
                "the switcher made way for the page of a Group"
            )
            expect(chips, toCount: 2, "and the Group was made: the strip has two Groups' chips")
            expect(button, toCount: 2, "the row has its two tabs still")
            XCTAssertEqual(site.requests.count, asked, "none of it loaded anything")

            // Opened again and done with: the tab in front is the one that was.
            press(button)
            XCTAssertTrue(done.waitForExistence(timeout: 30), "the switcher is open again")
            press(done)
            XCTAssertEqual(
                XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: gone, object: done)], timeout: 30), .completed,
                "Done puts the switcher away"
            )
            XCTAssertTrue(pageShows("Landed", in: app), "and nothing changed")
            XCTAssertEqual(site.requests.count, asked)
        }
    #endif
}

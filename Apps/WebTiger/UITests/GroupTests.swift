// Groups (041) in the app, in fake mode, with the real core and the real engine: a tab is put
// in a new Group and is kept by it; the Group's chip shuts it and opens it; the Group is
// edited; the Site is taken out and put back; and the Group is removed, which leaves the Site.
import XCTest

@MainActor
final class GroupTests: AppTestCase {
    /// Waits for an element of the strip to say this of itself beside its name.
    private func expect(_ element: XCUIElement, toSay said: String, _ what: String, line: UInt = #line) {
        let says = NSPredicate(format: "value == %@", said)
        let waited = XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: says, object: element)], timeout: 30)
        XCTAssertEqual(waited, .completed, "\(what): it says '\(element.value as? String ?? "")'", line: line)
    }

    /// Waits for there to be this many of something.
    private func expect(_ elements: XCUIElementQuery, toCount count: Int, _ what: String, line: UInt = #line) {
        let counted = NSPredicate(format: "count == %d", count)
        let waited = XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: counted, object: elements)], timeout: 30)
        XCTAssertEqual(waited, .completed, "\(what): there are \(elements.count)", line: line)
    }

    /// The item of a menu that is open now, by its words: the one that can be pressed.
    private func item(_ words: String, in app: XCUIApplication, line: UInt = #line) -> XCUIElement? {
        #if os(macOS)
            let named = app.menuItems.matching(NSPredicate(format: "title == %@", words))
        #else
            let named = app.buttons.matching(NSPredicate(format: "label == %@", words))
        #endif
        let deadline = Date().addingTimeInterval(30)
        var found: XCUIElement?
        repeat {
            found = (0 ..< named.count).map { named.element(boundBy: $0) }.first { $0.exists && $0.isHittable }
            if found == nil {
                usleep(200_000)
            }
        } while found == nil && Date() < deadline
        XCTAssertNotNil(found, "a menu has '\(words)'", line: line)
        return found
    }

    /// Chooses from "Move to Group" for a tab: in the File menu on a Mac, for the tab in
    /// front; in the tab's own menu on a phone.
    private func moveToGroup(_ choice: String, _ tab: XCUIElement, in app: XCUIApplication, line: UInt = #line) {
        #if os(macOS)
            app.menuBars.menuBarItems["File"].click()
        #else
            tab.press(forDuration: 1)
        #endif
        if let menu = item("Move to Group", in: app, line: line) {
            press(menu)
        }
        if let chosen = item(choice, in: app, line: line) {
            press(chosen)
        }
    }

    /// Chooses from a Group's own menu: a right-click on its chip on a Mac, a long press on a
    /// phone.
    private func choose(_ words: String, of chip: XCUIElement, in app: XCUIApplication, line: UInt = #line) {
        #if os(macOS)
            chip.rightClick()
        #else
            chip.press(forDuration: 1)
        #endif
        if let chosen = item(words, in: app, line: line) {
            press(chosen)
        }
    }

    /// Empties a field: every character it holds is deleted.
    private func clear(_ field: XCUIElement, in app: XCUIApplication) {
        press(field)
        #if os(macOS)
            app.typeKey("a", modifierFlags: .command)
            field.typeText(XCUIKeyboardKey.delete.rawValue)
        #else
            field.typeText(String(repeating: XCUIKeyboardKey.delete.rawValue, count: value(of: field).count))
        #endif
    }

    /// Makes a Group named "Work" for the first tab, by "New Group…", and opens a second tab
    /// somewhere else: something to show while the Group is shut.
    private func aGroupAndASecondTab(in app: XCUIApplication, chip: XCUIElement, tabs: XCUIElementQuery) {
        let first = tabs.element(boundBy: 0)
        // The page asks for a name, and makes nothing of none. Made, the Group has its chip
        // in the row, open, and the tab is a Site: put in a Group, it is kept.
        moveToGroup("New Group…", first, in: app)
        let name = app.textFields["group-name"]
        XCTAssertTrue(name.waitForExistence(timeout: 30), "the page that makes a Group")
        XCTAssertFalse(app.buttons["group-done"].isEnabled, "no name yet")
        press(name)
        name.typeText("Work")
        XCTAssertTrue(app.buttons["group-done"].isEnabled)
        audit(app, "making a Group")
        press(app.buttons["group-done"])
        XCTAssertTrue(chip.waitForExistence(timeout: 30), "the Group's chip is in the row")
        XCTAssertEqual(chip.label, "Work")
        expect(chip, toSay: "Group, open", "a new Group")
        expect(first, toSay: "Site", "put in a Group, the tab is kept")
        XCTAssertFalse(name.exists, "the page closed")

        let field = app.textFields["address-field"]
        press(app.buttons["new-tab"])
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "a second, empty tab")
        openLocation(field, in: app)
        field.typeText("127.0.0.1:\(site.port)/whoami\n")
        XCTAssertTrue(pageShows("who=none", in: app), "the second tab's page loaded")
        expect(tabs, toCount: 2, "two tabs")
    }

    func testATabIsPutInANewGroupWhichIsShutAndOpenedByItsChip() {
        let app = launch(start: site.url("/landed"))
        XCTAssertTrue(pageShows("Landed", in: app), "the first tab's page loaded")
        let tabs = app.buttons.matching(identifier: "tab")
        let chips = app.descendants(matching: .any).matching(identifier: "group")
        let chip = chips.firstMatch
        XCTAssertEqual(chips.count, 0, "no Group yet")
        aGroupAndASecondTab(in: app, chip: chip, tabs: tabs)
        let asked = site.requests.count

        // A press on the chip shuts the Group: its Site is out of the way, and the chip says
        // how many it holds. Pressed again it is open, and the Site is where it was. Neither
        // loads anything.
        press(chip)
        expect(chip, toSay: "Group, shut, Sites: 1", "shut")
        expect(tabs, toCount: 1, "the Group's Site is out of the way")
        XCTAssertTrue(pageShows("who=none", in: app), "the tab in front is as it was")
        audit(app, "a shut Group in the strip")
        press(chip)
        expect(chip, toSay: "Group, open", "open again")
        expect(tabs, toCount: 2, "the Site is back")
        XCTAssertEqual(site.requests.count, asked, "shutting and opening a Group loaded nothing")
    }

    func testAGroupIsEditedASiteIsTakenOutAndPutBackAndTheGroupIsRemoved() {
        let app = launch(start: site.url("/landed"))
        XCTAssertTrue(pageShows("Landed", in: app), "the first tab's page loaded")
        let tabs = app.buttons.matching(identifier: "tab")
        let first = tabs.element(boundBy: 0)
        let chips = app.descendants(matching: .any).matching(identifier: "group")
        let chip = chips.firstMatch
        aGroupAndASecondTab(in: app, chip: chip, tabs: tabs)

        // The Group's own menu: edited, it has another name.
        choose("Edit Group…", of: chip, in: app)
        let name = app.textFields["group-name"]
        XCTAssertTrue(name.waitForExistence(timeout: 30), "the page that edits a Group")
        XCTAssertEqual(value(of: name), "Work", "it holds the name the Group has")
        audit(app, "editing a Group")
        clear(name, in: app)
        name.typeText("Office")
        press(app.buttons["group-done"])
        let renamed = NSPredicate(format: "label == %@", "Office")
        XCTAssertEqual(
            XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: renamed, object: chip)], timeout: 30), .completed,
            "the chip has the new name: '\(chip.label)'"
        )

        // The Site in front, and taken out of its Group by "No Group": it is a Site still, and
        // the Group, shut, hides nothing and holds nothing.
        press(first)
        XCTAssertTrue(pageShows("Landed", in: app), "the Site is in front")
        moveToGroup("No Group", first, in: app)
        press(chip)
        expect(chip, toSay: "Group, shut, Sites: 0", "the Group is empty")
        expect(tabs, toCount: 2, "nothing is out of the way")
        expect(first, toSay: "Site", "taken out, it is a Site still")
        press(chip)
        expect(chip, toSay: "Group, open", "open again")

        // Put back by the Group's name; another tab in front, and the Group shut: out of the
        // way again.
        moveToGroup("Office", first, in: app)
        press(tabs.element(boundBy: 1))
        XCTAssertTrue(pageShows("who=none", in: app), "the other tab is in front")
        press(chip)
        expect(chip, toSay: "Group, shut, Sites: 1", "it holds the Site again")
        expect(tabs, toCount: 1, "the Site is out of the way")

        // Removed, shut as it is: the chip is gone, and its Site stays, a Site in no Group.
        choose("Remove Group", of: chip, in: app)
        expect(chips, toCount: 0, "the Group is gone")
        expect(tabs, toCount: 2, "its Site stays")
        expect(tabs.element(boundBy: 0), toSay: "Site", "a Site still")
    }
}

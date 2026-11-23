// The command bar (048) in the app, in fake mode, with the real core and the real engine: the
// address field's list has commands and Spaces in it. A command found by its words is a row
// that says it is one, and chosen it is done; a Space found by its name is shown in the
// window. Nothing is loaded by either.
import XCTest

@MainActor
final class CommandBarTests: AppTestCase {
    /// Types in the address field and waits for a row of the list that has these words and
    /// says it is this kind of thing.
    private func row(
        _ words: String, _ kind: String, afterTyping text: String, in app: XCUIApplication, line: UInt = #line
    ) -> XCUIElement {
        let field = app.textFields["address-field"]
        openLocation(field, in: app)
        field.typeText(text)
        let found = NSPredicate(format: "label CONTAINS %@ AND label CONTAINS %@", words, kind)
        let row = app.buttons.matching(identifier: "suggestion").matching(found).firstMatch
        XCTAssertTrue(row.waitForExistence(timeout: 30), "\"\(text)\" offers \(words), a \(kind)", line: line)
        return row
    }

    /// What the chip says: "Space: " and the name of the Space the window shows.
    private func shows(space name: String, in app: XCUIApplication) -> Bool {
        let chip = app.buttons["space-chip"]
        let says = NSPredicate(format: "label == %@", "Space: \(name)")
        return XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: says, object: chip)], timeout: 30) == .completed
    }

    func testACommandFoundByItsWordsIsDoneWhenItsRowIsChosen() {
        let app = launch(start: site.url("/landed"))
        XCTAssertTrue(pageShows("Landed", in: app), "the page loaded")
        let field = app.textFields["address-field"]
        let tabs = app.windows.element(boundBy: 0).buttons.matching(identifier: "tab")
        let asked = site.requests.count

        // "keep": the command that keeps the tab, by the words its menu has. The row says it
        // is a command.
        let keep = row("Keep as Site", "Command", afterTyping: "keep", in: app)
        audit(app, "a command offered")
        press(keep)
        expect(tabs.element(boundBy: 0), toSay: "Site", "chosen, the command was done: the tab is kept")
        // The field has the page's address again, and the list has gone.
        expect(field, toShow: "http://127.0.0.1:\(site.port)/landed")
        XCTAssertEqual(app.buttons.matching(identifier: "suggestion").count, 0, "the list is put away")

        // The same words find the other command now: what can be done is what is offered.
        let letGo = row("Stop Keeping", "Command", afterTyping: "keep", in: app)
        press(letGo)
        expect(tabs.element(boundBy: 0), toSay: "", "let go again")

        // "new t": a tab is opened.
        let new = row("New Tab", "Command", afterTyping: "new t", in: app)
        press(new)
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "chosen, a new tab was opened")
        expect(tabs, toCount: 2, "two tabs")
        #if os(macOS)
            // "side": the tabs go down the side; and then the same word finds the way back.
            press(row("Tabs at the Side", "Command", afterTyping: "side", in: app))
            expect(app.descendants(matching: .any).matching(identifier: "sidebar"), toCount: 1, "the tabs are down the side")
            press(row("Tabs at the Top", "Command", afterTyping: "tabs to", in: app))
            expect(app.descendants(matching: .any).matching(identifier: "sidebar"), toCount: 0, "and along the top again")
        #endif
        XCTAssertEqual(site.requests.count, asked, "no command loaded anything")
    }

    func testASpaceFoundByItsNameIsShownWhenItsRowIsChosen() {
        let app = launch(start: site.url("/landed"))
        XCTAssertTrue(pageShows("Landed", in: app), "the page loaded")
        XCTAssertTrue(shows(space: "Home", in: app))
        // A second Space, "Work": the window shows it once it is made.
        press(app.buttons["space-chip"])
        XCTAssertTrue(app.buttons["spaces-new"].waitForExistence(timeout: 30), "the list of Spaces")
        press(app.buttons["spaces-new"])
        let name = app.textFields["space-name"]
        XCTAssertTrue(name.waitForExistence(timeout: 30), "the page that makes a Space")
        press(name)
        name.typeText("Work")
        press(app.buttons["space-make"])
        XCTAssertTrue(shows(space: "Work", in: app), "the window shows the Space that was made")
        let asked = site.requests.count

        // Typed there, its own name finds nothing: it is where the person is. A third Space
        // is what is offered there; here there is none.
        let field = app.textFields["address-field"]
        openLocation(field, in: app)
        field.typeText("wor")
        usleep(1_500_000)
        let spaces = app.buttons.matching(identifier: "suggestion").matching(NSPredicate(format: "label CONTAINS %@", "Space"))
        XCTAssertEqual(spaces.count, 0, "the Space the window shows is not offered")

        // Back in the first Space by the list of Spaces; and from there "wor" finds Work.
        #if os(macOS)
            app.typeKey(.escape, modifierFlags: [])
        #endif
        press(app.buttons["space-chip"])
        let home = app.buttons["space-1"]
        XCTAssertTrue(home.waitForExistence(timeout: 30), "the list of Spaces")
        press(home)
        XCTAssertTrue(shows(space: "Home", in: app), "back in the first Space")
        let work = row("Work", "Space", afterTyping: "wor", in: app)
        audit(app, "a Space offered")
        press(work)
        XCTAssertTrue(shows(space: "Work", in: app), "chosen, the Space is shown")
        XCTAssertEqual(site.requests.count, asked, "showing a Space loaded nothing")
    }
}

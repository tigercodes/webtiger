// Undo for what is done to the tree (044, WT-D32) in the app, in fake mode, with the real core
// and the real engine. On a Mac: the Edit menu's command says what it undoes and undoes it;
// ⌘Z does the same; and where the keyboard is in a field with typing to undo, ⌘Z undoes the
// typing and leaves the tree alone. On a phone: the command is at the top of a tab's own menu
// and of a Group's, and undoes one thing after another, the newest first.
import XCTest

@MainActor
final class TreeUndoTests: AppTestCase {
    #if os(macOS)
        /// The Edit menu's first item now: its words, and whether it can be chosen. The menu
        /// is closed again.
        private func undoItem(in app: XCUIApplication) -> (words: String, enabled: Bool) {
            let edit = app.menuBars.menuBarItems["Edit"]
            edit.click()
            let first = edit.menuItems.element(boundBy: 0)
            XCTAssertTrue(first.waitForExistence(timeout: 30), "the Edit menu has its Undo")
            let said = (first.title, first.isEnabled)
            app.typeKey(.escape, modifierFlags: [])
            return said
        }

        func testTheEditMenusUndoIsTheTreesUnlessThereIsTypingToUndo() {
            let app = launch(start: site.url("/landed"))
            XCTAssertTrue(pageShows("Landed", in: app), "the first tab's page loaded")
            let field = app.textFields["address-field"]
            let tab = app.buttons.matching(identifier: "tab").element(boundBy: 0)
            // Nothing has been done to the row: the command says only "Undo", and is grey.
            var item = undoItem(in: app)
            XCTAssertEqual(item.words, "Undo")
            XCTAssertFalse(item.enabled, "there is nothing to undo")

            // The tab is kept: the command says what it would undo. Chosen, the tab is a tab
            // like any other again, and there is nothing more to undo.
            choose("Keep as Site", for: tab, in: app)
            expect(tab, toSay: "Site", "kept")
            item = undoItem(in: app)
            XCTAssertEqual(item.words, "Undo Keep as Site")
            XCTAssertTrue(item.enabled)
            choose("Undo Keep as Site", for: tab, in: app, menu: "Edit")
            expect(tab, toSay: "", "undone from the Edit menu")
            XCTAssertEqual(undoItem(in: app).words, "Undo")

            // ⌘Z does the same.
            choose("Keep as Site", for: tab, in: app)
            expect(tab, toSay: "Site", "kept again")
            app.typeKey("z", modifierFlags: .command)
            expect(tab, toSay: "", "undone by ⌘Z")

            // Typing comes first: with something typed in the address field, ⌘Z undoes the
            // typing, as it does everywhere, and the Site is a Site still.
            choose("Keep as Site", for: tab, in: app)
            expect(tab, toSay: "Site", "kept a third time")
            openLocation(field, in: app)
            field.typeText("abc")
            XCTAssertEqual(value(of: field), "abc")
            XCTAssertEqual(undoItem(in: app).words, "Undo", "with typing to undo, the command is the field's")
            app.typeKey("z", modifierFlags: .command)
            // The typing is undone: the field has the page's address again, whole, as it had
            // when the keyboard came to it. The tree was left alone.
            expect(field, toShow: site.url("/landed"))
            expect(tab, toSay: "Site", "and the Site is a Site still")
            // With no typing left to undo, the command is the tree's again, and says so.
            XCTAssertEqual(undoItem(in: app).words, "Undo Keep as Site")
            app.typeKey("z", modifierFlags: .command)
            expect(tab, toSay: "", "⌘Z undid the keeping once there was no typing to undo")
        }
    #else
        func testATabsMenuAndAGroupsMenuUndoWhatWasLastDoneToTheRow() {
            let app = launch(start: site.url("/landed"))
            XCTAssertTrue(pageShows("Landed", in: app), "the first tab's page loaded")
            let tab = app.windows.element(boundBy: 0).buttons.matching(identifier: "tab").element(boundBy: 0)
            let chips = app.descendants(matching: .any).matching(identifier: "group")
            let asked = site.requests.count

            // Kept, and then undone from the tab's own menu, where the command is at the top
            // and says what it undoes.
            choose("Keep as Site", for: tab, in: app)
            expect(tab, toSay: "Site", "kept")
            choose("Undo Keep as Site", for: tab, in: app)
            expect(tab, toSay: "", "undone from the tab's own menu")

            // A Group made for the tab: two things done, the Group made and the tab put in it.
            // From the Group's own menu they are undone the newest first: the tab is taken
            // out, and is a tab like any other again; then the Group is gone.
            makeGroup("Work", for: tab, in: app)
            let chip = chips.firstMatch
            XCTAssertTrue(chip.waitForExistence(timeout: 30), "the Group's chip is in the row")
            expect(tab, toSay: "Site", "put in a Group, the tab is kept")
            chip.press(forDuration: 1)
            item("Undo Move to Group", in: app)?.tap()
            expect(tab, toSay: "", "taken out of the Group again, and no Site")
            expect(chips, toCount: 1, "the Group is there still, with nothing in it")
            chip.press(forDuration: 1)
            item("Undo New Group", in: app)?.tap()
            expect(chips, toCount: 0, "and then the Group is gone")
            XCTAssertEqual(site.requests.count, asked, "none of it loaded anything")
        }
    #endif
}

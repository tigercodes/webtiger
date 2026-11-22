// Sites (039) in the app, in fake mode, with the real core and the real engine: a tab is kept,
// is first in the row and says it is a Site; closed, it stays, asleep; pressed, it wakes at
// its home; let go, it is a tab like any other.
import XCTest

@MainActor
final class SiteTests: AppTestCase {
    /// Waits for a tab of the strip to say this of itself beside its name.
    private func expect(_ tab: XCUIElement, toSay said: String, _ what: String, line: UInt = #line) {
        let says = NSPredicate(format: "value == %@", said)
        let waited = XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: says, object: tab)], timeout: 30)
        XCTAssertEqual(waited, .completed, "\(what): the tab says '\(tab.value as? String ?? "")'", line: line)
    }

    /// Asks for the one command by its name now: from the File menu on a Mac (for the tab in
    /// front), from the tab's own menu on a phone.
    private func choose(_ words: String, for tab: XCUIElement, in app: XCUIApplication, line: UInt = #line) {
        #if os(macOS)
            let file = app.menuBars.menuBarItems["File"]
            file.click()
            let item = file.menuItems[words]
            XCTAssertTrue(item.waitForExistence(timeout: 30), "the File menu has \(words)", line: line)
            item.click()
        #else
            tab.press(forDuration: 1)
            // The key that asks the same (⌘D, with a keyboard) is carried by a button nobody
            // sees, which has the same name: the menu's item is the one that can be pressed.
            let named = app.buttons.matching(NSPredicate(format: "label == %@", words))
            let deadline = Date().addingTimeInterval(30)
            var item: XCUIElement?
            repeat {
                item = (0 ..< named.count).map { named.element(boundBy: $0) }.first { $0.exists && $0.isHittable }
                if item == nil {
                    usleep(200_000)
                }
            } while item == nil && Date() < deadline
            XCTAssertNotNil(item, "the tab's menu has \(words)", line: line)
            item?.tap()
        #endif
    }

    func testATabIsKeptSleepsWhenItIsClosedAndWakesAtItsHome() {
        let app = launch(start: site.url("/landed"))
        XCTAssertTrue(pageShows("Landed", in: app), "the first tab's page loaded")
        let field = app.textFields["address-field"]
        let tabs = app.buttons.matching(identifier: "tab")
        let first = tabs.element(boundBy: 0)
        func askedForItsHome() -> Int {
            site.requests.count(where: { $0 == "/landed" })
        }

        // Kept: it says it is a Site, and the button at its end says what it now does.
        expect(first, toSay: "", "a tab that is no Site says nothing")
        choose("Keep as Site", for: first, in: app)
        expect(first, toSay: "Site", "kept")
        XCTAssertEqual(app.buttons.matching(identifier: "close-tab").firstMatch.label, "Put to Sleep")

        // It wanders from its home; and a second tab is opened, somewhere else.
        openLocation(field, in: app)
        field.typeText("127.0.0.1:\(site.port)/\n")
        XCTAssertTrue(pageShows("Home", in: app), "the Site went somewhere else")
        press(app.buttons["new-tab"])
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "a second, empty tab")
        openLocation(field, in: app)
        field.typeText("127.0.0.1:\(site.port)/whoami\n")
        XCTAssertTrue(pageShows("who=none", in: app), "the second tab's page loaded")
        XCTAssertEqual(tabs.count, 2)
        let asked = askedForItsHome()

        // The Site in front again, and closed by the button at its end: it is still there, by
        // the name it was kept with, asleep; the other tab has come in front; nothing was loaded.
        press(first)
        XCTAssertTrue(pageShows("Home", in: app), "the Site is where it had wandered")
        press(app.buttons.matching(identifier: "close-tab").firstMatch)
        XCTAssertTrue(pageShows("who=none", in: app), "the other tab came in front")
        XCTAssertEqual(tabs.count, 2)
        expect(first, toSay: "Site, Asleep", "closed")
        XCTAssertEqual(first.label, "Landed", "by the name it was kept with, not where it had wandered")
        XCTAssertEqual(
            app.buttons.matching(identifier: "close-tab").count,
            1,
            "a sleeping Site has no page to close: only the other tab has the button"
        )
        XCTAssertEqual(askedForItsHome(), asked)
        audit(app, "a Site asleep in the strip")

        // A press wakes it, at its home.
        press(first)
        XCTAssertTrue(pageShows("Landed", in: app), "it woke at its home")
        expect(first, toSay: "Site", "awake again")
        XCTAssertEqual(askedForItsHome(), asked + 1, "its home was loaded")

        // Let go: a tab like any other, where it now sits; closed, it is gone.
        choose("Stop Keeping", for: first, in: app)
        expect(first, toSay: "", "let go")
        XCTAssertEqual(tabs.count, 2)
        XCTAssertEqual(app.buttons.matching(identifier: "close-tab").firstMatch.label, "Close Tab")
    }

    #if os(macOS)
        /// ⌘D is the command's key, and ⌘W puts a Site to sleep as its button does. A window
        /// whose only tab to show would be an empty one beside sleeping Sites closes instead.
        func testTheMacsKeysKeepATabPutItToSleepAndLetItGo() {
            let app = launch(start: site.url("/landed"))
            XCTAssertTrue(pageShows("Landed", in: app), "the page loaded")
            let tabs = app.buttons.matching(identifier: "tab")
            let first = tabs.element(boundBy: 0)
            let file = app.menuBars.menuBarItems["File"]

            app.typeKey("d", modifierFlags: .command)
            expect(first, toSay: "Site", "⌘D kept the tab in front")
            // A second tab, somewhere else: the Site has a tab to give way to.
            app.typeKey("t", modifierFlags: .command)
            XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "a second, empty tab")
            // For an empty tab the command is grey: there is nothing to keep.
            file.click()
            XCTAssertFalse(file.menuItems["Keep as Site"].isEnabled, "an empty tab cannot be kept")
            app.typeKey(.escape, modifierFlags: [])
            app.typeKey("l", modifierFlags: .command)
            app.typeText("127.0.0.1:\(site.port)/\n")
            XCTAssertTrue(pageShows("Home", in: app), "the second tab's page loaded")

            // ⌘W on the Site: it sleeps and stays, and the other tab is in front.
            press(first)
            XCTAssertTrue(pageShows("Landed", in: app))
            app.typeKey("w", modifierFlags: .command)
            expect(first, toSay: "Site, Asleep", "⌘W put it to sleep")
            XCTAssertTrue(pageShows("Home", in: app), "the other tab came in front")
            XCTAssertEqual(tabs.count, 2)

            // The Site in front again: the menu names the command for it, and the key lets it go.
            press(first)
            XCTAssertTrue(pageShows("Landed", in: app))
            file.click()
            XCTAssertTrue(file.menuItems["Stop Keeping"].waitForExistence(timeout: 30), "the menu names the command for a Site")
            app.typeKey(.escape, modifierFlags: [])
            app.typeKey("d", modifierFlags: .command)
            expect(first, toSay: "", "⌘D let it go")

            // Kept again and put to sleep; then ⌘W on the last tab there is to show closes the
            // window, as it does with one tab: a sleeping Site is nothing to be left with.
            app.typeKey("d", modifierFlags: .command)
            expect(first, toSay: "Site", "kept again")
            app.typeKey("w", modifierFlags: .command)
            expect(first, toSay: "Site, Asleep", "asleep again")
            XCTAssertTrue(pageShows("Home", in: app))
            app.typeKey("w", modifierFlags: .command)
            let none = NSPredicate(format: "count == 0")
            XCTAssertEqual(
                XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: none, object: app.windows)], timeout: 30), .completed,
                "⌘W on the last tab to show closed the window: \(app.windows.count) windows"
            )
        }
    #endif
}

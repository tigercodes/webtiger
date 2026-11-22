// Favourites (042) in the app, in fake mode, with the real core and the real engine: a tab is
// made a Favourite and is a tile, first in the row, that says its whole name; closed it
// sleeps and stays; pressed it wakes; and it stops being one.
import XCTest

@MainActor
final class FavouriteTests: AppTestCase {
    /// Waits for a tab of the strip to say this of itself beside its name.
    private func expect(_ tab: XCUIElement, toSay said: String, _ what: String, line: UInt = #line) {
        let says = NSPredicate(format: "value == %@", said)
        let waited = XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: says, object: tab)], timeout: 30)
        XCTAssertEqual(waited, .completed, "\(what): the tab says '\(tab.value as? String ?? "")'", line: line)
    }

    /// Asks for a command by its name: from the File menu on a Mac, for the tab in front;
    /// from the tab's own menu on a phone.
    private func choose(_ words: String, for tab: XCUIElement, in app: XCUIApplication, line: UInt = #line) {
        #if os(macOS)
            let file = app.menuBars.menuBarItems["File"]
            file.click()
            let item = file.menuItems[words]
            XCTAssertTrue(item.waitForExistence(timeout: 30), "the File menu has \(words)", line: line)
            XCTAssertTrue(item.isEnabled, "\(words) can be chosen", line: line)
            item.click()
        #else
            tab.press(forDuration: 1)
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

    func testATabIsMadeAFavouriteIsATileFirstInTheRowAndStopsBeingOne() {
        let app = launch(start: site.url("/landed"))
        XCTAssertTrue(pageShows("Landed", in: app), "the first tab's page loaded")
        let field = app.textFields["address-field"]
        let tabs = app.buttons.matching(identifier: "tab")
        let first = tabs.element(boundBy: 0)
        let second = tabs.element(boundBy: 1)
        let closers = app.buttons.matching(identifier: "close-tab")
        // A second tab, somewhere else, in front.
        press(app.buttons["new-tab"])
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "a second, empty tab")
        openLocation(field, in: app)
        field.typeText("127.0.0.1:\(site.port)/whoami\n")
        XCTAssertTrue(pageShows("who=none", in: app), "the second tab's page loaded")
        XCTAssertEqual(second.label, "who=none")

        // Made a Favourite, the tab is kept, and is first in the row: a tile that says its
        // whole name, and that it is a Favourite. It has no button at its end.
        choose("Add to Favourites", for: second, in: app)
        expect(first, toSay: "Favourite", "the tab in front was made a Favourite, and is first")
        XCTAssertEqual(first.label, "who=none", "a tile says its whole name")
        XCTAssertEqual(second.label, "Landed")
        expect(second, toSay: "", "the other tab is a tab")
        XCTAssertEqual(closers.count, 1, "a tile has no button at its end: only the other tab has one")
        XCTAssertTrue(pageShows("who=none", in: app), "it is still in front")
        audit(app, "a Favourite's tile in the strip")

        // The other tab in front; the tile, pressed, brings its page back.
        press(second)
        XCTAssertTrue(pageShows("Landed", in: app))
        press(first)
        XCTAssertTrue(pageShows("who=none", in: app), "the tile brought its page in front")

        // Closed, a Favourite sleeps and stays, as a Site does: by ⌘W on a Mac, from the
        // tile's own menu on a phone. Pressed, it wakes.
        #if os(macOS)
            app.typeKey("w", modifierFlags: .command)
        #else
            choose("Put to Sleep", for: first, in: app)
        #endif
        expect(first, toSay: "Favourite, Asleep", "closed, it sleeps")
        XCTAssertTrue(pageShows("Landed", in: app), "the other tab came in front")
        XCTAssertEqual(tabs.count, 2)
        press(first)
        XCTAssertTrue(pageShows("who=none", in: app), "it woke")
        expect(first, toSay: "Favourite", "awake again")

        // No longer a Favourite: a Site with its name and the button at its end.
        choose("Remove from Favourites", for: first, in: app)
        expect(first, toSay: "Site", "a Site still")
        XCTAssertEqual(first.label, "who=none")
        XCTAssertEqual(closers.count, 2)
        #if os(macOS)
            // The command has its other name again; and for an empty tab it is grey.
            let file = app.menuBars.menuBarItems["File"]
            file.click()
            XCTAssertTrue(file.menuItems["Add to Favourites"].waitForExistence(timeout: 30))
            app.typeKey(.escape, modifierFlags: [])
            app.typeKey("t", modifierFlags: .command)
            XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "an empty tab")
            file.click()
            XCTAssertFalse(file.menuItems["Add to Favourites"].isEnabled, "an empty tab cannot be made a Favourite")
            app.typeKey(.escape, modifierFlags: [])
        #endif
    }
}

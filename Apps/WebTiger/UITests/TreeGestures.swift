// What the UI tests of the tree share (039, 041, 042, 043): what a part of the row says of
// itself, how many of something there are, and the menus a tab is arranged from.
import XCTest

extension AppTestCase {
    /// Waits for a part of the row (a tab, a Group's chip) to say this of itself beside its
    /// name.
    func expect(_ element: XCUIElement, toSay said: String, _ what: String, line: UInt = #line) {
        let says = NSPredicate(format: "value == %@", said)
        let waited = XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: says, object: element)], timeout: 30)
        XCTAssertEqual(waited, .completed, "\(what): it says '\(element.value as? String ?? "")'", line: line)
    }

    /// Waits for the row's tabs to be these, by their names, in this order.
    func expect(_ tabs: XCUIElementQuery, toBe names: [String], _ what: String, line: UInt = #line) {
        let deadline = Date().addingTimeInterval(30)
        var found: [String] = []
        repeat {
            found = (0 ..< tabs.count).map { tabs.element(boundBy: $0).label }
            if found != names {
                usleep(200_000)
            }
        } while found != names && Date() < deadline
        XCTAssertEqual(found, names, what, line: line)
    }

    /// Waits for there to be this many of something.
    func expect(_ elements: XCUIElementQuery, toCount count: Int, _ what: String, line: UInt = #line) {
        let counted = NSPredicate(format: "count == %d", count)
        let waited = XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: counted, object: elements)], timeout: 30)
        XCTAssertEqual(waited, .completed, "\(what): there are \(elements.count)", line: line)
    }

    /// The item of a menu that is open now, by its words: the one that can be pressed. (On a
    /// phone a key that asks the same, with a keyboard, is carried by a button nobody sees,
    /// which has the same name.)
    func item(_ words: String, in app: XCUIApplication, line: UInt = #line) -> XCUIElement? {
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

    /// Asks for a command by its name now: from the menu bar on a Mac (for the tab in front;
    /// the File menu, or the one named), from the tab's own menu on a phone.
    func choose(_ words: String, for tab: XCUIElement, in app: XCUIApplication, menu: String = "File", line: UInt = #line) {
        #if os(macOS)
            let title = app.menuBars.menuBarItems[menu]
            title.click()
            let item = title.menuItems[words]
            XCTAssertTrue(item.waitForExistence(timeout: 30), "the \(menu) menu has \(words)", line: line)
            XCTAssertTrue(item.isEnabled, "\(words) can be chosen", line: line)
            item.click()
        #else
            tab.press(forDuration: 1)
            item(words, in: app, line: line)?.tap()
        #endif
    }

    /// Chooses from "Move to Group" for a tab: in the File menu on a Mac, for the tab in
    /// front; in the tab's own menu on a phone.
    func moveToGroup(_ choice: String, _ tab: XCUIElement, in app: XCUIApplication, line: UInt = #line) {
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

    /// Whether an item of that name is on the screen now, in a menu of the app's.
    private func item(named words: String, isShownIn app: XCUIApplication) -> Bool {
        #if os(macOS)
            let named = app.menuItems.matching(NSPredicate(format: "title == %@", words))
        #else
            let named = app.buttons.matching(NSPredicate(format: "label == %@", words))
        #endif
        return (0 ..< named.count).contains { named.element(boundBy: $0).exists && named.element(boundBy: $0).isHittable }
    }

    /// Makes a Group for a tab by "New Group…", with this name.
    func makeGroup(_ name: String, for tab: XCUIElement, in app: XCUIApplication, line: UInt = #line) {
        moveToGroup("New Group…", tab, in: app, line: line)
        let field = app.textFields["group-name"]
        let came = field.waitForExistence(timeout: 30)
        // Once, on a Mac, the page did not come though the item was pressed (041 §9). Whether
        // the item is still on the screen then tells a click that was lost from a command
        // that did nothing.
        let stillOffered = came ? false : item(named: "New Group…", isShownIn: app)
        XCTAssertTrue(came, "the page that makes a Group; its item in the menu is \(stillOffered ? "still there" : "gone")", line: line)
        press(field)
        field.typeText(name)
        press(app.buttons["group-done"])
    }

    /// Asks for the tabs at the side, or back along the top, by the command's name: in the
    /// View menu on a Mac; on the window's button on a touch screen.
    func switchLayout(_ words: String, in app: XCUIApplication, line: UInt = #line) {
        #if os(macOS)
            let view = app.menuBars.menuBarItems["View"]
            view.click()
            let item = view.menuItems[words]
            XCTAssertTrue(item.waitForExistence(timeout: 30), "the View menu has '\(words)'", line: line)
            item.click()
        #else
            let button = app.buttons["layout"]
            XCTAssertEqual(button.label, words, "the window's button names what it does", line: line)
            button.tap()
        #endif
    }
}

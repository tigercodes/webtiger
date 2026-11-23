// The sidebar (043) in the app, in fake mode, with the real core and the real engine: the tabs
// go down the side and back along the top, and nothing else changes; at the side every row
// does what its tab does at the top. On a Mac by the View menu's command and its key; on a
// touch screen by the window's button, in a window wide enough. A phone's window is not: it
// has no switch, and that is its test.
import XCTest

@MainActor
final class SidebarTests: AppTestCase {
    /// Waits for there to be this many of something.
    private func expect(_ elements: XCUIElementQuery, toCount count: Int, _ what: String, line: UInt = #line) {
        let counted = NSPredicate(format: "count == %d", count)
        let waited = XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: counted, object: elements)], timeout: 30)
        XCTAssertEqual(waited, .completed, "\(what): there are \(elements.count)", line: line)
    }

    /// Asks for the tabs at the side, or back along the top, by the command's name: in the
    /// View menu on a Mac; on the window's button on a touch screen.
    private func switchLayout(_ words: String, in app: XCUIApplication, line: UInt = #line) {
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

    /// At the side, a row brings its tab in front; the address field goes where it is sent;
    /// "New Tab" opens a tab, and the button at a row's end closes it.
    private func useTheSidebar(of app: XCUIApplication, tabs: XCUIElementQuery) {
        let field = app.textFields["address-field"]
        press(tabs.element(boundBy: 0))
        XCTAssertTrue(pageShows("Landed", in: app), "a row pressed brought its tab in front")
        openLocation(field, in: app)
        field.typeText("127.0.0.1:\(site.port)/\n")
        XCTAssertTrue(pageShows("Home", in: app), "the address field at the side sent the tab somewhere")
        press(app.buttons["new-tab"])
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "a third, empty tab")
        expect(tabs, toCount: 3, "three tabs")
        press(app.buttons.matching(identifier: "close-tab").element(boundBy: 2))
        expect(tabs, toCount: 2, "the empty tab was closed by the button at its row's end")
    }

    func testTheTabsGoDownTheSideAndBackAndNothingElseChanges() {
        let app = launch(start: site.url("/landed"))
        XCTAssertTrue(pageShows("Landed", in: app), "the first tab's page loaded")
        let field = app.textFields["address-field"]
        // The tabs of the app's own window: an iPad's keyboard has a key by the same name.
        let tabs = app.windows.element(boundBy: 0).buttons.matching(identifier: "tab")
        let strip = app.descendants(matching: .any).matching(identifier: "tab-strip")
        let sidebar = app.descendants(matching: .any).matching(identifier: "sidebar")
        // A second tab, somewhere else, in front.
        press(app.buttons["new-tab"])
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "a second, empty tab")
        openLocation(field, in: app)
        field.typeText("127.0.0.1:\(site.port)/whoami\n")
        XCTAssertTrue(pageShows("who=none", in: app), "the second tab's page loaded")
        expect(strip, toCount: 1, "a window starts with its tabs along the top")
        expect(sidebar, toCount: 0, "and with no sidebar")
        #if !os(macOS)
            // A phone's window has no room for a sidebar beside a page: it has no switch.
            // (An iPad's has, and the rest of this is its test.)
            guard app.buttons["layout"].exists else { return }
        #endif
        let asked = site.requests.count

        // "Tabs at the Side": the sidebar is there, the strip is not; the same tabs, by the
        // same names, the same one in front; and nothing was loaded again.
        switchLayout("Tabs at the Side", in: app)
        expect(sidebar, toCount: 1, "the tabs are down the side")
        expect(strip, toCount: 0, "and not along the top")
        expect(tabs, toCount: 2, "the same tabs")
        XCTAssertEqual(tabs.element(boundBy: 0).label, "Landed")
        XCTAssertEqual(tabs.element(boundBy: 1).label, "who=none")
        XCTAssertTrue(pageShows("who=none", in: app), "the same tab is in front, with its page")
        XCTAssertEqual(site.requests.count, asked, "switching the layout loaded nothing")
        audit(app, "the window with its tabs at the side")
        useTheSidebar(of: app, tabs: tabs)

        // The command names the way back, and takes the tabs along the top again, as they are.
        let pages = site.requests.count
        switchLayout("Tabs at the Top", in: app)
        expect(strip, toCount: 1, "the tabs are along the top again")
        expect(sidebar, toCount: 0, "and the sidebar is gone")
        expect(tabs, toCount: 2, "the same tabs")
        XCTAssertEqual(tabs.element(boundBy: 0).label, "Home")
        XCTAssertEqual(site.requests.count, pages, "switching back loaded nothing")
        #if os(macOS)
            // Its key, ⌃⌘S, takes them to the side again, and back.
            app.typeKey("s", modifierFlags: [.control, .command])
            expect(sidebar, toCount: 1, "⌃⌘S: at the side")
            app.typeKey("s", modifierFlags: [.control, .command])
            expect(strip, toCount: 1, "⌃⌘S again: along the top")
        #endif
    }
}

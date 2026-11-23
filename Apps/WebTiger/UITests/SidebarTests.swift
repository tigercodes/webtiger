// The sidebar (043) in the Mac app, in fake mode, with the real core and the real engine: the
// tabs go down the side and back along the top by the View menu's command and its key, and
// nothing else changes; at the side every row does what its tab does at the top.
import XCTest

#if os(macOS)
    @MainActor
    final class SidebarTests: AppTestCase {
        /// Waits for there to be this many of something.
        private func expect(_ elements: XCUIElementQuery, toCount count: Int, _ what: String, line: UInt = #line) {
            let counted = NSPredicate(format: "count == %d", count)
            let waited = XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: counted, object: elements)], timeout: 30)
            XCTAssertEqual(waited, .completed, "\(what): there are \(elements.count)", line: line)
        }

        func testTheTabsGoDownTheSideAndBackAndNothingElseChanges() {
            let app = launch(start: site.url("/landed"))
            XCTAssertTrue(pageShows("Landed", in: app), "the first tab's page loaded")
            let tabs = app.buttons.matching(identifier: "tab")
            let strip = app.descendants(matching: .any).matching(identifier: "tab-strip")
            let sidebar = app.descendants(matching: .any).matching(identifier: "sidebar")
            let view = app.menuBars.menuBarItems["View"]
            // A second tab, somewhere else, in front.
            app.typeKey("t", modifierFlags: .command)
            XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "a second, empty tab")
            app.typeKey("l", modifierFlags: .command)
            app.typeText("127.0.0.1:\(site.port)/whoami\n")
            XCTAssertTrue(pageShows("who=none", in: app), "the second tab's page loaded")
            expect(strip, toCount: 1, "a window starts with its tabs along the top")
            expect(sidebar, toCount: 0, "and with no sidebar")
            let asked = site.requests.count

            // "Tabs at the Side": the sidebar is there, the strip is not; the same tabs, by
            // the same names, the same one in front; and nothing was loaded again.
            view.click()
            let toTheSide = view.menuItems["Tabs at the Side"]
            XCTAssertTrue(toTheSide.waitForExistence(timeout: 30), "the View menu has the command")
            toTheSide.click()
            expect(sidebar, toCount: 1, "the tabs are down the side")
            expect(strip, toCount: 0, "and not along the top")
            expect(tabs, toCount: 2, "the same tabs")
            XCTAssertEqual(tabs.element(boundBy: 0).label, "Landed")
            XCTAssertEqual(tabs.element(boundBy: 1).label, "who=none")
            XCTAssertTrue(pageShows("who=none", in: app), "the same tab is in front, with its page")
            XCTAssertEqual(site.requests.count, asked, "switching the layout loaded nothing")
            audit(app, "the window with its tabs at the side")

            // At the side, a row brings its tab in front; the address field goes where it is
            // sent; "New Tab" opens a tab, and the button at a row's end closes it.
            press(tabs.element(boundBy: 0))
            XCTAssertTrue(pageShows("Landed", in: app), "a row pressed brought its tab in front")
            app.typeKey("l", modifierFlags: .command)
            app.typeText("127.0.0.1:\(site.port)/\n")
            XCTAssertTrue(pageShows("Home", in: app), "the address field at the side sent the tab somewhere")
            press(app.buttons["new-tab"])
            XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "a third, empty tab")
            expect(tabs, toCount: 3, "three tabs")
            let closers = app.buttons.matching(identifier: "close-tab")
            press(closers.element(boundBy: 2))
            expect(tabs, toCount: 2, "the empty tab was closed by the button at its row's end")

            // The command has its other name; its key, ⌃⌘S, brings the tabs back along the
            // top, as they are.
            view.click()
            XCTAssertTrue(view.menuItems["Tabs at the Top"].waitForExistence(timeout: 30), "the command names the way back")
            app.typeKey(.escape, modifierFlags: [])
            let pages = site.requests.count
            app.typeKey("s", modifierFlags: [.control, .command])
            expect(strip, toCount: 1, "the tabs are along the top again")
            expect(sidebar, toCount: 0, "and the sidebar is gone")
            expect(tabs, toCount: 2, "the same tabs")
            XCTAssertEqual(tabs.element(boundBy: 0).label, "Home")
            XCTAssertEqual(site.requests.count, pages, "switching back loaded nothing")
            // And the key takes them to the side again.
            app.typeKey("s", modifierFlags: [.control, .command])
            expect(sidebar, toCount: 1, "⌃⌘S again: at the side")
        }
    }
#endif

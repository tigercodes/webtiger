// The Mac's menus (034) in the app, in fake mode, with the real core and the real engine: the
// commands of the window in front are in the menu bar, grey when they can do nothing, and the
// keys beside them work. Only the app's own menus are asked about, each item by its name.
import XCTest

#if os(macOS)
    @MainActor
    final class CommandTests: AppTestCase {
        /// Opens a menu of the app's menu bar and gives the item with these words.
        private func item(_ words: String, in menu: String, of app: XCUIApplication) -> XCUIElement {
            let title = app.menuBars.menuBarItems[menu]
            title.click()
            let found = title.menuItems[words]
            XCTAssertTrue(found.waitForExistence(timeout: 30), "\(menu) has \(words)")
            return found
        }

        /// Chooses an item of the app's menu bar.
        private func choose(_ words: String, in menu: String, of app: XCUIApplication, line: UInt = #line) {
            let found = item(words, in: menu, of: app)
            XCTAssertTrue(found.isEnabled, "\(words) can be chosen", line: line)
            found.click()
        }

        /// Whether an item of the app's menu bar can be chosen now. The menu is closed again.
        private func canChoose(_ words: String, in menu: String, of app: XCUIApplication) -> Bool {
            let enabled = item(words, in: menu, of: app).isEnabled
            app.typeKey(.escape, modifierFlags: [])
            return enabled
        }

        /// Waits for the window to have so many tabs.
        private func awaitTabs(_ count: Int, in app: XCUIApplication, _ what: String, line: UInt = #line) {
            let tabs = app.buttons.matching(identifier: "tab")
            let reached = NSPredicate(format: "count == %d", count)
            let waited = XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: reached, object: tabs)], timeout: 30)
            XCTAssertEqual(waited, .completed, "\(what): the window has \(tabs.count) tabs", line: line)
        }

        /// How many times the site was asked for a page.
        private func asked(for path: String) -> Int {
            site.requests.filter { $0 == path }.count
        }

        /// Waits until the site has been asked for a page so many times.
        private func awaitAsked(_ count: Int, for path: String, _ what: String, line: UInt = #line) {
            let deadline = Date().addingTimeInterval(30)
            while asked(for: path) < count, Date() < deadline {
                usleep(100_000)
            }
            XCTAssertEqual(asked(for: path), count, what, line: line)
        }

        /// The menus do what the window's buttons do, and are grey when there is nothing to do.
        func testTheMenuBarHasTheWindowsCommands() {
            let app = launch(start: site.url("/landed"))
            XCTAssertTrue(pageShows("Landed", in: app), "the window's page loaded")
            let tabs = app.buttons.matching(identifier: "tab")

            // Nothing was closed, and there is nowhere to go back or forward to: grey.
            XCTAssertFalse(canChoose("Reopen Closed Tab", in: "File", of: app))
            XCTAssertFalse(canChoose("Back", in: "History", of: app))
            XCTAssertFalse(canChoose("Forward", in: "History", of: app))
            // No find is open: there is no match to go to.
            XCTAssertFalse(canChoose("Next Match", in: "Edit", of: app))

            // File: a new tab, which is empty; there is nothing in it to reload or to find in.
            choose("New Tab", in: "File", of: app)
            awaitTabs(2, in: app, "New Tab opened a tab")
            XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30))
            XCTAssertFalse(canChoose("Reload", in: "View", of: app))
            XCTAssertFalse(canChoose("Find on Page", in: "Edit", of: app))
            XCTAssertFalse(canChoose("Zoom In", in: "View", of: app))

            // File: the page's tab is closed, and brought back.
            press(tabs.element(boundBy: 0))
            XCTAssertTrue(pageShows("Landed", in: app))
            choose("Close Tab", in: "File", of: app)
            awaitTabs(1, in: app, "Close Tab closed the tab in front")
            choose("Reopen Closed Tab", in: "File", of: app)
            awaitTabs(2, in: app, "Reopen Closed Tab brought it back")
            XCTAssertTrue(pageShows("Landed", in: app))
            awaitAsked(2, for: "/landed", "the page was loaded again when its tab came back")

            // View: the page is asked for again.
            choose("Reload", in: "View", of: app)
            awaitAsked(3, for: "/landed", "Reload asked the site for the page again")

            // Edit: the find bar opens, with the keyboard in it; then there are matches to go to.
            choose("Find on Page", in: "Edit", of: app)
            XCTAssertTrue(app.textFields["find-field"].waitForExistence(timeout: 30), "the find bar")
            XCTAssertTrue(canChoose("Next Match", in: "Edit", of: app))

            // History: the list opens.
            choose("History and Downloads", in: "History", of: app)
            XCTAssertTrue(app.buttons["history-done"].waitForExistence(timeout: 30), "the lists of what was visited")
            press(app.buttons["history-done"])
        }

        /// The keys the menus bring: ⌘[ and ⌘], ⌘R, and ⇧⌘W, which did nothing before; and
        /// going back and forward from the History menu.
        func testTheKeysBesideTheMenusItemsWork() {
            let app = launch(start: site.url("/"))
            XCTAssertTrue(pageShows("Home", in: app), "the window's page loaded")
            let field = app.textFields["address-field"]
            openLocation(field, in: app)
            field.typeText("127.0.0.1:\(site.port)/landed\n")
            XCTAssertTrue(pageShows("Landed", in: app))

            // Back and forward, by key and by menu.
            app.typeKey("[", modifierFlags: .command)
            XCTAssertTrue(pageShows("Home", in: app), "⌘[ went back")
            XCTAssertFalse(canChoose("Back", in: "History", of: app), "there is no way further back")
            app.typeKey("]", modifierFlags: .command)
            XCTAssertTrue(pageShows("Landed", in: app), "⌘] went forward")
            choose("Back", in: "History", of: app)
            XCTAssertTrue(pageShows("Home", in: app), "Back went back")
            choose("Forward", in: "History", of: app)
            XCTAssertTrue(pageShows("Landed", in: app), "Forward went forward")

            // ⌘R asks the site for the page again.
            let before = asked(for: "/landed")
            app.typeKey("r", modifierFlags: .command)
            awaitAsked(before + 1, for: "/landed", "⌘R reloaded the page")

            // ⇧⌘W closes the window, though it has two tabs; ⌘W would have closed one.
            app.typeKey("t", modifierFlags: .command)
            awaitTabs(2, in: app, "⌘T opened a tab")
            app.typeKey("w", modifierFlags: [.command, .shift])
            let gone = NSPredicate(format: "count == 0")
            let closed = XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: gone, object: app.windows)], timeout: 30)
            XCTAssertEqual(closed, .completed, "⇧⌘W closed the window with both its tabs")
        }

        /// Two browser windows: each has its own tabs, and the menu bar and the keys speak for
        /// the one in front.
        func testWithTwoWindowsTheMenuSpeaksForTheOneInFront() {
            let app = launch(start: site.url("/landed"))
            XCTAssertTrue(pageShows("Landed", in: app), "the first window's page loaded")
            let everyTab = app.buttons.matching(identifier: "tab")
            // Waits for the app to have so many windows, and so many tabs in them all.
            func expect(windows: Int, tabs: Int, _ what: String, line: UInt = #line) {
                let deadline = Date().addingTimeInterval(30)
                while app.windows.count != windows || everyTab.count != tabs, Date() < deadline {
                    usleep(200_000)
                }
                XCTAssertEqual(app.windows.count, windows, "\(what): windows", line: line)
                XCTAssertEqual(everyTab.count, tabs, "\(what): tabs in them all", line: line)
            }

            // A second window, from the system's own item: it opens in front, on the page a
            // new window opens on in fake mode, and is sent somewhere else to tell them apart.
            app.typeKey("n", modifierFlags: .command)
            expect(windows: 2, tabs: 2, "⌘N opened a window")
            app.typeKey("l", modifierFlags: .command)
            app.typeText("127.0.0.1:\(site.port)/\n")
            XCTAssertTrue(pageShows("Home", in: app), "the window in front went where it was sent")
            let first = app.windows["Landed"]
            XCTAssertTrue(first.exists, "the first window is where it was")
            XCTAssertEqual(first.buttons.matching(identifier: "tab").count, 1)

            // The window in front has been somewhere before; the menu says so, for it.
            XCTAssertTrue(canChoose("Back", in: "History", of: app), "the window in front has a way back")

            // New Tab is the front window's: the first window keeps its one tab.
            choose("New Tab", in: "File", of: app)
            expect(windows: 2, tabs: 3, "New Tab opened a tab in the window in front")
            XCTAssertEqual(first.buttons.matching(identifier: "tab").count, 1, "and not in the other")

            // ⌘W closes that tab; ⇧⌘W then closes the window in front, and the first is left.
            app.typeKey("w", modifierFlags: .command)
            expect(windows: 2, tabs: 2, "⌘W closed the tab in front")
            app.typeKey("w", modifierFlags: [.command, .shift])
            expect(windows: 1, tabs: 1, "⇧⌘W closed the window in front")
            XCTAssertTrue(first.exists, "the first window is the one that is left")

            // The menu now speaks for the first window, which has been nowhere else.
            XCTAssertFalse(canChoose("Back", in: "History", of: app), "the first window has no way back")
            choose("New Tab", in: "File", of: app)
            expect(windows: 1, tabs: 2, "New Tab opened a tab in the window that is left")
        }

        /// The system's own Close gave way to Close Tab and Close Window: a window that is not
        /// a browser's must still close by key. The design catalogue is such a window.
        func testAWindowThatIsNoBrowsersStillClosesByKey() {
            let app = XCUIApplication()
            app.launchEnvironment["WEBTIGER_CATALOGUE"] = "1"
            app.launchArguments += ["-ApplePersistenceIgnoreState", "YES"]
            app.launch()
            XCTAssertTrue(app.staticTexts["Design Catalogue"].waitForExistence(timeout: 30), "the catalogue opened")
            app.typeKey("w", modifierFlags: [.command, .shift])
            let gone = NSPredicate(format: "count == 0")
            let closed = XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: gone, object: app.windows)], timeout: 30)
            XCTAssertEqual(closed, .completed, "⇧⌘W closed the catalogue's window")
        }
    }
#endif

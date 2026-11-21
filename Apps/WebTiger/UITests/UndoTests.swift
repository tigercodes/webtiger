// Undo (033) in the app, in fake mode, with the real core and the real engine: the tab that was
// last closed is offered back by the button for a new tab, held, on a phone; and by the strip
// of tabs, right-clicked, on a Mac.
import XCTest

@MainActor
final class UndoTests: AppTestCase {
    /// Asks for the menu: a finger held on the button for a new tab on a phone; a right-click
    /// in the strip, beside the tabs, on a Mac (where a button takes a right-click for itself).
    private func askForTheMenu(in app: XCUIApplication) {
        #if os(macOS)
            let strip = app.descendants(matching: .any).matching(identifier: "tab-strip").firstMatch
            strip.coordinate(withNormalizedOffset: CGVector(dx: 0.9, dy: 0.5)).rightClick()
        #else
            app.buttons["new-tab"].press(forDuration: 1.2)
        #endif
    }

    /// "Reopen Closed Tab" in the menu, once it can be pressed; nil when none comes in time.
    private func reopenItem(in app: XCUIApplication, within seconds: TimeInterval) -> XCUIElement? {
        // By the name the shell gives it, or by its words where a system's menu keeps only
        // those.
        let named = NSPredicate(
            format: "identifier == %@ OR label == %@ OR title == %@",
            "reopen-closed-tab", "Reopen Closed Tab", "Reopen Closed Tab"
        )
        #if os(macOS)
            let found = app.menuItems.matching(named)
        #else
            let found = app.buttons.matching(named)
        #endif
        let deadline = Date().addingTimeInterval(seconds)
        repeat {
            // What a screen reader is offered is there by the same name, and is nowhere on
            // the screen: only what can be pressed is the menu's.
            for index in 0 ..< found.count {
                let item = found.element(boundBy: index)
                if item.exists, item.isHittable {
                    return item
                }
            }
            usleep(200_000)
        } while Date() < deadline
        return nil
    }

    /// Waits for the window to have so many tabs.
    private func awaitTabs(_ count: Int, in app: XCUIApplication, _ what: String, line: UInt = #line) {
        let tabs = app.buttons.matching(identifier: "tab")
        let reached = NSPredicate(format: "count == %d", count)
        let waited = XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: reached, object: tabs)], timeout: 30)
        XCTAssertEqual(waited, .completed, "\(what): the window has \(tabs.count) tabs", line: line)
    }

    /// A page's tab is closed with its button; the menu offers it back; chosen, the tab is
    /// where it was, in front, with its page. Asked for again, with nothing more to bring
    /// back, there is no menu.
    func testAClosedTabIsOfferedBackByAMenu() {
        let app = launch(start: site.url("/landed"))
        XCTAssertTrue(pageShows("Landed", in: app), "the window's page loaded")
        let tabs = app.buttons.matching(identifier: "tab")

        #if os(macOS)
            // Nothing was closed yet: there is no menu. (On a phone a finger held on a button
            // that has no menu is a press of it, which opens a tab.)
            askForTheMenu(in: app)
            XCTAssertNil(reopenItem(in: app, within: 2), "nothing to bring back, nothing offered")
        #endif

        // A second tab, empty; then the page's tab is brought in front and closed.
        press(app.buttons["new-tab"])
        awaitTabs(2, in: app, "a new tab was opened")
        press(tabs.element(boundBy: 0))
        XCTAssertTrue(pageShows("Landed", in: app))
        press(app.buttons.matching(identifier: "close-tab").element(boundBy: 0))
        awaitTabs(1, in: app, "the page's tab was closed")
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "the empty tab is what is left")

        askForTheMenu(in: app)
        guard let item = reopenItem(in: app, within: 30) else {
            return XCTFail("the closed tab is offered back")
        }
        // Asking opened no tab.
        XCTAssertEqual(tabs.count, 1)
        press(item)

        // Back where it was, the first of the two, in front, and its page loads again.
        awaitTabs(2, in: app, "the closed tab came back")
        XCTAssertTrue(pageShows("Landed", in: app), "with its page")
        XCTAssertTrue(tabs.element(boundBy: 0).isSelected, "where it was, and in front")
        expect(app.textFields["address-field"], toShow: "http://127.0.0.1:\(site.port)/landed")
        // The page was asked for twice: when the window opened, and when the tab came back.
        XCTAssertEqual(site.requests.filter { $0 == "/landed" }.count, 2, "\(site.requests)")

        // Nothing more was closed: asked for again, there is no menu.
        askForTheMenu(in: app)
        XCTAssertNil(reopenItem(in: app, within: 2), "nothing more to bring back")
    }
}

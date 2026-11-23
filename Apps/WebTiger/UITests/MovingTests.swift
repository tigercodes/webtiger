// Moving a tab in its row (045) in the app, in fake mode, with the real core and the real
// engine: the tab in front changes places with the one beside it, by the Window menu's
// commands on a Mac and the tab's own menu on a phone; the command the tab cannot go by is
// grey; nothing is loaded; and on a Mac, with the tabs at the side, the commands say up and
// down.
import XCTest

@MainActor
final class MovingTests: AppTestCase {
    /// Whether a command that moves a tab can be chosen now: in the Window menu on a Mac;
    /// in the tab's own menu on a phone. The menu is closed again.
    private func canChoose(_ words: String, for tab: XCUIElement, in app: XCUIApplication, line: UInt = #line) -> Bool {
        #if os(macOS)
            let window = app.menuBars.menuBarItems["Window"]
            window.click()
            let item = window.menuItems[words]
            XCTAssertTrue(item.waitForExistence(timeout: 30), "the Window menu has \(words)", line: line)
            let enabled = item.isEnabled
            app.typeKey(.escape, modifierFlags: [])
            return enabled
        #else
            tab.press(forDuration: 1)
            let enabled = item(words, in: app, line: line)?.isEnabled == true
            // A tap away from the menu closes it.
            app.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.9)).tap()
            return enabled
        #endif
    }

    func testTheTabInFrontChangesPlacesWithTheOneBesideIt() {
        let app = launch(start: site.url("/landed"))
        XCTAssertTrue(pageShows("Landed", in: app), "the first tab's page loaded")
        let field = app.textFields["address-field"]
        let tabs = app.windows.element(boundBy: 0).buttons.matching(identifier: "tab")
        // A second tab, somewhere else, in front.
        press(app.buttons["new-tab"])
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "a second, empty tab")
        openLocation(field, in: app)
        field.typeText("127.0.0.1:\(site.port)/whoami\n")
        XCTAssertTrue(pageShows("who=none", in: app), "the second tab's page loaded")
        expect(tabs, toBe: ["Landed", "who=none"], "two tabs, in the order they were opened")
        let asked = site.requests.count

        // The last tab can go left, and not right: the command it cannot go by is grey.
        XCTAssertFalse(canChoose("Move Tab Right", for: tabs.element(boundBy: 1), in: app), "the last tab has nowhere right to go")
        choose("Move Tab Left", for: tabs.element(boundBy: 1), in: app, menu: "Window")
        expect(tabs, toBe: ["who=none", "Landed"], "the tab in front changed places with the one before it")
        XCTAssertTrue(pageShows("who=none", in: app), "and is in front still, with its page")
        // Now the first: it can go right, and not left.
        XCTAssertFalse(canChoose("Move Tab Left", for: tabs.element(boundBy: 0), in: app), "the first tab has nowhere left to go")
        choose("Move Tab Right", for: tabs.element(boundBy: 0), in: app, menu: "Window")
        expect(tabs, toBe: ["Landed", "who=none"], "and back again")
        XCTAssertEqual(site.requests.count, asked, "moving a tab loaded nothing")

        #if os(macOS)
            // With the tabs down the side the same commands say up and down.
            switchLayout("Tabs at the Side", in: app)
            expect(app.descendants(matching: .any).matching(identifier: "sidebar"), toCount: 1, "the tabs are down the side")
            XCTAssertFalse(canChoose("Move Tab Down", for: tabs.element(boundBy: 1), in: app), "the last tab has nowhere down to go")
            choose("Move Tab Up", for: tabs.element(boundBy: 1), in: app, menu: "Window")
            expect(tabs, toBe: ["who=none", "Landed"], "the tab in front went up a place")
            choose("Move Tab Down", for: tabs.element(boundBy: 0), in: app, menu: "Window")
            expect(tabs, toBe: ["Landed", "who=none"], "and down again")
            XCTAssertEqual(site.requests.count, asked, "at the side as along the top: nothing loaded")
        #endif
    }
}

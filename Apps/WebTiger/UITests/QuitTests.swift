// The tree and the layout across a quit (039, 041, 042, 043), with the app itself on a profile
// of the suite's own: a store on disk, in a folder apart from anyone's data. A Favourite, a
// shut Group with its Site, a tab, and on a Mac the tabs at the side, are there when the app
// starts again: the Favourite and the Site asleep, the tab that was in front loaded.
import XCTest

@MainActor
final class QuitTests: AppTestCase {
    /// The parts of the row, in the app's own window.
    private struct Row {
        let tabs: XCUIElementQuery
        let chip: XCUIElement
        let strip: XCUIElementQuery
        let sidebar: XCUIElementQuery

        @MainActor
        init(of app: XCUIApplication) {
            tabs = app.windows.element(boundBy: 0).buttons.matching(identifier: "tab")
            chip = app.descendants(matching: .any).matching(identifier: "group").firstMatch
            strip = app.descendants(matching: .any).matching(identifier: "tab-strip")
            sidebar = app.descendants(matching: .any).matching(identifier: "sidebar")
        }
    }

    /// Opens a tab and sends it to a page of the test's site.
    private func openTab(at path: String, titled title: String, in app: XCUIApplication) {
        let field = app.textFields["address-field"]
        press(app.buttons["new-tab"])
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "an empty tab")
        openLocation(field, in: app)
        field.typeText("127.0.0.1:\(site.port)\(path)\n")
        XCTAssertTrue(pageShows(title, in: app), "the page at \(path) loaded")
    }

    /// Arranges a row from nothing: a Favourite ("who=none"), a Group "Work" that holds a Site
    /// ("Home") and is shut, and a tab ("Landed") in front; on a Mac, the tabs at the side.
    private func arrangeARow(in app: XCUIApplication) {
        let row = Row(of: app)
        let field = app.textFields["address-field"]
        openLocation(field, in: app)
        field.typeText("127.0.0.1:\(site.port)/landed\n")
        XCTAssertTrue(pageShows("Landed", in: app), "the first tab's page loaded")

        openTab(at: "/whoami", titled: "who=none", in: app)
        choose("Add to Favourites", for: row.tabs.element(boundBy: 1), in: app)
        expect(row.tabs.element(boundBy: 0), toSay: "Favourite", "the second tab is a Favourite, and first")

        openTab(at: "/", titled: "Home", in: app)
        makeGroup("Work", for: row.tabs.element(boundBy: 2), in: app)
        XCTAssertTrue(row.chip.waitForExistence(timeout: 30), "the Group's chip is in the row")
        expect(row.tabs.element(boundBy: 1), toSay: "Site", "put in a Group, the third tab is kept, before the tab that is none")
        XCTAssertEqual(row.tabs.element(boundBy: 1).label, "Home")

        // The tab that is no Site in front, and the Group shut: its Site is out of the way.
        press(row.tabs.element(boundBy: 2))
        XCTAssertTrue(pageShows("Landed", in: app), "the tab is in front")
        press(row.chip)
        expect(row.chip, toSay: "Group, shut, Sites: 1", "the Group is shut")
        expect(row.tabs, toCount: 2, "and its Site is out of the way")
        #if os(macOS)
            switchLayout("Tabs at the Side", in: app)
            expect(row.sidebar, toCount: 1, "the tabs are down the side")
        #endif
    }

    func testTheTreeAndTheLayoutComeBackWhenTheAppStartsAgain() {
        var app = launch(onProfile: "ui-test-tree", fresh: true)
        // A fresh profile is a first run (020): it asks where searches go before anything else.
        let question = app.buttons["search-engine-ecosia"]
        XCTAssertTrue(question.waitForExistence(timeout: 30), "a fresh profile is asked where searches go")
        press(question)
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "a fresh profile opens one empty tab")
        arrangeARow(in: app)

        // Ended without a word, as a crash or the system ends it, and started again.
        app.terminate()
        app = launch(onProfile: "ui-test-tree", fresh: false)
        let row = Row(of: app)

        // The tab that was in front is in front, and loaded. The Favourite is first, a tile
        // with its name, asleep; the Group is there by its name, shut, holding its Site.
        XCTAssertTrue(pageShows("Landed", in: app), "the tab that was in front is in front, and loaded")
        let asked = site.requests.count
        expect(row.tabs, toCount: 2, "the Favourite and the tab; the shut Group's Site is out of the way")
        let favourite = row.tabs.element(boundBy: 0)
        XCTAssertEqual(favourite.label, "who=none")
        expect(favourite, toSay: "Favourite, Asleep", "the Favourite is back, asleep")
        XCTAssertEqual(row.chip.label, "Work")
        expect(row.chip, toSay: "Group, shut, Sites: 1", "the Group is back, shut, with its Site")
        expect(row.tabs.element(boundBy: 1), toSay: "", "the tab is a tab")
        #if os(macOS)
            expect(row.sidebar, toCount: 1, "the tabs are down the side, as they were left")
            expect(row.strip, toCount: 0, "and not along the top")
        #else
            expect(row.strip, toCount: 1, "a phone has its tabs along the top")
        #endif

        // Opened, the Group shows its Site, asleep; nothing was loaded for it.
        press(row.chip)
        expect(row.chip, toSay: "Group, open", "the Group is open")
        expect(row.tabs, toCount: 3, "its Site is in the row")
        let kept = row.tabs.element(boundBy: 1)
        XCTAssertEqual(kept.label, "Home")
        expect(kept, toSay: "Site, Asleep", "the Group's Site is back, asleep")
        XCTAssertEqual(site.requests.count, asked, "opening a Group loads nothing")

        // Pressed, the Favourite wakes with its page; and the Site with its own.
        press(favourite)
        XCTAssertTrue(pageShows("who=none", in: app), "the Favourite woke")
        expect(favourite, toSay: "Favourite", "awake")
        press(kept)
        XCTAssertTrue(pageShows("Home", in: app), "the Group's Site woke at its home")
        expect(kept, toSay: "Site", "awake")
    }
}

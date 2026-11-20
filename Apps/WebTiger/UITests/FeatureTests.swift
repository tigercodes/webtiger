// The apps' tests of what W1 added after the first: the real app, the real core and the real
// engine, in fake mode, with pages from the loopback site.
import XCTest

@MainActor
final class FeatureTests: AppTestCase {
    /// Places offered as the address field is typed in (011), with the real core: a page that
    /// was visited is offered by the start of a word of its title, and choosing it goes there.
    func testAPlaceVisitedIsOfferedWhenPartOfItsNameIsTyped() {
        let app = launch(start: site.url("/"))
        XCTAssertTrue(pageShows("Home", in: app), "the first page loaded")
        let field = app.textFields["address-field"]
        let offered = app.buttons.matching(identifier: "suggestion")
        openLocation(field, in: app)
        XCTAssertEqual(offered.count, 0, "nothing is offered for what nobody typed")
        field.typeText("127.0.0.1:\(site.port)/landed\n")
        XCTAssertTrue(pageShows("Landed", in: app), "the second page loaded")

        // The start of a word of the first page's title: that page, and only it.
        openLocation(field, in: app)
        field.typeText("hom")
        XCTAssertTrue(offered.firstMatch.waitForExistence(timeout: 30), "a place is offered")
        XCTAssertEqual(offered.count, 1)
        XCTAssertTrue(offered.firstMatch.label.contains("Home"), "by its title: \(offered.firstMatch.label)")
        XCTAssertFalse(app.images["security-secure"].exists || app.images["security-insecure"].exists, "no mark vouches for what was typed")
        audit(app, "the places offered")

        #if os(macOS)
            // Escape puts the list away and leaves what was typed; typing on brings it back.
            app.typeKey(.escape, modifierFlags: [])
            let none = NSPredicate(format: "count == 0")
            XCTAssertEqual(XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: none, object: offered)], timeout: 30), .completed)
            XCTAssertEqual(value(of: field), "hom")
            field.typeText("e")
            XCTAssertTrue(offered.firstMatch.waitForExistence(timeout: 30), "it is offered again for what is typed now")
            // The arrow key marks it, and Return goes there.
            app.typeKey(.downArrow, modifierFlags: [])
            app.typeKey(.return, modifierFlags: [])
        #else
            press(offered.firstMatch)
        #endif

        XCTAssertTrue(pageShows("Home", in: app), "the place chosen is the page on screen")
        expect(field, toShow: "http://127.0.0.1:\(site.port)")
        XCTAssertEqual(offered.count, 0, "and the list is gone")
    }

    /// Zoom (012), with the real core and engine: the bar is shut until asked for; bigger,
    /// smaller and actual size go to the core, and the bar says the level the core says; the
    /// level is the site's, and is there on another page of it.
    func testZoomGoesThroughTheCoreAndTheBarSaysTheLevel() {
        let app = launch(start: site.url("/"))
        XCTAssertTrue(pageShows("Home", in: app), "the first page loaded")
        let level = app.buttons["zoom-level"]
        XCTAssertFalse(level.exists, "the bar is shut until asked for")

        press(app.buttons["zoom"])
        XCTAssertTrue(level.waitForExistence(timeout: 30), "the bar opened")
        expect(level, toShow: "100%")
        XCTAssertFalse(level.isEnabled, "at actual size there is nothing to put back")
        press(app.buttons["zoom-in"])
        expect(level, toShow: "110%")
        press(app.buttons["zoom-in"])
        expect(level, toShow: "125%")
        press(app.buttons["zoom-out"])
        expect(level, toShow: "110%")
        audit(app, "the zoom bar")

        // Pressing the level puts the page back.
        press(level)
        expect(level, toShow: "100%")

        #if os(macOS)
            // The keys, as in every browser; ⌘= is ⌘+ without Shift.
            app.typeKey("+", modifierFlags: .command)
            expect(level, toShow: "110%")
            app.typeKey("=", modifierFlags: .command)
            expect(level, toShow: "125%")
            app.typeKey("-", modifierFlags: .command)
            expect(level, toShow: "110%")
            app.typeKey("0", modifierFlags: .command)
            expect(level, toShow: "100%")
        #endif

        // The level is the site's: left at 110%, it is there on another page of the site.
        press(app.buttons["zoom-in"])
        expect(level, toShow: "110%")
        press(app.buttons["zoom-done"])
        let gone = NSPredicate(format: "exists == false")
        XCTAssertEqual(XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: gone, object: level)], timeout: 30), .completed)
        let field = app.textFields["address-field"]
        openLocation(field, in: app)
        field.typeText("127.0.0.1:\(site.port)/landed\n")
        XCTAssertTrue(pageShows("Landed", in: app), "the second page loaded")
        press(app.buttons["zoom"])
        XCTAssertTrue(level.waitForExistence(timeout: 30))
        expect(level, toShow: "110%")
    }

    /// Downloads (013), with the real core and engine: a link to a file that says it is to be
    /// saved is saved, the page stays, and the file is in the list of downloads with its name
    /// and size; taken off the list, the list is empty. In fake mode the file goes to a folder
    /// of the app's own among the temporary files.
    func testAFileALinkOffersIsDownloadedAndListed() {
        let app = launch(start: site.url("/downloads"))
        XCTAssertTrue(pageShows("Downloads", in: app), "the page of links loaded")
        let link = app.webViews.links["a file that says it is to be saved"]
        XCTAssertTrue(link.waitForExistence(timeout: 30))
        press(link)

        // The list is beside History, behind the same button.
        press(app.buttons["history"])
        let downloads = app.buttons["library-downloads"]
        XCTAssertTrue(downloads.waitForExistence(timeout: 30), "the sheet opened, with its switch")
        press(downloads)
        let rows = app.descendants(matching: .any).matching(identifier: "download-entry")
        XCTAssertTrue(rows.firstMatch.waitForExistence(timeout: 300), "the download is listed")
        // Its name is the site's suggestion, which is a plain one; and it finishes: its size
        // is said, where a download still coming says how far it has got. (A row's words are
        // its label on an iPhone and its value on a Mac.)
        func words() -> String {
            let row = rows.firstMatch
            return row.label.isEmpty ? value(of: row) : row.label
        }
        var said = words()
        let waitedSince = Date()
        while !(said.contains("26 bytes") && !said.contains(" of ")), Date().timeIntervalSince(waitedSince) < 60 {
            usleep(200_000)
            said = words()
        }
        XCTAssertTrue(said.contains("notes.txt"), "the row says \(said)")
        XCTAssertTrue(said.contains("26 bytes") && !said.contains(" of "), "it finished: the row says \(said)")
        XCTAssertEqual(rows.count, 1)
        audit(app, "the Downloads list")

        // Taken off the list, the list is empty; the file is not the list's to delete.
        press(app.buttons["download-forget"])
        XCTAssertTrue(app.staticTexts["downloads-nothing"].waitForExistence(timeout: 30))
        press(app.buttons["downloads-done"])
        XCTAssertTrue(pageShows("Downloads", in: app), "and the page that offered the file is still the page")
    }

    /// The Permissions list (014) is beside History and Downloads, behind the same button. No
    /// site was allowed anything here, and none can be: a page cannot be made to ask on this
    /// machine. So this is the list's place in the app, and what it says when it is empty.
    func testThePermissionsListIsBesideHistoryAndSaysWhenNoSiteWasAllowedAnything() {
        let app = launch(start: site.url("/"))
        XCTAssertTrue(pageShows("Home", in: app))

        press(app.buttons["history"])
        let permissions = app.buttons["library-permissions"]
        XCTAssertTrue(permissions.waitForExistence(timeout: 30), "the sheet opened, with its switch")
        press(permissions)

        let nothing = app.staticTexts["permissions-nothing"]
        XCTAssertTrue(nothing.waitForExistence(timeout: 30), "the core answered: no site was allowed anything")
        XCTAssertFalse(app.buttons["permissions-clear"].isEnabled, "with nothing to take back")
        audit(app, "the Permissions list")

        // The switch goes back to History, and the sheet closes on the page as it was.
        press(app.buttons["library-history"])
        XCTAssertTrue(app.buttons["history-done"].waitForExistence(timeout: 30))
        press(app.buttons["history-done"])
        XCTAssertTrue(pageShows("Home", in: app))
    }
}

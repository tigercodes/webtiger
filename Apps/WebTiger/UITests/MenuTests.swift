// What a menu saves (022) and a search for words that are selected (023), in the apps, in
// fake mode: with the real core and WebKit, against the test site. On a Mac the menu is
// WebKit's own, opened with the mouse; on an iPhone it is the shell's, for a link pressed and
// held. What is saved goes to a folder of the engine's own among the temporary files, never
// to anybody's Downloads. No search is made.
import XCTest

final class MenuTests: AppTestCase {
    /// Opens the list of downloads, and waits for its one row to say that `name` is whole.
    /// What the row says.
    private func listed(_ name: String, in app: XCUIApplication) -> String {
        press(app.buttons["history"])
        let downloads = app.buttons["library-downloads"]
        XCTAssertTrue(downloads.waitForExistence(timeout: 30), "the sheet opened, with its switch")
        press(downloads)
        let rows = app.descendants(matching: .any).matching(identifier: "download-entry")
        XCTAssertTrue(rows.firstMatch.waitForExistence(timeout: 300), "the download is listed")
        // A row's words are its label on an iPhone and its value on a Mac. One that is still
        // coming says how far it has got ("3 of 9 bytes").
        func words() -> String {
            let row = rows.firstMatch
            return row.label.isEmpty ? value(of: row) : row.label
        }
        var said = words()
        let waitedSince = Date()
        while !(said.contains(name) && said.contains("bytes") && !said.contains(" of ")), Date().timeIntervalSince(waitedSince) < 60 {
            usleep(200_000)
            said = words()
        }
        XCTAssertEqual(rows.count, 1)
        return said
    }

    #if os(macOS)
        func testAPictureIsSavedFromItsMenuWithNobodyAsked() {
            let app = launch(start: site.url("/pictures"))
            XCTAssertTrue(pageShows("Pictures", in: app), "the page of pictures loaded")
            let picture = app.webViews.images["a picture"]
            XCTAssertTrue(picture.waitForExistence(timeout: 30), "the page's picture is there to press")

            // The picture's menu: WebKit's own, with the shell's words for what its item for
            // a new window does here.
            picture.rightClick()
            XCTAssertTrue(app.menuItems["Open Image in New Tab"].waitForExistence(timeout: 30), "the menu's item has the shell's words")
            // There is no link under it: nothing offers to save one.
            XCTAssertFalse(app.menuItems["Download Linked File"].exists)
            let item = app.menuItems["Download Image"]
            XCTAssertTrue(item.exists, "the menu offers to save the picture")
            item.click()

            // The person chose it: it is saved, and nobody is asked whether the site may
            // download (which a file the page sent would be, 015).
            let said = listed("tiger.svg", in: app)
            XCTAssertTrue(
                said.contains("tiger.svg") && said.contains("bytes") && !said.contains(" of "),
                "it finished: the row says \(said)"
            )
            XCTAssertFalse(app.buttons["prompt-allow"].exists, "nobody was asked")
            press(app.buttons["downloads-done"])
            XCTAssertTrue(pageShows("Pictures", in: app), "and the page is where it was")
            XCTAssertEqual(app.buttons.matching(identifier: "tab").count, 1)
        }

        /// A search for words that are selected (023): WebKit's menu has the shell's item for
        /// it, named for the engine searches go to, and not WebKit's own, which goes to the
        /// system's engine in another app. The item is not chosen: a search goes to the
        /// internet.
        func testTheMenuOfSelectedWordsOffersASearchWithTheEngineThePersonChose() {
            let app = launch(start: site.url("/words"))
            XCTAssertTrue(pageShows("Words", in: app), "the page of words loaded")
            // The word is in a line of words: the line may be one thing to a test, or several.
            let line = app.webViews.staticTexts.matching(NSPredicate(format: "label CONTAINS 'tigers' OR value CONTAINS 'tigers'"))
                .firstMatch
            XCTAssertTrue(line.waitForExistence(timeout: 30), "the page's word is there to select")
            let said = value(of: line).isEmpty ? line.label : value(of: line)
            let across = said.trimmingCharacters(in: .whitespaces) == "tigers" ? 0.5 : 0.35

            // A right-click on a word selects it, and opens the menu of selected words.
            line.coordinate(withNormalizedOffset: CGVector(dx: across, dy: 0.5)).rightClick()
            // Fake mode takes the engine that stands in as chosen (020).
            XCTAssertTrue(app.menuItems["Search with DuckDuckGo"].waitForExistence(timeout: 30), "the shell's search, named for the engine")
            let webKits = NSPredicate(format: "identifier == 'WKMenuItemIdentifierSearchWeb'")
            XCTAssertEqual(app.menuItems.matching(webKits).count, 1, "one item searches")
            XCTAssertEqual(app.menuItems.matching(webKits).firstMatch.title, "Search with DuckDuckGo", "and it is the shell's")
            XCTAssertFalse(app.menuItems["Search with Google"].exists)
            // Put away, with nothing chosen: no tab opens, and nothing was asked of anybody.
            app.typeKey(.escape, modifierFlags: [])
            XCTAssertEqual(app.buttons.matching(identifier: "tab").count, 1)
            XCTAssertEqual(site.requests.filter { $0 != "/words" && $0 != "/favicon.ico" }, [], "\(site.requests)")
        }
    #else
        func testWhatALinkLeadsToIsSavedFromItsMenuWithNobodyAsked() {
            let app = launch(start: site.url("/links"))
            XCTAssertTrue(pageShows("Links", in: app), "the page of links loaded")
            let link = app.webViews.links["a link in words"]
            XCTAssertTrue(link.waitForExistence(timeout: 30), "the page's link is there to press")

            // A finger held on the link: the shell's menu, which offers to save what it leads to.
            link.press(forDuration: 1.2)
            let item = app.buttons["Download Linked File"]
            XCTAssertTrue(item.waitForExistence(timeout: 30), "the link's menu")
            item.tap()

            // The person chose it: it is saved, and nobody is asked whether the site may download.
            let said = listed("landed", in: app)
            XCTAssertTrue(said.contains("landed") && said.contains("bytes") && !said.contains(" of "), "it finished: the row says \(said)")
            XCTAssertFalse(app.buttons["prompt-allow"].exists, "nobody was asked")
            press(app.buttons["downloads-done"])
            XCTAssertTrue(pageShows("Links", in: app), "and the page is where it was")
            XCTAssertEqual(app.buttons.matching(identifier: "tab").count, 1)
        }
    #endif
}

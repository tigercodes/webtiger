// The W0 exit, as a test: the app starts the Rust core, the core asks the shell for a web view
// in the Space's login set, WebKit loads a page, and what the address field shows is what the
// core published. Fake mode keeps the store in memory, so the test leaves nothing behind.
import XCTest
#if os(macOS)
    import AppKit
#endif

@MainActor
final class LaunchTests: AppTestCase {
    func testAPageLoadsThroughTheCore() {
        let app = launch(start: site.url("/"))
        let field = app.textFields["address-field"]
        XCTAssertTrue(field.waitForExistence(timeout: 30))
        XCTAssertTrue(pageShows("Home", in: app), "the page loaded")
        // The field shows what the core says the address is: plain http keeps its scheme.
        expect(field, toShow: "http://127.0.0.1:\(site.port)")
        XCTAssertTrue(app.images["security-insecure"].exists, "an http page is marked not secure")
    }

    func testTypingAnAddressLoadsIt() {
        let app = launch(start: nil)
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "a new tab is empty")
        let field = app.textFields["address-field"]
        XCTAssertTrue(field.waitForExistence(timeout: 30), "the address field exists")
        openLocation(field, in: app)
        // No scheme typed: the core decides this is an address on the loopback interface.
        field.typeText("127.0.0.1:\(site.port)/landed\n")
        XCTAssertTrue(pageShows("Landed", in: app), "the page loaded")
        expect(field, toShow: "http://127.0.0.1:\(site.port)/landed")

        // Back is offered once there is somewhere to go back to.
        openLocation(field, in: app)
        field.typeText("127.0.0.1:\(site.port)/\n")
        XCTAssertTrue(pageShows("Home", in: app), "typing replaced the address; the field shows \(value(of: field))")
        let back = app.buttons["back"]
        XCTAssertTrue(back.isEnabled, "back is offered")
        press(back)
        XCTAssertTrue(pageShows("Landed", in: app), "back goes back")
    }

    /// Several tabs (005): a new one is empty, in front, and has the keyboard; each tab is
    /// where it was left; closing the one in front brings the other forward.
    func testTabsOpenSwitchAndClose() {
        let app = launch(start: site.url("/"))
        XCTAssertTrue(pageShows("Home", in: app), "the first tab's page loaded")
        let tabs = app.buttons.matching(identifier: "tab")
        XCTAssertEqual(tabs.count, 1)

        press(app.buttons["new-tab"])
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "a new tab is empty, and in front")
        XCTAssertEqual(tabs.count, 2)
        // The keyboard is in the new tab's address field already: nothing is tapped first.
        let field = app.textFields["address-field"]
        field.typeText("127.0.0.1:\(site.port)/landed\n")
        XCTAssertTrue(pageShows("Landed", in: app), "the new tab loaded what was typed")

        // The first tab is where it was left.
        press(tabs.element(boundBy: 0))
        XCTAssertTrue(pageShows("Home", in: app), "the first tab came in front")
        expect(field, toShow: "http://127.0.0.1:\(site.port)")

        // Closing the tab in front brings the other forward.
        press(app.buttons.matching(identifier: "close-tab").element(boundBy: 0))
        XCTAssertTrue(pageShows("Landed", in: app), "the tab that was left came forward")
        XCTAssertEqual(tabs.count, 1)
    }

    /// A page that asks for a new window (006): a link with a target opens in a tab of its own,
    /// in front, beside the tab it came from; closing it brings that tab forward again.
    func testALinkThatOpensAWindowOpensATab() {
        let app = launch(start: site.url("/opener"))
        XCTAssertTrue(pageShows("Opener", in: app), "the first tab's page loaded")
        let tabs = app.buttons.matching(identifier: "tab")
        // A tab after the first, so that "beside" is not "at the end", and "the tab it came
        // from" is not "the tab after it".
        press(app.buttons["new-tab"])
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30))
        press(tabs.element(boundBy: 0))
        XCTAssertTrue(pageShows("Opener", in: app), "the first tab is in front again")

        let link = app.webViews.links["new window"]
        XCTAssertTrue(link.waitForExistence(timeout: 30), "the page's link is there to click")
        press(link)

        XCTAssertTrue(pageShows("Opened", in: app), "the link's page is in front")
        XCTAssertEqual(tabs.count, 3, "in a tab of its own")
        XCTAssertTrue(tabs.element(boundBy: 1).isSelected, "beside the tab it came from")
        let field = app.textFields["address-field"]
        expect(field, toShow: "http://127.0.0.1:\(site.port)/opened")

        // Closing it brings forward the tab it came from, not the tab after it.
        press(app.buttons.matching(identifier: "close-tab").element(boundBy: 1))
        XCTAssertTrue(pageShows("Opener", in: app), "back in the tab the link was in")
        XCTAssertEqual(tabs.count, 2)
        XCTAssertTrue(tabs.element(boundBy: 0).isSelected)
        expect(field, toShow: "http://127.0.0.1:\(site.port)/opener")
    }

    /// A window that is refused the only place it was going leaves no tab behind (006). The
    /// page's button opens a window on a number to ring, which is for the system and not for
    /// a browser: the core's policy refuses it, and the window has nothing to show.
    func testAWindowRefusedItsAddressLeavesNoTab() {
        let app = launch(start: site.url("/opener"))
        XCTAssertTrue(pageShows("Opener", in: app), "the page loaded")
        let tabs = app.buttons.matching(identifier: "tab")

        let ring = app.webViews.buttons["ring"]
        XCTAssertTrue(ring.waitForExistence(timeout: 30), "the page's button is there to click")
        press(ring)

        // The page is in front again, and its link works: nothing was left in the way.
        let link = app.webViews.links["new window"]
        XCTAssertTrue(link.waitForExistence(timeout: 30), "the page that asked is in front")
        press(link)
        XCTAssertTrue(pageShows("Opened", in: app), "the link's page is in front")
        XCTAssertEqual(tabs.count, 2, "the page and what its link opened: the refused window left no tab")
    }

    /// A link for the system (007), with the real core: a click on a link to write a letter
    /// leaves the page as it was. In fake mode the shell opens nothing, so the suite starts no
    /// mail app on the machine it runs on.
    func testALinkForTheSystemLeavesThePageAsItWas() {
        #if os(macOS)
            // Whether the mail app is running before, to know afterwards that this did not
            // start it. (If it was running already, that much cannot be told.)
            let mail = "com.apple.mail"
            let wasRunning = !NSRunningApplication.runningApplications(withBundleIdentifier: mail).isEmpty
        #endif
        let app = launch(start: site.url("/contact"))
        XCTAssertTrue(pageShows("Contact", in: app), "the page loaded")
        let field = app.textFields["address-field"]
        expect(field, toShow: "http://127.0.0.1:\(site.port)/contact")

        let link = app.webViews.links["write a letter"]
        XCTAssertTrue(link.waitForExistence(timeout: 30), "the page's link is there to click")
        press(link)

        // Long enough for the core to have heard of the link and asked the shell to open it.
        _ = app.webViews.links["no such link"].waitForExistence(timeout: 2)
        XCTAssertTrue(pageShows("Contact", in: app), "the page is still the one on screen")
        expect(field, toShow: "http://127.0.0.1:\(site.port)/contact")
        XCTAssertEqual(app.buttons.matching(identifier: "tab").count, 1, "and no tab was opened for it")
        XCTAssertEqual(app.state, .runningForeground, "the app is still in front")
        #if os(macOS)
            if !wasRunning {
                XCTAssertTrue(
                    NSRunningApplication.runningApplications(withBundleIdentifier: mail).isEmpty,
                    "fake mode started the mail app"
                )
            }
        #endif
    }

    /// History (008), with the real core: what was visited is listed, newest first; it is found
    /// by a word; choosing a visit loads it again; a visit is forgotten, and then all of them.
    func testHistoryListsWhatWasVisitedFindsItAndForgetsIt() {
        let app = launch(start: site.url("/"))
        XCTAssertTrue(pageShows("Home", in: app), "the first page loaded")
        let field = app.textFields["address-field"]
        openLocation(field, in: app)
        field.typeText("127.0.0.1:\(site.port)/landed\n")
        XCTAssertTrue(pageShows("Landed", in: app), "the second page loaded")

        press(app.buttons["history"])
        let visits = app.buttons.matching(identifier: "history-entry")
        XCTAssertTrue(visits.firstMatch.waitForExistence(timeout: 30), "the list opened, with visits in it")
        XCTAssertEqual(visits.count, 2)
        XCTAssertTrue(visits.element(boundBy: 0).label.contains("Landed"), "newest first: \(visits.element(boundBy: 0).label)")
        XCTAssertTrue(visits.element(boundBy: 1).label.contains("Home"))
        audit(app, "the History list")

        // A word of a title finds its visit, and only it.
        let search = app.textFields["history-search"]
        press(search)
        search.typeText("hom")
        let one = NSPredicate(format: "count == 1")
        XCTAssertEqual(XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: one, object: visits)], timeout: 30), .completed)
        XCTAssertTrue(visits.firstMatch.label.contains("Home"))

        // Choosing it loads it again, in the tab in front, and the list is gone.
        press(visits.firstMatch)
        XCTAssertTrue(pageShows("Home", in: app), "the visit chosen is the page on screen")
        expect(field, toShow: "http://127.0.0.1:\(site.port)")

        // That was a visit too. One is forgotten, and then all of them.
        press(app.buttons["history"])
        XCTAssertTrue(visits.firstMatch.waitForExistence(timeout: 30))
        XCTAssertEqual(visits.count, 3)
        press(app.buttons.matching(identifier: "history-forget").firstMatch)
        let two = NSPredicate(format: "count == 2")
        XCTAssertEqual(XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: two, object: visits)], timeout: 30), .completed)
        press(app.buttons["history-clear"])
        let everything = app.buttons["history-clear-everything"].firstMatch
        XCTAssertTrue(everything.waitForExistence(timeout: 30), "it asks which visits to forget")
        press(everything)
        XCTAssertTrue(app.staticTexts["history-nothing"].waitForExistence(timeout: 30), "nothing is left to list")
        XCTAssertEqual(visits.count, 0)
        press(app.buttons["history-done"])
        XCTAssertTrue(pageShows("Home", in: app), "and the page is as it was")
    }

    /// The tabs come back (009), with the app itself: it is ended without a word, as a crash
    /// or the system ends it, and started again; its tabs are there, the one that was in front
    /// is in front and loaded, and the other loads when it is chosen.
    func testTheTabsComeBackWhenTheAppStartsAgain() {
        var app = launch(onProfile: "ui-test-session", fresh: true)
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "a fresh profile opens one empty tab")
        var field = app.textFields["address-field"]
        openLocation(field, in: app)
        field.typeText("127.0.0.1:\(site.port)/landed\n")
        XCTAssertTrue(pageShows("Landed", in: app), "the first tab's page loaded")
        press(app.buttons["new-tab"])
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30))
        field.typeText("127.0.0.1:\(site.port)/\n")
        XCTAssertTrue(pageShows("Home", in: app), "the second tab's page loaded")

        app.terminate()
        app = launch(onProfile: "ui-test-session", fresh: false)

        let tabs = app.buttons.matching(identifier: "tab")
        XCTAssertTrue(pageShows("Home", in: app), "the tab that was in front is in front, and loaded")
        XCTAssertEqual(tabs.count, 2, "both tabs are back")
        XCTAssertTrue(
            tabs.element(boundBy: 0).label.contains("Landed"),
            "in their order, by their names: \(tabs.element(boundBy: 0).label)"
        )
        XCTAssertTrue(tabs.element(boundBy: 1).isSelected)
        field = app.textFields["address-field"]
        expect(field, toShow: "http://127.0.0.1:\(site.port)")

        // The other was asleep: it loads now that it is chosen.
        press(tabs.element(boundBy: 0))
        XCTAssertTrue(pageShows("Landed", in: app), "the tab behind loads when it comes in front")
        expect(field, toShow: "http://127.0.0.1:\(site.port)/landed")
    }

    /// Find on the page (010), with the real core and the real engine: text that is on the
    /// page is found without a word (WebKit does not count), text that is not there is said
    /// not to be, and closing the bar ends it.
    func testFindOnThePageSaysWhenTheTextIsNotThere() {
        let app = launch(start: site.url("/find"))
        XCTAssertTrue(pageShows("Find", in: app), "the page loaded")
        let field = app.textFields["find-field"]
        XCTAssertFalse(field.exists, "the find bar is shut until it is asked for")

        press(app.buttons["find"])
        XCTAssertTrue(field.waitForExistence(timeout: 30), "the find bar opened")
        press(field)
        field.typeText("needle")
        // Long enough for the engine to have answered: the text is there, and nothing is said.
        let outcome = app.staticTexts["find-outcome"]
        XCTAssertFalse(outcome.waitForExistence(timeout: 2), "text that is on the page is not \"not found\"")
        XCTAssertTrue(app.buttons["find-next"].isEnabled)
        press(app.buttons["find-next"])
        press(app.buttons["find-previous"])
        XCTAssertFalse(outcome.exists)
        audit(app, "the find bar")

        // Closing the bar ends the find; opened again it is empty.
        press(app.buttons["find-done"])
        let gone = NSPredicate(format: "exists == false")
        XCTAssertEqual(XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: gone, object: field)], timeout: 30), .completed)
        press(app.buttons["find"])
        XCTAssertTrue(field.waitForExistence(timeout: 30))
        // An empty field's value is nothing on a Mac and its placeholder on an iPhone.
        XCTAssertTrue(["", "Find on page"].contains(value(of: field)), "the bar opens empty: \(value(of: field))")
        press(field)
        field.typeText("zebra")

        XCTAssertTrue(outcome.waitForExistence(timeout: 30), "text that is not on the page is said not to be")
        // A text's words are its label on an iPhone and its value on a Mac.
        XCTAssertEqual(outcome.label.isEmpty ? value(of: outcome) : outcome.label, "Not found")
        press(app.buttons["find-done"])
        XCTAssertEqual(XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: gone, object: field)], timeout: 30), .completed)
        XCTAssertTrue(pageShows("Find", in: app), "and the page is as it was")
    }

    #if os(macOS)
        /// ⌘T opens a tab; ⌘W closes the one in front, and on the last tab the window.
        func testTheMacsTabShortcuts() {
            let app = launch(start: nil)
            XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30))
            let tabs = app.buttons.matching(identifier: "tab")
            app.typeKey("t", modifierFlags: .command)
            XCTAssertTrue(tabs.element(boundBy: 1).waitForExistence(timeout: 30), "⌘T opened a tab")
            app.typeKey("w", modifierFlags: .command)
            let one = NSPredicate(format: "count == 1")
            XCTAssertEqual(
                XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: one, object: tabs)], timeout: 30),
                .completed,
                "⌘W closed it"
            )
            XCTAssertTrue(app.windows.firstMatch.exists, "and left the window")
            app.typeKey("w", modifierFlags: .command)
            let none = NSPredicate(format: "exists == false")
            let closed = XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: none, object: app.windows.firstMatch)], timeout: 30)
            XCTAssertEqual(closed, .completed, "⌘W on the last tab closes the window")
        }
    #endif

    func testAPageThatCannotBeReachedSaysSoInWords() {
        // RFC 6761 reserves .invalid: it never resolves.
        let app = launch(start: "http://webtiger-test.invalid/")
        XCTAssertTrue(app.staticTexts["This page didn’t load"].waitForExistence(timeout: 30))
        XCTAssertTrue(app.staticTexts["The site’s address couldn’t be found."].exists, "the code became words")
        XCTAssertTrue(app.buttons["retry"].exists)
    }

    /// The design catalogue opens in place of a browser, shows its sections, and passes
    /// Apple's accessibility audit (004).
    func testTheDesignCatalogueOpensAndPassesTheAccessibilityAudit() {
        continueAfterFailure = true
        let app = XCUIApplication()
        app.launchEnvironment["WEBTIGER_CATALOGUE"] = "1"
        app.launch()
        XCTAssertTrue(app.staticTexts["Design Catalogue"].waitForExistence(timeout: 30), "the catalogue opened")
        XCTAssertTrue(app.staticTexts["Colours"].exists)
        XCTAssertTrue(app.staticTexts["The Window"].exists, "the window's states are in it")
        let fields = app.textFields.matching(identifier: "address-field")
        XCTAssertGreaterThan(fields.count, 1, "each state is the window itself, with an address field of its own")
        audit(app, "the design catalogue")
    }

    /// Apple's accessibility audit over each state of the window (000 §8.6): labels, contrast,
    /// hit regions, clipped text.
    func testEveryStateOfTheWindowPassesTheAccessibilityAudit() {
        continueAfterFailure = true
        let app = launch(start: nil)
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30))
        audit(app, "an empty tab")

        // The failed page, in the same run of the app. The app used to be ended here and
        // started again on the failing address; twice on GitHub's simulator it did not end
        // within a minute of being asked, straight after the audit, and nowhere else.
        let field = app.textFields["address-field"]
        openLocation(field, in: app)
        field.typeText("http://webtiger-test.invalid/\n")
        XCTAssertTrue(app.buttons["retry"].waitForExistence(timeout: 30))
        audit(app, "a failed page")
    }
}

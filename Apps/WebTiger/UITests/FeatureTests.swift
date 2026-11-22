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

    /// An open tab is offered while typing (038), with the real core: a page that is open in
    /// another tab is offered in a row that says it is a tab, and not a second time as the
    /// place it also is; choosing it brings that tab in front, and the site is not asked for
    /// the page again.
    func testAnOpenTabIsOfferedAndChoosingItBringsItInFront() {
        let app = launch(start: site.url("/landed"))
        XCTAssertTrue(pageShows("Landed", in: app), "the first tab's page loaded")
        let field = app.textFields["address-field"]
        let offered = app.buttons.matching(identifier: "suggestion")
        let tabs = app.buttons.matching(identifier: "tab")
        func askedForThePage() -> Int {
            site.requests.count(where: { $0 == "/landed" })
        }

        // A second tab, gone somewhere else.
        press(app.buttons["new-tab"])
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "a second, empty tab")
        openLocation(field, in: app)
        field.typeText("127.0.0.1:\(site.port)/\n")
        XCTAssertTrue(pageShows("Home", in: app), "the second tab's page loaded")
        XCTAssertEqual(tabs.count, 2)
        #if !os(macOS)
            // A phone's strip scrolls: the tab in front is brought into view, whatever is
            // then cut off of the other. (The audit below is told that a tab the strip cuts
            // off has no contrast to measure; a tab in view is measured.)
            let strip = app.descendants(matching: .any).matching(identifier: "tab-strip").firstMatch
            XCTAssertTrue(strip.frame.contains(tabs.element(boundBy: 1).frame), "the tab in front is in view in the strip")
        #endif
        let asked = askedForThePage()

        // Typed there, the start of a word of the first tab's title: one row, the tab.
        openLocation(field, in: app)
        field.typeText("lan")
        XCTAssertTrue(offered.firstMatch.waitForExistence(timeout: 30), "the open tab is offered")
        XCTAssertEqual(offered.count, 1, "once: not again as the place it also is")
        let row = offered.firstMatch.label
        XCTAssertTrue(row.contains("Landed"), "by its title: \(row)")
        XCTAssertTrue(row.contains("Switch to Tab"), "and the row says it is a tab: \(row)")
        audit(app, "an open tab offered")

        #if os(macOS)
            // The arrow key marks it, and Return goes to it.
            app.typeKey(.downArrow, modifierFlags: [])
            app.typeKey(.return, modifierFlags: [])
        #else
            press(offered.firstMatch)
        #endif

        XCTAssertTrue(pageShows("Landed", in: app), "the tab chosen is in front")
        expect(field, toShow: "http://127.0.0.1:\(site.port)/landed")
        XCTAssertEqual(offered.count, 0, "and the list is gone")
        XCTAssertEqual(tabs.count, 2, "no tab was opened, and none closed")
        XCTAssertEqual(askedForThePage(), asked, "the page was open: the site is not asked for it again")
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

        // It is the site's first download: the person is asked, in the tab, by the site's
        // name and the file's (015), and says yes. Not at once: a prompt is not answered in
        // the moment it appears.
        let allow = app.buttons["prompt-allow"]
        XCTAssertTrue(allow.waitForExistence(timeout: 300), "the tab asks whether the site may download")
        let sentence = app.staticTexts["prompt-words"]
        let asked = sentence.label.isEmpty ? value(of: sentence) : sentence.label
        XCTAssertTrue(asked.hasSuffix("wants to download “notes.txt”."), "the bar says \(asked)")
        audit(app, "the prompt for a download")
        usleep(700_000)
        press(allow)

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

    /// What a page says or asks (016), with the real core and WebKit: a question is shown in
    /// the tab under the site's name, and the page gets the answer; a prompt's field holds
    /// what the page offers; a page that has said something before can be stopped, and then
    /// says no more.
    func testWhatAPageSaysIsShownInItsTabAndThePageGetsTheAnswer() {
        let app = launch(start: site.url("/says"))
        XCTAssertTrue(pageShows("Says", in: app), "the page that says and asks loaded")
        func shownWords(_ identifier: String) -> String {
            let text = app.staticTexts[identifier]
            return text.label.isEmpty ? value(of: text) : text.label
        }

        // A press on the page's button: it asks yes or no, and its script waits.
        press(app.webViews.buttons["asks yes or no"])
        let ok = app.buttons["prompt-ok"]
        XCTAssertTrue(ok.waitForExistence(timeout: 30), "the tab shows what the page asks")
        XCTAssertEqual(shownWords("prompt-words"), "Delete it?")
        XCTAssertTrue(shownWords("prompt-site").hasSuffix(" says:"), "under the site's name: \(shownWords("prompt-site"))")
        XCTAssertFalse(app.buttons["prompt-silence"].exists, "a page's first is not to be stopped")
        audit(app, "what a page asks")
        // Not at once: a prompt is not answered in the moment it appears.
        usleep(700_000)
        press(ok)
        XCTAssertTrue(pageShows("confirm true", in: app), "the page has its answer")

        // It asks for a word: the field holds what the page offers. It is the page's
        // second: it could be stopped.
        press(app.webViews.buttons["asks for a word"])
        let field = app.textFields["prompt-field"]
        XCTAssertTrue(field.waitForExistence(timeout: 30))
        XCTAssertEqual(value(of: field), "Ada")
        XCTAssertTrue(app.buttons["prompt-silence"].exists)
        audit(app, "a page's prompt")
        usleep(700_000)
        press(app.buttons["prompt-ok"])
        XCTAssertTrue(pageShows("prompt Ada", in: app))

        // It says something more, and is stopped: what it asks after is answered as if it
        // had been closed, with nothing shown.
        press(app.webViews.buttons["says something"])
        let stop = app.buttons["prompt-silence"]
        XCTAssertTrue(stop.waitForExistence(timeout: 30))
        XCTAssertEqual(shownWords("prompt-words"), "Your basket is empty.")
        usleep(700_000)
        press(stop)
        XCTAssertTrue(pageShows("alert returned", in: app))
        press(app.webViews.buttons["asks yes or no"])
        XCTAssertTrue(pageShows("confirm false", in: app))
        XCTAssertFalse(app.buttons["prompt-ok"].exists)
    }

    /// A server asks who is there (017), with the real core and WebKit: the tab asks for a
    /// name and a password under the server's name; a wrong answer is said to be one and
    /// asked for again; the right one loads the page.
    func testAServerThatAsksWhoIsThereIsAnsweredInTheTab() {
        let app = launch(start: site.url("/staff"))
        func shownWords(_ identifier: String) -> String {
            let text = app.staticTexts[identifier]
            return text.label.isEmpty ? value(of: text) : text.label
        }
        // Types a name and a password, and signs in once the bar has been there long enough
        // to be answered.
        func signIn(_ name: String, _ password: String) {
            let nameField = app.textFields["prompt-name"]
            XCTAssertTrue(nameField.waitForExistence(timeout: 30), "the tab asks for a name")
            press(nameField)
            nameField.typeText(name)
            let passwordField = app.secureTextFields["prompt-password"]
            press(passwordField)
            passwordField.typeText(password)
            usleep(700_000)
            press(app.buttons["prompt-sign-in"])
        }

        // The load waits: the tab asks under the name of the server that gets what is typed.
        XCTAssertTrue(app.buttons["prompt-sign-in"].waitForExistence(timeout: 30), "the tab asks who is there")
        XCTAssertTrue(shownWords("prompt-site").hasPrefix("Sign in to "), shownWords("prompt-site"))
        // This device itself: nothing travels, and it is the first time of asking.
        XCTAssertFalse(app.staticTexts["prompt-not-private"].exists)
        XCTAssertFalse(app.staticTexts["prompt-again"].exists)
        audit(app, "a server asks who is there")

        // A wrong password: the server asks again, and the bar says so.
        signIn("ada", "wrong")
        XCTAssertTrue(app.staticTexts["prompt-again"].waitForExistence(timeout: 30), "said to be refused")
        audit(app, "a server asks again")

        // The right one: the page loads, and nothing more is asked.
        signIn("ada", "lovelace")
        XCTAssertTrue(pageShows("Staff only", in: app), "the page the server kept")
        XCTAssertFalse(app.buttons["prompt-sign-in"].exists)
    }

    /// A link the person asks for in a new tab (018), with the real core and WebKit. On a
    /// Mac: by the link's menu, which is WebKit's own with the shell's words for the item,
    /// and by a click with ⌘ held, which opens the tab behind. On an iPhone: by the menu a
    /// long press on the link shows, which is the shell's.
    func testALinkIsOpenedInANewTabAsThePersonAsks() {
        let app = launch(start: site.url("/links"))
        XCTAssertTrue(pageShows("Links", in: app), "the page of links loaded")
        let tabs = app.buttons.matching(identifier: "tab")
        let link = app.webViews.links["a link in words"]
        XCTAssertTrue(link.waitForExistence(timeout: 30), "the page's link is there to press")
        XCTAssertEqual(tabs.count, 1)

        #if os(macOS)
            // A click with ⌘ held: a tab behind, and the page being read stays in front.
            XCUIElement.perform(withKeyModifiers: .command) { link.click() }
            let behind = NSPredicate(format: "count == 2")
            XCTAssertEqual(
                XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: behind, object: tabs)], timeout: 30),
                .completed,
                "a second tab"
            )
            XCTAssertTrue(tabs.element(boundBy: 0).isSelected, "the page being read is still in front")
            XCTAssertTrue(pageShows("Links", in: app))

            // The link's menu: WebKit's own, with the shell's words for what its item does.
            link.rightClick()
            let item = app.menuItems["Open Link in New Tab"]
            XCTAssertTrue(item.waitForExistence(timeout: 30), "the menu's item has the shell's words")
            item.click()
            // WebKit's item asks for a window (006): its tab comes in front.
            XCTAssertTrue(pageShows("Landed", in: app), "the link's page, in a tab of its own")
            XCTAssertEqual(tabs.count, 3)
        #else
            // A finger held on the link: the shell's menu.
            link.press(forDuration: 1.2)
            let item = app.buttons["Open in New Tab"]
            XCTAssertTrue(item.waitForExistence(timeout: 30), "the link's menu")
            XCTAssertTrue(app.buttons["Copy Link"].exists)
            // No preview: nobody has asked the site for the link's page yet.
            XCTAssertFalse(site.requests.contains("/landed"), "the menu loaded the link: \(site.requests)")
            item.tap()
            let behind = NSPredicate(format: "count == 2")
            XCTAssertEqual(
                XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: behind, object: tabs)], timeout: 30),
                .completed,
                "a second tab"
            )
            XCTAssertTrue(tabs.element(boundBy: 0).isSelected, "the page being read is still in front")
            XCTAssertTrue(pageShows("Links", in: app))
            // Brought forward, it is the link's page.
            press(tabs.element(boundBy: 1))
            XCTAssertTrue(pageShows("Landed", in: app))
        #endif
    }

    #if os(macOS)
        /// A page's file input (019), in fake mode: the engine has no picker of the system's,
        /// so nothing of the system's is shown, and the page is told that nothing was chosen.
        /// (The system's own picker is shown by the app started for real, and by no test.)
        func testAFileInputShowsNothingOfTheSystemsInFakeMode() {
            let app = launch(start: site.url("/upload"))
            XCTAssertTrue(pageShows("Upload", in: app), "the page of file inputs loaded")
            let input = app.webViews.buttons.firstMatch
            XCTAssertTrue(input.waitForExistence(timeout: 30), "the page's file input is there to press")

            press(input)

            XCTAssertTrue(pageShows("cancelled one", in: app), "the page is told nothing was chosen")
            XCTAssertEqual(app.sheets.count, 0, "no panel of the system's")
            XCTAssertEqual(app.dialogs.count, 0)
        }

        /// A link pressed in another app (031), when WebTiger is not running: the Mac starts
        /// it and hands it the address, and its one window opens the link in a new tab in
        /// front, beside the tab the window starts with. The address is handed over as the
        /// system hands one, to the app under test and to no other: no other app is started,
        /// and the address is the test site's. (A link handed to a copy that is already
        /// running is not something a test can do: 031 §8 says how it was seen.)
        func testStartedByALinkFromAnotherAppTheWindowOpensItInANewTabInFront() throws {
            let asked = site.requests.count

            let app = try launch(start: site.url("/"), byLink: XCTUnwrap(URL(string: site.url("/landed"))))

            XCTAssertTrue(pageShows("Landed", in: app), "the link's page loaded")
            let tabs = app.buttons.matching(identifier: "tab")
            let two = NSPredicate(format: "count == 2")
            XCTAssertEqual(
                XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: two, object: tabs)], timeout: 30), .completed,
                "tabs \(tabs.count), windows \(app.windows.count), the site was asked \(site.requests.dropFirst(asked))"
            )
            // After the tab the window started with, in front; the field says where it is,
            // and the mark says what its connection is.
            XCTAssertTrue(tabs.element(boundBy: 1).isSelected, "the link's tab is in front")
            let first = tabs.element(boundBy: 0)
            XCTAssertTrue((first.label + value(of: first) + first.title).contains("Home"), "the window's own tab is before it")
            expect(app.textFields["address-field"], toShow: "http://127.0.0.1:\(site.port)/landed")
            // One window: the system opened no second one for the link, and the window that
            // was started took it.
            XCTAssertEqual(app.windows.count, 1)
            // The Mac counts this copy of the app among those that show an address of the
            // web: that is what its list of browsers is made of. (Only this app is looked
            // for: which others there are is nobody's business here.)
            let running = NSRunningApplication.runningApplications(withBundleIdentifier: "com.tigercodes.webtiger").compactMap(\.bundleURL)
            XCTAssertFalse(running.isEmpty)
            let showsTheWeb = try NSWorkspace.shared.urlsForApplications(toOpen: XCTUnwrap(URL(string: "https://example.com/")))
            XCTAssertTrue(
                running.contains { copy in showsTheWeb.contains { $0.standardizedFileURL == copy.standardizedFileURL } },
                "the Mac does not list the app under test for an address of the web"
            )
            // The site was asked for the window's own page and for the link, once each (the
            // two load at once: which reaches the site first is not said).
            XCTAssertEqual(site.requests.dropFirst(asked).filter { $0 != "/favicon.ico" }.sorted(), ["/", "/landed"], "\(site.requests)")

            // The copy of the app this test started is ended here, and seen to be gone: left
            // running, its window is in the way of the tests that come after (seen: one test
            // in each run failed, a different one each time, until this was here).
            endEveryCopy()
        }
    #endif

    /// The first run's default (020), with the real core: the list is offered with DuckDuckGo
    /// to keep; one press keeps it, an empty tab says so, and opened again the list has it in
    /// use by somebody's choice. No search is made: a search goes to the internet.
    func testTheFirstRunsDefaultIsKeptWithOnePress() {
        let app = launch(start: nil, firstRun: true)
        let keep = app.buttons["search-keep"]
        XCTAssertTrue(keep.waitForExistence(timeout: 30), "the first run's list")
        XCTAssertEqual(keep.label, "Keep DuckDuckGo")

        press(keep)

        let goesTo = app.staticTexts["search-goes-to"]
        XCTAssertTrue(goesTo.waitForExistence(timeout: 30), "the empty tab's line")
        XCTAssertEqual(goesTo.label.isEmpty ? value(of: goesTo) : goesTo.label, "Searches go to DuckDuckGo.")
        XCTAssertFalse(app.buttons["search-engine-ecosia"].exists, "kept, the list is gone")

        press(app.buttons["search-change"])
        let duckDuckGo = app.buttons["search-engine-duckduckgo"]
        XCTAssertTrue(duckDuckGo.waitForExistence(timeout: 30))
        XCTAssertTrue(duckDuckGo.label.contains("In use"), duckDuckGo.label)
        XCTAssertEqual(app.buttons["search-keep"].label, "Cancel")
    }

    /// Where searches go (020), with the real core, started as for a first run: the list is
    /// there before anything else, with the default marked; one press chooses another; an empty
    /// tab then says where searches go; and "Change…" opens the list again, with that engine
    /// marked. No search is made: a search goes to the internet.
    func testTheFirstRunAsksWhereSearchesGoAndAnEmptyTabSaysSo() {
        let app = launch(start: nil, firstRun: true)
        func shownWords(_ identifier: String) -> String {
            let text = app.staticTexts[identifier]
            return text.label.isEmpty ? value(of: text) : text.label
        }

        // The list, over the window.
        let ecosia = app.buttons["search-engine-ecosia"]
        XCTAssertTrue(ecosia.waitForExistence(timeout: 30), "the first run's question")
        XCTAssertTrue(app.staticTexts["search-question"].exists)
        XCTAssertTrue(app.buttons["search-engine-duckduckgo"].exists)
        // The default is marked as that, and one press keeps it; none is anybody's choice yet.
        XCTAssertTrue(app.buttons["search-engine-duckduckgo"].label.contains("Default"), app.buttons["search-engine-duckduckgo"].label)
        XCTAssertEqual(app.buttons["search-keep"].label, "Keep DuckDuckGo")
        XCTAssertFalse(ecosia.label.contains("In use"), "none is anybody's choice yet: \(ecosia.label)")
        audit(app, "the first run's question")

        press(ecosia)

        // Answered, it is gone, and the empty tab says where searches go.
        let goesTo = app.staticTexts["search-goes-to"]
        XCTAssertTrue(goesTo.waitForExistence(timeout: 30), "the empty tab's line")
        XCTAssertEqual(shownWords("search-goes-to"), "Searches go to Ecosia.")
        XCTAssertFalse(app.buttons["search-engine-ecosia"].exists)
        audit(app, "an empty tab that says where searches go")

        // "Change…": the list again, with the engine in use marked, and a way to leave it.
        press(app.buttons["search-change"])
        XCTAssertTrue(ecosia.waitForExistence(timeout: 30))
        XCTAssertTrue(ecosia.label.contains("In use"), ecosia.label)
        XCTAssertTrue(app.buttons["search-keep"].exists)
        audit(app, "the list of search engines, opened to change")
        press(app.buttons["search-keep"])
        XCTAssertTrue(goesTo.waitForExistence(timeout: 30))
        XCTAssertEqual(shownWords("search-goes-to"), "Searches go to Ecosia.", "nothing changed")
    }
}

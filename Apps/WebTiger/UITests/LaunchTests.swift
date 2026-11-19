// The W0 exit, as a test: the app starts the Rust core, the core asks the shell for a web view
// in the Space's login set, WebKit loads a page, and what the address field shows is what the
// core published. Fake mode keeps the store in memory, so the test leaves nothing behind.
import XCTest
#if os(macOS)
    import AppKit
#endif

@MainActor
final class LaunchTests: XCTestCase {
    private let site = TestSite()

    override func setUp() async throws {
        continueAfterFailure = false
        XCTAssertNotEqual(site.port, 0, "the loopback site is listening")
    }

    private func launch(start: String?) -> XCUIApplication {
        let app = XCUIApplication()
        app.launchEnvironment["WEBTIGER_FAKE"] = "1"
        if let start {
            app.launchEnvironment["WEBTIGER_FAKE_START"] = start
        }
        app.launch()
        return app
    }

    /// Launches the app itself, not in fake mode, on a profile of the suite's own: a store on
    /// disk in a folder apart from anyone's data, which the next launch finds as it was left.
    private func launch(onProfile profile: String, fresh: Bool) -> XCUIApplication {
        let app = XCUIApplication()
        app.launchEnvironment["WEBTIGER_PROFILE"] = profile
        if fresh {
            app.launchEnvironment["WEBTIGER_PROFILE_FRESH"] = "1"
        }
        app.launch()
        return app
    }

    /// Waits for the page titled `title` to be the one on screen.
    private func pageShows(_ title: String, in app: XCUIApplication) -> Bool {
        #if os(macOS)
            // The window takes the page's title, which the core publishes.
            app.windows[title].waitForExistence(timeout: 30)
        #else
            app.webViews.staticTexts[title].waitForExistence(timeout: 30)
        #endif
    }

    /// Puts the keyboard in the address field with the whole address selected, so what is typed
    /// next replaces it: ⌘L on a Mac, a tap on a phone.
    private func openLocation(_ field: XCUIElement, in app: XCUIApplication) {
        #if os(macOS)
            app.typeKey("l", modifierFlags: .command)
        #else
            field.tap()
        #endif
    }

    private func press(_ button: XCUIElement) {
        #if os(macOS)
            button.click()
        #else
            button.tap()
        #endif
    }

    private func value(of field: XCUIElement) -> String {
        field.value as? String ?? ""
    }

    /// Waits for the address field to show `text`.
    private func expect(_ field: XCUIElement, toShow text: String, file: StaticString = #filePath, line: UInt = #line) {
        let shown = NSPredicate(format: "value == %@", text)
        let waited = XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: shown, object: field)], timeout: 30)
        XCTAssertEqual(waited, .completed, "the address field shows \(value(of: field)), not \(text)", file: file, line: line)
    }

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

    /// Whether a point of the screen is a single pixel: too coarse for contrast to be measured.
    private static var screenIsCoarse: Bool {
        #if os(macOS)
            (NSScreen.main?.backingScaleFactor ?? 2) < 2
        #else
            false
        #endif
    }

    /// Runs Apple's accessibility audit and names every element it faults: its kind, its
    /// identifier and its label, never the whole tree.
    private func audit(_ app: XCUIApplication, _ state: String, file: StaticString = #filePath, line: UInt = #line) {
        var faults: [String] = []
        do {
            try app.performAccessibilityAudit { issue in
                let element = issue.element
                // A container that only groups other elements needs no description of its own.
                if issue.auditType == .sufficientElementDescription, let kind = element?.elementType,
                   kind == .group || kind == .other, element?.identifier.isEmpty ?? true {
                    return true
                }
                // A tab with no title is named by its site, and a site's name is what it is: the
                // audit takes a host for a label nobody could read.
                if issue.auditType == .sufficientElementDescription, element?.identifier == "tab" {
                    return true
                }
                // An element cut by the window's edge is not drawn whole, and what is not drawn
                // has no contrast to measure. (A long screen that scrolls has one at its foot.)
                if issue.auditType == .contrast, let frame = element?.frame, !app.windows.firstMatch.frame.contains(frame) {
                    return true
                }
                // Contrast is measured from the screen's pixels. Where a point is one pixel,
                // small text is drawn in greys that never reach its colour, and plain black on
                // white is faulted: GitHub's Mac does it, and the recording in its results
                // shows why. It is measured where a point has more pixels than one.
                if issue.auditType == .contrast, Self.screenIsCoarse {
                    return true
                }
                #if os(macOS)
                    // A fault with no element cannot be named, and so cannot be mended. macOS
                    // 27.0 reports one such mismatch in the hierarchy for every state of the
                    // window; 27.2 reports none.
                    if issue.auditType == .parentChild, element == nil {
                        return true
                    }
                #endif
                #if !os(macOS)
                    // A one-line text field scrolls what does not fit: that is not clipped text.
                    if issue.auditType == .textClipped, element?.elementType == .textField {
                        return true
                    }
                    // A tab in the strip shows one line of its title, and is read out whole.
                    if issue.auditType == .textClipped, element?.identifier == "tab" {
                        return true
                    }
                    // A visit's address in the History list may be any length: it is cut to
                    // two lines, in the middle, and is read out whole.
                    if issue.auditType == .textClipped, element?.identifier == "history-address" {
                        return true
                    }
                #endif
                // What the element is, where it is, and Apple's own account of the fault: enough
                // to find it from a runner's log, where nobody can look at the window.
                let frame = element.map { "\(Int($0.frame.minX)),\(Int($0.frame.minY)) \(Int($0.frame.width))x\(Int($0.frame.height))" }
                let what = [element.map { "\($0.elementType.rawValue)" }, element?.identifier, element?.label, frame]
                    .compactMap(\.self).filter { !$0.isEmpty }.joined(separator: " / ")
                faults.append("\(issue.compactDescription) [\(what)]: \(issue.detailedDescription)")
                return true
            }
        } catch {
            faults.append("the audit could not run: \(error.localizedDescription)")
        }
        XCTAssertTrue(faults.isEmpty, "\(state): \(faults.joined(separator: "; "))", file: file, line: line)
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
        let empty = launch(start: nil)
        XCTAssertTrue(empty.staticTexts["empty-tab"].waitForExistence(timeout: 30))
        audit(empty, "an empty tab")
        empty.terminate()

        let failed = launch(start: "http://webtiger-test.invalid/")
        XCTAssertTrue(failed.buttons["retry"].waitForExistence(timeout: 30))
        audit(failed, "a failed page")
    }
}

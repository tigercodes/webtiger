// No more live tabs than the device can hold (054), in the app.
//
// In fake mode, with the real core and the real engine and a limit of two given by the suite
// (`WEBTIGER_FAKE_LIVE_TABS`), where fake mode has none: a third tab's arrival puts the first
// to sleep at once, and says so; pressed, it wakes at the page it was at with its way back,
// and the tab that is then the longest out of sight sleeps in its turn.
//
// And on an iPhone or an iPad, the app itself on a profile of the suite's own, with nothing
// said of a limit: the app says what it runs on, the core holds three tabs alive on the one
// and six on the other, and one more puts the first to sleep.
import XCTest

@MainActor
final class LiveTabsTests: AppTestCase {
    /// Sends the tab in front to a page of the test's site.
    private func go(to path: String, titled title: String, in app: XCUIApplication) {
        let field = app.textFields["address-field"]
        openLocation(field, in: app)
        field.typeText("127.0.0.1:\(site.port)\(path)\n")
        XCTAssertTrue(pageShows(title, in: app), "the page at \(path) loaded")
    }

    /// Opens a tab in front of the others, at a page of the test's site.
    private func openTab(at path: String, titled title: String, in app: XCUIApplication) {
        press(app.buttons["new-tab"])
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "an empty tab")
        go(to: path, titled: title, in: app)
    }

    /// Waits for a button to be one that can be pressed.
    private func expectEnabled(_ button: XCUIElement, _ why: String, file: StaticString = #filePath, line: UInt = #line) {
        let enabled = XCTNSPredicateExpectation(predicate: NSPredicate(format: "isEnabled == true"), object: button)
        XCTAssertEqual(XCTWaiter.wait(for: [enabled], timeout: 30), .completed, why, file: file, line: line)
    }

    func testAThirdTabPutsTheFirstToSleepWhereTwoAreHeld() {
        let app = launch(start: site.url("/"), liveTabs: 2)
        XCTAssertTrue(pageShows("Home", in: app), "the first tab's page loaded")
        // The first tab goes on to Landed: it has somewhere to go back to.
        go(to: "/landed", titled: "Landed", in: app)
        openTab(at: "/whoami", titled: "who=none", in: app)
        let tabs = app.windows.element(boundBy: 0).buttons.matching(identifier: "tab")
        let first = tabs.element(boundBy: 0)
        let second = tabs.element(boundBy: 1)
        let third = tabs.element(boundBy: 2)
        expect(first, toSay: "", "two are held, and two are alive")

        // A third: the first, the longest out of sight, sleeps as it arrives, where it was in
        // its row and by the name it had. The second has only just left the front.
        openTab(at: "/tall", titled: "Tall", in: app)
        expect(first, toSay: "Asleep", "the tab longest out of sight is put to sleep by the third's arrival")
        XCTAssertEqual(first.label, "Landed")
        expect(second, toSay: "", "the tab that has just left the front is awake")
        expect(third, toSay: "", "and the tab in front")

        // Pressed, it wakes at the page it was at, and its way back came with it. That is a
        // tab come alive: the second, now the longest out of sight, sleeps in its turn.
        press(first)
        XCTAssertTrue(pageShows("Landed", in: app), "the tab woke where it was")
        expect(first, toSay: "", "and is awake")
        expect(second, toSay: "Asleep", "the tab that is now the longest out of sight sleeps")
        expect(third, toSay: "", "the tab that has just left the front is awake")
        let back = app.buttons["back"]
        expectEnabled(back, "a tab that wakes from its state has the list it had")
        press(back)
        XCTAssertTrue(pageShows("Home", in: app), "back goes back, to the page before it slept")
    }

    #if os(iOS)
        /// The app itself, on a profile of the suite's own, with nothing said of a limit: it says
        /// what it runs on, and the core holds three live tabs on an iPhone and six on an iPad.
        func testOneTabMoreThanTheDeviceHoldsPutsTheFirstToSleep() {
            // The simulator the suite runs on.
            let held = UIDevice.current.userInterfaceIdiom == .pad ? 6 : 3
            let pages = [
                ("/", "Home"), ("/landed", "Landed"), ("/whoami", "who=none"), ("/tall", "Tall"),
                ("/find", "Find"), ("/contact", "Contact"), ("/pictures", "Pictures"),
            ]
            let app = launch(onProfile: "ui-test-live-tabs", fresh: true)
            // A fresh profile is a first run (020): it asks where searches go before anything else.
            let question = app.buttons["search-engine-ecosia"]
            XCTAssertTrue(question.waitForExistence(timeout: 30), "a fresh profile is asked where searches go")
            press(question)
            XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "a fresh profile opens one empty tab")

            // As many tabs as it holds, each at a page of its own: all are awake.
            go(to: pages[0].0, titled: pages[0].1, in: app)
            for page in pages[1 ..< held] {
                openTab(at: page.0, titled: page.1, in: app)
            }
            let tabs = app.windows.element(boundBy: 0).buttons.matching(identifier: "tab")
            let first = tabs.element(boundBy: 0)
            expect(first, toSay: "", "it holds \(held)")

            // One more: the first sleeps. Nobody said a limit: the app said what it runs on.
            openTab(at: pages[held].0, titled: pages[held].1, in: app)
            expect(first, toSay: "Asleep", "one tab more than it holds puts the first to sleep")
            XCTAssertEqual(first.label, "Home")
            for other in 1 ... held {
                expect(tabs.element(boundBy: other), toSay: "", "tab \(other + 1) is awake")
            }
        }
    #endif
}

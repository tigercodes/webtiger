// A tab sleeps by itself (053), in the app, in fake mode: the real core, the real engine, and
// a sleep time of seconds where a person's is half an hour (`WEBTIGER_FAKE_SLEEP_AFTER`). A
// tab that is left behind is asleep a few seconds later, and says so; pressed, it wakes at the
// page it was at, and Back in it goes back: the core asked the engine for its state as it put
// it to sleep, and made its web view from that state when it woke.
import XCTest

@MainActor
final class SleepTests: AppTestCase {
    /// Sends the tab in front to a page of the test's site.
    private func go(to path: String, titled title: String, in app: XCUIApplication) {
        let field = app.textFields["address-field"]
        openLocation(field, in: app)
        field.typeText("127.0.0.1:\(site.port)\(path)\n")
        XCTAssertTrue(pageShows(title, in: app), "the page at \(path) loaded")
    }

    /// Waits for a button to be one that can be pressed.
    private func expectEnabled(_ button: XCUIElement, _ why: String, file: StaticString = #filePath, line: UInt = #line) {
        let enabled = XCTNSPredicateExpectation(predicate: NSPredicate(format: "isEnabled == true"), object: button)
        XCTAssertEqual(XCTWaiter.wait(for: [enabled], timeout: 30), .completed, why, file: file, line: line)
    }

    func testATabLeftBehindSleepsAndWakesWhereItWasWithItsWayBack() {
        let app = launch(start: site.url("/"), sleepAfter: 4)
        XCTAssertTrue(pageShows("Home", in: app), "the first tab's page loaded")
        // The first tab goes on to Landed: it has somewhere to go back to.
        go(to: "/landed", titled: "Landed", in: app)
        XCTAssertTrue(app.buttons["back"].isEnabled, "back is offered")

        // Another tab comes in front of it. The first is out of sight from now.
        press(app.buttons["new-tab"])
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "an empty tab")
        go(to: "/whoami", titled: "who=none", in: app)
        let tabs = app.windows.element(boundBy: 0).buttons.matching(identifier: "tab")
        let first = tabs.element(boundBy: 0)
        let second = tabs.element(boundBy: 1)

        // A few seconds on, the first is asleep, where it was in its row and by the name it
        // had; the tab in front is awake, however long it is looked at.
        expect(first, toSay: "Asleep", "the tab that was left behind is put to sleep")
        XCTAssertEqual(first.label, "Landed")
        sleep(6)
        expect(second, toSay: "", "the tab in front does not sleep")
        XCTAssertTrue(pageShows("who=none", in: app))

        // Pressed, it wakes at the page it was at, and its way back came with it.
        press(first)
        XCTAssertTrue(pageShows("Landed", in: app), "the tab woke where it was")
        expect(first, toSay: "", "and is awake")
        let back = app.buttons["back"]
        expectEnabled(back, "a tab that wakes from its state has the list it had")
        press(back)
        XCTAssertTrue(pageShows("Home", in: app), "back goes back, to the page before it slept")
        // And the tab it was woken over is out of sight in its turn: it sleeps.
        expect(second, toSay: "Asleep", "the tab that is now behind sleeps in its turn")
    }
}

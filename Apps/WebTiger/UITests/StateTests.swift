// A tab's kept state across a quit (051), with the app itself on a profile of the suite's own:
// a store on disk, in a folder apart from anyone's data. A tab that has been to two pages and
// has left the front comes back, when the app starts again and it is pressed, at the page it
// was at, and Back in it goes back: the core asked the engine for its state when it left the
// front, kept it in the store, and handed it back when the tab woke.
import XCTest

@MainActor
final class StateTests: AppTestCase {
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

    func testATabThatLeftTheFrontHasItsWayBackWhenTheAppStartsAgain() {
        var app = launch(onProfile: "ui-test-state", fresh: true)
        // A fresh profile is a first run (020): it asks where searches go before anything else.
        let question = app.buttons["search-engine-ecosia"]
        XCTAssertTrue(question.waitForExistence(timeout: 30), "a fresh profile is asked where searches go")
        press(question)
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "a fresh profile opens one empty tab")

        // The first tab goes to Home and on to Landed: it has somewhere to go back to.
        go(to: "/", titled: "Home", in: app)
        go(to: "/landed", titled: "Landed", in: app)
        XCTAssertTrue(app.buttons["back"].isEnabled, "back is offered")
        // Another tab comes in front of it: the first has left the front.
        press(app.buttons["new-tab"])
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "an empty tab")
        go(to: "/whoami", titled: "who=none", in: app)
        XCTAssertFalse(app.buttons["back"].isEnabled, "the new tab has been nowhere else")

        // Ended without a word, as a crash or the system ends it, and started again.
        app.terminate()
        app = launch(onProfile: "ui-test-state", fresh: false)
        XCTAssertTrue(pageShows("who=none", in: app), "the tab that was in front is in front, and loaded")
        XCTAssertFalse(app.buttons["back"].isEnabled, "it never left the front: it was loaded, and has been nowhere else")

        // The first tab is asleep. Pressed, it wakes at the page it was at, and its way back
        // came with it: before a tab kept its state, Back was grey here.
        let tabs = app.windows.element(boundBy: 0).buttons.matching(identifier: "tab")
        press(tabs.element(boundBy: 0))
        XCTAssertTrue(pageShows("Landed", in: app), "the first tab woke where it was")
        let back = app.buttons["back"]
        expectEnabled(back, "a tab that wakes from its state has the list it had")
        press(back)
        XCTAssertTrue(pageShows("Home", in: app), "back goes back, to the page before the quit")
        expectEnabled(app.buttons["forward"], "and forward goes where it came from")
    }
}

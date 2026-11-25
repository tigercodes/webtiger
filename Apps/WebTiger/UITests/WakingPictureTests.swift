// A sleeping tab's picture (059), in the app, in fake mode: the real core, the real engine,
// and a sleep time of seconds. A tab at the suite's slow page, which answers after three
// seconds, is left behind and sleeps: the core asked the engine for a picture of its page as
// it put it to sleep. Pressed, the tab is on its way to that page again for three seconds,
// and the page as it was stands where the page will be; when the page comes, it is gone.
import XCTest

@MainActor
final class WakingPictureTests: AppTestCase {
    /// Sends the tab in front to a page of the test's site.
    private func go(to path: String, titled title: String, in app: XCUIApplication) {
        let field = app.textFields["address-field"]
        openLocation(field, in: app)
        field.typeText("127.0.0.1:\(site.port)\(path)\n")
        XCTAssertTrue(pageShows(title, in: app), "the page at \(path) loaded")
    }

    func testATabThatWakesShowsItsPageAsItWasUntilThePageComes() {
        let app = launch(start: site.url("/slow"), sleepAfter: 4)
        XCTAssertTrue(pageShows("Slow", in: app), "the first tab's page loaded")
        let picture = app.images["page-picture"]
        XCTAssertFalse(picture.exists, "a page that came by itself has no picture over it")

        // Another tab comes in front of it. The first is out of sight from now, and sleeps.
        press(app.buttons["new-tab"])
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "an empty tab")
        go(to: "/whoami", titled: "who=none", in: app)
        let first = app.windows.element(boundBy: 0).buttons.matching(identifier: "tab").element(boundBy: 0)
        expect(first, toSay: "Asleep", "the tab that was left behind is put to sleep")
        XCTAssertFalse(picture.exists, "a sleeping tab's picture is not over another tab's page")

        // Pressed, it is on its way to its page, which takes three seconds to answer: the page
        // as it was is where the page will be, and says what it is.
        press(first)
        XCTAssertTrue(picture.waitForExistence(timeout: 3), "the page as it was stands where the page will be")
        XCTAssertEqual(picture.label, "The page as it was")

        // The page comes, and the picture is gone.
        XCTAssertTrue(pageShows("Slow", in: app), "the tab woke where it was")
        let gone = XCTNSPredicateExpectation(predicate: NSPredicate(format: "exists == false"), object: picture)
        XCTAssertEqual(XCTWaiter.wait(for: [gone], timeout: 30), .completed, "the picture goes when the page has come")
        expect(first, toSay: "", "and the tab is awake")
    }
}

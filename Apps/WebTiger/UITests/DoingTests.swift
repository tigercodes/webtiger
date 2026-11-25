// Marks for what a tab is doing (058), in the app, in fake mode: the real core and the real
// engine. A page of the suite's own plays a sound (silence, as a sound file) when its button
// is pressed: its tab then says so in the row, in front and behind, and no longer once the
// sound is stopped. The camera's mark is not tried here: no test lets a page have a camera.
import XCTest

@MainActor
final class DoingTests: AppTestCase {
    /// Sends the tab in front to a page of the test's site.
    private func go(to path: String, titled title: String, in app: XCUIApplication) {
        let field = app.textFields["address-field"]
        openLocation(field, in: app)
        field.typeText("127.0.0.1:\(site.port)\(path)\n")
        XCTAssertTrue(pageShows(title, in: app), "the page at \(path) loaded")
    }

    func testATabThatPlaysASoundSaysSoInTheRowUntilTheSoundStops() {
        let app = launch(start: site.url("/doing"))
        XCTAssertTrue(pageShows("Doing", in: app), "the page loaded")
        let tabs = app.windows.element(boundBy: 0).buttons.matching(identifier: "tab")
        let first = tabs.element(boundBy: 0)
        expect(first, toSay: "", "a tab whose page does nothing says nothing")

        // The page's own button starts its sound: the tab says so.
        let play = app.webViews.buttons["play"]
        XCTAssertTrue(play.waitForExistence(timeout: 30), "the page has its button")
        press(play)
        expect(first, toSay: "Playing sound", "a tab whose page plays a sound says so")

        // Another tab comes in front: the first says so still, from behind.
        press(app.buttons["new-tab"])
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "an empty tab")
        go(to: "/landed", titled: "Landed", in: app)
        expect(first, toSay: "Playing sound", "it says so from behind")
        expect(tabs.element(boundBy: 1), toSay: "", "and the tab in front says nothing")
        // Audited here, with a page in front that has nothing of its own: Apple's audit looks
        // at the page too, and the suite's page of fields has fields with no names.
        audit(app, "a tab that plays a sound, behind another")

        // In front again, and its sound stopped: it says nothing.
        press(first)
        XCTAssertTrue(pageShows("Doing", in: app))
        let stop = app.webViews.buttons["stop"]
        XCTAssertTrue(stop.waitForExistence(timeout: 30))
        press(stop)
        expect(first, toSay: "", "the sound has stopped: the tab says nothing")
    }
}

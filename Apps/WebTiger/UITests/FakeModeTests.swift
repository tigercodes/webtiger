// Fake mode (001 §7), where every suite of the app's runs: nothing of one run is there in the
// next. The core's store is in memory there, and so is what pages keep: a test that signs
// in, even with a cookie that is to be kept, leaves the next test's window signed in by
// nobody.
import XCTest

@MainActor
final class FakeModeTests: AppTestCase {
    func testWhatAPageKeepsDoesNotOutliveTheApp() {
        // Signed in at the site with a cookie that is to be kept for an hour.
        var app = launch(start: site.url("/set?v=ada"))
        XCTAssertTrue(pageShows("Set", in: app), "signed in")
        let field = app.textFields["address-field"]
        openLocation(field, in: app)
        field.typeText("127.0.0.1:\(site.port)/whoami\n")
        XCTAssertTrue(pageShows("who=ada", in: app), "signed in, while the app runs")

        // The app ends, and starts again: nobody is signed in.
        app.terminate()
        app = launch(start: site.url("/whoami"))
        XCTAssertTrue(pageShows("who=none", in: app), "what the page kept went with the app")
        XCTAssertEqual(site.headers(of: "/whoami").map { $0["cookie"] ?? "" }, ["who=ada", ""], "\(site.requests)")
    }
}

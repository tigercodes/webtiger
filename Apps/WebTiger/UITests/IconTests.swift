// A page's icon in the app (046 §5a), in fake mode, with the real core and the real engine.
// On Apple's systems WebKit hands an app no icon: the app asks the site for the one a page
// names. What a test sees of that from outside is what the site was asked for: once in a run
// for an address, and as nobody, though the page itself is asked for as whoever is signed in.
// (That the picture is then drawn is the pictures' to show, and the window's model's tests.)
//
// In fake mode the app fetches no icon unless a suite says so, and then from this device
// only, where the suite's site is.
import XCTest

@MainActor
final class IconTests: AppTestCase {
    /// How many times the site was asked for a path.
    private func asked(_ path: String) -> Int {
        site.requests.count { $0 == path }
    }

    /// Waits until the site has been asked for a path.
    private func awaitRequest(_ path: String, line: UInt = #line) {
        let deadline = Date().addingTimeInterval(30)
        while asked(path) == 0, Date() < deadline {
            usleep(100_000)
        }
        XCTAssertEqual(asked(path), 1, "the site was asked for \(path): \(site.requests)", line: line)
    }

    func testTheAppAsksForAPagesIconOnceAndAsNobody() throws {
        // Somebody signs in at the site, in the window's login set: for the session only, with
        // a cookie the engine keeps in memory. (The app's web data outlives a test: a cookie
        // that is kept would sign the next test's window in too.) The page names no icon: the
        // app asks for the one every site may have, and there is none.
        let app = launch(start: site.url("/session?v=ada"), icons: true)
        XCTAssertTrue(pageShows("Session", in: app), "signed in")
        awaitRequest("/favicon.ico")
        let field = app.textFields["address-field"]

        // A page that names its icon: the app asks the site for it.
        openLocation(field, in: app)
        field.typeText("127.0.0.1:\(site.port)/with-served-icon\n")
        XCTAssertTrue(pageShows("With served icon", in: app), "the page loaded")
        awaitRequest("/icon.png")

        // The page was asked for as who is signed in; its icon as nobody, saying nothing of
        // where it came from.
        let page = try XCTUnwrap(site.headers(of: "/with-served-icon").last)
        XCTAssertEqual(page["cookie"], "who=ada", "the page is asked for with its login set's cookie")
        let icon = try XCTUnwrap(site.headers(of: "/icon.png").last)
        XCTAssertNil(icon["cookie"], "the icon was asked for with the page's cookie")
        XCTAssertNil(icon["referer"])
        XCTAssertNil(icon["authorization"])
        XCTAssertEqual(icon["user-agent"], page["user-agent"], "the fetch says it is what the page says it is")

        // The same page in a second tab, and a page of the site that names none: neither
        // address is asked for again in this run.
        press(app.buttons["new-tab"])
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "a second, empty tab")
        openLocation(field, in: app)
        field.typeText("127.0.0.1:\(site.port)/with-served-icon\n")
        XCTAssertTrue(pageShows("With served icon", in: app), "the page loaded in the second tab")
        openLocation(field, in: app)
        field.typeText("127.0.0.1:\(site.port)/landed\n")
        XCTAssertTrue(pageShows("Landed", in: app), "a page that names no icon")
        // (Time for a fetch that should not happen to have happened.)
        usleep(1_500_000)
        XCTAssertEqual(asked("/icon.png"), 1, "\(site.requests)")
        XCTAssertEqual(asked("/favicon.ico"), 1, "\(site.requests)")
    }

    func testInFakeModeNoIconIsAskedForUnlessASuiteSaysSo() {
        let app = launch(start: site.url("/with-served-icon"))
        XCTAssertTrue(pageShows("With served icon", in: app), "the page loaded")
        usleep(1_500_000)
        XCTAssertEqual(site.requests.filter { $0 == "/icon.png" || $0 == "/favicon.ico" }, [], "\(site.requests)")
    }
}

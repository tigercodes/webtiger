// Third-party cookies (070), in the app, in fake mode, with the real core and the real engine:
// a site that has kept a name is, in another site's page, asked with nothing. What the engine
// does is its adapter's suite's; this is the app as it is put together.
import XCTest

@MainActor
final class ThirdPartyCookieTests: AppTestCase {
    func testASiteInAnotherSitesPageIsNotSentItsCookies() {
        // The site, by its other name, keeps a name: in a cookie, and in one that says it
        // may go anywhere.
        let app = launch(start: "http://localhost:\(site.port)/keep?v=ada")
        XCTAssertTrue(pageShows("Kept", in: app), "the site kept a name")

        // Then a page of another site holds it in a frame, and a picture of it.
        let field = app.textFields["address-field"]
        openLocation(field, in: app)
        field.typeText("127.0.0.1:\(site.port)/holds-another-site\n")
        let shutOut = "frame: asked none/none, reads none, writes no, stored none"
        XCTAssertTrue(pageShows(shutOut, in: app), "the site's script in the frame has nothing")
        // And by the site's own record, neither was asked for with a cookie.
        XCTAssertEqual(site.headers(of: "/kept").map { $0["cookie"] }, [nil])
        XCTAssertEqual(site.headers(of: "/pixel").map { $0["cookie"] }, [nil])

        // As itself, the site has what it kept.
        openLocation(field, in: app)
        field.typeText("http://localhost:\(site.port)/kept\n")
        XCTAssertTrue(pageShows("asked ada/ada, reads anywhere+who, writes yes, stored ada", in: app), "the site has what it kept")
    }
}

// Going on past a certificate that is not trusted (026) in the app, in fake mode, with the real
// core and the real engine, and a site over https in this process whose certificate signs
// itself: its identity is made in memory, and no keychain is touched.
import XCTest

@MainActor
final class CertificateTests: AppTestCase {
    /// A site whose certificate signs itself is refused, and the page says why. Nothing on it
    /// goes on by one press: "Advanced…" shows the warning, which names the site; and until
    /// the button under the warning is pressed the site is never asked for its page. Pressed,
    /// the page loads, and the mark beside its address says it is not secure. Apple's
    /// accessibility audit passes over the failed page and over the warning.
    func testACertificateThatIsNotTrustedIsWarnedOfBeforeItIsGoneOnPast() throws {
        let secure = SecureSite()
        try XCTSkipIf(secure.port == 0, "this system would not make a TLS identity in memory, or would not listen")
        let app = launch(start: site.url("/"))
        XCTAssertTrue(pageShows("Home", in: app))
        let field = app.textFields["address-field"]
        openLocation(field, in: app)

        field.typeText("\(secure.address)\n")

        // Refused: the page says which trouble it is, and offers to try again or to say more.
        let advanced = app.buttons["certificate-advanced"]
        XCTAssertTrue(advanced.waitForExistence(timeout: 30), "the failed page offers to say more")
        XCTAssertTrue(app.staticTexts["This device doesn’t trust this site’s certificate."].exists, "which trouble it is")
        XCTAssertTrue(app.buttons["retry"].exists)
        let goOn = app.buttons["certificate-go-on"]
        XCTAssertFalse(goOn.exists, "no way on before the warning")
        // Nothing vouches for an address whose certificate was just refused.
        XCTAssertFalse(app.images["security-secure"].exists, "no padlock beside a refused certificate")
        XCTAssertEqual(secure.served, 0, "the site was not asked for its page")
        audit(app, "a certificate that is not trusted")

        press(advanced)

        // The warning, which names the site; and still nothing was asked of it.
        XCTAssertTrue(goOn.waitForExistence(timeout: 30), "the warning, and the button under it")
        XCTAssertEqual(goOn.label, "Go On to 127.0.0.1 (Not Safe)")
        XCTAssertTrue(app.staticTexts["Going on isn’t safe"].exists)
        XCTAssertFalse(advanced.exists, "the warning is in the place of the button that showed it")
        XCTAssertEqual(secure.served, 0, "a warning lets nothing through")
        audit(app, "the warning before going on")

        press(goOn)

        // Gone on: the site's page, asked for once, and marked as not secure.
        XCTAssertTrue(pageShows(SecureSite.title, in: app), "the site's page")
        XCTAssertEqual(secure.served, 1)
        XCTAssertTrue(app.images["security-insecure"].waitForExistence(timeout: 30), "marked as not secure")
        XCTAssertFalse(app.images["security-secure"].exists, "a page past a certificate is never shown as secure")
        XCTAssertFalse(app.buttons["retry"].exists)
    }
}

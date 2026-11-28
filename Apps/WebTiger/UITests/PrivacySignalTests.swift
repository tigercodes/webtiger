// Global Privacy Control (069), in the app, in fake mode, with the real core and the real
// engine: a page the app loads reads the signal, and the request for it carried the header.
// What the adapter can do is its own suite's; this is that the core's word reaches it.
import XCTest

@MainActor
final class PrivacySignalTests: AppTestCase {
    func testAPageTheAppLoadsIsToldTheSignal() {
        // The suite's page says what its script read of the signal, first and after trying
        // to make it say otherwise.
        let app = launch(start: site.url("/gpc"))
        XCTAssertTrue(pageShows("gpc=true then true", in: app), "the page read the signal, and could not change it")
        // And the request the app began for it said so to the server.
        XCTAssertEqual(site.headers(of: "/gpc").first?["sec-gpc"], "1", "\(site.headers(of: "/gpc"))")
    }
}

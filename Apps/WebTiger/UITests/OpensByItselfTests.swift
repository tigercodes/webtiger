// Fake mode's list (063), in the app: the core opens each address by itself, so many seconds
// apart, each as a link from another app is opened. A run that is measured with a list of
// sites open (`Tools/perf`) is an app started this way; this is the test that such an app
// opens its list, with nothing pressed or typed.
import XCTest

@MainActor
final class OpensByItselfTests: AppTestCase {
    func testTheAppOpensItsListOfAddressesByItself() {
        let app = launch(start: site.url("/"), open: [site.url("/landed"), site.url("/tall")], openEvery: 4)
        // Each in its turn, in a new tab in front.
        XCTAssertTrue(pageShows("Landed", in: app), "the first of the list")
        XCTAssertTrue(pageShows("Tall", in: app), "the second")
        #if os(macOS)
            // The window's own tab, and the two.
            expect(app.windows.element(boundBy: 0).buttons.matching(identifier: "tab"), toCount: 3, "three tabs")
        #endif
        // The site was asked for each page once: the window's own first page, and the list's.
        for path in ["/", "/landed", "/tall"] {
            XCTAssertEqual(site.requests.filter { $0 == path }.count, 1, "\(path): \(site.requests)")
        }
    }
}

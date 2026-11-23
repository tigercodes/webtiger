// The test site itself, where the suites lean on it (046): a page of bytes, a picture, is
// sent as its bytes, so that an engine that asks the site for one has a picture. Asked for by
// a page of the site, in a real web view.
import WTContract
import WTWebKit
import XCTest

@MainActor
final class TestSitePictureTests: AdapterTestCase {
    func testAPictureOfTheSiteIsAPictureToTheEngine() async {
        await create(1)
        _ = await load(1, site.url("/landed"))
        await wait(1, "the page to finish") { self.finished($0) && self.hasTitle("Landed")($0) }

        // The page asks for the site's picture, and says how large it is when it has it.
        await run(
            """
            window.__picture = "";
            const picture = new Image();
            picture.onload = () => { window.__picture = picture.naturalWidth + "x" + picture.naturalHeight; };
            picture.onerror = () => { window.__picture = "no picture"; };
            picture.src = "/icon.png";
            """,
            in: 1
        )
        var said = ""
        for _ in 0 ..< 250 where said.isEmpty {
            said = await run("window.__picture", in: 1) as? String ?? ""
            if said.isEmpty {
                try? await Task.sleep(for: .milliseconds(20))
            }
        }
        XCTAssertEqual(said, "32x32", "the site's picture is a picture, 32 pixels each way")
        XCTAssertTrue(site.requests.contains("/icon.png"), "and the site was asked for it: \(site.requests)")
    }
}

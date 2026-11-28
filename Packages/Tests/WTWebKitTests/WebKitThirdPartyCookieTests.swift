// Third-party cookies (070), against real WebKit and the suite's own pages. A site that is in
// another site's page (a frame, a picture) is not sent the cookies it has; what it sets
// there is not kept; and its script in a frame reads none and can write none. Visited as
// itself it has what it had. WebKit does this by itself: the suite says so, so that a WebKit
// that stopped would be noticed. The test site answers by two names, 127.0.0.1 and
// localhost: two sites, to an engine.
import WebKit
import WTContract
@testable import WTWebKit
import XCTest

@MainActor
final class WebKitThirdPartyCookieTests: AdapterTestCase {
    /// What `/kept` says of a site that kept "ada" (`/keep?v=ada`), asked as itself: what
    /// it was asked with, what its script reads and can write, and what its storage has.
    private static let itself = "asked ada/ada, reads anywhere+who, writes yes, stored ada"
    /// What it says of a site that has kept nothing.
    private static let nobody = "asked none/none, reads none, writes yes, stored none"
    /// What it says in a frame of another site's page: asked with nothing, nothing to
    /// read, nothing written.
    private static let shutOut = "asked none/none, reads none, writes no, stored none"

    /// The address of a path of the site by its other name: another site, to an engine.
    private func other(_ path: String) -> String {
        "http://localhost:\(site.port)\(path)"
    }

    /// The cookies each request for a path came with, in the order asked: none is empty.
    private func cookies(at path: String) -> [Set<String>] {
        site.headers(of: path).map { said in
            Set((said["cookie"] ?? "").components(separatedBy: "; ").filter { !$0.isEmpty })
        }
    }

    /// A web view in a login set, at an address, waited for by the title its page comes to.
    private func show(_ id: UInt64, in loginSet: UUID, _ url: String, titled title: String, ephemeral: Bool = false) async {
        _ = await create(id, loginSet: loginSet, ephemeral: ephemeral)
        _ = await load(id, url)
        await wait(id, "the page titled \(title)") { self.hasTitle(title)($0) }
    }

    func testASiteInAnotherSitesPageIsNotSentItsCookiesAndItsScriptHasNone() async {
        let set = newLoginSet()
        await show(1, in: set, other("/keep?v=ada"), titled: "Kept")
        // As itself, it is asked with what it kept, and its script reads it.
        await show(2, in: set, other("/kept"), titled: Self.itself)

        // In a page of another site: its frame's page and its picture are asked for with
        // nothing, and its script in the frame has nothing and can keep nothing.
        await show(3, in: set, site.url("/holds-another-site"), titled: "frame: " + Self.shutOut)
        XCTAssertEqual(cookies(at: "/kept"), [["who=ada", "anywhere=ada"], []])
        XCTAssertEqual(cookies(at: "/pixel"), [[]])

        // And it has lost nothing: as itself again, it has what it had.
        await show(4, in: set, other("/kept"), titled: Self.itself)
        XCTAssertEqual(cookies(at: "/kept").last, ["who=ada", "anywhere=ada"])
    }

    func testWhatASiteSetsInAFrameOfAnotherSitesPageIsNotKept() async {
        let set = newLoginSet()
        // The frame's page is one that keeps a name every way it can, and says when it has.
        await show(1, in: set, site.url("/holds-another-site?frame=keep"), titled: "frame: kept")
        XCTAssertEqual(site.requests.count(where: { $0 == "/keep" }), 1, "the frame's page was asked for")
        // As itself, the site has nothing.
        await show(2, in: set, other("/kept"), titled: Self.nobody)
        XCTAssertEqual(cookies(at: "/kept"), [[]])
    }

    func testWhatASiteSetsWithAPictureInAnotherSitesPageIsNotKept() async {
        let set = newLoginSet()
        // The picture is no picture: its answer sets the cookies. The page says what its
        // frame said once the picture has been answered.
        await show(1, in: set, site.url("/holds-another-site?picture=keep"), titled: "frame: " + Self.shutOut)
        XCTAssertEqual(site.requests.count(where: { $0 == "/keep" }), 1, "the picture was asked for")
        await show(2, in: set, other("/kept"), titled: Self.nobody)
        XCTAssertEqual(cookies(at: "/kept").last, [])
    }

    /// The other way round, so that these tests are known to see a cookie where there is
    /// one: a frame and a picture of the page's own site have that site's cookies.
    func testAFrameAndAPictureOfThePagesOwnSiteHaveItsCookies() async {
        let set = newLoginSet()
        await show(1, in: set, site.url("/keep?v=ada"), titled: "Kept")
        await show(2, in: set, site.url("/holds-another-site?at=127.0.0.1"), titled: "frame: " + Self.itself)
        XCTAssertEqual(cookies(at: "/kept"), [["who=ada", "anywhere=ada"]])
        XCTAssertEqual(cookies(at: "/pixel"), [["who=ada", "anywhere=ada"]])
    }

    #if os(macOS)
        /// A site a person has used as itself (a click in its page) is, to WebKit, one they
        /// have to do with, and WebKit has had rules that let such a site more than one a
        /// person only passed through. It is shut out of another site's page all the same:
        /// the story of 070 is a site a person is signed in to.
        func testASiteAPersonHasUsedIsShutOutOfAnotherSitesPageAllTheSame() async throws {
            let set = newLoginSet()
            await show(1, in: set, other("/keep?v=ada"), titled: "Kept")
            let window = try window(for: 1)
            defer { window.close() }
            try await click("use", in: 1, of: window, holding: [])
            await wait(1, "the click to reach the site's page") { self.hasTitle("Used")($0) }

            await show(2, in: set, site.url("/holds-another-site"), titled: "frame: " + Self.shutOut)
            XCTAssertEqual(cookies(at: "/kept"), [[]])
            XCTAssertEqual(cookies(at: "/pixel"), [[]])
            // And in the very web view the person used it in.
            _ = await load(1, site.url("/holds-another-site"))
            await wait(1, "the frame to say what it has") { self.hasTitle("frame: " + Self.shutOut)($0) }
            XCTAssertEqual(cookies(at: "/kept"), [[], []])
        }
    #endif

    /// A private window's login set (037), which is kept in memory: the same.
    func testInAPrivateWindowsLoginSetTheSame() async {
        let set = UUID()
        await show(1, in: set, other("/keep?v=ada"), titled: "Kept", ephemeral: true)
        await show(2, in: set, other("/kept"), titled: Self.itself, ephemeral: true)
        await show(3, in: set, site.url("/holds-another-site"), titled: "frame: " + Self.shutOut, ephemeral: true)
        XCTAssertEqual(cookies(at: "/kept"), [["who=ada", "anywhere=ada"], []])
        XCTAssertEqual(cookies(at: "/pixel"), [[]])
    }
}

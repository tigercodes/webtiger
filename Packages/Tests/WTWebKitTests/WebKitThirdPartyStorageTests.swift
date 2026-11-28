// A third party's storage (071), against real WebKit and the suite's own pages. WebKit keeps
// it apart by itself: a site in a frame of another site's page finds nothing of what it keeps
// as itself, in any store, and what it keeps there is not there when it is visited as itself.
// Nothing of the adapter's does this: the suite says so, store by store, so that a WebKit
// that stopped would be noticed. (On Android the WebView does not, and the adapter does:
// SystemWebViewThirdPartyStorageTest.) The test site answers by two names, 127.0.0.1 and
// localhost: two sites, to an engine.
import WebKit
import WTContract
@testable import WTWebKit
import XCTest

@MainActor
final class WebKitThirdPartyStorageTests: AdapterTestCase {
    // What `/stores` says after the stores a name is kept in. An app's web view on an
    // iPhone is offered no worker that serves pages; on the Mac it is.
    #if os(macOS)
        private static let theRest = "files given, worker offered, shared offered"
    #else
        private static let theRest = "files given, worker absent, shared offered"
    #endif
    /// What `/stores` says of a site that has kept nothing, as itself.
    private static let nothingKept = "local none, session none, database none, cache none, " + theRest
    /// What it says of a site that kept "ada" in every store, as itself, in another web view.
    private static let adaKept = "local ada, session none, database ada, cache ada, " + theRest
    /// What `/storage-works` says of storage that works.
    private static let works = "local works, session works"

    /// The address of a path of the site by its other name: another site, to an engine.
    private func other(_ path: String) -> String {
        "http://localhost:\(site.port)\(path)"
    }

    /// Sends a web view to an address, and gives the title its page comes to that begins
    /// as told: what the page said.
    private func said(_ id: UInt64, at url: String, beginning: String) async -> String {
        _ = await load(id, url)
        var heard = ""
        await wait(id, "a page that says \(beginning)…") { reports in
            for case let .titleChanged(changed) in reports where changed.title.hasPrefix(beginning) {
                heard = changed.title
            }
            return !heard.isEmpty
        }
        return heard
    }

    func testEveryStoreOfASiteInAnotherSitesPageIsApartFromItsOwn() async {
        let set = newLoginSet()
        // As itself, the site keeps a name in every store it has.
        _ = await create(1, loginSet: set)
        let first = await said(1, at: other("/stores?v=ada"), beginning: "local ")
        XCTAssertEqual(first, Self.nothingKept)

        // In another site's page, in the very web view it was kept in: it finds the name in
        // no store.
        let framed = await said(1, at: site.url("/holds-another-site?frame=stores"), beginning: "frame: local ")
        XCTAssertFalse(framed.contains("ada"), framed)
        XCTAssertEqual(framed, "frame: " + Self.nothingKept, "what WebKit gives a third party, store by store")

        // What it kept there ("eve") is nowhere: as itself it has the name it kept, in each.
        _ = await create(2, loginSet: set)
        let after = await said(2, at: other("/stores"), beginning: "local ")
        XCTAssertFalse(after.contains("eve"), after)
        XCTAssertEqual(after, Self.adaKept)
    }

    /// The two ways round a script that keeps storage apart (071 §8: an empty frame of
    /// the site's own making, and a worker) are no ways round WebKit, which keeps the
    /// storage itself apart: through either, a site in another site's page finds nothing.
    func testNeitherAnEmptyFrameNorAWorkerReachesTheSitesOwnStorage() async {
        let set = newLoginSet()
        _ = await create(1, loginSet: set)
        let first = await said(1, at: other("/stores?v=ada"), beginning: "local ")
        XCTAssertEqual(first, Self.nothingKept)
        // As itself, a worker of the site's has what the site kept: the page is right
        // about workers.
        _ = await create(2, loginSet: set)
        let itself = await said(2, at: other("/kept-through-a-worker"), beginning: "worker: ")
        XCTAssertEqual(itself, "worker: ada")

        _ = await create(3, loginSet: set)
        let throughAFrame = await said(3, at: site.url("/holds-another-site?frame=kept-through-a-frame"), beginning: "frame: at once ")
        XCTAssertEqual(throughAFrame, "frame: at once none, later none")
        _ = await create(4, loginSet: set)
        let throughAWorker = await said(4, at: site.url("/holds-another-site?frame=kept-through-a-worker"), beginning: "frame: worker: ")
        XCTAssertEqual(throughAWorker, "frame: worker: none")
    }

    /// The page that tries storage as a page does is right about storage: WebKit's own
    /// passes it, as itself and in a frame of another site's page. (The same page is what
    /// Android's stand-in for a third party's storage is held to.)
    func testWebKitsStorageWorksAsThePageThatTriesItSays() async {
        let set = newLoginSet()
        _ = await create(1, loginSet: set)
        let itself = await said(1, at: other("/storage-works"), beginning: "local ")
        XCTAssertEqual(itself, Self.works)
        _ = await create(2, loginSet: set)
        let framed = await said(2, at: site.url("/holds-another-site?frame=storage-works"), beginning: "frame: local ")
        XCTAssertEqual(framed, "frame: " + Self.works)
    }
}

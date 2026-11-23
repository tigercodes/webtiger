// A page's icon, on WebKit (046 §5a), against the real engine and the test site: the adapter
// says which icons a page names, fetches the one it is asked for, and hands over a small PNG.
// The fetch is made as nobody, which is tested as a security property: it carries no cookie
// of the page's login set and keeps none it is given, it says of itself only what the page
// says, and it follows a server only where it may ask.
//
// Every fetch here is of the test's own site, in the test's own process: the engine is let
// ask this device and nothing beyond it.
import WebKit
import WTContract
@testable import WTWebKit
import XCTest

@MainActor
final class WebKitIconTests: AdapterTestCase {
    override func setUp() async throws {
        try await super.setUp()
        engine.iconFetching = .thisDevice
    }

    /// Which icons the pages of a web view have named, in order.
    private func named(_ id: UInt64) -> [WTIconNamed] {
        reports(id).compactMap {
            if case let .iconNamed(named) = $0 {
                named
            } else {
                nil
            }
        }
    }

    /// The icons handed over for a web view, in order.
    private func received(_ id: UInt64) -> [WTIconReceived] {
        reports(id).compactMap {
            if case let .iconReceived(received) = $0 {
                received
            } else {
                nil
            }
        }
    }

    /// Loads a page and waits until it has said which icons it names.
    private func open(_ id: UInt64, _ path: String, line: UInt = #line) async -> WTIconNamed {
        await load(id, site.url(path))
        await wait(id, "the icons \(path) names", line: line) { _ in self.named(id).last?.pageURL == self.site.url(path) }
        return named(id).last ?? WTIconNamed()
    }

    /// Asks for an icon, as the core does.
    @discardableResult
    private func fetch(_ id: UInt64, _ path: String, of page: String) async -> WTHostResponse.OneOf_Kind {
        var fetch = WTWebViewFetchIcon()
        fetch.webViewID = id
        fetch.url = site.url(path)
        fetch.pageURL = site.url(page)
        return await engine.answer(.webViewFetchIcon(fetch))
    }

    /// Waits until the site has been asked for a path this many times.
    private func awaitRequest(_ path: String, times: Int = 1, line: UInt = #line) async {
        for _ in 0 ..< Patience.steps where site.requests.count(where: { $0 == path }) < times {
            try? await Task.sleep(for: .milliseconds(10))
        }
        XCTAssertEqual(site.requests.count { $0 == path }, times, "the site was asked for \(path): \(site.requests)", line: line)
    }

    /// What the site was asked for, without the page every suite's first test loads to wake
    /// WebKit.
    private var asked: [String] {
        site.requests.filter { $0 != "/" }
    }

    private func address(_ path: String) -> URL {
        URL(string: site.url(path)) ?? URL(fileURLWithPath: "/")
    }

    func testAPageSaysWhichIconsItNamesAndNothingIsFetchedUnasked() async {
        await create(1)
        // An icon at an address of the site: named whole, with what the page says of it.
        let served = await open(1, "/with-served-icon")
        // (The page links to something that is no icon too, which is not named.)
        XCTAssertEqual(served.icons.map(\.url), [site.url("/icon.png")], "the address is made whole against the page's own")
        XCTAssertEqual(served.icons.map(\.rel), ["icon"])
        XCTAssertEqual(served.icons.map(\.type), ["image/png"])
        XCTAssertEqual(served.icons.map(\.sizes), ["32x32"])
        // A page that names none says so: the core then asks for the one every site may have.
        let none = await open(1, "/landed")
        XCTAssertEqual(none.icons.count, 0)
        // A picture given in the page itself is named as the page wrote it.
        let given = await open(1, "/with-icon")
        XCTAssertEqual(given.icons.count, 1)
        XCTAssertTrue(given.icons.first?.url.hasPrefix("data:image/png;base64,") == true)
        // Each page said it once. And nothing was fetched: WebKit asks for no icon, and the
        // adapter for none it was not asked for.
        XCTAssertEqual(named(1).map(\.pageURL), ["/with-served-icon", "/landed", "/with-icon"].map(site.url))
        XCTAssertEqual(site.requests.filter { $0 == "/icon.png" || $0 == "/favicon.ico" }, [])
        XCTAssertEqual(received(1).count, 0)
    }

    func testTheIconAskedForIsFetchedAsNobodyAndHandedOverAsAPNG() async throws {
        // Somebody is signed in, in the page's login set.
        await create(1)
        await load(1, site.url("/set?v=ada"))
        await wait(1, "signed in", until: hasTitle("Set"))
        await load(1, site.url("/whoami"))
        await wait(1, "the page is asked for as ada", until: hasTitle("who=ada"))
        _ = await open(1, "/with-served-icon")

        // The icon is asked for, and comes as the page's: a PNG, the size it was served.
        let answer = await fetch(1, "/icon.png", of: "/with-served-icon")
        guard case .ack = answer else { return XCTFail("\(answer)") }
        await wait(1, "the icon") { _ in !self.received(1).isEmpty }
        let icon = try XCTUnwrap(received(1).last)
        XCTAssertEqual(icon.pageURL, site.url("/with-served-icon"))
        XCTAssertEqual(IconPicture.kind(of: icon.png), .png)
        XCTAssertEqual(TestPictures.size(of: icon.png), [32, 32])

        // What the fetch said of itself: no cookie, though the page has one; nowhere it came
        // from; nobody; and to be what the page says it is, so that it tells the site nothing
        // the page's own requests did not.
        let said = try XCTUnwrap(site.headers(of: "/icon.png").last)
        XCTAssertNil(said["cookie"], "the fetch carried the page's cookie")
        XCTAssertNil(said["referer"])
        XCTAssertNil(said["origin"])
        XCTAssertNil(said["authorization"])
        let agent = await run("navigator.userAgent", in: 1) as? String
        XCTAssertEqual(said["user-agent"], agent)
        XCTAssertFalse((agent ?? "").isEmpty)

        // The site sends whoever asks on to an address named after their cookie: the fetch is
        // sent on as nobody. The page itself, asking for the same, is sent on as ada.
        await fetch(1, "/icon-by-cookie", of: "/with-served-icon")
        await awaitRequest("/icon-of-none")
        XCTAssertEqual(site.requests.filter { $0.hasPrefix("/icon-of-") }, ["/icon-of-none"])
        await load(1, site.url("/icon-by-cookie"))
        await awaitRequest("/icon-of-ada")
        XCTAssertEqual(received(1).count, 1, "what is at neither address is no picture, and nothing was handed over for it")
    }

    func testWhatAFetchIsGivenToKeepIsNotKept() async {
        // A server sets a cookie on the way to somewhere else: it is not sent along.
        let file = await IconFetch.picture(at: address("/signs-in-and-sends?v=mallory"), agent: "", reach: .thisDevice)
        XCTAssertNil(file, "words are no picture")
        XCTAssertEqual(asked, ["/signs-in-and-sends", "/whose-file"], "the fetch followed the server within the site")
        XCTAssertEqual(site.headers(of: "/whose-file").compactMap { $0["cookie"] }, [], "a cookie set on the way was sent on")

        // A server signs the fetch in: the next fetch is nobody still.
        let page = await IconFetch.picture(at: address("/set?v=mallory"), agent: "", reach: .thisDevice)
        XCTAssertNil(page)
        _ = await IconFetch.picture(at: address("/icon-by-cookie"), agent: "", reach: .thisDevice)
        XCTAssertEqual(site.requests.filter { $0.hasPrefix("/icon-of-") }, ["/icon-of-none"])
        XCTAssertEqual(site.headers(of: "/icon-by-cookie").compactMap { $0["cookie"] }, [])
        // Nor did it go to where this process keeps cookies for whatever else asks.
        let kept = HTTPCookieStorage.shared.cookies(for: address("/")) ?? []
        XCTAssertEqual(kept.filter { $0.name == "who" }.map(\.value), [])

        // A web view of another login set, signed in by nobody, is nobody: the fetch signed
        // nobody in anywhere.
        await create(1)
        await load(1, site.url("/whoami"))
        await wait(1, "nobody is signed in", until: hasTitle("who=none"))
    }

    func testAFetchFollowsAServerOnlyWhereItMayAskAndTakesOnlyAPicture() async throws {
        // A picture that has moved within the site is followed to.
        let moved = await IconFetch.picture(at: address("/icon-moved"), agent: "", reach: .thisDevice)
        XCTAssertEqual(TestPictures.size(of: moved), [32, 32])
        XCTAssertEqual(asked, ["/icon-moved", "/icon.png"])
        // A server that sends the fetch on to an address that names somebody is not followed:
        // the picture there was not asked for again.
        let somebody = await IconFetch.picture(at: address("/icon-as-somebody?port=\(site.port)"), agent: "", reach: .thisDevice)
        XCTAssertNil(somebody)
        XCTAssertEqual(asked, ["/icon-moved", "/icon.png", "/icon-as-somebody"])

        // Words, and a page that is not there, are no picture.
        let words = await IconFetch.picture(at: address("/landed"), agent: "", reach: .thisDevice)
        XCTAssertNil(words)
        let missing = await IconFetch.picture(at: address("/favicon.ico"), agent: "", reach: .thisDevice)
        XCTAssertNil(missing)

        // No more bytes are taken than an icon has: the picture is that many exactly, and
        // one fewer allowed is too few.
        let served = try XCTUnwrap(TestSitePages.all.first { $0.path == "/icon.png" })
        let bytes = try XCTUnwrap(Data(base64Encoded: served.bytes)).count
        let whole = await IconFetch.picture(at: address("/icon.png"), agent: "", reach: .thisDevice, most: bytes)
        XCTAssertEqual(TestPictures.size(of: whole), [32, 32])
        let cut = await IconFetch.picture(at: address("/icon.png"), agent: "", reach: .thisDevice, most: bytes - 1)
        XCTAssertNil(cut)

        // An address that may not be asked is not asked at all.
        let before = site.requests.count
        let never = await IconFetch.picture(at: address("/icon.png"), agent: "", reach: .never)
        XCTAssertNil(never)
        let named = URL(string: "http://ada:secret@127.0.0.1:\(site.port)/icon.png") ?? address("/")
        let asSomebody = await IconFetch.picture(at: named, agent: "", reach: .thisDevice)
        XCTAssertNil(asSomebody)
        XCTAssertEqual(site.requests.count, before)
    }

    func testAnEngineNotLetFetchSaysNothingOfIconsAndFetchesNone() async {
        // An engine is not let, unless it is made otherwise.
        XCTAssertEqual(WebKitEngine().iconFetching, .never)
        engine.iconFetching = .never
        await create(1)
        await load(1, site.url("/with-served-icon"))
        await wait(1, "the page") { self.finished($0) }
        await wait(1, "its title", until: hasTitle("With served icon"))
        // The page has answered a question since it loaded: it was asked none about its icons.
        let title = await run("document.title", in: 1) as? String
        XCTAssertEqual(title, "With served icon")
        XCTAssertEqual(named(1).count, 0)

        // Asked for an icon, it answers, and fetches nothing.
        let answer = await fetch(1, "/icon.png", of: "/with-served-icon")
        guard case .ack = answer else { return XCTFail("\(answer)") }
        try? await Task.sleep(for: .milliseconds(300))
        XCTAssertEqual(site.requests.filter { $0 == "/icon.png" }, [])
        XCTAssertEqual(received(1).count, 0)

        // A web view there is none of has no icon to fetch.
        guard case let .error(error) = await fetch(99, "/icon.png", of: "/with-served-icon") else { return XCTFail("no error") }
        XCTAssertEqual(error.code, "webview.unknown")
    }

    func testAWebViewThatHasGoneIsToldOfNoIcon() async {
        await create(1)
        _ = await open(1, "/with-served-icon")
        // Asked for, and the web view destroyed before the picture has come.
        await fetch(1, "/icon.png", of: "/with-served-icon")
        await destroy(1)
        await awaitRequest("/icon.png")
        try? await Task.sleep(for: .milliseconds(300))
        XCTAssertEqual(received(1).count, 0)
    }
}

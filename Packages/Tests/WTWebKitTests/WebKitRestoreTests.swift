// What WebKit gives a tab that is put to sleep and woken (000 §5.7, spike Q5): the web view is
// released, and a new one is handed the old one's `interactionState`. Sleeping tabs are W3;
// these tests pin the platform behaviour that design rests on, so a WebKit that changes it
// fails here first.
import WebKit
import XCTest

@MainActor
final class WebKitRestoreTests: XCTestCase {
    private let site = TestSite()
    private let loginSet = UUID()

    override func setUp() async throws {
        XCTAssertNotEqual(site.port, 0, "the loopback site is listening")
    }

    override func tearDown() async throws {
        for _ in 0 ..< 100 {
            if await (try? WKWebsiteDataStore.remove(forIdentifier: loginSet)) != nil {
                break
            }
            try? await Task.sleep(for: .milliseconds(50))
        }
    }

    private func makeWebView() -> WKWebView {
        let configuration = WKWebViewConfiguration()
        configuration.websiteDataStore = WKWebsiteDataStore(forIdentifier: loginSet)
        return WKWebView(frame: CGRect(x: 0, y: 0, width: 800, height: 600), configuration: configuration)
    }

    /// Waits until the web view's *document* is `path` and has finished loading.
    ///
    /// The web view's own `url` and `isLoading` are not enough. `url` changes when a load is
    /// asked for and when a state is restored, a moment before `isLoading` does, so the two can
    /// read "there, and idle" while the old document (or none) is still the one on show. A slow
    /// machine finds that moment: GitHub's runner did. So the document itself is asked.
    private func wait(for webView: WKWebView, toShow path: String, file: StaticString = #filePath, line: UInt = #line) async {
        let wanted = site.url(path)
        for _ in 0 ..< 1500 {
            if !webView.isLoading, webView.url?.absoluteString == wanted {
                let shown = try? await webView.evaluateJavaScript("document.readyState + ' ' + location.href") as? String
                if shown == "complete \(wanted)" {
                    return
                }
            }
            try? await Task.sleep(for: .milliseconds(10))
        }
        XCTFail("the web view shows \(webView.url?.path ?? "nothing"), not \(path)", file: file, line: line)
    }

    /// The page's title once WebKit has it. It arrives a moment after the load ends, so reading
    /// it at once is a race a slower machine loses.
    private func title(of webView: WKWebView) async -> String {
        for _ in 0 ..< 1000 {
            if let title = webView.title, !title.isEmpty {
                return title
            }
            try? await Task.sleep(for: .milliseconds(10))
        }
        return webView.title ?? ""
    }

    private func load(_ webView: WKWebView, _ path: String) async {
        guard let url = URL(string: site.url(path)) else { return XCTFail("a bad test address") }
        webView.load(URLRequest(url: url))
        await wait(for: webView, toShow: path)
    }

    func testASleepingTabsStateIsSmallAndRestoresItsHistoryInANewWebView() async throws {
        var asleep: Any?
        do {
            let awake = makeWebView()
            await load(awake, "/")
            await load(awake, "/landed")
            await load(awake, "/tall")
            asleep = awake.interactionState
            // `awake` is released here: the tab has no web view while it sleeps.
        }
        // The state can be kept: it is data, and small.
        let data = try XCTUnwrap(asleep as? Data, "interactionState is Data, so it can be written to the store")
        XCTAssertLessThan(data.count, 250_000, "000 §7: a sleeping tab is at most 250 KB on disk")
        print("WebKit interactionState for three pages of history: \(data.count) bytes")

        let woken = makeWebView()
        woken.interactionState = data
        await wait(for: woken, toShow: "/tall")
        let restored = await title(of: woken)
        XCTAssertEqual(restored, "Tall")
        XCTAssertTrue(woken.canGoBack, "the history came back with it")
        XCTAssertFalse(woken.canGoForward)
        XCTAssertEqual(woken.backForwardList.backList.map(\.url.absoluteString), [site.url("/"), site.url("/landed")])

        woken.goBack()
        await wait(for: woken, toShow: "/landed")
        XCTAssertTrue(woken.canGoForward)
    }

    func testAWokenTabIsWhereItWasScrolledTo() async throws {
        let awake = makeWebView()
        await load(awake, "/tall")
        _ = try await awake.evaluateJavaScript("window.scrollTo(0, 2400); window.scrollY")
        // WebKit records the scroll position in the history item a moment after the scroll.
        try await Task.sleep(for: .milliseconds(600))
        let asleep = try XCTUnwrap(awake.interactionState as? Data)

        let woken = makeWebView()
        woken.interactionState = asleep
        await wait(for: woken, toShow: "/tall")
        var scrolled = 0.0
        for _ in 0 ..< 300 {
            scrolled = await (try? woken.evaluateJavaScript("window.scrollY") as? Double) ?? 0
            if scrolled > 0 {
                break
            }
            try await Task.sleep(for: .milliseconds(10))
        }
        XCTAssertEqual(scrolled, 2400, accuracy: 1, "a woken tab is where it was left")
    }

    func testAWokenTabKeepsItsLoginSet() async throws {
        let awake = makeWebView()
        await load(awake, "/set?v=ada")
        let asleep = try XCTUnwrap(awake.interactionState as? Data)
        let woken = makeWebView()
        woken.interactionState = asleep
        await wait(for: woken, toShow: "/set?v=ada")
        await load(woken, "/whoami")
        let answer = await title(of: woken)
        XCTAssertEqual(answer, "who=ada", "waking a tab does not sign it out")
    }
}

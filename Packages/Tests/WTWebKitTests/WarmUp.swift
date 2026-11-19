// WebKit's own processes, started once before the first test that needs them.
import WebKit

/// The first web view a process makes waits for WebKit's networking and web content processes
/// to start. On a slow machine (a simulator on a shared runner) that has taken minutes: longer
/// than any test should wait for a page. The first tests of a run then failed with a load
/// that had started and nothing more, one after the other, until WebKit was there.
///
/// So WebKit is started here, once, before the first test: a page is loaded from the suite's
/// own loopback site, the way every test loads one, and waited for. A load given to a WebKit
/// that was not there yet has been seen never to end, so the page is asked for again every
/// half minute. Eight minutes, because a test may take ten (`Tools/test.sh`): a runner that
/// slow is a slow runner, and what it says about WebTiger is nothing.
@MainActor
enum WebKitWarmUp {
    private static var done = false

    /// Whether WebKit loaded the page. When it did not, no test of the suite can pass, and
    /// each says so at once instead of waiting for its own page.
    private(set) static var started = false

    /// Loads `url`, a page of the test site whose title is `title`, and waits for it.
    static func run(loading url: String, titled title: String) async {
        guard !done else { return }
        done = true
        let configuration = WKWebViewConfiguration()
        // A store of its own, kept in memory: the warm-up leaves nothing behind.
        configuration.websiteDataStore = .nonPersistent()
        let webView = WKWebView(frame: .zero, configuration: configuration)
        guard let page = URL(string: url) else { return }
        let clock = ContinuousClock()
        let end = clock.now + .seconds(480)
        var askedAgainAt = clock.now
        while webView.title != title, clock.now < end {
            if clock.now >= askedAgainAt {
                webView.load(URLRequest(url: page))
                askedAgainAt = clock.now + .seconds(30)
            }
            try? await Task.sleep(for: .milliseconds(20))
        }
        started = webView.title == title
    }
}

// WebKit's own processes, started once before the first test that needs them.
import WebKit

/// The first web view a process makes waits for WebKit's networking and web content processes
/// to start. On a slow machine (a simulator on a shared runner) that has taken longer than any
/// test should wait for a page: the first test of a run then failed with nothing reported at
/// all. So they are started here, once, and given a minute.
@MainActor
enum WebKitWarmUp {
    private static var done = false

    static func run() async {
        guard !done else { return }
        done = true
        let configuration = WKWebViewConfiguration()
        // A store of its own, kept in memory: the warm-up leaves nothing behind.
        configuration.websiteDataStore = .nonPersistent()
        let webView = WKWebView(frame: .zero, configuration: configuration)
        webView.loadHTMLString("<title>warm</title>", baseURL: nil)
        for _ in 0 ..< 6000 where webView.title != "warm" {
            try? await Task.sleep(for: .milliseconds(10))
        }
    }
}

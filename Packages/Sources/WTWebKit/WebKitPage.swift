// One web view of the WebKit adapter's (001 §5): the delegates that turn what WebKit says of
// it into the contract's events. It decides nothing: whether a navigation may happen is the
// core's answer, asked through the engine.
import WebKit
import WTContract

/// One web view and the delegates that turn its callbacks into the contract's events
/// (001 §5 has the mapping).
@MainActor
final class Page: NSObject, WKNavigationDelegate, WKUIDelegate {
    let id: UInt64
    let webView: WKWebView
    /// The login set the web view's pages run in, and whether it is a private one.
    let loginSet: UUID
    let isPrivate: Bool
    private(set) weak var engine: WebKitEngine?
    private var observations: [NSKeyValueObservation] = []
    /// A main-frame load has started and not yet committed or failed.
    private(set) var provisional = false
    /// The address last reported as the page on screen, by a commit or an address change.
    private var reportedURL: URL?
    /// A navigation was refused on the core's verdict while a load had started and not yet
    /// committed. If that load now ends as interrupted, the refusal was of where it was
    /// redirected to, and the load's end is the refusal's.
    var refusedWhileProvisional = false

    init(id: UInt64, webView: WKWebView, loginSet: UUID, isPrivate: Bool, engine: WebKitEngine) {
        self.id = id
        self.webView = webView
        self.loginSet = loginSet
        self.isPrivate = isPrivate
        self.engine = engine
        super.init()
        webView.navigationDelegate = self
        webView.uiDelegate = self
        observe()
    }

    /// Finds text in the page and shows the match, or with empty text clears what a find
    /// showed (010). WebKit's own find: it ignores case, wraps round, shows a match by
    /// selecting it, and says whether there was one. It does not count them.
    func find(_ text: String, backwards: Bool) {
        guard !text.isEmpty else {
            // What a find showed is the page's selection: letting go of it is how the marks
            // go. Asked in a world of the app's own, where no script of the page's runs.
            webView.evaluateJavaScript("getSelection().removeAllRanges()", in: nil, in: .defaultClient) { _ in }
            return
        }
        let configuration = WKFindConfiguration()
        configuration.backwards = backwards
        configuration.caseSensitive = false
        configuration.wraps = true
        webView.find(text, configuration: configuration) { [weak self] result in
            MainActor.assumeIsolated {
                guard let self else { return }
                var found = WTFindResult()
                found.found = result.matchFound
                self.engine?.report(self.id, .findResult(found))
            }
        }
    }

    /// Makes the web view what a state says (051): the list it goes back and forward through,
    /// at the page it was at, which WebKit then loads as it loads any page. False when WebKit
    /// would not take the state (it is not WebKit's, or is damaged, or says no page): the web
    /// view is as it was. WebKit says nothing either way; a state it took is seen by the place
    /// in its list, which is then another.
    func restore(_ state: Data) -> Bool {
        let before = webView.backForwardList.currentItem
        webView.interactionState = state
        let now = webView.backForwardList.currentItem
        return now != nil && now !== before
    }

    func close() {
        observations.removeAll()
        webView.navigationDelegate = nil
        webView.uiDelegate = nil
        webView.stopLoading()
        webView.removeFromSuperview()
    }

    private func observe() {
        // WebKit changes these on the main thread; the handlers say so to the compiler.
        observations = [
            webView.observe(\.title) { [weak self] _, _ in
                MainActor.assumeIsolated { self?.titleChanged() }
            },
            webView.observe(\.estimatedProgress) { [weak self] _, _ in
                MainActor.assumeIsolated { self?.progressChanged() }
            },
            webView.observe(\.canGoBack) { [weak self] _, _ in
                MainActor.assumeIsolated { self?.historyChanged() }
            },
            webView.observe(\.canGoForward) { [weak self] _, _ in
                MainActor.assumeIsolated { self?.historyChanged() }
            },
            webView.observe(\.url) { [weak self] _, _ in
                MainActor.assumeIsolated { self?.urlChanged() }
            },
        ]
    }

    private func titleChanged() {
        var changed = WTTitleChanged()
        changed.title = webView.title ?? ""
        engine?.report(id, .titleChanged(changed))
    }

    private func progressChanged() {
        var changed = WTProgressChanged()
        changed.fraction = webView.estimatedProgress
        engine?.report(id, .progressChanged(changed))
    }

    private func historyChanged() {
        var changed = WTHistoryChanged()
        changed.canGoBack = webView.canGoBack
        changed.canGoForward = webView.canGoForward
        engine?.report(id, .historyChanged(changed))
    }

    /// WebKit's `url` changed. It changes for many reasons, and only one of them is the page
    /// on screen changing its own address (a fragment, or the History API):
    ///
    /// - `load()` sets it to the address asked for, at once, and a state that is restored
    ///   (051) to the page the state is at;
    /// - a load that starts, or is redirected, sets it to where the load is heading;
    /// - a load that fails sets it back.
    ///
    /// None of those is reported from here: an address that has not committed is not the
    /// page's address, and reporting it would let a page put an address in the field that it
    /// has not become. WebKit is loading in every one of those cases, so a change is reported
    /// only while it is not. A page that changes its address while it is still loading is
    /// caught up with when the load ends (`reconcileAddress`).
    private func urlChanged() {
        guard !provisional, !webView.isLoading else { return }
        reconcileAddress()
    }

    /// Reports the page's address if it is not the one last reported.
    private func reconcileAddress() {
        guard let url = webView.url, url != reportedURL else { return }
        reportedURL = url
        var changed = WTAddressChanged()
        changed.url = url.absoluteString
        engine?.report(id, .addressChanged(changed))
    }

    // MARK: WKNavigationDelegate

    /// What WebKit cannot show, and what says it is to be saved, is a download (013). WebKit
    /// ends the load it was ("interrupted"), which the adapter reports as a load that stopped:
    /// the page on screen stays.
    func webView(_: WKWebView, decidePolicyFor response: WKNavigationResponse) async -> WKNavigationResponsePolicy {
        let disposition = (response.response as? HTTPURLResponse)?.value(forHTTPHeaderField: "Content-Disposition") ?? ""
        let attachment = disposition.trimmingCharacters(in: .whitespaces).lowercased().hasPrefix("attachment")
        return attachment || !response.canShowMIMEType ? .download : .allow
    }

    func webView(_: WKWebView, navigationResponse _: WKNavigationResponse, didBecome download: WKDownload) {
        engine?.downloads.adopt(download, from: id)
    }

    func webView(_: WKWebView, navigationAction _: WKNavigationAction, didBecome download: WKDownload) {
        engine?.downloads.adopt(download, from: id)
    }

    func webView(_ webView: WKWebView, didStartProvisionalNavigation _: WKNavigation?) {
        provisional = true
        engine?.certificates.loadBegan(in: id)
        var started = WTNavigationStarted()
        started.url = webView.url?.absoluteString ?? ""
        engine?.report(id, .navigationStarted(started))
    }

    func webView(_ webView: WKWebView, didCommit _: WKNavigation?) {
        provisional = false
        refusedWhileProvisional = false
        engine?.certificates.loadBegan(in: id)
        reportedURL = webView.url
        var committed = WTNavigationCommitted()
        committed.url = webView.url?.absoluteString ?? ""
        engine?.report(id, .navigationCommitted(committed))
        historyChanged()
    }

    func webView(_ webView: WKWebView, didFinish _: WKNavigation?) {
        reconcileAddress()
        engine?.report(id, .navigationFinished(WTNavigationFinished()))
        engine?.icons.read(webView, id: id)
    }

    func webView(_: WKWebView, didFailProvisionalNavigation _: WKNavigation?, withError error: any Error) {
        provisional = false
        // "Frame load interrupted" after a refusal: the load was redirected to the address
        // that was refused, and ends with it.
        let interrupted = (error as NSError).domain == "WebKitErrorDomain" && (error as NSError).code == 102
        failed(error, refused: refusedWhileProvisional && interrupted, ownLoad: true)
        refusedWhileProvisional = false
    }

    func webView(_: WKWebView, didFail _: WKNavigation?, withError error: any Error) {
        failed(error)
    }

    func webViewWebContentProcessDidTerminate(_: WKWebView) {
        provisional = false
        engine?.report(id, .rendererGone(WTRendererGone()))
    }

    /// `ownLoad`: the page's own load, which never committed. Only then can the certificate it
    /// failed for be one a person is asked about (026).
    private func failed(_ error: any Error, refused: Bool = false, ownLoad: Bool = false) {
        let error = error as NSError
        let url = error.userInfo[NSURLErrorFailingURLErrorKey] as? URL
        var failed = WTNavigationFailed()
        failed.code = WebKitEngine.code(for: error)
        failed.url = url?.absoluteString ?? ""
        failed.refused = refused
        failed.certificateHeld = ownLoad && holdsCertificate(refusedWith: failed.code, at: url)
        engine?.report(id, .navigationFailed(failed))
    }

    // MARK: WKUIDelegate

    /// A page asked for a new window: a link with a target, or `window.open`. The engine makes
    /// the web view and reports it; WebKit loads the address in it. Its size and place, when
    /// the page asked for any, are not followed: every window a page opens is a tab.
    func webView(
        _: WKWebView,
        createWebViewWith configuration: WKWebViewConfiguration,
        for action: WKNavigationAction,
        windowFeatures _: WKWindowFeatures
    ) -> WKWebView? {
        engine?.newWindow(askedBy: self, configuration: configuration, url: action.request.url)
    }

    /// The page asked for its own window to be closed (`window.close()`). Whether it may is
    /// the core's answer: it closes a tab a page opened, and no other.
    func webViewDidClose(_: WKWebView) {
        engine?.report(id, .closeRequested(WTCloseRequested()))
    }
}

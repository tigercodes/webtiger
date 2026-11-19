// The WebKit engine adapter (000 §4.4, 001 §5): WKWebView behind the WebViews host service.
//
// The adapter creates, loads and destroys web views when the core asks, and reports what each
// one does. It decides nothing: whether a navigation may happen is the core's answer
// (`navigationVerdict`), and what the address field shows is the core's snapshot.
//
// One web view is made unasked: the one a page asks for as a new window, which WebKit must be
// handed at once (006). The adapter makes it, names it and reports it; what becomes of it is
// the core's to say.
public import WebKit
public import WTContract
public import WTCoreClient

@MainActor
public final class WebKitEngine: HostServices {
    private var pages: [UInt64: Page] = [:]
    private var core: (any CoreClient)?
    /// The data stores of the login sets used so far, held here and not left to the web views.
    /// A sign-in for the session (a cookie with no expiry) lives in its store's memory and goes
    /// with the store: a login set whose last tab sleeps must still have its store when the
    /// tab wakes. Each is held until `deleteLoginSet`.
    private var stores: [UUID: WKWebsiteDataStore] = [:]
    /// The stores of private login sets, which exist nowhere else: one for all of a login
    /// set's web views, forgotten when the last of them is destroyed.
    private var privateStores: [UUID: WKWebsiteDataStore] = [:]
    /// How many web views this engine has made unasked, for pages that asked for a window.
    /// Each is named by the next id of the shell's own, and no id is used twice.
    private var ownWebViews: UInt64 = 0

    /// The bit that marks a web view's id as the shell's own: the core's ids never have it
    /// (`NewWindowRequested`, engine.proto).
    public static let ownID: UInt64 = 1 << 63

    public init() {}

    /// What this engine is, for `CoreConfig.engines`.
    public static var info: WTEngineInfo {
        var info = WTEngineInfo()
        info.kind = .webkit
        info.version = Bundle(for: WKWebView.self).infoDictionary?["CFBundleVersion"] as? String ?? ""
        return info
    }

    /// Gives the engine the core its reports go to. Called once, after the core has started.
    public func attach(_ core: any CoreClient) {
        self.core = core
    }

    /// The web view the core named `id`, for a window to show.
    public func webView(for id: UInt64) -> WKWebView? {
        pages[id]?.webView
    }

    /// How many web views exist. A tab that is asleep has none.
    public var webViewCount: Int {
        pages.count
    }

    public func answer(_ request: WTHostRequest.OneOf_Kind) async -> WTHostResponse.OneOf_Kind {
        switch request {
        case let .ping(ping):
            var pong = WTHostPong()
            pong.payload = ping.payload
            return .pong(pong)
        case let .webViewCreate(create):
            return self.create(create)
        case let .webViewLoad(load):
            guard let page = pages[load.webViewID] else { return failure("webview.unknown") }
            guard let url = URL(string: load.url) else { return failure("webview.bad_address") }
            page.webView.load(URLRequest(url: url))
            return .ack(WTAck())
        case let .webViewCommand(command):
            guard let page = pages[command.webViewID] else { return failure("webview.unknown") }
            switch command.command {
            case .back: page.webView.goBack()
            case .forward: page.webView.goForward()
            case .reload: page.webView.reload()
            case .stop: page.webView.stopLoading()
            case .unspecified, .UNRECOGNIZED: return failure("webview.bad_command")
            }
            return .ack(WTAck())
        case let .webViewDestroy(destroy):
            if let page = pages.removeValue(forKey: destroy.webViewID) {
                page.close()
                if page.isPrivate, !pages.values.contains(where: { $0.isPrivate && $0.loginSet == page.loginSet }) {
                    // A private login set is kept in memory and nowhere else: with its last
                    // web view gone, letting go of its store is what forgets it.
                    privateStores[page.loginSet] = nil
                }
            }
            return .ack(WTAck())
        case .openOutside:
            // The System service's, which the shell answers itself (007): opening a link
            // with another app is nothing to do with a web engine.
            return failure("host.unsupported")
        case .webViewFind:
            // Find on the page (010): not built in this adapter yet.
            return failure("host.unsupported")
        }
    }

    /// Deletes a login set: everything its pages stored, and the store itself. False, and
    /// nothing is deleted, while a web view still uses it. (Deleting a login set is the
    /// LoginSets host service's, which the contract does not have yet: until then this is for
    /// the tests that delete one.)
    public func deleteLoginSet(_ loginSet: UUID) async -> Bool {
        guard !pages.values.contains(where: { !$0.isPrivate && $0.loginSet == loginSet }) else { return false }
        do {
            // Cleared through the store while it is still held. Deleting the files alone is not
            // enough: WebKit lets a store's session go a moment after the store, and until
            // then what is in its memory (its cookies) is given to a store made anew for the
            // same login set.
            let store = stores[loginSet] ?? WKWebsiteDataStore(forIdentifier: loginSet)
            guard await Self.clear(store, within: .seconds(20)) else { return false }
            stores[loginSet] = nil
        }
        // WebKit refuses while the store is in use, and a released store lets go a moment later.
        for _ in 0 ..< 100 {
            do {
                try await WKWebsiteDataStore.remove(forIdentifier: loginSet)
                return true
            } catch {
                try? await Task.sleep(for: .milliseconds(50))
            }
        }
        return false
    }

    /// Clears everything a store holds, and says whether WebKit was done within `limit`. WebKit
    /// sets no limit of its own: on a busy machine a clear has been seen not to end for minutes,
    /// and whoever asked for a login set to be deleted must be answered.
    private static func clear(_ store: WKWebsiteDataStore, within limit: Duration) async -> Bool {
        await withCheckedContinuation { continuation in
            let answer = Answer(continuation)
            store.removeData(ofTypes: WKWebsiteDataStore.allWebsiteDataTypes(), modifiedSince: .distantPast) {
                MainActor.assumeIsolated { answer.give(true) }
            }
            Task { @MainActor in
                try? await Task.sleep(for: limit)
                answer.give(false)
            }
        }
    }

    /// An answer given once: by WebKit when it is done, or by the clock when it is not.
    @MainActor
    private final class Answer {
        private var continuation: CheckedContinuation<Bool, Never>?

        init(_ continuation: CheckedContinuation<Bool, Never>) {
            self.continuation = continuation
        }

        func give(_ done: Bool) {
            continuation?.resume(returning: done)
            continuation = nil
        }
    }

    private func create(_ create: WTWebViewCreate) -> WTHostResponse.OneOf_Kind {
        guard create.engine == .webkit else { return failure("engine.unavailable") }
        guard pages[create.webViewID] == nil else { return failure("webview.exists") }
        guard let loginSet = UUID(uuidString: create.loginSet.id) else { return failure("webview.bad_login_set") }
        let configuration = WKWebViewConfiguration()
        // One data store per login set: cookies, storage, cache, service workers and
        // permissions are all kept apart by WebKit itself.
        configuration.websiteDataStore = dataStore(for: loginSet, private: create.loginSet.ephemeral)
        // A page gets a window when somebody clicked for one, and not by asking as it loads:
        // WebKit's own blocker, which is off by default on a Mac.
        configuration.preferences.javaScriptCanOpenWindowsAutomatically = false
        pages[create.webViewID] = Page(
            id: create.webViewID,
            webView: Self.webView(configuration),
            loginSet: loginSet,
            isPrivate: create.loginSet.ephemeral,
            engine: self
        )
        return .ack(WTAck())
    }

    private static func webView(_ configuration: WKWebViewConfiguration) -> WKWebView {
        let webView = WKWebView(frame: .zero, configuration: configuration)
        #if os(macOS)
            webView.allowsMagnification = true
        #endif
        webView.allowsBackForwardNavigationGestures = true
        return webView
    }

    /// A page asked for a new window, and WebKit must be handed its web view before the call
    /// returns: that is what lets the page that asked keep its hold on the one it opened. The
    /// web view is made from the configuration WebKit gives (the asking page's, and so its
    /// login set's data store), named with an id of the shell's own, shown nowhere, and
    /// reported. The core adopts it into a tab or has it destroyed (006).
    fileprivate func newWindow(askedBy asking: Page, configuration: WKWebViewConfiguration, url: URL?) -> WKWebView {
        ownWebViews += 1
        let id = Self.ownID | ownWebViews
        let page = Page(
            id: id, webView: Self.webView(configuration), loginSet: asking.loginSet, isPrivate: asking.isPrivate, engine: self
        )
        pages[id] = page
        var asked = WTNewWindowRequested()
        asked.newWebViewID = id
        asked.url = url?.absoluteString ?? ""
        // WebKit asks only for the windows its blocker let through: those somebody clicked for.
        asked.userGesture = true
        report(asking.id, .newWindowRequested(asked))
        return page.webView
    }

    /// A login set's data store: the one already held, or a new one, held from now on.
    private func dataStore(for loginSet: UUID, private isPrivate: Bool) -> WKWebsiteDataStore {
        if isPrivate {
            if let held = privateStores[loginSet] {
                return held
            }
            // A private window's: kept in memory, never written.
            let store = WKWebsiteDataStore.nonPersistent()
            privateStores[loginSet] = store
            return store
        }
        if let held = stores[loginSet] {
            return held
        }
        let store = WKWebsiteDataStore(forIdentifier: loginSet)
        stores[loginSet] = store
        return store
    }

    private func failure(_ code: String) -> WTHostResponse.OneOf_Kind {
        var error = WTError()
        error.code = code
        return .error(error)
    }

    /// An engine error as one of the contract's codes. The error's own text never leaves the shell.
    public nonisolated static func code(for error: NSError) -> String {
        if error.domain == "WebKitErrorDomain" {
            // 102: "Frame load interrupted", which is how a refused or replaced load ends.
            return error.code == 102 ? "net.cancelled" : "net.failed"
        }
        guard error.domain == NSURLErrorDomain else { return "net.failed" }
        switch error.code {
        case NSURLErrorCancelled: return "net.cancelled"
        case NSURLErrorNotConnectedToInternet, NSURLErrorDataNotAllowed, NSURLErrorInternationalRoamingOff: return "net.offline"
        case NSURLErrorCannotFindHost, NSURLErrorDNSLookupFailed: return "net.dns"
        case NSURLErrorCannotConnectToHost, NSURLErrorNetworkConnectionLost: return "net.connect"
        case NSURLErrorTimedOut: return "net.timeout"
        case NSURLErrorSecureConnectionFailed, NSURLErrorServerCertificateHasBadDate, NSURLErrorServerCertificateUntrusted,
             NSURLErrorServerCertificateHasUnknownRoot, NSURLErrorServerCertificateNotYetValid, NSURLErrorClientCertificateRejected,
             NSURLErrorClientCertificateRequired, NSURLErrorAppTransportSecurityRequiresSecureConnection:
            return "net.tls"
        default: return "net.failed"
        }
    }

    fileprivate func report(_ id: UInt64, _ kind: WTEngineEvent.OneOf_Kind) {
        var event = WTEngineEvent()
        event.webViewID = id
        event.kind = kind
        core?.report(event)
    }

    fileprivate func verdict(for url: URL?, webViewID: UInt64, mainFrame: Bool, userGesture: Bool) -> NavigationVerdict {
        guard let core, let url else { return .cancel }
        return core.navigationVerdict(for: url.absoluteString, webViewID: webViewID, mainFrame: mainFrame, userGesture: userGesture)
    }
}

/// One web view and the delegates that turn its callbacks into the contract's events
/// (001 §5 has the mapping).
@MainActor
private final class Page: NSObject, WKNavigationDelegate, WKUIDelegate {
    let id: UInt64
    let webView: WKWebView
    /// The login set the web view's pages run in, and whether it is a private one.
    let loginSet: UUID
    let isPrivate: Bool
    private weak var engine: WebKitEngine?
    private var observations: [NSKeyValueObservation] = []
    /// A main-frame load has started and not yet committed or failed.
    private var provisional = false
    /// The address last reported as the page on screen, by a commit or an address change.
    private var reportedURL: URL?
    /// A navigation was refused on the core's verdict while a load had started and not yet
    /// committed. If that load now ends as interrupted, the refusal was of where it was
    /// redirected to, and the load's end is the refusal's.
    private var refusedWhileProvisional = false

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
    /// - `load()` sets it to the address asked for, at once;
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

    func webView(_ webView: WKWebView, decidePolicyFor action: WKNavigationAction) async -> WKNavigationActionPolicy {
        // A navigation with no target frame opens a new window; it is decided as a top-level one.
        let mainFrame = action.targetFrame?.isMainFrame ?? true
        let userGesture = action.navigationType == .linkActivated || action.navigationType == .formSubmitted
        let verdict = engine?.verdict(for: action.request.url, webViewID: id, mainFrame: mainFrame, userGesture: userGesture)
        if verdict == .external {
            // An address for the system, not for a browser: refused here like any other, and
            // reported first, for a frame as for the page. Whether it is handed to the system
            // is the core's to say (007); the adapter opens nothing.
            var asked = WTOutsideLinkRequested()
            asked.url = action.request.url?.absoluteString ?? ""
            asked.userGesture = userGesture
            engine?.report(id, .outsideLinkRequested(asked))
        }
        switch verdict {
        case .allow:
            return .allow
        case .cancel, .external, nil:
            // A refused navigation with no target frame was for a new window: nothing of this
            // web view's own was going anywhere, and nothing is said about it.
            if mainFrame, action.targetFrame != nil {
                // WebKit reports nothing about a navigation refused here, so the adapter says
                // it ended: otherwise a tab the user sent somewhere would wait for ever.
                var failed = WTNavigationFailed()
                failed.code = "net.cancelled"
                failed.url = action.request.url?.absoluteString ?? ""
                // A refusal that leaves the web view loading nothing is said to be one: a
                // window a page opened then has nothing to show (006). With a load on its
                // way, it is that load's end that will say.
                refusedWhileProvisional = provisional
                failed.refused = !provisional && !webView.isLoading
                engine?.report(id, .navigationFailed(failed))
            }
            return .cancel
        }
    }

    func webView(_ webView: WKWebView, didStartProvisionalNavigation _: WKNavigation?) {
        provisional = true
        var started = WTNavigationStarted()
        started.url = webView.url?.absoluteString ?? ""
        engine?.report(id, .navigationStarted(started))
    }

    func webView(_ webView: WKWebView, didCommit _: WKNavigation?) {
        provisional = false
        refusedWhileProvisional = false
        reportedURL = webView.url
        var committed = WTNavigationCommitted()
        committed.url = webView.url?.absoluteString ?? ""
        engine?.report(id, .navigationCommitted(committed))
        historyChanged()
    }

    func webView(_: WKWebView, didFinish _: WKNavigation?) {
        reconcileAddress()
        engine?.report(id, .navigationFinished(WTNavigationFinished()))
    }

    func webView(_: WKWebView, didFailProvisionalNavigation _: WKNavigation?, withError error: any Error) {
        provisional = false
        // "Frame load interrupted" after a refusal: the load was redirected to the address
        // that was refused, and ends with it.
        let interrupted = (error as NSError).domain == "WebKitErrorDomain" && (error as NSError).code == 102
        failed(error, refused: refusedWhileProvisional && interrupted)
        refusedWhileProvisional = false
    }

    func webView(_: WKWebView, didFail _: WKNavigation?, withError error: any Error) {
        failed(error)
    }

    func webViewWebContentProcessDidTerminate(_: WKWebView) {
        provisional = false
        engine?.report(id, .rendererGone(WTRendererGone()))
    }

    private func failed(_ error: any Error, refused: Bool = false) {
        let error = error as NSError
        var failed = WTNavigationFailed()
        failed.code = WebKitEngine.code(for: error)
        failed.url = (error.userInfo[NSURLErrorFailingURLErrorKey] as? URL)?.absoluteString ?? ""
        failed.refused = refused
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

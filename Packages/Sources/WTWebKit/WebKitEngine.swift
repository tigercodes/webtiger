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
    /// How many web views this engine has made: a window watches it, to show one that was
    /// made after the window was told its number.
    public let made = WebViewsMade()
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

    /// Where downloads are saved (013). A folder of the engine's own, among the temporary
    /// files, unless one is given: only the app itself, started for real, names the system's
    /// place for downloads, so nothing else can write there.
    public let downloadsFolder: URL

    /// Downloads: what WebKit is fetching for a page, and what waits for the core's word.
    private(set) lazy var downloads = Downloads(folder: downloadsFolder, report: reporting)

    /// Whether a page is let have what the core grants it (014). Never, unless the engine is
    /// made otherwise: only the app itself, started for real, may bring up the system's own
    /// question about the camera, the microphone or the location.
    public let deviceAccess: DeviceAccess

    /// What pages ask to use of the device's, while the core has not said.
    private(set) lazy var permissions = Permissions(access: deviceAccess, report: reporting)

    /// What pages say or ask (016), while the core has not said how each is answered.
    private(set) lazy var dialogs = Dialogs(report: reporting)

    /// The servers that asked who is there (017), while the core has not said how each is answered.
    private(set) lazy var signIns = SignIns(report: reporting)

    /// The certificates a person went on past (026): until the process ends, and kept nowhere else.
    let certificates = Certificates()

    /// A link's menu (018): the shell's words for it, and where it copies to. With none,
    /// WebKit's own menu is shown as it is.
    public var linkMenu: LinkMenu?
    #if os(macOS)
        /// What was right-clicked, as the adapter's own script says (022).
        let pressed = PressedReports()
    #endif
    /// What each page is doing that would keep its tab awake, as the adapter's own script
    /// says of sound and of typing (052).
    let activity = ActivityReports()

    /// How a page's file input is given files (019). Nothing is picked, and nothing of the
    /// system's shown, unless the app itself gives its engine the system's picker.
    public var filePicker = FilePicker.none
    /// The web views whose pages have a picker open.
    let files = Files()
    /// Which icons each page names, and the fetch of the one the core asks for (046 §5a).
    private(set) lazy var icons = Icons(report: reporting)

    /// Where each of those sends what it has to say of a web view: to the core.
    private var reporting: @MainActor (UInt64, WTEngineEvent.OneOf_Kind) -> Void {
        { [weak self] in self?.report($0, $1) }
    }

    /// Where what pages keep is kept: their cookies, their storage, what the engine caches.
    public enum WebData: Sendable {
        /// On disk, a store for each login set: how the app keeps it.
        case kept
        /// In memory, a store for each login set all the same, and written nowhere: for a
        /// mode in which nothing outlives the app (fake mode, 001 §7).
        case inMemory
    }

    /// Kept, unless the engine is made otherwise.
    public let webData: WebData

    public init(downloads folder: URL? = nil, deviceAccess: DeviceAccess = .never, webData: WebData = .kept) {
        downloadsFolder = folder ?? FileManager.default.temporaryDirectory.appendingPathComponent("webtiger-downloads-\(UUID().uuidString)")
        self.deviceAccess = deviceAccess
        self.webData = webData
    }

    /// The system's place for downloads: the Downloads folder on a Mac; on an iPhone the app's
    /// own Downloads folder, which the Files app shows.
    public static var systemDownloads: URL? {
        #if os(macOS)
            FileManager.default.urls(for: .downloadsDirectory, in: .userDomainMask).first
        #else
            FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first?.appendingPathComponent("Downloads")
        #endif
    }

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

    /// A web view's picture (059) is begun in its turn, of the web view as it then is, and
    /// answered when WebKit has drawn it: the requests after it are not held up. The first
    /// of them lets the web view go, and WebKit draws its picture all the same (the adapter
    /// suite holds it to that, on a Mac and an iPhone).
    public func begin(_ request: WTHostRequest.OneOf_Kind) -> (@MainActor @Sendable () async -> WTHostResponse.OneOf_Kind)? {
        guard case let .webViewSnapshot(asked) = request, let page = pages[asked.webViewID] else { return nil }
        let flight = WebKitPictures.begin(of: page.webView, mostWide: Int(asked.maxWidth), mostBytes: Int(asked.maxBytes))
        return { await .webViewPicture(flight.picture()) }
    }

    public func answer(_ request: WTHostRequest.OneOf_Kind) async -> WTHostResponse.OneOf_Kind {
        switch request {
        case let .ping(ping):
            var pong = WTHostPong()
            pong.payload = ping.payload
            return .pong(pong)
        case let .webViewCreate(create): return self.create(create)
        case let .webViewLoad(load): return self.load(load.url, in: load.webViewID)
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
            // What its page was asking goes with it: refused, while WebKit can be told.
            permissions.drop(webViewID: destroy.webViewID)
            dialogs.drop(webViewID: destroy.webViewID)
            signIns.drop(webViewID: destroy.webViewID)
            files.drop(webViewID: destroy.webViewID)
            icons.drop(webViewID: destroy.webViewID)
            certificates.dropped(destroy.webViewID)
            if let page = pages.removeValue(forKey: destroy.webViewID) {
                // A web view whose picture is being drawn (059) is closed when it is drawn:
                // it is the tab's no longer from now, and WebKit has it a moment more.
                page.close()
                if page.isPrivate, !pages.values.contains(where: { $0.isPrivate && $0.loginSet == page.loginSet }) {
                    // A private login set is kept in memory and nowhere else: with its last
                    // web view gone, letting go of its store is what forgets it.
                    privateStores[page.loginSet] = nil
                }
            }
            return .ack(WTAck())
        case .openOutside, .openFile:
            // The System service's, which the shell answers itself (007, 030): opening a
            // link or a file with another app is nothing to do with a web engine.
            return failure("host.unsupported")
        case let .webViewFind(find):
            guard let page = pages[find.webViewID] else { return failure("webview.unknown") }
            page.find(find.text, backwards: find.backwards)
            return .ack(WTAck())
        case let .webViewSetZoom(zoom):
            guard let page = pages[zoom.webViewID] else { return failure("webview.unknown") }
            // WebKit's own page zoom (012): the page is laid out again at the level, and the
            // web view keeps it for whatever page it goes on to. The core sends its own
            // levels, which lie between half and three times; anything else is held to half
            // and five times, as far as the suite has tried WebKit on a Mac and an iPhone.
            page.webView.pageZoom = CGFloat(min(max(zoom.percent, 50), 500)) / 100
            return .ack(WTAck())
        case let .downloadStart(start):
            // The core's word on a download this adapter reported (013): it starts, under the
            // name the core made.
            return downloads.start(start.downloadID, named: start.fileName) ? .ack(WTAck()) : failure("download.unknown")
        case let .downloadCancel(cancel):
            downloads.cancel(cancel.downloadID)
            return .ack(WTAck())
        case .dialogAnswer, .signInAnswer, .permissionAnswer:
            return settle(request)
        case let .webViewTrustCertificate(trust):
            // The person went on past a certificate (026): what this web view last refused is
            // let through from now, in its login set.
            guard let page = pages[trust.webViewID] else { return failure("webview.unknown") }
            let trusted = certificates.trust(heldBy: page.id, loginSet: page.loginSet, isPrivate: page.isPrivate)
            return trusted ? .ack(WTAck()) : failure("certificate.unknown")
        case let .webViewFetchIcon(fetch): return fetchIcon(fetch)
        case let .webViewSaveState(save): return state(of: save.webViewID)
        case let .webViewRestore(restore): return self.restore(restore)
        case let .webViewSnapshot(asked): return await picture(asked)
        }
    }

    /// A picture of a web view as it is (059), for where its page will be when its tab
    /// wakes. The requests that follow wait for it: the next lets the web view go.
    private func picture(_ asked: WTWebViewSnapshot) async -> WTHostResponse.OneOf_Kind {
        guard let page = pages[asked.webViewID] else { return failure("webview.unknown") }
        return await .webViewPicture(WebKitPictures.picture(
            of: page.webView,
            mostWide: Int(asked.maxWidth),
            mostBytes: Int(asked.maxBytes)
        ))
    }

    /// Loads an address in a web view, as the core asked.
    private func load(_ address: String, in webViewID: UInt64) -> WTHostResponse.OneOf_Kind {
        guard let page = pages[webViewID] else { return failure("webview.unknown") }
        guard let url = URL(string: address) else { return failure("webview.bad_address") }
        page.webView.load(PrivacySignal.request(for: url, told: page.tellsPrivacySignal))
        return .ack(WTAck())
    }

    /// A web view's own state (051): WebKit's `interactionState`, which is data (the adapter
    /// suite holds it to that). None, should WebKit give none or something that is not data:
    /// the tab then wakes at its address.
    private func state(of webViewID: UInt64) -> WTHostResponse.OneOf_Kind {
        guard let page = pages[webViewID] else { return failure("webview.unknown") }
        var state = WTWebViewState()
        state.state = page.webView.interactionState as? Data ?? Data()
        return .webViewState(state)
    }

    /// Makes a web view what a state says (051), and where WebKit will not take the state
    /// loads the address that came with it: the tab wakes either way.
    private func restore(_ restore: WTWebViewRestore) -> WTHostResponse.OneOf_Kind {
        guard let page = pages[restore.webViewID] else { return failure("webview.unknown") }
        return page.restore(restore.state) ? .ack(WTAck()) : load(restore.url, in: restore.webViewID)
    }

    /// The core asked for a page's icon (046 §5a): answered at once, whatever comes of it.
    private func fetchIcon(_ fetch: WTWebViewFetchIcon) -> WTHostResponse.OneOf_Kind {
        guard pages[fetch.webViewID] != nil else { return failure("webview.unknown") }
        icons.fetch(fetch)
        return .ack(WTAck())
    }

    /// Deletes a login set: everything its pages stored, and the store itself. False, and
    /// nothing is deleted, while a web view still uses it. (Deleting a login set is the
    /// LoginSets host service's, which the contract does not have yet: until then this is for
    /// the tests that delete one.)
    public func deleteLoginSet(_ loginSet: UUID) async -> Bool {
        guard !pages.values.contains(where: { !$0.isPrivate && $0.loginSet == loginSet }) else { return false }
        if webData == .inMemory {
            // There is nothing on disk: the store this engine holds is all there is of it.
            guard let store = stores[loginSet] else { return true }
            guard await Self.clear(store, within: .seconds(20)) else { return false }
            stores[loginSet] = nil
            return true
        }
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
        #if os(macOS)
            pressed.listen(in: configuration.userContentController)
        #endif
        activity.listen(in: configuration.userContentController)
        // Global Privacy Control (069), when the core says this web view's pages are told it.
        if create.globalPrivacyControl {
            PrivacySignal.tell(configuration.userContentController)
        }
        pages[create.webViewID] = Page(
            id: create.webViewID,
            webView: Self.webView(configuration),
            loginSet: loginSet,
            isPrivate: create.loginSet.ephemeral,
            tellsPrivacySignal: create.globalPrivacyControl,
            engine: self
        )
        made.one()
        return .ack(WTAck())
    }

    private static func webView(_ configuration: WKWebViewConfiguration) -> WKWebView {
        let webView = MenuWebView(frame: .zero, configuration: configuration)
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
    func newWindow(askedBy asking: Page, configuration: WKWebViewConfiguration, url: URL?) -> WKWebView {
        ownWebViews += 1
        let id = Self.ownID | ownWebViews
        // (The configuration is the asking page's: with its scripts, and so with the signal
        // when the asking page's pages are told it.)
        let page = Page(
            id: id,
            webView: Self.webView(configuration),
            loginSet: asking.loginSet,
            isPrivate: asking.isPrivate,
            tellsPrivacySignal: asking.tellsPrivacySignal,
            engine: self
        )
        pages[id] = page
        made.one()
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
        // In memory, the store is the login set's own all the same, and is held while the
        // engine lives: a tab that sleeps wakes with what it had, and nothing is written.
        let store = webData == .inMemory ? WKWebsiteDataStore.nonPersistent() : WKWebsiteDataStore(forIdentifier: loginSet)
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
        // A certificate that is not trusted says why (021), where WebKit says. One that is
        // another site's it calls untrusted, like one that signs itself.
        case NSURLErrorServerCertificateUntrusted, NSURLErrorServerCertificateHasUnknownRoot: return "net.tls.untrusted"
        case NSURLErrorServerCertificateHasBadDate, NSURLErrorServerCertificateNotYetValid: return "net.tls.date"
        case NSURLErrorSecureConnectionFailed, NSURLErrorClientCertificateRejected, NSURLErrorClientCertificateRequired,
             NSURLErrorAppTransportSecurityRequiresSecureConnection:
            return "net.tls"
        default: return "net.failed"
        }
    }

    func report(_ id: UInt64, _ kind: WTEngineEvent.OneOf_Kind) {
        var event = WTEngineEvent()
        event.webViewID = id
        event.kind = kind
        core?.report(event)
    }

    func verdict(for url: URL?, webViewID: UInt64, mainFrame: Bool, userGesture: Bool) -> NavigationVerdict {
        guard let core, let url else { return .cancel }
        return core.navigationVerdict(for: url.absoluteString, webViewID: webViewID, mainFrame: mainFrame, userGesture: userGesture)
    }
}

/// How many web views an engine has made. The core tells a window the number of its tab's web
/// view as soon as it has asked the shell for one, and the shell makes it a moment later: a
/// window that looked for it in between found none. It looks again when this changes.
@MainActor
@Observable
public final class WebViewsMade {
    public private(set) var count = 0

    func one() {
        count += 1
    }
}

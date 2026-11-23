// A page's icon, on WebKit (046 §5a). WebKit hands an app no icon, so the adapter does two
// things, and decides neither: it says which icons a page names, and it fetches the one the
// core then asks for.
//
// The fetch is made as nobody. It is not the page's request and is given nothing of the
// page's: no cookie is sent and none is kept, there is no login to offer, nothing is cached,
// and no address is said to have referred it. What comes back is a picture or it is nothing:
// only four kinds are decoded, each known by its own first bytes and not by what a server
// calls it, and what is handed on is a small PNG made here.
import Foundation
import ImageIO
import UniformTypeIdentifiers
import WebKit
import WTContract

public extension WebKitEngine {
    /// Whether a page's icon is fetched when the core asks for it, and from where.
    enum IconFetching: Sendable {
        /// Nothing is read of a page's icons, and nothing is fetched.
        case never
        /// Only from this device: `localhost`, a loopback address. For a suite, whose site
        /// is in its own process.
        case thisDevice
        /// From this device, and from anywhere over `https`.
        case web
    }

    /// Never, unless the engine is made otherwise: only the app itself, started for real,
    /// fetches anything from the web that a page did not.
    var iconFetching: IconFetching {
        get { icons.fetching }
        set { icons.fetching = newValue }
    }
}

/// The icons of the engine's pages: which each names, and the fetch of the one asked for.
@MainActor
final class Icons {
    var fetching = WebKitEngine.IconFetching.never
    /// What each web view's page last said it is to a server: the fetch says the same, so
    /// that it tells a site nothing the page's own requests did not.
    private var agents: [UInt64: String] = [:]
    private let report: @MainActor (UInt64, WTEngineEvent.OneOf_Kind) -> Void

    /// A world of the adapter's own, where no script of the page's runs.
    private static let world = WKContentWorld.world(name: "webtiger-icons")

    /// The most icons taken from one page, and the longest address and words.
    private static let script = """
    (function () {
        var found = [];
        var links = document.querySelectorAll('link[rel][href]');
        for (var i = 0; i < links.length && found.length < 16; i += 1) {
            var rel = String(links[i].getAttribute('rel') || '');
            if (!/(^|\\s)(icon|apple-touch-icon|apple-touch-icon-precomposed)(\\s|$)/i.test(rel)) { continue; }
            var url = String(links[i].href || '');
            if (url.length === 0 || url.length > 2048) { continue; }
            found.push({
                url: url,
                rel: rel.slice(0, 64),
                sizes: String(links[i].getAttribute('sizes') || '').slice(0, 64),
                type: String(links[i].getAttribute('type') || '').slice(0, 64)
            });
        }
        return { page: String(location.href), agent: String(navigator.userAgent || ''), icons: found };
    }())
    """

    init(report: @escaping @MainActor (UInt64, WTEngineEvent.OneOf_Kind) -> Void) {
        self.report = report
    }

    /// A page has loaded: the core is told which icons it names. The page is asked in the
    /// adapter's own world, and says of itself which page it is: the web view may have gone on
    /// to another by the time it answers.
    func read(_ webView: WKWebView, id: UInt64) {
        guard fetching != .never else { return }
        webView.evaluateJavaScript(Self.script, in: nil, in: Self.world) { [weak self] result in
            MainActor.assumeIsolated {
                guard let self, case let .success(value) = result, let said = value as? [String: Any],
                      let page = said["page"] as? String
                else { return }
                self.agents[id] = said["agent"] as? String ?? ""
                var named = WTIconNamed()
                named.pageURL = page
                named.icons = (said["icons"] as? [[String: String]] ?? []).map { link in
                    var icon = WTNamedIcon()
                    icon.url = link["url"] ?? ""
                    icon.rel = link["rel"] ?? ""
                    icon.sizes = link["sizes"] ?? ""
                    icon.type = link["type"] ?? ""
                    return icon
                }
                self.report(id, .iconNamed(named))
            }
        }
    }

    /// The core asked for an icon. It is answered at once; the picture, when there is one, is
    /// reported as the page's, and whether it is that page's still is the core's to say.
    func fetch(_ asked: WTWebViewFetchIcon) {
        // (A page that was never asked which icons it names has none fetched.)
        guard let url = URL(string: asked.url), let agent = agents[asked.webViewID] else { return }
        let (id, page, reach) = (asked.webViewID, asked.pageURL, fetching)
        Task { [weak self] in
            let png = await IconFetch.picture(at: url, agent: agent, reach: reach)
            // A web view that has gone since has no icon to be told of.
            guard let self, let png, agents[id] != nil else { return }
            var received = WTIconReceived()
            received.png = png
            received.pageURL = page
            report(id, .iconReceived(received))
        }
    }

    func drop(webViewID: UInt64) {
        agents[webViewID] = nil
    }
}

/// The fetch of an icon, as nobody.
enum IconFetch {
    /// The most that is taken of what a server sends: nothing over this is an icon.
    static let mostBytes = 200 * 1024
    /// How long a server is waited for.
    static let seconds: TimeInterval = 20

    /// Whether an address may be asked for a picture: one of this device always, any other
    /// only over `https` and only by an engine that may reach the web. Never one that names
    /// somebody: a fetch has no login.
    static func allows(_ url: URL, _ reach: WebKitEngine.IconFetching) -> Bool {
        guard let scheme = url.scheme?.lowercased(), let host = url.host()?.lowercased(), !host.isEmpty,
              url.user == nil, url.password == nil
        else { return false }
        let own = host == "localhost" || host.hasSuffix(".localhost") || host == "::1" || isLoopback(host)
        switch reach {
        case .never: return false
        case .thisDevice: return own && (scheme == "http" || scheme == "https")
        case .web: return scheme == "https" || (own && scheme == "http")
        }
    }

    /// Whether a host is written as an address that never leaves this device: 127 and three
    /// more numbers.
    private static func isLoopback(_ host: String) -> Bool {
        let parts = host.split(separator: ".", omittingEmptySubsequences: false)
        return parts.count == 4 && parts.first == "127" && parts.allSatisfy { UInt8($0) != nil }
    }

    /// The picture at an address as a small PNG, or nil: for an address that may not be
    /// asked, a server that does not answer with one, more bytes than an icon has, and
    /// whatever is no picture of the four kinds.
    static func picture(at url: URL, agent: String, reach: WebKitEngine.IconFetching, most: Int = mostBytes) async -> Data? {
        guard allows(url, reach) else { return nil }
        // A session of its own for each fetch, which keeps nothing and is given nothing to
        // send: it ends with the fetch. No cookie is sent, and none is taken: said to the
        // request, and to the session, which has nowhere to keep one.
        var request = URLRequest(url: url)
        request.httpShouldHandleCookies = false
        let configuration = URLSessionConfiguration.ephemeral
        configuration.httpCookieStorage = nil
        configuration.httpShouldSetCookies = false
        configuration.httpCookieAcceptPolicy = .never
        configuration.urlCredentialStorage = nil
        configuration.urlCache = nil
        configuration.requestCachePolicy = .reloadIgnoringLocalCacheData
        configuration.timeoutIntervalForRequest = seconds
        configuration.timeoutIntervalForResource = seconds
        let session = URLSession(configuration: configuration, delegate: Nobody(reach: reach), delegateQueue: nil)
        defer { session.invalidateAndCancel() }
        if !agent.isEmpty {
            request.setValue(agent, forHTTPHeaderField: "User-Agent")
        }
        request.setValue("image/png, image/x-icon, image/vnd.microsoft.icon, image/jpeg, image/gif", forHTTPHeaderField: "Accept")
        do {
            let (bytes, response) = try await session.bytes(for: request)
            guard let answer = response as? HTTPURLResponse, (200 ..< 300).contains(answer.statusCode) else { return nil }
            var taken = Data()
            for try await byte in bytes {
                taken.append(byte)
                if taken.count > most {
                    return nil
                }
            }
            return IconPicture.png(from: taken)
        } catch {
            return nil
        }
    }

    /// What the fetch's session is told to do where a server asks something of it.
    private final class Nobody: NSObject, URLSessionTaskDelegate, Sendable {
        private let reach: WebKitEngine.IconFetching

        init(reach: WebKitEngine.IconFetching) {
            self.reach = reach
        }

        /// A server sends the fetch on: followed only to an address that may be asked.
        /// Otherwise the fetch ends there, with what is no picture.
        func urlSession(
            _: URLSession,
            task _: URLSessionTask,
            willPerformHTTPRedirection _: HTTPURLResponse,
            newRequest request: URLRequest
        ) async -> URLRequest? {
            guard let url = request.url, IconFetch.allows(url, reach) else { return nil }
            return request
        }

        /// A server's certificate is the system's to judge, as for any request. A server that
        /// asks who is there is told nothing, and the fetch ends: nobody is there.
        func urlSession(
            _: URLSession,
            task _: URLSessionTask,
            didReceive challenge: URLAuthenticationChallenge
        ) async -> (URLSession.AuthChallengeDisposition, URLCredential?) {
            challenge.protectionSpace.authenticationMethod == NSURLAuthenticationMethodServerTrust
                ? (.performDefaultHandling, nil) : (.cancelAuthenticationChallenge, nil)
        }
    }
}

/// What a server sent, made the picture the core is handed.
enum IconPicture {
    /// The widest and the tallest an icon is handed over as, in pixels.
    static let pixels = 128
    /// The widest and the tallest picture that is decoded at all.
    static let mostPixels = 2048
    /// The most bytes of a picture the core keeps.
    static let mostBytes = 65536

    /// The kind of picture these bytes begin as, of the four that are decoded.
    static func kind(of data: Data) -> UTType? {
        let first = [UInt8](data.prefix(8))
        if first == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
            return .png
        }
        if first.starts(with: [0xFF, 0xD8, 0xFF]) {
            return .jpeg
        }
        if first.starts(with: Array("GIF87a".utf8)) || first.starts(with: Array("GIF89a".utf8)) {
            return .gif
        }
        return first.starts(with: [0, 0, 1, 0]) ? .ico : nil
    }

    /// The picture as a PNG of at most `pixels` a side, in its own shape. Nil for what is none
    /// of the four kinds, what the system's decoder does not take for the kind it begins as,
    /// a picture of no size or of more than `mostPixels` a side, and one whose PNG is more
    /// bytes than the core keeps even when it is made smaller.
    static func png(from data: Data, mostBytes: Int = mostBytes) -> Data? {
        guard let kind = kind(of: data),
              let source = CGImageSourceCreateWithData(data as CFData, [kCGImageSourceTypeIdentifierHint: kind.identifier] as CFDictionary),
              CGImageSourceGetType(source) as String? == kind.identifier,
              let index = largest(in: source, kind: kind)
        else { return nil }
        for side in [pixels, pixels * 3 / 4, pixels / 2] {
            let options: [CFString: Any] = [
                kCGImageSourceCreateThumbnailFromImageAlways: true,
                kCGImageSourceCreateThumbnailWithTransform: true,
                kCGImageSourceThumbnailMaxPixelSize: side,
            ]
            let written = NSMutableData()
            guard let image = CGImageSourceCreateThumbnailAtIndex(source, index, options as CFDictionary),
                  let destination = CGImageDestinationCreateWithData(written, UTType.png.identifier as CFString, 1, nil)
            else { return nil }
            CGImageDestinationAddImage(destination, image, nil)
            guard CGImageDestinationFinalize(destination) else { return nil }
            if written.length <= mostBytes {
                return written as Data
            }
        }
        return nil
    }

    /// Which of a file's pictures to take: the largest that is not too large to decode. An
    /// `.ico` holds several sizes of one; of any other kind the first is the picture.
    private static func largest(in source: CGImageSource, kind: UTType) -> Int? {
        var best: (index: Int, side: Int)?
        for index in 0 ..< min(CGImageSourceGetCount(source), kind == .ico ? 16 : 1) {
            guard let said = CGImageSourceCopyPropertiesAtIndex(source, index, nil) as? [CFString: Any],
                  let wide = said[kCGImagePropertyPixelWidth] as? Int, let tall = said[kCGImagePropertyPixelHeight] as? Int,
                  wide > 0, tall > 0, wide <= mostPixels, tall <= mostPixels
            else { continue }
            if max(wide, tall) > best?.side ?? 0 {
                best = (index, max(wide, tall))
            }
        }
        return best?.index
    }
}

// A server asks who is there before it answers (017): HTTP authentication, on WebKit. Left
// alone WebKit gives the server no answer. The adapter reports that it asks and waits: the
// core says whether it is answered, and with what. It decides nothing, shows nothing, and
// keeps nothing of what was typed; WebKit keeps the answer for the server until the app
// quits, in the web view's login set.
import Foundation
import WebKit
import WTContract

/// The servers' questions of one engine's web views that wait for the core's word.
@MainActor
final class SignIns {
    /// What a person typed for a server.
    struct Typed {
        let name: String
        let password: String
    }

    private struct Waiting {
        let webViewID: UInt64
        let answer: (Typed?) -> Void
    }

    private let report: @MainActor (UInt64, WTEngineEvent.OneOf_Kind) -> Void
    /// How many questions this engine has named. Each has the next number, and none is used
    /// twice: the ids are the shell's own (engine.proto, `SignInRequested`).
    private var named: UInt64 = 0
    private var waiting: [UInt64: Waiting] = [:]

    init(report: @escaping @MainActor (UInt64, WTEngineEvent.OneOf_Kind) -> Void) {
        self.report = report
    }

    /// A server asks who is there in the web view `webViewID`. It is reported, and the load
    /// waits until the core has said: what was typed, or nil for no answer.
    func ask(webViewID: UInt64, server: URLProtectionSpace, heading: String, again: Bool) async -> Typed? {
        named += 1
        let id = named
        var asked = WTSignInRequested()
        asked.requestID = id
        asked.host = server.host
        asked.origin = Self.origin(of: server)
        asked.realm = server.realm ?? ""
        asked.heading = heading
        asked.again = again
        return await withCheckedContinuation { continuation in
            waiting[id] = Waiting(webViewID: webViewID) { continuation.resume(returning: $0) }
            report(webViewID, .signInRequested(asked))
        }
    }

    /// The core's word on a server's question. One that no longer waits has no server to
    /// answer.
    func answer(_ id: UInt64, accepted: Bool, name: String, password: String) {
        waiting.removeValue(forKey: id)?.answer(accepted ? Typed(name: name, password: password) : nil)
    }

    /// A web view is going: the servers that asked in it are given no answer. WebKit must be
    /// given its answer: one it never gets is the app's fault, and it says so by stopping it.
    func drop(webViewID: UInt64) {
        for (id, asking) in waiting where asking.webViewID == webViewID {
            waiting[id] = nil
            asking.answer(nil)
        }
    }

    /// Whether a name and a password answer what the server asks. Its certificate, a
    /// certificate of the person's, and a ticket of the system's are other questions, and
    /// the engine's (017 §2).
    static func isForAPassword(_ server: URLProtectionSpace) -> Bool {
        [
            NSURLAuthenticationMethodDefault,
            NSURLAuthenticationMethodHTTPBasic,
            NSURLAuthenticationMethodHTTPDigest,
            NSURLAuthenticationMethodNTLM,
        ]
        .contains(server.authenticationMethod)
    }

    /// The server, as the contract writes an origin: without the port its scheme implies.
    static func origin(of server: URLProtectionSpace) -> String {
        let scheme = server.protocol ?? ""
        let implied = (scheme == "https" && server.port == 443) || (scheme == "http" && server.port == 80)
        return Permissions.origin(scheme: scheme, host: server.host, port: implied ? 0 : server.port)
    }
}

/// What WebKit asks the page's delegate when a server wants something before it answers (the
/// rest of the delegate is beside the page, in WebKitEngine.swift).
extension Page {
    /// Where the load on its way is heading, redirects followed: WebKit's own address for the
    /// web view is that, while a load has not committed. Empty when none is on its way.
    var heading: String {
        provisional ? webView.url?.absoluteString ?? "" : ""
    }

    func webView(
        _ webView: WKWebView,
        respondTo challenge: URLAuthenticationChallenge
    ) async -> (URLSession.AuthChallengeDisposition, URLCredential?) {
        let server = challenge.protectionSpace
        // The server's certificate (026): let through where a person went on past it, and
        // WebKit's own judgement otherwise.
        if server.authenticationMethod == NSURLAuthenticationMethodServerTrust {
            return answerToACertificate(server)
        }
        // A proxy's question, and one that a name and a password do not answer, are WebKit's,
        // as they were.
        guard let engine, !server.isProxy(), SignIns.isForAPassword(server) else {
            return (.performDefaultHandling, nil)
        }
        let typed = await engine.signIns.ask(webViewID: id, server: server, heading: heading, again: challenge.previousFailureCount > 0)
        guard let typed else {
            // No answer: WebKit goes on without one, and shows what the server sent with its
            // refusal.
            return (.rejectProtectionSpace, nil)
        }
        // Kept by WebKit for this server until the app quits, in the web view's login set.
        return (.useCredential, URLCredential(user: typed.name, password: typed.password, persistence: .forSession))
    }
}

extension WebKitEngine {
    /// The core's word on what waited for the person: what a page said or asked (016), a
    /// server that asked who is there (017), what a page asked to use (014). Done, also for
    /// one that no longer waits: its page went away.
    func settle(_ request: WTHostRequest.OneOf_Kind) -> WTHostResponse.OneOf_Kind {
        switch request {
        case let .dialogAnswer(answer):
            dialogs.answer(answer.requestID, accepted: answer.accepted, text: answer.text)
        case let .signInAnswer(answer):
            signIns.answer(answer.requestID, accepted: answer.accepted, name: answer.name, password: answer.password)
        case let .permissionAnswer(answer):
            permissions.answer(answer.requestID, granted: answer.granted)
        default:
            break
        }
        return .ack(WTAck())
    }
}

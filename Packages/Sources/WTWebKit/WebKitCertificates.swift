// Going on past a certificate that is not trusted (026), WebKit's part: which certificates are
// let through and for whom, and which one a web view holds to be asked about. Kept in memory,
// for as long as the process lives, and nowhere else.
import CryptoKit
import Foundation
import Security
import WebKit

/// A certificate a server presented: whose it was (the scheme, host and port it was presented
/// at) and which (a hash of the certificate itself).
struct PresentedCertificate: Hashable, Sendable {
    let origin: String
    let fingerprint: String
}

/// What is let through (026 §3): one certificate, at one host and port, in one login set. The
/// same host with another certificate, another host with the same one, and another login set
/// are each refused as before.
@MainActor
final class Certificates {
    private struct Exception: Hashable {
        let loginSet: UUID
        let isPrivate: Bool
        let certificate: PresentedCertificate
    }

    private var trusted: Set<Exception> = []
    /// What each web view's servers presented since its load began, by origin: WebKit asks
    /// about a certificate before it says whether the load failed for it.
    private var presented: [UInt64: [String: String]] = [:]
    /// The certificate each web view's own page was last refused: what it would be told to
    /// let through.
    private var held: [UInt64: PresentedCertificate] = [:]

    /// Whether a certificate is let through, in a login set.
    func trusts(loginSet: UUID, isPrivate: Bool, _ certificate: PresentedCertificate) -> Bool {
        trusted.contains(Exception(loginSet: loginSet, isPrivate: isPrivate, certificate: certificate))
    }

    /// A server presented a web view this certificate, which is not one that is let through.
    func wasPresented(_ certificate: PresentedCertificate, to webViewID: UInt64) {
        presented[webViewID, default: [:]][certificate.origin] = certificate.fingerprint
    }

    /// A web view begins a load of its own, or its load commits: what it was presented and
    /// held before is the last load's, and a page that is shown holds nothing.
    func loadBegan(in webViewID: UInt64) {
        presented[webViewID] = nil
        held[webViewID] = nil
    }

    /// A web view's own page failed for the certificate of `origin`. Whether the certificate is
    /// held: it is, when that origin presented one since the load began.
    func refused(at origin: String, in webViewID: UInt64) -> Bool {
        guard let fingerprint = presented[webViewID]?[origin] else { return false }
        held[webViewID] = PresentedCertificate(origin: origin, fingerprint: fingerprint)
        return true
    }

    /// Lets through, in its login set, what a web view last refused. False when it holds nothing.
    func trust(heldBy webViewID: UInt64, loginSet: UUID, isPrivate: Bool) -> Bool {
        guard let certificate = held[webViewID] else { return false }
        trusted.insert(Exception(loginSet: loginSet, isPrivate: isPrivate, certificate: certificate))
        return true
    }

    /// The web view is gone, and with it what it held. What was let through stays, for its
    /// login set.
    func dropped(_ webViewID: UInt64) {
        presented[webViewID] = nil
        held[webViewID] = nil
    }

    /// Whether a failure's code is one of the kinds of trouble a certificate has (021 §3), which
    /// a person may go on past: not every way a secure connection fails is.
    nonisolated static func isACertificatesTrouble(_ code: String) -> Bool {
        ["net.tls.untrusted", "net.tls.date", "net.tls.name"].contains(code)
    }

    /// Whose certificate a connection presents: the scheme, host and port, with the port
    /// written also where the scheme implies it, so that two ways of writing an address are one
    /// site.
    nonisolated static func origin(host: String, port: Int) -> String {
        "https://\(host.lowercased()):\(port)"
    }

    /// The same, of an address. `nil` for what is no address over `https`.
    nonisolated static func origin(of url: URL) -> String? {
        guard url.scheme?.lowercased() == "https", let host = url.host(percentEncoded: false), !host.isEmpty else { return nil }
        return origin(host: host, port: url.port ?? 443)
    }

    /// A hash of the certificate a server presented, the first of its chain: SHA-256 of the
    /// certificate itself, in hex. `nil` when the system gives no certificate.
    nonisolated static func fingerprint(of trust: SecTrust) -> String? {
        guard let chain = SecTrustCopyCertificateChain(trust) as? [SecCertificate], let leaf = chain.first else { return nil }
        return SHA256.hash(data: SecCertificateCopyData(leaf) as Data).map { String(format: "%02x", $0) }.joined()
    }
}

extension Page {
    /// What WebKit is told when a server presents its certificate (026): let through, when a
    /// person went on past that very certificate at that host and port, in this web view's
    /// login set; otherwise WebKit's own judgement, as before, with the certificate remembered
    /// in case the load fails for it.
    func answerToACertificate(_ server: URLProtectionSpace) -> (URLSession.AuthChallengeDisposition, URLCredential?) {
        guard let engine, let trust = server.serverTrust, let fingerprint = Certificates.fingerprint(of: trust) else {
            return (.performDefaultHandling, nil)
        }
        let certificate = PresentedCertificate(origin: Certificates.origin(host: server.host, port: server.port), fingerprint: fingerprint)
        if engine.certificates.trusts(loginSet: loginSet, isPrivate: isPrivate, certificate) {
            return (.useCredential, URLCredential(trust: trust))
        }
        // Remembered only while the page's own load has not committed: only that load's
        // failure is one a person is asked about, and a page that stays for hours is not to
        // gather the certificate of every server it ever asked.
        if provisional {
            engine.certificates.wasPresented(certificate, to: id)
        }
        return (.performDefaultHandling, nil)
    }

    /// Whether the certificate that a failed load of this page's own was refused is held, to be
    /// let through if the core says: only for the kinds of trouble a certificate has (021), and
    /// only when the server of the address that failed presented one.
    func holdsCertificate(refusedWith code: String, at url: URL?) -> Bool {
        guard Certificates.isACertificatesTrouble(code), let url, let origin = Certificates.origin(of: url) else {
            return false
        }
        return engine?.certificates.refused(at: origin, in: id) ?? false
    }
}

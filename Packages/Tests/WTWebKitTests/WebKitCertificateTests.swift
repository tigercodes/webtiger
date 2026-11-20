// The adapter suite's WebKit run, for a certificate that is not trusted and for going on past
// one (021, 026): a site over https whose certificate signs itself, served from this process
// with an identity that is made in memory. What the adapter says the trouble was, that nothing
// of the site is shown or asked for, and what it lets through once it is told to.
import Foundation
import WTContract
@testable import WTWebKit
import XCTest

@MainActor
final class WebKitCertificateTests: AdapterTestCase {
    /// The failures of a web view's loads, in order.
    private func failures(_ id: UInt64) -> [WTNavigationFailed] {
        reports(id).compactMap { report -> WTNavigationFailed? in
            if case let .navigationFailed(failed) = report {
                failed
            } else {
                nil
            }
        }
    }

    private func failedTimes(_ times: Int) -> ([WTEngineEvent.OneOf_Kind]) -> Bool {
        { reports in
            reports.count(where: {
                if case .navigationFailed = $0 {
                    true
                } else {
                    false
                }
            }) >= times
        }
    }

    /// Tells the adapter to let through what a web view last refused. The error's code, or
    /// `nil` when it did.
    private func trust(_ id: UInt64) async -> String? {
        var trust = WTWebViewTrustCertificate()
        trust.webViewID = id
        if case let .error(refused) = await engine.answer(.webViewTrustCertificate(trust)) {
            return refused.code
        }
        return nil
    }

    private func secureSite(_ identity: TestIdentity = .first, port: UInt16 = 0) throws -> SecureSite {
        let secure = SecureSite(identity, port: port)
        try XCTSkipIf(secure.port == 0, "this system would not make a TLS identity in memory, or would not listen")
        return secure
    }

    func testACertificateThatSignsItselfIsNotTrustedAndThePageIsNeverAskedFor() async throws {
        let secure = try secureSite()
        await create(1)
        await load(1, secure.address)

        await wait(1, "the load to fail", until: failedTimes(1))

        let failed = failures(1)
        XCTAssertEqual(failed.map(\.code), ["net.tls.untrusted"], "which kind of trouble the certificate was")
        XCTAssertEqual(failed.map(\.url), [secure.address])
        // Nothing of the site was shown, and nothing was accepted on anybody's behalf: the
        // site was never asked for its page.
        XCTAssertFalse(names(reports(1)).contains("committed"), "\(names(reports(1)))")
        XCTAssertEqual(secure.served, 0)
        // The adapter holds what it refused (026): it can be told to let it through.
        XCTAssertEqual(failed.map(\.certificateHeld), [true])
    }

    /// Going on past a certificate (026), against real WebKit: nothing is let through until the
    /// adapter is told; told, the certificate it refused is let through and the page loads;
    /// and it is let through for the login set it was told for, and no other.
    func testACertificateIsLetThroughWhenTheAdapterIsToldAndOnlyInThatLoginSet() async throws {
        let secure = try secureSite()
        let home = newLoginSet()
        await create(1, loginSet: home)
        // A web view that refused nothing holds nothing; and no web view, nothing.
        let nothingHeld = await trust(1)
        XCTAssertEqual(nothingHeld, "certificate.unknown")
        let nobody = await trust(99)
        XCTAssertEqual(nobody, "webview.unknown")
        await load(1, secure.address)
        await wait(1, "the load to fail", until: failedTimes(1))
        XCTAssertEqual(secure.served, 0)

        let refused = await trust(1)
        XCTAssertNil(refused)
        await load(1, secure.address)

        await wait(1, "the site's page") { self.hasTitle(SecureSite.title)($0) && self.finished($0) }
        XCTAssertEqual(secure.served, 1, "the site was asked for its page, once")
        XCTAssertEqual(failures(1).map(\.code), ["net.tls.untrusted"], "no failure but the first")

        // Another web view of the same login set: let through already.
        await create(2, loginSet: home)
        await load(2, secure.address)
        await wait(2, "the site's page in the same login set") { self.hasTitle(SecureSite.title)($0) && self.finished($0) }
        XCTAssertEqual(failures(2).map(\.code), [])

        // Another login set is asked for itself: refused, held, and the site not asked.
        let asked = secure.served
        await create(3)
        await load(3, secure.address)
        await wait(3, "the load to fail in another login set", until: failedTimes(1))
        XCTAssertEqual(failures(3).map(\.code), ["net.tls.untrusted"])
        XCTAssertEqual(failures(3).map(\.certificateHeld), [true])
        try? await Task.sleep(for: .milliseconds(500))
        XCTAssertEqual(secure.served, asked, "nothing was let through for it")
    }

    /// One certificate and no other (026 §3): the same host and port with another certificate
    /// is refused again, though a person went on past the one before.
    func testAnotherCertificateAtTheSameHostAndPortIsRefusedAgain() async throws {
        let first = try secureSite()
        let address = first.address
        let port = first.port
        await create(1)
        await load(1, address)
        await wait(1, "the load to fail", until: failedTimes(1))
        let refused = await trust(1)
        XCTAssertNil(refused)
        await load(1, address)
        await wait(1, "the site's page") { self.hasTitle(SecureSite.title)($0) && self.finished($0) }
        first.stop()
        try? await Task.sleep(for: .milliseconds(300))

        // The same place, with another key and certificate.
        let second = try secureSite(.second, port: port)
        XCTAssertEqual(second.port, port, "the second site is where the first was")
        await load(1, address)
        await wait(1, "the second load to fail", until: failedTimes(2))
        XCTAssertEqual(failures(1).map(\.code), ["net.tls.untrusted", "net.tls.untrusted"])
        XCTAssertEqual(failures(1).map(\.certificateHeld), [true, true], "held, to be gone on with afresh")
        try? await Task.sleep(for: .milliseconds(500))
        XCTAssertEqual(second.served, 0, "its page was never asked for")

        // Gone on with again, it is this one that is let through.
        let again = await trust(1)
        XCTAssertNil(again)
        await load(1, address)
        await wait(1, "the site's page again") { self.finished($0, times: 2) }
        XCTAssertEqual(second.served, 1)
    }

    /// A page that is shown asks a site whose certificate is not trusted for something: the
    /// site is not asked, and nothing of it is held to go on past. Only a page's own load is.
    func testWhatAPageThatIsShownAsksOfAnUntrustedSiteIsRefusedAndNothingIsHeld() async throws {
        let secure = try secureSite()
        await create(1)
        await load(1, site.url("/landed"))
        await wait(1, "the page") { self.finished($0) }

        await run("fetch('\(secure.address)').then(() => { document.title = 'fetched' }, () => { document.title = 'refused' })", in: 1)

        await wait(1, "the page's request to be refused") { self.hasTitle("refused")($0) || self.hasTitle("fetched")($0) }
        XCTAssertEqual(lastTitle(1), "refused")
        XCTAssertEqual(secure.served, 0)
        XCTAssertEqual(failures(1).map(\.code), [], "the page itself did not fail")
        let origin = try XCTUnwrap(URL(string: secure.address).flatMap(Certificates.origin(of:)))
        XCTAssertFalse(engine.certificates.refused(at: origin, in: 1), "nothing is remembered of it")
        let held = await trust(1)
        XCTAssertEqual(held, "certificate.unknown")
    }

    /// Whose certificate an address presents, and that one is told from another.
    func testWhoseCertificateAnAddressPresentsIsItsHostAndPortOverHTTPS() {
        let origin = { (address: String) in URL(string: address).flatMap(Certificates.origin(of:)) }
        XCTAssertEqual(origin("https://router.example.com/"), "https://router.example.com:443")
        XCTAssertEqual(origin("https://Router.Example.com:443/status?x=1#top"), "https://router.example.com:443")
        XCTAssertEqual(origin("https://127.0.0.1:8443/"), "https://127.0.0.1:8443")
        XCTAssertEqual(Certificates.origin(host: "Router.Example.com", port: 443), "https://router.example.com:443")
        // No certificate is presented at any of these.
        for other in ["http://router.example.com/", "wss://router.example.com/", "about:blank", "https://"] {
            XCTAssertNil(origin(other), other)
        }
        // What is held is held for a certificate's own trouble, and no other failure.
        for code in ["net.tls.untrusted", "net.tls.date", "net.tls.name"] {
            XCTAssertTrue(Certificates.isACertificatesTrouble(code), code)
        }
        for code in ["net.tls", "net.dns", "net.connect", "net.cancelled", "net.failed", ""] {
            XCTAssertFalse(Certificates.isACertificatesTrouble(code), code)
        }
    }

    /// The adapter's own rule, with no engine: what is let through, and for whom.
    func testWhatAWebViewLastRefusedIsLetThroughInItsLoginSetAndNoOther() {
        let certificates = Certificates()
        let home = UUID()
        let work = UUID()
        let router = PresentedCertificate(origin: "https://router.example.com:443", fingerprint: "aa11")
        let changed = PresentedCertificate(origin: router.origin, fingerprint: "bb22")

        // Presented, and refused: held, and nothing let through.
        XCTAssertFalse(certificates.refused(at: router.origin, in: 4), "nothing was presented there yet")
        certificates.wasPresented(router, to: 4)
        XCTAssertTrue(certificates.refused(at: router.origin, in: 4))
        XCTAssertFalse(certificates.trusts(loginSet: home, isPrivate: false, router))
        XCTAssertFalse(certificates.trust(heldBy: 5, loginSet: home, isPrivate: false), "a web view that refused nothing holds nothing")

        XCTAssertTrue(certificates.trust(heldBy: 4, loginSet: home, isPrivate: false))

        XCTAssertTrue(certificates.trusts(loginSet: home, isPrivate: false, router))
        XCTAssertFalse(certificates.trusts(loginSet: work, isPrivate: false, router), "another login set is asked for itself")
        XCTAssertFalse(certificates.trusts(loginSet: home, isPrivate: true, router), "and so is a private window")
        XCTAssertFalse(certificates.trusts(loginSet: home, isPrivate: false, changed), "another certificate at the same host")
        let elsewhere = PresentedCertificate(origin: "https://printer.example.com:443", fingerprint: router.fingerprint)
        XCTAssertFalse(certificates.trusts(loginSet: home, isPrivate: false, elsewhere), "the same certificate at another host")

        // A load that begins, and a web view that is gone, hold nothing; what was let through stays.
        certificates.wasPresented(changed, to: 4)
        XCTAssertTrue(certificates.refused(at: router.origin, in: 4))
        certificates.loadBegan(in: 4)
        XCTAssertFalse(certificates.trust(heldBy: 4, loginSet: work, isPrivate: false))
        certificates.wasPresented(changed, to: 6)
        XCTAssertTrue(certificates.refused(at: router.origin, in: 6))
        certificates.dropped(6)
        XCTAssertFalse(certificates.trust(heldBy: 6, loginSet: work, isPrivate: false))
        XCTAssertTrue(certificates.trusts(loginSet: home, isPrivate: false, router))
    }
}

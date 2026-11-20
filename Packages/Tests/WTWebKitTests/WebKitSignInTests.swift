// The adapter suite's WebKit run, for a server that asks who is there (017): HTTP
// authentication, against the test site in a real web view. What the adapter reports, that the
// load waits for the core's word, what each word does, and that what WebKit keeps of an answer
// stays in its login set.
import WebKit
import WTContract
import WTCoreClient
@testable import WTWebKit
import XCTest

@MainActor
final class WebKitSignInTests: AdapterTestCase {
    /// The questions the adapter has reported of a web view's servers, in order.
    private func asked(_ id: UInt64 = 1) -> [WTSignInRequested] {
        reports(id).compactMap {
            if case let .signInRequested(asked) = $0 {
                asked
            } else {
                nil
            }
        }
    }

    /// Waits until so many servers have asked in a web view, and gives the last.
    private func awaitAsked(_ id: UInt64 = 1, count: Int = 1) async -> WTSignInRequested {
        await wait(id, "a server to ask who is there (\(count))") { _ in self.asked(id).count >= count }
        return asked(id).last ?? WTSignInRequested()
    }

    /// The core's word on a server's question: a name and a password, or with none, no answer.
    private func answer(_ asked: WTSignInRequested, name: String? = nil, password: String = "") async {
        var answer = WTSignInAnswer()
        answer.requestID = asked.requestID
        answer.accepted = name != nil
        answer.name = name ?? ""
        answer.password = password
        guard case .ack = await engine.answer(.signInAnswer(answer)) else {
            return XCTFail("a word about a server's question is acknowledged")
        }
    }

    private func awaitPage(_ id: UInt64, _ title: String) async {
        await wait(id, "the page \"\(title)\"; it last said \"\(lastTitle(id))\"") { self.finished($0) && self.hasTitle(title)($0) }
    }

    /// The addresses a web view's pages have committed at, in order.
    private func committed(_ id: UInt64 = 1) -> [String] {
        reports(id).compactMap {
            if case let .navigationCommitted(page) = $0 {
                page.url
            } else {
                nil
            }
        }
    }

    /// The site's own origin, as an origin is written.
    private var origin: String {
        String(site.url("/").dropLast())
    }

    func testAServerThatAsksIsReportedAndTheLoadWaitsForTheCoresWord() async {
        await create(1)
        await load(1, site.url("/staff"))

        let asked = await awaitAsked()

        XCTAssertEqual(asked.host, "127.0.0.1")
        XCTAssertEqual(asked.origin, origin, "WebKit names the server's origin")
        XCTAssertEqual(asked.realm, "Staff")
        XCTAssertEqual(asked.heading, site.url("/staff"), "where the load is heading")
        XCTAssertFalse(asked.again, "the first time")
        XCTAssertNotEqual(asked.requestID, 0, "named by the shell")
        // The load waits: no page has come of it.
        try? await Task.sleep(for: .milliseconds(500))
        XCTAssertEqual(committed(), [])

        await answer(asked, name: "ada", password: "lovelace")

        await awaitPage(1, "Staff only")
        XCTAssertEqual(committed(), [site.url("/staff")])
        // The page's picture is the same server's: WebKit answers for itself.
        try? await Task.sleep(for: .milliseconds(500))
        XCTAssertEqual(self.asked().count, 1, "asked once")
    }

    func testAWrongAnswerAsksAgainAndSaysSo() async {
        await create(1)
        await load(1, site.url("/staff"))
        let first = await awaitAsked()

        await answer(first, name: "ada", password: "wrong")

        let second = await awaitAsked(count: 2)
        XCTAssertTrue(second.again, "the server refused an answer already")
        XCTAssertNotEqual(second.requestID, first.requestID, "each has a name of its own")
        XCTAssertEqual(second.heading, site.url("/staff"), "still the same load")
        await answer(second, name: "ada", password: "lovelace")
        await awaitPage(1, "Staff only")
    }

    func testNoAnswerShowsWhatTheServerSentWithItsRefusal() async {
        await create(1)
        await load(1, site.url("/staff"))

        await answer(awaitAsked())

        await awaitPage(1, "Sign in needed")
        XCTAssertEqual(committed(), [site.url("/staff")], "the refusal is a page, at the address asked for")
        XCTAssertEqual(asked().count, 1, "and nobody is asked again")
    }

    func testWhatWebKitKeepsOfAnAnswerStaysInItsLoginSet() async {
        let one = newLoginSet()
        await create(1, loginSet: one)
        await load(1, site.url("/staff"))
        await answer(awaitAsked(1), name: "ada", password: "lovelace")
        await awaitPage(1, "Staff only")

        // Another web view of the same login set: WebKit answers for itself.
        await create(2, loginSet: one)
        await load(2, site.url("/staff"))
        await awaitPage(2, "Staff only")
        XCTAssertEqual(asked(2).count, 0, "not asked again in the same login set")

        // Another login set knows nothing of it: the server asks.
        await create(3, loginSet: newLoginSet())
        await load(3, site.url("/staff"))
        let again = await awaitAsked(3)
        XCTAssertFalse(again.again, "and it is that login set's first time")
        await answer(again)
        await awaitPage(3, "Sign in needed")
    }

    func testWhereARedirectTookTheLoadIsWhereItIsHeading() async {
        let loginSet = newLoginSet()
        await create(1, loginSet: loginSet)
        await load(1, site.url("/to-staff?port=\(site.port)"))

        let asked = await awaitAsked()

        // The same server by its other name: where the load was sent on to.
        XCTAssertEqual(asked.host, "localhost")
        XCTAssertEqual(asked.origin, "http://localhost:\(site.port)")
        XCTAssertEqual(asked.heading, "http://localhost:\(site.port)/staff")
        await answer(asked, name: "ada", password: "lovelace")
        await wait(1, "the page to arrive") { self.hasTitle("Staff only")($0) }
        XCTAssertEqual(committed(), ["http://localhost:\(site.port)/staff"])

        // Seen here: an answer given to a server a load was sent on to is not used by WebKit
        // for that page's own picture. The server asks once more, now as the page's own.
        let picture = await awaitAsked(count: 2)
        XCTAssertEqual(picture.origin, "http://localhost:\(site.port)")
        XCTAssertEqual(picture.heading, "", "the page has arrived")
        XCTAssertFalse(picture.again, "not a refusal: it was not offered the answer")
        await answer(picture, name: "ada", password: "lovelace")
        await awaitPage(1, "Staff only")
        // That answer is kept: the same address again asks nothing.
        await create(2, loginSet: loginSet)
        await load(2, "http://localhost:\(site.port)/staff")
        await awaitPage(2, "Staff only")
        XCTAssertEqual(self.asked(2).count, 0)
    }

    func testAPictureOfThePagesOwnServerAsksWithNoLoadOnItsWay() async {
        await create(1)
        await load(1, site.url("/shows-staff-picture?from=own"))

        let asked = await awaitAsked()

        XCTAssertEqual(asked.origin, origin)
        XCTAssertEqual(asked.heading, "", "the page has arrived: nothing is heading anywhere")
        await answer(asked, name: "ada", password: "lovelace")
        await wait(1, "the page to say its picture is shown; it last said \"\(lastTitle(1))\"") { self.hasTitle("picture shown")($0) }
    }

    func testAPictureFromAnotherServerIsGivenNoAnswerByWebKitItself() async {
        await create(1)
        await load(1, site.url("/shows-staff-picture"))

        // WebKit does not ask on behalf of a picture from another server: nothing is reported,
        // and the picture is refused. (An engine that did ask would be told the same by the
        // core's rule, 017 §3.)
        await wait(1, "the page to say its picture was refused; it last said \"\(lastTitle(1))\"") { self.hasTitle("picture refused")($0) }
        XCTAssertEqual(asked().count, 0)
    }

    func testAWebViewDestroyedWhileAServerWaitsIsGivenItsAnswer() async {
        await create(1)
        await load(1, site.url("/staff"))
        let asked = await awaitAsked()

        // WebKit stops an app that never answers it: destroying the web view answers.
        await destroy(1)

        // A word for it afterwards is for nothing, and is acknowledged.
        await answer(asked, name: "ada", password: "lovelace")
        XCTAssertEqual(self.asked().count, 1)
    }

    func testAWebViewThatIsDestroyedGivesItsServersNoAnswerAndNotAnothers() async {
        // At the seam, where what WebKit is answered can be seen: a server asks in each of
        // two web views, and one of the web views is destroyed.
        let server = URLProtectionSpace(
            host: "staff.example.com",
            port: 443,
            protocol: "https",
            realm: "Staff",
            authenticationMethod: NSURLAuthenticationMethodHTTPBasic
        )
        var answers: [UInt64: String] = [:]
        for webView in [UInt64(4), 5, 6] {
            let before = core.reports.count
            Task {
                let typed = await self.engine.signIns.ask(webViewID: webView, server: server, heading: "", again: false)
                answers[webView] = typed.map { "\($0.name) / \($0.password)" } ?? "no answer"
            }
            while core.reports.count == before {
                await Task.yield()
            }
        }

        var destroy = WTWebViewDestroy()
        destroy.webViewID = 4
        _ = await engine.answer(.webViewDestroy(destroy))
        for _ in 0 ..< 200 where answers[4] == nil {
            await Task.yield()
        }

        XCTAssertEqual(answers[4], "no answer")
        XCTAssertNil(answers[5], "another web view's server still waits")
        // And is answered by the core's word, when it comes: the name and the password, each
        // as itself.
        let waiting = core.reports.compactMap { event -> UInt64? in
            if case let .signInRequested(asked)? = event.kind, event.webViewID == 5 {
                asked.requestID
            } else {
                nil
            }
        }
        var answer = WTSignInAnswer()
        answer.requestID = waiting.last ?? 0
        answer.accepted = true
        answer.name = "ada"
        answer.password = "love lace"
        _ = await engine.answer(.signInAnswer(answer))
        for _ in 0 ..< 200 where answers[5] == nil {
            await Task.yield()
        }
        XCTAssertEqual(answers[5], "ada / love lace")
        // The core's no is no answer, whatever came with it.
        let refused = core.reports.compactMap { event -> UInt64? in
            if case let .signInRequested(asked)? = event.kind, event.webViewID == 6 {
                asked.requestID
            } else {
                nil
            }
        }
        answer.requestID = refused.last ?? 0
        answer.accepted = false
        _ = await engine.answer(.signInAnswer(answer))
        for _ in 0 ..< 200 where answers[6] == nil {
            await Task.yield()
        }
        XCTAssertEqual(answers[6], "no answer")
    }

    func testAServerIsWrittenAsAnOriginWithoutThePortItsSchemeImplies() {
        func server(_ scheme: String, _ host: String, _ port: Int) -> URLProtectionSpace {
            URLProtectionSpace(
                host: host,
                port: port,
                protocol: scheme,
                realm: "Staff",
                authenticationMethod: NSURLAuthenticationMethodHTTPBasic
            )
        }
        XCTAssertEqual(SignIns.origin(of: server("https", "example.com", 443)), "https://example.com")
        XCTAssertEqual(SignIns.origin(of: server("http", "example.com", 80)), "http://example.com")
        XCTAssertEqual(SignIns.origin(of: server("https", "example.com", 8443)), "https://example.com:8443")
        XCTAssertEqual(SignIns.origin(of: server("http", "example.com", 443)), "http://example.com:443")
        XCTAssertEqual(SignIns.origin(of: server("http", "::1", 8080)), "http://[::1]:8080")
        // A name and a password answer these; a certificate is another question.
        XCTAssertTrue(SignIns.isForAPassword(server("https", "example.com", 443)))
        let trust = URLProtectionSpace(
            host: "example.com",
            port: 443,
            protocol: "https",
            realm: nil,
            authenticationMethod: NSURLAuthenticationMethodServerTrust
        )
        XCTAssertFalse(SignIns.isForAPassword(trust))
        let certificate = URLProtectionSpace(
            host: "example.com",
            port: 443,
            protocol: "https",
            realm: nil,
            authenticationMethod: NSURLAuthenticationMethodClientCertificate
        )
        XCTAssertFalse(SignIns.isForAPassword(certificate))
    }
}

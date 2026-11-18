// The adapter suite's WebKit run (000 §8.3): the behaviours every engine adapter must show,
// against a real WKWebView and a site on the loopback address. The core is scripted, so what
// is tested is the adapter alone: what it reports, in what order, and what it keeps apart.
import WebKit
import WTContract
import WTCoreClient
import WTWebKit
import XCTest

@MainActor
final class WebKitAdapterTests: AdapterTestCase {
    // MARK: Event order

    func testALoadReportsStartedThenCommittedThenFinished() async {
        await create(1)
        let answer = await load(1, site.url("/"))
        guard case .ack = answer else { return XCTFail("a load is acknowledged") }
        await wait(1, "the page to finish") { self.finished($0) && self.hasTitle("Home")($0) }

        let order = names(reports(1)).filter { ["started", "committed", "finished"].contains($0) }
        XCTAssertEqual(order, ["started", "committed", "finished"])
        guard case let .navigationStarted(started) = reports(1).first(where: {
            if case .navigationStarted = $0 {
                true
            } else {
                false
            }
        }),
            case let .navigationCommitted(committed) = reports(1)
            .first(where: {
                if case .navigationCommitted = $0 {
                    true
                } else {
                    false
                }
            })
        else { return XCTFail("expected a start and a commit") }
        XCTAssertEqual(started.url, site.url("/"))
        XCTAssertEqual(committed.url, site.url("/"))
        XCTAssertEqual(lastTitle(1), "Home")
        XCTAssertNotNil(engine.webView(for: 1))
    }

    func testARedirectCommitsWhereItLanded() async {
        await create(1)
        await load(1, site.url("/redirect"))
        await wait(1, "the redirect to land") { self.finished($0) && self.hasTitle("Landed")($0) }
        let commits = reports(1).compactMap {
            if case let .navigationCommitted(committed) = $0 {
                committed.url
            } else {
                nil
            }
        }
        XCTAssertEqual(commits, [site.url("/landed")])
    }

    func testAPageChangingItsOwnAddressReportsAnAddressChange() async {
        await create(1)
        await load(1, site.url("/push"))
        await wait(1, "the page to push its address") { reports in
            reports.contains {
                if case let .addressChanged(changed) = $0 {
                    changed.url == self.site.url("/pushed")
                } else {
                    false
                }
            }
        }
        let commits = reports(1).count(where: {
            if case .navigationCommitted = $0 {
                true
            } else {
                false
            }
        })
        XCTAssertEqual(commits, 1, "a pushed address is not a new page")
    }

    func testBackAndForwardReportHistory() async {
        await create(1)
        await load(1, site.url("/"))
        await wait(1, "the first page") { self.finished($0) }
        await load(1, site.url("/landed"))
        await wait(1, "the second page") { self.finished($0, times: 2) }
        guard case let .historyChanged(afterTwo) = reports(1).last(where: {
            if case .historyChanged = $0 {
                true
            } else {
                false
            }
        }) else {
            return XCTFail("expected history reports")
        }
        XCTAssertTrue(afterTwo.canGoBack)
        XCTAssertFalse(afterTwo.canGoForward)

        await command(1, .back)
        await wait(1, "going back") { reports in
            reports.contains {
                if case let .historyChanged(changed) = $0 {
                    !changed.canGoBack && changed.canGoForward
                } else {
                    false
                }
            }
        }
        await wait(1, "the first page's title again") { _ in self.lastTitle(1) == "Home" }
    }

    // MARK: Failures

    /// A loopback port nothing listens on: one the system just handed out and took back.
    private static func unusedPort() -> UInt16 {
        let descriptor = socket(AF_INET, SOCK_STREAM, 0)
        defer { Darwin.close(descriptor) }
        var address = sockaddr_in()
        address.sin_len = UInt8(MemoryLayout<sockaddr_in>.size)
        address.sin_family = sa_family_t(AF_INET)
        address.sin_addr.s_addr = inet_addr("127.0.0.1")
        var length = socklen_t(MemoryLayout<sockaddr_in>.size)
        _ = withUnsafeMutablePointer(to: &address) { pointer in
            pointer.withMemoryRebound(to: sockaddr.self, capacity: 1) { generic in
                _ = Darwin.bind(descriptor, generic, length)
                return getsockname(descriptor, generic, &length)
            }
        }
        return UInt16(bigEndian: address.sin_port)
    }

    func testAConnectionThatIsRefusedReportsACodeAndNoText() async {
        await create(1)
        await load(1, "http://127.0.0.1:\(Self.unusedPort())/")
        await wait(1, "the load to fail") { reports in
            reports.contains {
                if case .navigationFailed = $0 {
                    true
                } else {
                    false
                }
            }
        }
        guard case let .navigationFailed(failed) = reports(1).first(where: {
            if case .navigationFailed = $0 {
                true
            } else {
                false
            }
        })
        else { return }
        XCTAssertEqual(failed.code, "net.connect")
        XCTAssertFalse(names(reports(1)).contains("committed"))
    }

    func testANameThatDoesNotResolveReportsDNS() async {
        await create(1)
        // RFC 6761 reserves .invalid: it never resolves.
        await load(1, "http://webtiger-test.invalid/")
        await wait(1, "the load to fail") { reports in
            reports.contains {
                if case .navigationFailed = $0 {
                    true
                } else {
                    false
                }
            }
        }
        guard case let .navigationFailed(failed) = reports(1).first(where: {
            if case .navigationFailed = $0 {
                true
            } else {
                false
            }
        })
        else { return }
        XCTAssertEqual(failed.code, "net.dns")
    }

    func testEngineErrorsBecomeCodes() {
        // The mapping is the whole of what the shell knows about a failure; no engine text passes it.
        let cases: [(Int, String)] = [
            (NSURLErrorCancelled, "net.cancelled"), (NSURLErrorNotConnectedToInternet, "net.offline"),
            (NSURLErrorCannotFindHost, "net.dns"), (NSURLErrorCannotConnectToHost, "net.connect"),
            (NSURLErrorTimedOut, "net.timeout"), (NSURLErrorServerCertificateUntrusted, "net.tls"),
            (NSURLErrorBadServerResponse, "net.failed"),
        ]
        for (code, expected) in cases {
            XCTAssertEqual(WebKitEngine.code(for: NSError(domain: NSURLErrorDomain, code: code)), expected)
        }
        XCTAssertEqual(WebKitEngine.code(for: NSError(domain: "WebKitErrorDomain", code: 102)), "net.cancelled")
        XCTAssertEqual(WebKitEngine.code(for: NSError(domain: "SomethingElse", code: 1)), "net.failed")
    }

    // MARK: The navigation policy

    func testANavigationTheCoreRefusesNeverCommits() async {
        core.setVerdict(.cancel, for: site.url("/landed"))
        await create(1)
        await load(1, site.url("/landed"))
        await wait(1, "the refusal") { reports in
            reports.contains {
                if case let .navigationFailed(failed) = $0 {
                    failed.code == "net.cancelled"
                } else {
                    false
                }
            }
        }
        XCTAssertFalse(names(reports(1)).contains("committed"))
        XCTAssertFalse(names(reports(1)).contains("address"), "an address that never committed is never reported as the page's")
    }

    func testAnExternalAddressIsNotLoaded() async {
        core.setVerdict(.external, for: "mailto:ada@example.com")
        await create(1)
        await load(1, site.url("/"))
        await wait(1, "the page") { self.finished($0) }
        await load(1, "mailto:ada@example.com")
        try? await Task.sleep(for: .milliseconds(300))
        let commits = reports(1).compactMap {
            if case let .navigationCommitted(committed) = $0 {
                committed.url
            } else {
                nil
            }
        }
        XCTAssertEqual(commits, [site.url("/")], "the page on screen is still the page on screen")
        XCTAssertFalse(names(reports(1)).contains("address"))
        XCTAssertEqual(names(reports(1)).last, "failed(net.cancelled)", "the refused navigation is reported as ended")
    }

    func testAnAddressIsReportedOnlyOnceItHasCommitted() async {
        await create(1)
        await load(1, site.url("/"))
        await wait(1, "the first page") { self.finished($0) }
        await load(1, site.url("/redirect"))
        await wait(1, "the second page") { self.finished($0, times: 2) }
        // Every address reported as the page's is one that committed: never the address asked
        // for, and never the redirect it passed through.
        let addresses = reports(1).compactMap { report -> String? in
            switch report {
            case let .navigationCommitted(committed): committed.url
            case let .addressChanged(changed): changed.url
            default: nil
            }
        }
        XCTAssertEqual(addresses, [site.url("/"), site.url("/landed")])
    }

    // MARK: Host requests

    func testRequestsAboutWebViewsThatDoNotExistAreRefusedWithCodes() async {
        func code(_ answer: WTHostResponse.OneOf_Kind) -> String {
            if case let .error(error) = answer {
                error.code
            } else {
                ""
            }
        }
        let loaded = await load(42, site.url("/"))
        XCTAssertEqual(code(loaded), "webview.unknown")
        let chromium = await create(1, engine: .chromium)
        XCTAssertEqual(code(chromium), "engine.unavailable")
        await create(2)
        let twice = await create(2)
        XCTAssertEqual(code(twice), "webview.exists")
        var bad = WTWebViewCreate()
        bad.webViewID = 3
        bad.engine = .webkit
        bad.loginSet.id = "not-a-uuid"
        let refused = await engine.answer(.webViewCreate(bad))
        XCTAssertEqual(code(refused), "webview.bad_login_set")
    }

    func testDestroyingAWebViewReleasesIt() async {
        await create(1)
        XCTAssertEqual(engine.webViewCount, 1)
        weak let webView = engine.webView(for: 1)
        await destroy(1)
        XCTAssertEqual(engine.webViewCount, 0)
        XCTAssertNil(engine.webView(for: 1))
        // Destroying it again is not an error.
        var again = WTWebViewDestroy()
        again.webViewID = 1
        guard case .ack = await engine.answer(.webViewDestroy(again)) else { return XCTFail("destroy is idempotent") }
        for _ in 0 ..< 100 where webView != nil {
            try? await Task.sleep(for: .milliseconds(10))
        }
        XCTAssertNil(webView, "the engine's resources are released, not just forgotten")
    }

    func testThePingIsAnswered() async {
        var ping = WTHostPing()
        ping.payload = Data("hello".utf8)
        guard case let .pong(pong) = await engine.answer(.ping(ping)) else { return XCTFail("expected a pong") }
        XCTAssertEqual(pong.payload, Data("hello".utf8))
    }
}

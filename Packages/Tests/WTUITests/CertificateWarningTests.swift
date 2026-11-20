// Going on past a certificate that is not trusted (026 §6), the model against a scripted core:
// what the failed page is given to offer when the core says the load may be gone on with, that
// nothing goes on by one press, and that the warning is one failure's.
import WTContract
import WTCoreClient
@testable import WTUI
import WTWebKit
import XCTest

@MainActor
final class CertificateWarningTests: XCTestCase {
    /// A model over a core that opens window 1 with tab 2, and says Ack to the rest.
    private func opened() async -> (BrowserModel, ScriptedCore) {
        let core = ScriptedCore { request in
            switch request {
            case .openWindow:
                var opened = WTWindowOpened()
                opened.windowID = 1
                opened.tabID = 2
                return .windowOpened(opened)
            case .subscribe:
                var subscribed = WTSubscribed()
                subscribed.subscriptionID = 3
                return .subscribed(subscribed)
            default:
                return .ack(WTAck())
            }
        }
        let model = BrowserModel(session: BrowserSession(core: core, engine: WebKitEngine()))
        await model.open()
        return (model, core)
    }

    /// What the core says of the tab in front.
    private struct Said {
        var state = WTLoadState.failed
        var passable = true
        var host = "router.example.com"
        var code = "net.tls.untrusted"
        var tab: UInt64 = 2
        /// A core names the host with what may be gone on with; one that named it without is
        /// still not to be believed about the warning.
        var namesHost: Bool?
    }

    /// The core publishes a snapshot, and the model has taken it.
    private func says(_ said: Said = Said(), by core: ScriptedCore, to model: BrowserModel) async {
        var window = WTWindowSnapshot()
        window.windowID = 1
        window.tab.tabID = said.tab
        window.tab.loadState = said.state
        window.tab.address.url = "https://\(said.host)/"
        window.tab.address.display = said.host
        if said.state == .failed {
            window.tab.errorCode = said.code
            window.tab.certificatePassable = said.passable
            window.tab.certificateHost = said.namesHost ?? said.passable ? said.host : ""
        }
        // Every snapshot differs from the one before, so that the model's taking it can be seen.
        published += 1
        window.tab.progress = Double(published) / 1000
        core.emit(window, subscription: 3)
        let progress = window.tab.progress
        for _ in 0 ..< 500 where model.tab.progress != progress {
            await Task.yield()
            try? await Task.sleep(for: .milliseconds(2))
        }
        XCTAssertEqual(model.tab.progress, progress, "the model took the snapshot")
    }

    private var published = 0

    /// The tabs the core was asked to go on in, in order.
    private func goneOn(_ core: ScriptedCore) -> [UInt64] {
        core.requests.compactMap {
            if case let .proceedPastCertificate(proceed) = $0 {
                proceed.tabID
            } else {
                nil
            }
        }
    }

    /// Lets the model's request tasks run.
    private func settle(until done: () -> Bool = { false }) async {
        for _ in 0 ..< 200 where !done() {
            await Task.yield()
            try? await Task.sleep(for: .milliseconds(2))
        }
    }

    func testNothingGoesOnByOnePressAndTheWarningComesFirst() async throws {
        let (model, core) = await opened()
        await says(by: core, to: model)

        // The failed page is given the way to say more, and nothing that goes on.
        XCTAssertTrue(model.canGoOnPastCertificate)
        var offered = try XCTUnwrap(model.goingOn)
        XCTAssertEqual(offered.host, "router.example.com")
        XCTAssertFalse(offered.isWarned)

        offered.advanced()

        // The warning is open, and the core was asked nothing.
        XCTAssertTrue(model.isWarnedOfCertificate)
        offered = try XCTUnwrap(model.goingOn)
        XCTAssertTrue(offered.isWarned)
        await settle()
        XCTAssertEqual(goneOn(core), [])

        offered.goOn()
        await settle { !self.goneOn(core).isEmpty }

        // The core is asked, for the tab in front, once; what comes of it is the core's.
        XCTAssertEqual(goneOn(core), [2])
        XCTAssertFalse(model.isWarnedOfCertificate)
    }

    func testAFailureTheCoreDoesNotSayMayBeGoneOnWithOffersNoWayOn() async {
        let (model, core) = await opened()
        // A certificate's trouble, but not one to go on past (the adapter holds nothing); a
        // failure that is no certificate's; and a page that loaded.
        for said in [Said(passable: false), Said(passable: false, code: "net.dns"), Said(state: .idle)] {
            await says(said, by: core, to: model)
            XCTAssertFalse(model.canGoOnPastCertificate)
            XCTAssertNil(model.goingOn)

            // And the model asks the core nothing, whoever calls it.
            model.warnOfCertificate()
            XCTAssertFalse(model.isWarnedOfCertificate)
            model.goOnPastCertificate()
        }
        await settle()
        XCTAssertEqual(goneOn(core), [])
    }

    func testWithoutTheWarningOnTheScreenTheCoreIsNotAsked() async {
        let (model, core) = await opened()
        await says(by: core, to: model)

        model.goOnPastCertificate()
        await settle()

        XCTAssertEqual(goneOn(core), [], "going on is only ever the warning's button")
    }

    func testTheWarningIsOneFailuresAndAnotherStartsAgain() async {
        let (model, core) = await opened()
        await says(by: core, to: model)
        model.warnOfCertificate()
        XCTAssertTrue(model.isWarnedOfCertificate)

        // The same failure said again (a snapshot for something else) leaves the warning open.
        await says(by: core, to: model)
        XCTAssertTrue(model.isWarnedOfCertificate)

        // Another load of the tab, which fails the same way: the warning is not the new failure's.
        await says(Said(state: .loading), by: core, to: model)
        XCTAssertFalse(model.isWarnedOfCertificate)
        await says(by: core, to: model)
        XCTAssertFalse(model.isWarnedOfCertificate)
        model.warnOfCertificate()

        // A failure there that may not be gone on with ends it too, whatever host a core names.
        await says(Said(passable: false, namesHost: true), by: core, to: model)
        await says(by: core, to: model)
        XCTAssertFalse(model.isWarnedOfCertificate, "the warning was the failure before's")
        model.warnOfCertificate()

        // Another site's certificate, in the same tab.
        await says(Said(host: "printer.example.com"), by: core, to: model)
        XCTAssertFalse(model.isWarnedOfCertificate, "the warning named another site")
        model.warnOfCertificate()
        XCTAssertEqual(model.goingOn?.host, "printer.example.com")

        // Another tab in front, with the same trouble.
        await says(Said(host: "printer.example.com", tab: 7), by: core, to: model)
        XCTAssertFalse(model.isWarnedOfCertificate, "the warning was another tab's")
        await settle()
        XCTAssertEqual(goneOn(core), [])
    }

    /// The warning's words name the site, and the button that goes on says what it does.
    func testTheWordsNameTheSiteAndTheButtonSaysItIsNotSafe() {
        let words = { (key: String) in String(
            format: Bundle.module.localizedString(forKey: key, value: nil, table: nil),
            "router.example.com"
        ) }
        XCTAssertEqual(words("certificate.proceed"), "Go On to router.example.com (Not Safe)")
        XCTAssertTrue(words("certificate.warning").hasPrefix("WebTiger can’t confirm that this is really router.example.com."))
        XCTAssertEqual(words("certificate.advanced"), "Advanced…")
        XCTAssertEqual(words("certificate.warning.title"), "Going on isn’t safe")
    }
}

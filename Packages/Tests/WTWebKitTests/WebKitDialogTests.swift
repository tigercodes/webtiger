// The adapter suite's WebKit run, for what a page says or asks (016): alert, confirm and
// prompt, called by a page of the test site in a real web view. What the adapter reports,
// that the page's script waits for the core's word, and what each word gives the page.
import WebKit
import WTContract
import WTCoreClient
@testable import WTWebKit
import XCTest

@MainActor
final class WebKitDialogTests: AdapterTestCase {
    /// A web view on the page that says and asks by its buttons.
    private func page(_ id: UInt64 = 1) async {
        await create(id)
        await load(id, site.url("/says"))
        await wait(id, "the page to load") { self.finished($0) && self.hasTitle("Says")($0) }
    }

    /// The dialogs the adapter has reported of a web view's pages, in order.
    private func said(_ id: UInt64 = 1) -> [WTDialogRequested] {
        reports(id).compactMap {
            if case let .dialogRequested(dialog) = $0 {
                dialog
            } else {
                nil
            }
        }
    }

    /// Presses one of the page's buttons, and waits for what it says to be reported. The
    /// press is made a moment later, from a timer of the page's: the page's script stops at
    /// what it says, and a script that waited for the press would stop with it.
    private func press(_ button: String, in id: UInt64 = 1) async -> WTDialogRequested {
        let before = said(id).count
        await run("setTimeout(() => document.getElementById('\(button)').click(), 0)", in: id)
        for _ in 0 ..< Patience.steps where said(id).count == before {
            try? await Task.sleep(for: .milliseconds(10))
        }
        XCTAssertGreaterThan(said(id).count, before, "the page said something")
        return said(id).last ?? WTDialogRequested()
    }

    /// The core's word on a dialog.
    private func answer(_ dialog: WTDialogRequested, accepted: Bool, text: String = "") async {
        var answer = WTDialogAnswer()
        answer.requestID = dialog.requestID
        answer.accepted = accepted
        answer.text = text
        guard case .ack = await engine.answer(.dialogAnswer(answer)) else {
            return XCTFail("a word about a dialog is acknowledged")
        }
    }

    /// Waits for the page to say, in its title, what it got.
    private func got(_ words: String, in id: UInt64 = 1) async {
        await wait(id, "the page to say \"\(words)\"; it last said \"\(lastTitle(id))\"") { self.hasTitle(words)($0) }
    }

    /// The page's own origin, as an origin is written.
    private var origin: String {
        String(site.url("/").dropLast())
    }

    func testAnAlertIsReportedAndThePagesScriptWaitsForTheCoresWord() async {
        await page()

        let alert = await press("alert")

        XCTAssertEqual(alert.kind, .alert)
        XCTAssertEqual(alert.message, "Your basket is empty.")
        XCTAssertEqual(alert.origin, origin, "who called: the page's origin")
        XCTAssertNotEqual(alert.requestID, 0, "named by the shell")
        // The script waits: the line after the alert has not run.
        try? await Task.sleep(for: .milliseconds(500))
        XCTAssertEqual(lastTitle(1), "Says")

        await answer(alert, accepted: true)

        await got("alert returned")
    }

    func testAConfirmIsTrueOrFalseAsTheCoreSays() async {
        await page()

        let first = await press("confirm")
        XCTAssertEqual(first.kind, .confirm)
        XCTAssertEqual(first.message, "Delete it?")
        await answer(first, accepted: true)
        await got("confirm true")

        let second = await press("confirm")
        XCTAssertNotEqual(second.requestID, first.requestID, "each has a name of its own")
        await answer(second, accepted: false)
        await got("confirm false")
    }

    func testAPromptOffersItsTextAndGetsTheAnswerOrNothing() async {
        await page()

        let first = await press("prompt")
        XCTAssertEqual(first.kind, .prompt)
        XCTAssertEqual(first.message, "Your name?")
        XCTAssertEqual(first.defaultText, "Ada", "the text the page offers")
        await answer(first, accepted: true, text: "Grace Hopper")
        await got("prompt Grace Hopper")

        await answer(press("prompt"), accepted: false)
        await got("prompt nothing")
    }

    func testAFrameFromAnotherOriginIsReportedAsItself() async {
        await create(1)
        await load(1, site.url("/says-in-a-frame?now=confirm"))

        await wait(1, "the frame to ask") { _ in !self.said().isEmpty }

        // The frame is the same server by its other name: another origin than the page's.
        // The adapter says who called; whether a frame may is the core's rule (016 §3).
        let asked = said().last ?? WTDialogRequested()
        XCTAssertEqual(asked.origin, "http://localhost:\(site.port)")
        XCTAssertEqual(asked.kind, .confirm)
        await answer(asked, accepted: false)
        await got("frame: confirm false")
    }

    func testAWebViewDestroyedWhileItsPageWaitsIsGivenItsAnswer() async {
        await page()
        let alert = await press("alert")

        // WebKit stops an app that never answers it: destroying the web view answers.
        await destroy(1)

        // A word for it afterwards is for nothing, and is acknowledged.
        await answer(alert, accepted: true)
        XCTAssertEqual(said().count, 1)
    }

    func testAWebViewThatIsDestroyedDismissesWhatItsPagesSaidAndNotAnothers() async {
        // At the seam, where what WebKit is answered can be seen: a page of each of two web
        // views asks, and one of the web views is destroyed.
        var answers: [UInt64: Bool] = [:]
        for webView in [UInt64(4), 5] {
            let before = core.reports.count
            Task {
                let answer = await self.engine.dialogs.ask(webViewID: webView, origin: self.origin, kind: .confirm, message: "Leave?")
                answers[webView] = answer.accepted
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

        XCTAssertEqual(answers[4], false, "its page is answered as if the person had closed it")
        XCTAssertNil(answers[5], "another web view's page still waits")
        // And is answered by the core's word, when it comes.
        let waiting = core.reports.compactMap { event -> UInt64? in
            if case let .dialogRequested(dialog)? = event.kind, event.webViewID == 5 {
                dialog.requestID
            } else {
                nil
            }
        }
        var answer = WTDialogAnswer()
        answer.requestID = waiting.last ?? 0
        answer.accepted = true
        _ = await engine.answer(.dialogAnswer(answer))
        for _ in 0 ..< 200 where answers[5] == nil {
            await Task.yield()
        }
        XCTAssertEqual(answers[5], true)
    }

    func testAWordAboutADialogThereIsNot() async {
        var answer = WTDialogAnswer()
        answer.requestID = 99
        answer.accepted = true
        guard case .ack = await engine.answer(.dialogAnswer(answer)) else {
            return XCTFail("acknowledged, as for one whose page went away")
        }
    }
}

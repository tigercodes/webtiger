// What a page says or asks (016), against a scripted core: when the model has it to show, and
// what each answer sends.
import WTContract
import WTCoreClient
@testable import WTUI
import WTWebKit
import XCTest

@MainActor
final class DialogModelTests: XCTestCase {
    /// A model over a core that opens a window and says Ack to the rest.
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

    /// A window's snapshot whose page of example.com says `message`, or asks, under the
    /// core's name `prompt`.
    private func saying(
        _ prompt: UInt64,
        _ kind: WTDialogKind,
        _ message: String,
        offered: String = "",
        maySilence: Bool = false
    ) -> WTWindowSnapshot {
        var window = WTWindowSnapshot()
        window.windowID = 1
        window.tab.tabID = 2
        window.tab.webViewID = 4
        window.tab.address.url = "https://example.com/"
        window.tab.address.display = "example.com"
        window.tab.prompt.promptID = prompt
        window.tab.prompt.dialog.site = "example.com"
        window.tab.prompt.dialog.kind = kind
        window.tab.prompt.dialog.message = message
        window.tab.prompt.dialog.defaultText = offered
        window.tab.prompt.dialog.maySilence = maySilence
        return window
    }

    /// Lets the session's event task and the model's request tasks run.
    private func settle(until done: () -> Bool = { false }) async {
        for _ in 0 ..< 200 where !done() {
            await Task.yield()
            try? await Task.sleep(for: .milliseconds(2))
        }
    }

    /// What was sent as answers, each in a few words.
    private func answers(_ core: ScriptedCore) -> [String] {
        core.requests.compactMap { request -> String? in
            guard case let .answerPrompt(answer) = request else { return nil }
            let what = answer.silence ? "stop" : (answer.allow ? "ok" : "cancel")
            return "prompt \(answer.promptID): \(what)" + (answer.text.isEmpty ? "" : " \"\(answer.text)\"")
        }
    }

    func testTheModelHasWhatThePageSaysAndEachAnswerIsSentByThePromptsName() async {
        let (model, core) = await opened()

        core.emit(saying(9, .prompt, "What should we call you?", offered: "Ada", maySilence: true), subscription: 3)
        await settle { model.prompt != nil }

        guard case let .dialog(dialog)? = model.prompt?.kind else { return XCTFail("the model has the dialog") }
        XCTAssertEqual(dialog.message, "What should we call you?")
        XCTAssertEqual(dialog.defaultText, "Ada")

        model.answerPrompt(allow: true, text: "Grace Hopper")
        model.answerPrompt(allow: false)
        model.answerPrompt(allow: false, silence: true)
        await settle { self.answers(core).count == 3 }

        XCTAssertEqual(answers(core), ["prompt 9: ok \"Grace Hopper\"", "prompt 9: cancel", "prompt 9: stop"])
        // The prompt stays until the core says it is gone.
        XCTAssertEqual(model.prompt?.promptID, 9)
    }

    func testSomethingThisBuildHasNoBarForIsNotShown() async {
        let (model, core) = await opened()

        core.emit(saying(9, .unspecified, "Hello"), subscription: 3)
        await settle()
        XCTAssertNil(model.prompt)
        core.emit(saying(10, .UNRECOGNIZED(99), "Hello"), subscription: 3)
        await settle()
        XCTAssertNil(model.prompt)

        model.answerPrompt(allow: true)
        await settle()
        XCTAssertEqual(answers(core), [])
        // Each of the three it knows is shown.
        for (prompt, kind) in [(UInt64(11), WTDialogKind.alert), (12, .confirm), (13, .prompt)] {
            core.emit(saying(prompt, kind, "Hello"), subscription: 3)
            await settle { model.prompt?.promptID == prompt }
            XCTAssertEqual(model.prompt?.promptID, prompt)
        }
    }

    func testThePageIsSaidToWaitOnlyWhileWhatItSaidIsShown() async {
        let (model, core) = await opened()
        XCTAssertFalse(model.pageWaits, "nothing is asked")

        // What a page says stops its script, and with it all its process does.
        core.emit(saying(9, .confirm, "Delete it?"), subscription: 3)
        await settle { model.pageWaits }
        XCTAssertTrue(model.pageWaits)

        // A page that asks to use the camera goes on running; so does one a server keeps
        // waiting, and one that wants to download.
        var asking = saying(10, .confirm, "")
        asking.tab.prompt = WTPromptView()
        asking.tab.prompt.promptID = 10
        asking.tab.prompt.permission.site = "example.com"
        asking.tab.prompt.permission.kinds = [.camera]
        core.emit(asking, subscription: 3)
        await settle { model.prompt?.promptID == 10 }
        XCTAssertFalse(model.pageWaits)
        asking.tab.prompt = WTPromptView()
        asking.tab.prompt.promptID = 11
        asking.tab.prompt.signIn.site = "example.com"
        core.emit(asking, subscription: 3)
        await settle { model.prompt?.promptID == 11 }
        XCTAssertFalse(model.pageWaits)

        // Answered and gone, the page is itself again.
        asking.tab.clearPrompt()
        core.emit(asking, subscription: 3)
        await settle { model.prompt == nil }
        XCTAssertFalse(model.pageWaits)
    }

    #if os(macOS)
        func testAPageThatWaitsIsNotOfferedToAssistiveTechnologies() {
            // The view that holds the web view: what it holds is what it offers, until its
            // page waits for an answer to what it said.
            let container = WebViewContainer(frame: NSRect(x: 0, y: 0, width: 200, height: 100))
            let held = NSButton(title: "A page", target: nil, action: nil)
            container.addSubview(held)
            XCTAssertEqual(container.accessibilityChildren()?.count, 1)

            container.pageWaits = true
            XCTAssertEqual(container.accessibilityChildren()?.count, 0, "its process could not answer for it")

            container.pageWaits = false
            XCTAssertEqual(container.accessibilityChildren()?.count, 1, "answered, the page is there again")
        }
    #endif

    func testTheBarsFirstLineIsWhoSaysIt() {
        XCTAssertEqual(DialogBar.says("example.com", locale: Locale(identifier: "en_GB")), "example.com says:")
        XCTAssertEqual(DialogBar.says("http://127.0.0.1:8080", locale: Locale(identifier: "en_GB")), "http://127.0.0.1:8080 says:")
    }
}

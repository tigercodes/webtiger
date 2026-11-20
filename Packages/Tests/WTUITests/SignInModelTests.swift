// A server that asks who is there (017), against a scripted core: when the model has it to
// show, and what an answer sends.
import WTContract
import WTCoreClient
@testable import WTUI
import WTWebKit
import XCTest

@MainActor
final class SignInModelTests: XCTestCase {
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

    /// A window's snapshot whose page of example.com is left for a server that asks who is
    /// there, under the core's name `prompt`.
    private func signingIn(_ prompt: UInt64, confidential: Bool = true, again: Bool = false) -> WTWindowSnapshot {
        var window = WTWindowSnapshot()
        window.windowID = 1
        window.tab.tabID = 2
        window.tab.webViewID = 4
        window.tab.address.url = "https://example.com/"
        window.tab.address.display = "example.com"
        window.tab.prompt.promptID = prompt
        window.tab.prompt.signIn.site = "staff.example.com"
        window.tab.prompt.signIn.confidential = confidential
        window.tab.prompt.signIn.again = again
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
            let what = answer.allow ? "sign in" : "cancel"
            return "tab \(answer.tabID) prompt \(answer.promptID): \(what) \"\(answer.text)\" \"\(answer.secret)\"" +
                (answer.silence ? " stop" : "")
        }
    }

    func testTheModelHasTheServersQuestionAndTheAnswerCarriesTheNameAndThePassword() async {
        let (model, core) = await opened()

        core.emit(signingIn(9, confidential: false, again: true), subscription: 3)
        await settle { model.prompt != nil }

        guard case let .signIn(asked)? = model.prompt?.kind else { return XCTFail("the model has the server's question") }
        XCTAssertEqual(asked.site, "staff.example.com")
        XCTAssertFalse(asked.confidential)
        XCTAssertTrue(asked.again)

        model.answerPrompt(allow: true, text: "ada", secret: "love lace")
        model.answerPrompt(allow: false)
        await settle { self.answers(core).count == 2 }

        XCTAssertEqual(answers(core), ["tab 2 prompt 9: sign in \"ada\" \"love lace\"", "tab 2 prompt 9: cancel \"\" \"\""])
        // The prompt stays until the core says it is gone.
        XCTAssertEqual(model.prompt?.promptID, 9)
    }

    func testTheBarsFirstLineIsTheServerThatGetsWhatIsTyped() {
        let british = Locale(identifier: "en_GB")
        XCTAssertEqual(SignInBar.title("staff.example.com", locale: british), "Sign in to staff.example.com")
        XCTAssertEqual(SignInBar.title("http://127.0.0.1:8080", locale: british), "Sign in to http://127.0.0.1:8080")
    }
}

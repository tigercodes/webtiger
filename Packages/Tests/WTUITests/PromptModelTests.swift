// What a tab asks the person (014), against a scripted core: when the model has a prompt, what
// the bar says, and what an answer sends.
import WTContract
import WTCoreClient
@testable import WTUI
import WTWebKit
import XCTest

@MainActor
final class PromptModelTests: XCTestCase {
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

    /// A window's snapshot whose tab in front is on a page that asks for these, under the
    /// core's name `prompt`; with no kinds, a page that asks nothing.
    private func window(prompt: UInt64 = 0, _ kinds: [WTPermissionKind] = [], site: String = "meet.example.com") -> WTWindowSnapshot {
        var window = WTWindowSnapshot()
        window.windowID = 1
        window.tab.tabID = 2
        window.tab.webViewID = 4
        window.tab.address.url = "https://meet.example.com/"
        window.tab.address.display = "meet.example.com"
        if prompt != 0 {
            window.tab.prompt.promptID = prompt
            window.tab.prompt.permission.site = site
            window.tab.prompt.permission.kinds = kinds
        }
        return window
    }

    /// Lets the session's event task and the model's request tasks run.
    private func settle(until done: () -> Bool = { false }) async {
        for _ in 0 ..< 200 where !done() {
            await Task.yield()
            try? await Task.sleep(for: .milliseconds(2))
        }
    }

    private func answers(_ core: ScriptedCore) -> [String] {
        core.requests.compactMap { request -> String? in
            if case let .answerPrompt(answer) = request {
                "tab \(answer.tabID) prompt \(answer.promptID) \(answer.allow ? "allow" : "refuse")"
            } else {
                nil
            }
        }
    }

    func testTheModelHasThePromptTheCoreSaysTheTabShows() async {
        let (model, core) = await opened()
        XCTAssertNil(model.prompt)

        core.emit(window(prompt: 9, [.camera]), subscription: 3)
        await settle { model.prompt != nil }

        XCTAssertEqual(model.prompt?.promptID, 9)
        // Answered, or its page gone: the core's next snapshot has no prompt.
        core.emit(window(), subscription: 3)
        await settle { model.prompt == nil }
        XCTAssertNil(model.prompt)
    }

    func testAnAnswerNamesThePromptThatIsShown() async {
        let (model, core) = await opened()
        core.emit(window(prompt: 9, [.camera]), subscription: 3)
        await settle { model.prompt != nil }

        model.answerPrompt(allow: false)
        await settle { !self.answers(core).isEmpty }

        XCTAssertEqual(answers(core), ["tab 2 prompt 9 refuse"])
        // The prompt stays until the core says it is gone: an answer that came too soon
        // answered nothing. The next one shown has a name of its own.
        XCTAssertEqual(model.prompt?.promptID, 9)
        core.emit(window(prompt: 12, [.location]), subscription: 3)
        await settle { model.prompt?.promptID == 12 }
        model.answerPrompt(allow: true)
        await settle { self.answers(core).count == 2 }
        XCTAssertEqual(answers(core), ["tab 2 prompt 9 refuse", "tab 2 prompt 12 allow"])
    }

    func testAPromptThisBuildHasNoWordsForIsNotPutBeforeAPerson() async {
        let (model, core) = await opened()

        // One that names nothing, and one that names only what a later contract knows.
        var nothing = window(prompt: 9, [])
        core.emit(nothing, subscription: 3)
        await settle()
        XCTAssertNil(model.prompt)
        nothing.tab.prompt.permission.kinds = [.UNRECOGNIZED(99)]
        core.emit(nothing, subscription: 3)
        await settle()
        XCTAssertNil(model.prompt)

        // Nothing is sent for what is not shown.
        model.answerPrompt(allow: true)
        await settle()
        XCTAssertEqual(answers(core), [])
    }

    func testTheSentenceNamesWhoAsksInTheCoresWordsAndWhatFor() {
        let british = Locale(identifier: "en_GB")
        func words(_ kinds: [WTPermissionKind], site: String = "meet.example.com", locale: Locale = british) -> String {
            PromptBar.words(window(prompt: 9, kinds, site: site).tab.prompt, locale: locale)
        }
        XCTAssertEqual(words([.camera]), "meet.example.com wants to use your camera.")
        XCTAssertEqual(words([.microphone]), "meet.example.com wants to use your microphone.")
        XCTAssertEqual(words([.location]), "meet.example.com wants to use your location.")
        XCTAssertEqual(words([.camera, .microphone]), "meet.example.com wants to use your camera and your microphone.")
        // Joined as the language joins a list: with a comma before "and" where it has one.
        XCTAssertEqual(
            words([.camera, .microphone, .location]),
            "meet.example.com wants to use your camera, your microphone and your location."
        )
        XCTAssertEqual(
            words([.camera, .microphone, .location], locale: Locale(identifier: "en_US")),
            "meet.example.com wants to use your camera, your microphone, and your location."
        )
        // Who asks is what the core says, as it says it: a page not over https has its scheme.
        XCTAssertEqual(words([.camera], site: "http://127.0.0.1:8080"), "http://127.0.0.1:8080 wants to use your camera.")
        // A kind with no words is left out of the list.
        XCTAssertEqual(words([.camera, .UNRECOGNIZED(99)]), "meet.example.com wants to use your camera.")
    }
}

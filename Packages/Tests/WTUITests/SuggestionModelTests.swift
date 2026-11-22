// The window's model and the places offered as its address field is typed in (011), against
// a scripted core: what it asks, what it shows, and when the list goes.
import WTContract
import WTCoreClient
@testable import WTUI
import WTWebKit
import XCTest

@MainActor
final class SuggestionModelTests: XCTestCase {
    /// A window's snapshot: the tab in front and what it shows, and the tabs behind it.
    private func tabs(front: UInt64, url: String, display: String, others: [UInt64] = []) -> WTWindowSnapshot {
        var window = WTWindowSnapshot()
        window.windowID = 1
        window.tab.tabID = front
        window.tab.address.url = url
        window.tab.address.display = display
        window.tabs = ([front] + others).map { id in
            var summary = WTTabSummary()
            summary.tabID = id
            return summary
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

    /// A core that opens a window, and offers three places named after what was typed; the
    /// first of them as a tab that is open, when one is named.
    private func offering(firstIsTab: UInt64 = 0) async -> (BrowserModel, ScriptedCore) {
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
            case let .suggestAddresses(ask):
                var offered = WTAddressSuggestions()
                offered.suggestions = (1 ... 3).map { number in
                    var place = WTAddressSuggestion()
                    place.title = "\(ask.text) \(number)"
                    place.url = "https://example.com/\(ask.text)/\(number)"
                    place.display = "example.com/\(ask.text)/\(number)"
                    place.tabID = number == 1 ? firstIsTab : 0
                    return place
                }
                return .addressSuggestions(offered)
            default:
                return .ack(WTAck())
            }
        }
        let model = BrowserModel(session: BrowserSession(core: core, engine: WebKitEngine()))
        await model.open()
        return (model, core)
    }

    private func asked(_ core: ScriptedCore) -> [String] {
        core.requests.compactMap { request -> String? in
            if case let .suggestAddresses(ask) = request {
                "\(ask.windowID) \(ask.text)"
            } else {
                nil
            }
        }
    }

    func testTypingInTheAddressFieldAsksWhatToOfferAndShowsTheLastAnswer() async {
        let (model, core) = await offering()
        // What the model itself puts in the field is not typed: nothing is asked.
        XCTAssertEqual(asked(core), [])

        model.isEditingAddress = true
        model.addressText = "r"
        model.addressText = "ru"
        model.addressText = "rus"
        await settle { model.suggestions.first?.title == "rus 1" }

        XCTAssertEqual(asked(core), ["1 r", "1 ru", "1 rus"], "each letter, for the window")
        XCTAssertEqual(model.suggestions.map(\.title), ["rus 1", "rus 2", "rus 3"], "the answer to what is now in the field")
        XCTAssertNil(model.markedSuggestion)
    }

    func testTheListGoesWhenTheFieldIsEmptiedOrLosesTheKeyboard() async {
        let (model, core) = await offering()
        model.isEditingAddress = true
        model.addressText = "rus"
        await settle { !model.suggestions.isEmpty }

        model.addressText = ""
        XCTAssertEqual(model.suggestions, [], "nothing typed, nothing offered")
        XCTAssertEqual(asked(core).count, 1, "and nothing is asked for nothing")

        model.addressText = "rus"
        await settle { !model.suggestions.isEmpty }
        model.isEditingAddress = false
        XCTAssertEqual(model.suggestions, [])
        // An answer that arrives after the list was put away is for nothing.
        model.isEditingAddress = true
        model.addressText = "late"
        model.putSuggestionsAway()
        await settle()
        XCTAssertEqual(model.suggestions, [])
    }

    func testChoosingAPlaceGoesThereAndPutsTheListAway() async {
        let (model, core) = await offering()
        model.isEditingAddress = true
        model.addressText = "rus"
        await settle { !model.suggestions.isEmpty }

        model.choose(model.suggestions[1])

        guard case let .submitAddress(submit) = core.requests.last else { return XCTFail("expected SubmitAddress") }
        XCTAssertEqual(submit.text, "https://example.com/rus/2")
        XCTAssertEqual(submit.tabID, 2)
        XCTAssertFalse(model.isEditingAddress)
        XCTAssertEqual(model.suggestions, [])
    }

    /// A tab that is open (038): choosing its row brings it in front, and nothing is loaded.
    func testChoosingATabThatIsOpenBringsItInFrontAndLoadsNothing() async {
        let (model, core) = await offering(firstIsTab: 7)
        func loads() -> Int {
            core.requests.count(where: {
                if case .submitAddress = $0 {
                    true
                } else {
                    false
                }
            })
        }
        model.isEditingAddress = true
        model.addressText = "rus"
        await settle { !model.suggestions.isEmpty }
        XCTAssertEqual(model.suggestions.map(\.tabID), [7, 0, 0])

        model.choose(model.suggestions[0])

        guard case let .activateTab(activate) = core.requests.last else { return XCTFail("expected ActivateTab") }
        XCTAssertEqual(activate.tabID, 7)
        XCTAssertEqual(loads(), 0, "the page is open: nothing is loaded")
        XCTAssertFalse(model.isEditingAddress)
        XCTAssertEqual(model.suggestions, [])
        XCTAssertEqual(model.addressText, "", "what was typed is gone from the field of the tab that is left")

        // By the arrow keys and Return, the same.
        model.isEditingAddress = true
        model.addressText = "rust"
        await settle { model.suggestions.first?.title == "rust 1" }
        model.markNextSuggestion()
        model.submit()
        guard case let .activateTab(again) = core.requests.last else { return XCTFail("expected ActivateTab") }
        XCTAssertEqual(again.tabID, 7)
        XCTAssertEqual(loads(), 0)

        // And a place beside it is still gone to.
        model.isEditingAddress = true
        model.addressText = "rusty"
        await settle { model.suggestions.first?.title == "rusty 1" }
        model.choose(model.suggestions[1])
        guard case let .submitAddress(submit) = core.requests.last else { return XCTFail("expected SubmitAddress") }
        XCTAssertEqual(submit.text, "https://example.com/rusty/2")
    }

    func testTheArrowKeysMarkAPlaceAndReturnGoesThere() async {
        let (model, core) = await offering()
        model.isEditingAddress = true
        model.addressText = "rus"
        await settle { !model.suggestions.isEmpty }

        // Down from none: the first, the second, the third, and none again.
        var marks: [Int?] = []
        for _ in 0 ..< 4 {
            model.markNextSuggestion()
            marks.append(model.markedSuggestion)
        }
        XCTAssertEqual(marks, [0, 1, 2, nil])
        // Up from none: the last; and on, to the first and to none.
        marks = []
        for _ in 0 ..< 4 {
            model.markPreviousSuggestion()
            marks.append(model.markedSuggestion)
        }
        XCTAssertEqual(marks, [2, 1, 0, nil])

        // With none marked, Return sends what was typed, as it was typed.
        model.submit()
        guard case let .submitAddress(typed) = core.requests.last else { return XCTFail("expected SubmitAddress") }
        XCTAssertEqual(typed.text, "rus")

        // With one marked, Return goes there.
        model.isEditingAddress = true
        model.addressText = "rust"
        await settle { model.suggestions.first?.title == "rust 1" }
        model.markNextSuggestion()
        model.markNextSuggestion()
        model.submit()
        guard case let .submitAddress(chosen) = core.requests.last else { return XCTFail("expected SubmitAddress") }
        XCTAssertEqual(chosen.text, "https://example.com/rust/2")
    }

    func testWhatWasTypedBeforeTheWindowsFirstSnapshotStays() async {
        let (model, core) = await offering()
        // Typed at once, before the core has said what the window's tab shows.
        model.isEditingAddress = true
        model.addressText = "exam"

        core.emit(tabs(front: 2, url: "", display: ""), subscription: 3)
        await settle { model.tab.tabID == 2 }

        XCTAssertEqual(model.addressText, "exam", "the first snapshot names the first tab: it is no other tab coming in front")
    }

    func testThePadlockIsThePagesAndNotWhatWasTypeds() async {
        let (model, core) = await offering()
        var secure = tabs(front: 2, url: "https://example.com/", display: "example.com")
        secure.tab.address.security = .secure
        core.emit(secure, subscription: 3)
        await settle { model.tab.address.security == .secure }
        XCTAssertEqual(model.fieldSecurity, .secure, "the field shows the page's address")

        // With the keyboard and nothing typed, the field still shows the page's address.
        model.isEditingAddress = true
        XCTAssertEqual(model.fieldSecurity, .secure)

        model.addressText = "exa"
        XCTAssertEqual(model.fieldSecurity, .unspecified, "what was typed is nobody's page yet")

        // Given up, the field shows the page again, and its mark.
        model.isEditingAddress = false
        XCTAssertEqual(model.addressText, "example.com")
        XCTAssertEqual(model.fieldSecurity, .secure)
    }

    func testAnotherTabInFrontPutsTheListAway() async {
        let (model, core) = await offering()
        core.emit(tabs(front: 2, url: "https://example.com/", display: "example.com", others: [9]), subscription: 3)
        await settle { model.tabs.count == 2 }
        model.isEditingAddress = true
        model.addressText = "rus"
        await settle { !model.suggestions.isEmpty }

        core.emit(tabs(front: 9, url: "https://example.org/", display: "example.org", others: [2]), subscription: 3)
        await settle { model.tab.tabID == 9 }

        XCTAssertEqual(model.suggestions, [], "they were offered for what was typed for the other tab")
        XCTAssertEqual(model.addressText, "https://example.org/")
    }

    func testTypingAgainUnmarksWhatTheArrowsMarked() async {
        let (model, _) = await offering()
        model.isEditingAddress = true
        model.addressText = "rus"
        await settle { !model.suggestions.isEmpty }
        model.markNextSuggestion()

        model.addressText = "rust"
        await settle { model.suggestions.first?.title == "rust 1" }

        XCTAssertNil(model.markedSuggestion, "the list is another list")
    }
}

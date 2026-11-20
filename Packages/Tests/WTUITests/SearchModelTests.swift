// Where searches go (020), against a scripted core: when the model has the question to show,
// what a choice sends, and what the empty tab is told.
import WTContract
import WTCoreClient
@testable import WTUI
import WTWebKit
import XCTest

@MainActor
final class SearchModelTests: XCTestCase {
    /// What the scripted core says of where searches go, and whether it takes a choice.
    private final class Told: @unchecked Sendable {
        private let lock = NSLock()
        private var inUse = "duckduckgo"
        private var chosen = false
        private var takes = true
        private var known = true

        func set(inUse: String? = nil, chosen: Bool? = nil, takes: Bool? = nil, known: Bool? = nil) {
            lock.withLock {
                self.inUse = inUse ?? self.inUse
                self.chosen = chosen ?? self.chosen
                self.takes = takes ?? self.takes
                self.known = known ?? self.known
            }
        }

        func answer(_ request: WTRequest.OneOf_Kind) -> WTResponse.OneOf_Kind? {
            lock.withLock {
                switch request {
                case .querySearch where known:
                    var settings = WTSearchSettings()
                    settings.engines = [("duckduckgo", "DuckDuckGo", "duckduckgo.com"), ("ecosia", "Ecosia", "www.ecosia.org")]
                        .map { id, name, host in
                            var engine = WTSearchEngine()
                            engine.id = id
                            engine.name = name
                            engine.host = host
                            return engine
                        }
                    settings.chosenID = inUse
                    settings.chosen = chosen
                    return .searchSettings(settings)
                case let .chooseSearchEngine(choice):
                    guard takes else {
                        var refused = WTError()
                        refused.code = "search.unknown_engine"
                        return .error(refused)
                    }
                    inUse = choice.id
                    chosen = true
                    return .ack(WTAck())
                default:
                    return nil
                }
            }
        }
    }

    private let told = Told()

    /// A model over a core that opens a window, says where searches go, and says Ack to the rest.
    private func opened() async -> (BrowserModel, ScriptedCore) {
        let told = told
        let core = ScriptedCore { request in
            if let answer = told.answer(request) {
                return answer
            }
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
        await settle { model.search != nil }
        return (model, core)
    }

    /// Lets the model's request tasks run.
    private func settle(until done: () -> Bool = { false }) async {
        for _ in 0 ..< 200 where !done() {
            await Task.yield()
            try? await Task.sleep(for: .milliseconds(2))
        }
    }

    /// The engines chosen, as the core was told, in order.
    private func choices(_ core: ScriptedCore) -> [String] {
        core.requests.compactMap {
            if case let .chooseSearchEngine(choice) = $0 {
                choice.id
            } else {
                nil
            }
        }
    }

    func testAtAFirstRunTheQuestionIsAskedAndAChoiceAnswersIt() async {
        let (model, core) = await opened()

        XCTAssertTrue(model.asksWhereSearchesGo, "nobody has chosen: the list is before the person")
        XCTAssertEqual(model.search?.chosen, false)

        model.chooseSearchEngine("ecosia")
        await settle { !model.asksWhereSearchesGo }

        XCTAssertEqual(choices(core), ["ecosia"])
        XCTAssertFalse(model.asksWhereSearchesGo, "answered, and gone")
        XCTAssertEqual(model.searchEngineName, "Ecosia", "what the empty tab says")
    }

    func testAChoiceTheCoreRefusesLeavesTheListAsItIs() async {
        told.set(takes: false)
        let (model, core) = await opened()

        model.chooseSearchEngine("ecosia")
        await settle { self.choices(core).count == 1 }
        await settle()

        XCTAssertTrue(model.asksWhereSearchesGo)
        XCTAssertEqual(model.searchEngineName, "DuckDuckGo")

        // The same when the list was opened to change: it stays open.
        told.set(chosen: true)
        model.askWhereSearchesGo()
        await settle { model.search?.chosen == true }
        model.changeSearch()
        model.chooseSearchEngine("ecosia")
        await settle { self.choices(core).count == 2 }
        await settle()
        XCTAssertTrue(model.isChangingSearch)
        XCTAssertTrue(model.asksWhereSearchesGo)
    }

    func testWhenSomebodyHasChosenNothingIsAskedUntilTheyAskToChange() async {
        told.set(inUse: "ecosia", chosen: true)
        let (model, core) = await opened()

        XCTAssertFalse(model.asksWhereSearchesGo)
        XCTAssertEqual(model.searchEngineName, "Ecosia")

        // "Change…", and "Cancel": nothing is chosen.
        model.changeSearch()
        XCTAssertTrue(model.asksWhereSearchesGo)
        model.keepSearch()
        XCTAssertFalse(model.asksWhereSearchesGo)
        XCTAssertEqual(choices(core), [])

        // "Change…", and another engine.
        model.changeSearch()
        model.chooseSearchEngine("duckduckgo")
        await settle { model.searchEngineName == "DuckDuckGo" }
        XCTAssertEqual(choices(core), ["duckduckgo"])
        XCTAssertFalse(model.asksWhereSearchesGo)
        XCTAssertEqual(model.searchEngineName, "DuckDuckGo")
    }

    func testUntilTheCoreHasSaidNothingIsAskedAndNothingIsSaid() async {
        // A core that knows nothing of search engines: an older one.
        told.set(known: false)
        let core = ScriptedCore { _ in .ack(WTAck()) }
        let model = BrowserModel(session: BrowserSession(core: core, engine: WebKitEngine()))
        await model.open()
        await settle()

        XCTAssertNil(model.search)
        XCTAssertFalse(model.asksWhereSearchesGo)
        XCTAssertNil(model.searchEngineName)
        // Even "Change…" shows nothing there is nothing to show of.
        model.changeSearch()
        XCTAssertFalse(model.asksWhereSearchesGo)
    }

    func testTheEmptyTabsSentenceNamesTheEngine() {
        XCTAssertEqual(EmptyTabView.goesTo("Ecosia", locale: Locale(identifier: "en_GB")), "Searches go to Ecosia.")
    }
}

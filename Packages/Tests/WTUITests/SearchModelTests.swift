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
    private func opened(awaiting: Bool = true) async -> (BrowserModel, ScriptedCore) {
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
        if awaiting {
            await settle { model.search != nil }
        }
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

    func testAtAFirstRunTheDefaultIsKeptByClosingTheListAndIsNotOfferedAgain() async {
        let (model, core) = await opened()
        XCTAssertTrue(model.asksWhereSearchesGo)
        XCTAssertEqual(model.searchEngineName, "DuckDuckGo", "the default, while nobody has chosen")

        model.keepSearch()

        // Gone at once, before the core has answered: a list that was closed does not come back.
        XCTAssertFalse(model.asksWhereSearchesGo)
        await settle { model.search?.chosen == true }
        // Keeping the default is choosing it: the core is told, so that it is not offered again.
        XCTAssertEqual(choices(core), ["duckduckgo"])
        XCTAssertEqual(model.search?.chosen, true)
        XCTAssertEqual(model.searchEngineName, "DuckDuckGo")

        // Where somebody has chosen, closing the list changes nothing and tells the core nothing.
        model.changeSearch()
        XCTAssertTrue(model.asksWhereSearchesGo)
        model.keepSearch()
        XCTAssertFalse(model.asksWhereSearchesGo)
        XCTAssertEqual(choices(core), ["duckduckgo"])
    }

    func testTheMenuOfSelectedWordsNamesTheEngineASearchGoesTo() async {
        // Until the core has said where searches go, the menu has no words for a search.
        told.set(known: false)
        let (unknowing, _) = await opened(awaiting: false)
        await settle()
        XCTAssertNotNil(unknowing.session.engine.linkMenu, "the session's menu is there")
        XCTAssertNil(unknowing.session.engine.linkMenu?.searchWords)

        // Told, it names the engine in use; and the one chosen after.
        told.set(known: true)
        let (model, _) = await opened()
        XCTAssertEqual(model.session.engine.linkMenu?.searchWords, "Search with DuckDuckGo")

        model.chooseSearchEngine("ecosia")
        await settle { model.searchEngineName == "Ecosia" }
        XCTAssertEqual(model.session.engine.linkMenu?.searchWords, "Search with Ecosia")
    }

    /// The menu's words are the session's, and every model of it shares them. SwiftUI makes a
    /// window's view again whenever it likes, each time with a model that is thrown away: one
    /// that has been told nothing yet must not take the words away. (It did: in the Mac's
    /// app the menu of selected words had no search, and the app's test said so.)
    func testAModelThatIsMadeAndThrownAwayLeavesTheMenusWordsAsTheyAre() async {
        let (model, _) = await opened()
        XCTAssertEqual(model.session.engine.linkMenu?.searchWords, "Search with DuckDuckGo")

        // As SwiftUI does: the window's view again, over the same session.
        _ = BrowserWindowView(session: model.session)
        XCTAssertEqual(model.session.engine.linkMenu?.searchWords, "Search with DuckDuckGo")
        // A model alone, before it is opened; and one for a picture, which is handed where
        // searches go and was told it by no core.
        _ = BrowserModel(session: model.session)
        XCTAssertEqual(model.session.engine.linkMenu?.searchWords, "Search with DuckDuckGo")
        var elsewhere = WTSearchSettings()
        elsewhere.engines = [WTSearchEngine()]
        _ = BrowserModel(session: model.session, finding: nil, searching: elsewhere)
        XCTAssertEqual(model.session.engine.linkMenu?.searchWords, "Search with DuckDuckGo")
        // The model that is kept still shows where searches go.
        XCTAssertEqual(model.searchEngineName, "DuckDuckGo")
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

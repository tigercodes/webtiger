// The command bar (048), against a scripted core: what the window sends with each question
// (the commands it can do now, and that it shows Spaces), and what choosing a Space's row or a
// command's row does. Which rows there are for what is typed is the core's.
import WTContract
import WTCoreClient
@testable import WTUI
import WTWebKit
import XCTest

@MainActor
final class CommandBarModelTests: XCTestCase {
    /// A core that opens window 1 with tab 2, and offers back whatever rows a test gives it.
    private func model(offering rows: [WTAddressSuggestion] = []) async -> (BrowserModel, ScriptedCore) {
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
            case .suggestAddresses:
                var offered = WTAddressSuggestions()
                offered.suggestions = rows
                return .addressSuggestions(offered)
            default:
                return .ack(WTAck())
            }
        }
        let model = BrowserModel(session: BrowserSession(core: core, engine: WebKitEngine()))
        // A window's width is its own: every test says whether there is room for a sidebar.
        model.hasRoomForSidebar = false
        await model.open()
        return (model, core)
    }

    /// The window as the core says it is: tab 2 in front, with a page or without.
    private func snapshot(page: Bool = true, change: (inout WTWindowSnapshot) -> Void = { _ in }) -> WTWindowSnapshot {
        var said = WTWindowSnapshot()
        said.windowID = 1
        said.tab.tabID = 2
        if page {
            said.tab.webViewID = 4
            said.tab.address.url = "https://example.com/"
            said.tab.address.display = "example.com"
            said.tab.zoom.percent = 100
        }
        var front = WTTabSummary()
        front.tabID = 2
        front.host = page ? "example.com" : ""
        said.tabs = [front]
        change(&said)
        return said
    }

    private func settle(until done: () -> Bool = { false }) async {
        for _ in 0 ..< 200 where !done() {
            await Task.yield()
            try? await Task.sleep(for: .milliseconds(2))
        }
    }

    /// Types in the address field, and gives the last question the core was then asked.
    private func type(_ text: String, in model: BrowserModel, _ core: ScriptedCore) async -> WTSuggestAddresses {
        let before = questions(core).count
        model.isEditingAddress = true
        model.addressText = text
        await settle { self.questions(core).count > before }
        return questions(core).last ?? WTSuggestAddresses()
    }

    private func questions(_ core: ScriptedCore) -> [WTSuggestAddresses] {
        core.requests.compactMap {
            if case let .suggestAddresses(ask) = $0 {
                ask
            } else {
                nil
            }
        }
    }

    private func space(_ id: UInt64, _ name: String) -> WTAddressSuggestion {
        var row = WTAddressSuggestion()
        row.spaceID = id
        row.title = name
        return row
    }

    private func command(_ name: String, _ words: String) -> WTAddressSuggestion {
        var row = WTAddressSuggestion()
        row.command = name
        row.title = words
        return row
    }

    func testAQuestionCarriesTheCommandsThatCanBeDoneNow() async {
        let (model, core) = await model()
        // An empty tab, alone, in a window with no room for a sidebar and nothing of its own
        // to offer: a tab can be opened, and this one closed.
        core.emit(snapshot(page: false), subscription: 3)
        await settle { model.tabs.count == 1 }
        var asked = await type("n", in: model, core)
        XCTAssertEqual(asked.commands.map(\.name), ["tab.new", "tab.close"])
        XCTAssertEqual(asked.commands.map(\.words), ["New Tab", "Close Tab"], "by the words their menus have")
        XCTAssertTrue(asked.spaces, "and the window shows Spaces")

        // A page: it can be kept, made a Favourite and zoomed.
        core.emit(snapshot(), subscription: 3)
        await settle { model.hasPage }
        asked = await type("ne", in: model, core)
        XCTAssertEqual(asked.commands.map(\.name), ["tab.new", "tab.close", "site.keep", "favourite.add", "page.zoom"])

        // What the window itself can do: find, the lists, a private window; and room for a
        // sidebar. Each is offered where its menu has it.
        var found = 0
        var listed = 0
        var opened = 0
        model.windowDoes = WindowDoes(find: { found += 1 }, showHistory: { listed += 1 }, openPrivateWindow: { opened += 1 })
        model.hasRoomForSidebar = true
        asked = await type("new", in: model, core)
        XCTAssertEqual(asked.commands.map(\.name), [
            "tab.new", "window.private", "tab.close", "site.keep", "favourite.add", "layout.side", "page.find", "page.zoom",
            "history.show",
        ])
        XCTAssertEqual(asked.commands.first { $0.name == "layout.side" }?.words, "Tabs at the Side")
        XCTAssertEqual([found, listed, opened], [0, 0, 0], "nothing is done by being offered")

        // The same window with an empty tab: there is nothing to find in, to keep or to zoom.
        core.emit(snapshot(page: false), subscription: 3)
        await settle { !model.hasPage }
        asked = await type("newe", in: model, core)
        XCTAssertEqual(asked.commands.map(\.name), ["tab.new", "window.private", "tab.close", "layout.side", "history.show"])
        core.emit(snapshot(), subscription: 3)
        await settle { model.hasPage }

        // A Site that is a Favourite and has wandered, with a closed tab to bring back, tabs
        // down the side, and a tab beside it that it can change places with: each command says
        // what it would do now.
        core.emit(snapshot { said in
            said.tab.kept = true
            said.tab.favourite = true
            said.tab.away = true
            said.undo = "tab.closed"
            said.layout = .sidebar
            said.tabs[0].kept = true
            said.tabs[0].favourite = true
            said.tabs[0].stepsForward = true
            var other = WTTabSummary()
            other.tabID = 5
            other.kept = true
            said.tabs.append(other)
        }, subscription: 3)
        await settle { model.tabs.count == 2 }
        asked = await type("newer", in: model, core)
        XCTAssertEqual(asked.commands.map(\.name), [
            "tab.new", "window.private", "tab.reopen", "tab.close", "site.let_go", "favourite.remove", "site.home", "layout.top",
            "page.find", "page.zoom", "history.show", "tab.move_forward", "tab.next", "tab.previous",
        ])
        XCTAssertEqual(
            asked.commands.filter { ["site.let_go", "favourite.remove", "site.home", "layout.top"].contains($0.name) }.map(\.words),
            ["Stop Keeping", "Remove from Favourites", "Return to Site", "Tabs at the Top"]
        )
        // A Favourite's tile is in a row wherever the tabs are: it moves right, not down.
        XCTAssertEqual(asked.commands.first { $0.name == "tab.move_forward" }?.words, "Move Tab Right")
        // Where the row runs from right to left, as the window says, towards its end is left.
        model.windowDoes.rightToLeft = true
        asked = await type("newest", in: model, core)
        XCTAssertEqual(asked.commands.first { $0.name == "tab.move_forward" }?.words, "Move Tab Left")
    }

    func testAPrivateWindowAsksForNoSpacesAndBeforeTheWindowIsOpenNothingIsOffered() async {
        let (model, core) = await model()
        core.emit(snapshot { $0.privateWindow = true }, subscription: 3)
        await settle { model.isPrivate }
        let asked = await type("w", in: model, core)
        XCTAssertFalse(asked.spaces, "a private window has no Space")
        // Nothing of a private window is kept, so neither a Site nor a Favourite is offered;
        // what a page can have done to it is.
        XCTAssertEqual(asked.commands.map(\.name), ["tab.new", "tab.close", "site.keep", "page.zoom"])

        // A model whose window has not opened has no commands.
        let core2 = ScriptedCore { _ in .ack(WTAck()) }
        let unopened = BrowserModel(session: BrowserSession(core: core2, engine: WebKitEngine()))
        XCTAssertEqual(unopened.barCommands().count, 0)
    }

    func testChoosingASpacesRowShowsItAndChoosingACommandsRowDoesIt() async {
        let rows = [space(7, "Work"), command("tab.new", "New Tab"), command("page.zoom", "Zoom"), command("no.such", "Nothing")]
        let (model, core) = await model(offering: rows)
        core.emit(snapshot(), subscription: 3)
        await settle { model.hasPage }
        _ = await type("w", in: model, core)
        await settle { model.suggestions.count == 4 }

        // A Space: the core is asked to show it in this window, and the field gives up what
        // was typed.
        model.choose(model.suggestions[0])
        XCTAssertFalse(model.isEditingAddress)
        XCTAssertTrue(core.requests.contains {
            if case let .switchSpace(ask) = $0 {
                ask.windowID == 1 && ask.spaceID == 7
            } else {
                false
            }
        })
        XCTAssertEqual(model.suggestions.count, 0, "the list is put away")

        // A command: what its menu's item does.
        _ = await type("n", in: model, core)
        await settle { model.suggestions.count == 4 }
        let opened = core.requests.count {
            if case .openTab = $0 {
                true
            } else {
                false
            }
        }
        model.choose(model.suggestions[1])
        XCTAssertEqual(core.requests.count {
            if case .openTab = $0 {
                true
            } else {
                false
            }
        }, opened + 1)
        XCTAssertFalse(model.isEditingAddress)

        _ = await type("z", in: model, core)
        await settle { model.suggestions.count == 4 }
        XCTAssertFalse(model.isZooming)
        model.choose(model.suggestions[2])
        XCTAssertTrue(model.isZooming, "the zoom bar is open")

        // A command the window did not send is done by nobody; nothing is loaded for it either.
        _ = await type("x", in: model, core)
        await settle { model.suggestions.count == 4 }
        let before = core.requests.count
        model.choose(model.suggestions[3])
        XCTAssertEqual(core.requests.count, before, "nothing was asked of the core")
        XCTAssertFalse(model.isEditingAddress)
    }

    func testWhatOnlyTheWindowCanDoIsDoneByTheWindowAndACommandIsTheOneThatWasOffered() async {
        let rows = [
            command("page.find", "Find on Page"),
            command("history.show", "History and Downloads"),
            command("site.keep", "Keep as Site"),
        ]
        let (model, core) = await model(offering: rows)
        core.emit(snapshot(), subscription: 3)
        await settle { model.hasPage }
        var done: [String] = []
        model.windowDoes = WindowDoes(find: { done.append("find") }, showHistory: { done.append("history") })
        _ = await type("f", in: model, core)
        await settle { model.suggestions.count == 3 }
        model.choose(model.suggestions[0])
        XCTAssertEqual(done, ["find"])
        XCTAssertTrue(model.isFinding, "the find bar is open, and the window was asked to give it the keyboard")
        _ = await type("h", in: model, core)
        await settle { model.suggestions.count == 3 }
        model.choose(model.suggestions[1])
        XCTAssertEqual(done, ["find", "history"])

        // The command done is the one that was offered when the question was asked: the tab
        // was no Site then, and choosing "Keep as Site" keeps it.
        _ = await type("k", in: model, core)
        await settle { model.suggestions.count == 3 }
        model.choose(model.suggestions[2])
        XCTAssertTrue(core.requests.contains {
            if case let .keepTab(keep) = $0 {
                keep.tabID == 2
            } else {
                false
            }
        })
    }

    func testARowSaysWhatItIsWhenItIsNoPlace() {
        var place = WTAddressSuggestion()
        place.url = "https://example.com/"
        XCTAssertNil(SuggestionList.kind(of: place))
        var tab = place
        tab.tabID = 5
        XCTAssertEqual(SuggestionList.kind(of: tab), Words.suggestionsSwitchTab)
        XCTAssertEqual(SuggestionList.kind(of: space(7, "Work")), Words.suggestionsSpace)
        XCTAssertEqual(SuggestionList.kind(of: command("tab.new", "New Tab")), Words.suggestionsCommand)
        XCTAssertEqual(Words.suggestionsSpace.string(), "Space")
        XCTAssertEqual(Words.suggestionsCommand.string(), "Command")
    }

    func testTheLastTabOfAMacsWindowClosesTheWindow() async {
        let (model, core) = await model()
        core.emit(snapshot(), subscription: 3)
        await settle { model.hasPage }
        var closed = 0
        // Where the window says how it closes (a Mac), the last tab takes the window with it.
        model.windowDoes = WindowDoes(closeWindow: { closed += 1 })
        model.closeTheTabInFront()
        XCTAssertEqual(closed, 1)
        XCTAssertFalse(core.requests.contains {
            if case .closeTab = $0 {
                true
            } else {
                false
            }
        })
        // With another tab to show, the tab is closed and the window stays.
        core.emit(snapshot { said in
            var other = WTTabSummary()
            other.tabID = 5
            said.tabs.append(other)
        }, subscription: 3)
        await settle { model.tabs.count == 2 }
        model.closeTheTabInFront()
        XCTAssertEqual(closed, 1)
        XCTAssertTrue(core.requests.contains {
            if case let .closeTab(close) = $0 {
                close.tabID == 2
            } else {
                false
            }
        })
        // A Site that is asleep is no tab to be left with.
        core.emit(snapshot { said in
            var asleep = WTTabSummary()
            asleep.tabID = 6
            asleep.kept = true
            asleep.asleep = true
            said.tabs.append(asleep)
        }, subscription: 3)
        await settle { model.tabs.last?.tabID == 6 }
        model.closeTheTabInFront()
        XCTAssertEqual(closed, 2)
        // Where the window does not say (a touch screen), the tab is closed, and the core
        // opens another.
        model.windowDoes = WindowDoes()
        let before = core.requests.count {
            if case .closeTab = $0 {
                true
            } else {
                false
            }
        }
        model.closeTheTabInFront()
        XCTAssertEqual(core.requests.count {
            if case .closeTab = $0 {
                true
            } else {
                false
            }
        }, before + 1)
    }
}

// Several Spaces (024 §6.1), against a scripted core: what the window says of the Space it
// shows, what the list is, and what a press and a new Space ask of the core.
import WTContract
import WTCoreClient
import WTDesign
@testable import WTUI
import WTWebKit
import XCTest

@MainActor
final class SpacesModelTests: XCTestCase {
    /// What the scripted core lists, and whether it makes a Space when asked.
    private final class Told: @unchecked Sendable {
        private let lock = NSLock()
        private var listed = [Told.space(1, "", logins: 1), Told.space(2, "Work", logins: 2)]
        private var refusal: String?
        private var renameRefusal: String?
        private var colourRefusal: String?

        private static func space(_ id: UInt64, _ name: String, logins: UInt32) -> WTSpaceSummary {
            var space = WTSpaceSummary()
            space.spaceID = id
            space.name = name
            space.logins = logins
            return space
        }

        func refuse(with code: String?) {
            lock.withLock { refusal = code }
        }

        func refuseNames(with code: String?) {
            lock.withLock { renameRefusal = code }
        }

        func refuseColours(with code: String?) {
            lock.withLock { colourRefusal = code }
        }

        func answer(_ request: WTRequest.OneOf_Kind) -> WTResponse.OneOf_Kind? {
            lock.withLock {
                switch request {
                case .querySpaces:
                    var page = WTSpacesPage()
                    page.spaces = listed
                    page.colours = ["red", "orange", "blue"]
                    return .spacesPage(page)
                case .colourSpace:
                    guard let colourRefusal else { return .ack(WTAck()) }
                    var refused = WTError()
                    refused.code = colourRefusal
                    return .error(refused)
                case let .createSpace(create):
                    if let refusal {
                        var refused = WTError()
                        refused.code = refusal
                        return .error(refused)
                    }
                    let id = UInt64(listed.count + 1)
                    listed.append(Self.space(id, create.name, logins: UInt32(id)))
                    var made = WTSpaceCreated()
                    made.spaceID = id
                    return .spaceCreated(made)
                case .renameSpace:
                    guard let renameRefusal else { return .ack(WTAck()) }
                    var refused = WTError()
                    refused.code = renameRefusal
                    return .error(refused)
                default:
                    return nil
                }
            }
        }
    }

    private let told = Told()

    /// A model over a core that opens a window, lists the Spaces, and says Ack to the rest.
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
        return (model, core)
    }

    /// Lets the model's request tasks run.
    private func settle(until done: () -> Bool = { false }) async {
        for _ in 0 ..< 200 where !done() {
            await Task.yield()
            try? await Task.sleep(for: .milliseconds(2))
        }
    }

    /// What was asked of the core about Spaces, in order, in a few words each.
    private func asked(_ core: ScriptedCore) -> [String] {
        core.requests.compactMap {
            switch $0 {
            case .querySpaces: "list"
            case let .switchSpace(ask): "show \(ask.spaceID) in \(ask.windowID)"
            case let .createSpace(ask): "make '\(ask.name)' sharing \(ask.shareLoginsWith)" +
                (ask.colour.isEmpty ? "" : " in \(ask.colour)")
            case let .renameSpace(ask): "name \(ask.spaceID) '\(ask.name)'"
            case let .colourSpace(ask): "colour \(ask.spaceID) '\(ask.colour)'"
            default: nil
            }
        }
    }

    func testTheWindowSaysWhichSpaceItShowsAndTheOneNobodyNamedHasTheShellsWord() async {
        let (model, core) = await opened()
        var window = WTWindowSnapshot()
        window.windowID = 1
        window.tab.tabID = 2
        window.spaceID = 1
        core.emit(window, subscription: 3)
        await settle { model.spaceID == 1 }
        XCTAssertEqual(model.spaceID, 1)
        XCTAssertEqual(model.spaceName, "")
        XCTAssertEqual(SpaceName.shown(model.spaceName), "Home")

        window.spaceID = 2
        window.spaceName = "Work"
        core.emit(window, subscription: 3)
        await settle { model.spaceID == 2 }
        XCTAssertEqual(model.spaceName, "Work")
        XCTAssertEqual(SpaceName.shown(model.spaceName), "Work", "a name is shown as it is")
    }

    func testTheListIsTheCoresAnswerAskedEachTimeItIsOpened() async {
        let (model, core) = await opened()
        XCTAssertNil(model.spaces, "nothing is listed until the core has said")
        XCTAssertEqual(asked(core), [], "and nothing is asked until the list is opened")

        model.openSpaces()
        await settle { model.spaces != nil }
        XCTAssertEqual(model.spaces?.map(\.name), ["", "Work"])
        XCTAssertEqual(model.spaces?.map(\.logins), [1, 2])

        model.openSpaces()
        await settle { asked(core).count == 2 }
        XCTAssertEqual(asked(core), ["list", "list"], "a list that is opened again is asked for again")
    }

    func testASpaceThatIsPressedIsAskedOfTheCore() async {
        let (model, core) = await opened()
        model.showSpace(2)
        XCTAssertEqual(asked(core), ["show 2 in 1"])
        // The model shows nothing by itself: the Space it shows is the core's word.
        XCTAssertEqual(model.spaceID, 0)
    }

    func testASpaceIsMadeWithItsNameAsTypedAndIsThenShown() async {
        let (model, core) = await opened()

        let made = await model.makeSpace(named: "  Travel ", sharingLoginsWith: nil)
        XCTAssertTrue(made)
        // The name goes as it was typed: tidying it is the core's.
        XCTAssertEqual(asked(core), ["make '  Travel ' sharing 0", "show 3 in 1"])
        XCTAssertNil(model.spaceRefusal)

        let shared = await model.makeSpace(named: "Project", sharingLoginsWith: 2)
        XCTAssertTrue(shared)
        XCTAssertEqual(Array(asked(core).suffix(2)), ["make 'Project' sharing 2", "show 4 in 1"])
    }

    func testASpaceTheCoreWillNotMakeSaysWhyAndNothingIsShown() async {
        let (model, core) = await opened()
        told.refuse(with: "space.too_many")

        let made = await model.makeSpace(named: "One more", sharingLoginsWith: nil)

        XCTAssertFalse(made)
        XCTAssertEqual(model.spaceRefusal, "space.too_many")
        XCTAssertEqual(asked(core), ["make 'One more' sharing 0"], "nothing is shown")

        // Tried again and made, it says nothing of before.
        told.refuse(with: nil)
        let again = await model.makeSpace(named: "One more", sharingLoginsWith: nil)
        XCTAssertTrue(again)
        XCTAssertNil(model.spaceRefusal)

        // And a list that is opened anew starts with nothing said.
        told.refuse(with: "space.name_empty")
        _ = await model.makeSpace(named: " ", sharingLoginsWith: nil)
        XCTAssertEqual(model.spaceRefusal, "space.name_empty")
        model.openSpaces()
        XCTAssertNil(model.spaceRefusal)
    }

    /// A model whose window the core says shows the Space `id`.
    private func showing(_ id: UInt64, named name: String) async -> (BrowserModel, ScriptedCore) {
        let (model, core) = await opened()
        var window = WTWindowSnapshot()
        window.windowID = 1
        window.tab.tabID = 2
        window.spaceID = id
        window.spaceName = name
        core.emit(window, subscription: 3)
        await settle { model.spaceID == id }
        return (model, core)
    }

    func testTheSpaceShownIsNamedWithTheNameAsTyped() async {
        let (model, core) = await showing(2, named: "Work")

        let named = await model.renameSpace(to: "  The office ")

        XCTAssertTrue(named)
        // The Space the window shows, and the name as typed: tidying it is the core's.
        XCTAssertEqual(asked(core), ["name 2 '  The office '"])
        XCTAssertNil(model.spaceRefusal)
        // The model names nothing by itself: the name it shows is the core's word.
        XCTAssertEqual(model.spaceName, "Work")
    }

    func testANameTheCoreWillNotTakeSaysWhyAndBeforeTheCoreHasSaidWhichSpaceNothingIsAsked() async {
        let (unopened, quiet) = await opened()
        let nothing = await unopened.renameSpace(to: "Anything")
        XCTAssertFalse(nothing)
        XCTAssertEqual(asked(quiet), [], "no Space is shown yet: there is none to name")

        let (model, core) = await showing(2, named: "Work")
        told.refuseNames(with: "space.name_empty")
        let named = await model.renameSpace(to: " ")
        XCTAssertFalse(named)
        XCTAssertEqual(model.spaceRefusal, "space.name_empty")
        XCTAssertEqual(asked(core), ["name 2 ' '"])

        // Tried again and taken, it says nothing of before.
        told.refuseNames(with: nil)
        let again = await model.renameSpace(to: "Office")
        XCTAssertTrue(again)
        XCTAssertNil(model.spaceRefusal)
    }

    // A Space's colour (025)

    func testTheWindowSaysItsSpacesColourAndTheListSaysTheColoursThereAre() async {
        let (model, core) = await showing(2, named: "Work")
        XCTAssertEqual(model.spaceColour, "", "none until the core says one")
        XCTAssertEqual(model.spaceColours, [], "and none to choose among until the list is asked for")

        var window = WTWindowSnapshot()
        window.windowID = 1
        window.tab.tabID = 2
        window.spaceID = 2
        window.spaceName = "Work"
        window.spaceColour = "blue"
        core.emit(window, subscription: 3)
        await settle { model.spaceColour == "blue" }
        XCTAssertEqual(model.spaceColour, "blue")

        model.openSpaces()
        await settle { !model.spaceColours.isEmpty }
        XCTAssertEqual(model.spaceColours, ["red", "orange", "blue"], "the core's, in its order")
    }

    func testASpaceIsMadeWithTheColourChosenOrWithNoneForTheCoreToChoose() async {
        let (model, core) = await opened()
        _ = await model.makeSpace(named: "Work", sharingLoginsWith: nil, colour: "blue")
        _ = await model.makeSpace(named: "Play", sharingLoginsWith: 2, colour: "")
        XCTAssertEqual(asked(core), ["make 'Work' sharing 0 in blue", "show 3 in 1", "make 'Play' sharing 2", "show 4 in 1"])
    }

    func testEditingASpaceAsksForWhatChangedTheNameFirst() async {
        let (model, core) = await showing(2, named: "Work")

        let named = await model.editSpace(SpaceEdit(name: "Office"))
        let coloured = await model.editSpace(SpaceEdit(colour: "green"))
        let none = await model.editSpace(SpaceEdit(colour: ""))
        let both = await model.editSpace(SpaceEdit(name: "Desk", colour: "red"))
        let nothing = await model.editSpace(SpaceEdit())

        XCTAssertTrue(named && coloured && none && both && nothing)
        XCTAssertEqual(asked(core), ["name 2 'Office'", "colour 2 'green'", "colour 2 ''", "name 2 'Desk'", "colour 2 'red'"])
        // The model colours nothing by itself: what it shows is the core's word.
        XCTAssertEqual(model.spaceColour, "")
    }

    func testANameTheCoreWillNotTakeLeavesTheColourUnaskedAndAColourItWillNotTakeSaysWhy() async {
        let (model, core) = await showing(2, named: "Work")
        told.refuseNames(with: "space.name_empty")
        let refused = await model.editSpace(SpaceEdit(name: " ", colour: "red"))
        XCTAssertFalse(refused)
        XCTAssertEqual(asked(core), ["name 2 ' '"], "the colour was not asked for")
        XCTAssertEqual(model.spaceRefusal, "space.name_empty")

        told.refuseNames(with: nil)
        told.refuseColours(with: "space.colour_unknown")
        let faint = await model.editSpace(SpaceEdit(colour: "mauve"))
        XCTAssertFalse(faint)
        XCTAssertEqual(model.spaceRefusal, "space.colour_unknown")
        told.refuseColours(with: nil)
        let taken = await model.editSpace(SpaceEdit(colour: "blue"))
        XCTAssertTrue(taken)
        XCTAssertNil(model.spaceRefusal)
    }

    func testEveryColourHasAWordAndAPaintAndAnotherNameHasNeither() {
        let names = Tokens.SpaceColour.named.map(\.name)
        XCTAssertEqual(names, ["red", "orange", "yellow", "green", "teal", "blue", "purple", "pink", "grey"])
        XCTAssertEqual(names.map { SpaceColour.word($0) }, ["Red", "Orange", "Yellow", "Green", "Teal", "Blue", "Purple", "Pink", "Grey"])
        for name in names {
            XCTAssertNotNil(Tokens.SpaceColour.paint(name), name)
        }
        // None, and a colour a newer core might say: no paint, and never its own text.
        for other in ["", "mauve", "Blue", "none", "chosen", "automatic"] {
            XCTAssertNil(Tokens.SpaceColour.paint(other), other)
            XCTAssertEqual(SpaceColour.word(other), "", other)
        }
    }

    #if WEBTIGER_CORE
        /// The colours the real core lists are the ones this shell has a paint and a word for:
        /// each of them, by the same names.
        func testTheShellHasAPaintAndAWordForEveryColourTheCoreLists() async throws {
            let session = try BrowserSession.live(environment: ["WEBTIGER_FAKE": "1"])
            defer { session.core.stop() }
            guard case let .spacesPage(page) = try await session.core.send(.querySpaces(WTQuerySpaces())).value else {
                return XCTFail("the core lists the Spaces")
            }
            XCTAssertEqual(page.colours, Tokens.SpaceColour.named.map(\.name))
            XCTAssertFalse(page.colours.isEmpty)
            for colour in page.colours {
                XCTAssertNotNil(Tokens.SpaceColour.paint(colour), colour)
                XCTAssertFalse(SpaceColour.word(colour).isEmpty, colour)
            }
        }
    #endif

    func testWhereThereIsNoRoomForANameASpaceIsItsInitial() {
        XCTAssertEqual(SpaceName.initial(of: "Home"), "H")
        XCTAssertEqual(SpaceName.initial(of: "work"), "W", "as a capital")
        XCTAssertEqual(SpaceName.initial(of: "élan"), "É")
        // A picture is one letter, whatever it is made of.
        XCTAssertEqual(SpaceName.initial(of: "👩‍🚀 Missions"), "👩‍🚀")
        XCTAssertEqual(SpaceName.initial(of: ""), "")
    }

    func testARefusalIsSaidInWordsAndNeverByItsCode() {
        XCTAssertEqual(SpacesView.words(for: "space.too_many"), "There are as many Spaces as there can be.")
        XCTAssertEqual(SpacesView.words(for: "space.name_empty"), "A Space needs a name.")
        XCTAssertEqual(SpacesView.words(for: "space.unknown"), "That Space isn’t there any more.")
        // A code with no words of its own, a newer core's or the store's.
        for code in ["space.random", "store.query", "space.failed", ""] {
            XCTAssertEqual(SpacesView.words(for: code), "The Space couldn’t be made.", code)
        }
        // On the page that edits a Space, it is said of changing it.
        for code in ["space.random", "store.query", "space.rename_failed", ""] {
            XCTAssertEqual(SpacesView.words(for: code, unknown: "error.space.rename_failed"), "The Space couldn’t be changed.", code)
        }
        XCTAssertEqual(SpacesView.words(for: "space.colour_unknown"), "That colour isn’t one a Space can have.")
        XCTAssertEqual(SpacesView.words(for: "space.unknown", unknown: "error.space.rename_failed"), "That Space isn’t there any more.")
    }
}

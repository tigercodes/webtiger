// Groups (041), against a scripted core: how the strip's row is made of the core's two lists,
// what a chip shows and says, and what the window asks of the core to shut a Group, to move a
// tab, to make a Group and to edit one, with what it does when the core refuses.
import WTContract
import WTCoreClient
@testable import WTUI
import WTWebKit
import XCTest

@MainActor
final class GroupModelTests: XCTestCase {
    /// A model over a core that opens window 1 with tab 2, makes Group 30 when asked, and
    /// refuses what `refused` says, by code; it says Ack to the rest.
    private func model(refusing refused: [String: String] = [:]) -> (BrowserModel, ScriptedCore) {
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
            case .querySpaces:
                var page = WTSpacesPage()
                page.colours = ["red", "blue"]
                return .spacesPage(page)
            case .makeGroup:
                if let code = refused["make"] {
                    return Self.error(code)
                }
                var made = WTGroupMade()
                made.groupID = 30
                return .groupMade(made)
            case .renameGroup:
                return refused["rename"].map(Self.error) ?? .ack(WTAck())
            case .colourGroup:
                return refused["colour"].map(Self.error) ?? .ack(WTAck())
            default:
                return .ack(WTAck())
            }
        }
        return (BrowserModel(session: BrowserSession(core: core, engine: WebKitEngine())), core)
    }

    private nonisolated static func error(_ code: String) -> WTResponse.OneOf_Kind {
        var error = WTError()
        error.code = code
        return .error(error)
    }

    private func tab(_ id: UInt64, kept: Bool = false, group: UInt64 = 0, host: String = "example.com") -> WTTabSummary {
        var summary = WTTabSummary()
        summary.tabID = id
        summary.kept = kept
        summary.groupID = group
        summary.host = host
        return summary
    }

    private func group(_ id: UInt64, _ name: String, colour: String = "", shut: Bool = false, sites: UInt32 = 0) -> WTGroupSummary {
        var summary = WTGroupSummary()
        summary.groupID = id
        summary.name = name
        summary.colour = colour
        summary.shut = shut
        summary.sites = sites
        return summary
    }

    /// The window as the core says it is: tab 2 in front with a page, these tabs and Groups.
    private func snapshot(tabs: [WTTabSummary], groups: [WTGroupSummary] = [], private isPrivate: Bool = false) -> WTWindowSnapshot {
        var said = WTWindowSnapshot()
        said.windowID = 1
        said.tab.tabID = 2
        said.tab.webViewID = 4
        said.tabs = tabs
        said.groups = groups
        said.privateWindow = isPrivate
        return said
    }

    /// What the core was asked about Groups, in order.
    private func asked(_ core: ScriptedCore) -> [String] {
        core.requests.compactMap { request -> String? in
            switch request {
            case let .makeGroup(make): "make '\(make.name)' in \(make.windowID)"
            case let .renameGroup(rename): "name \(rename.groupID) '\(rename.name)'"
            case let .colourGroup(colour): "colour \(colour.groupID) '\(colour.colour)'"
            case let .shutGroup(shut): "\(shut.shut ? "shut" : "open") \(shut.groupID)"
            case let .removeGroup(remove): "remove \(remove.groupID)"
            case let .moveToGroup(move): "move \(move.tabID) to \(move.groupID)"
            default: nil
            }
        }
    }

    /// Lets the session's event task run.
    private func settle(until done: () -> Bool) async {
        for _ in 0 ..< 200 where !done() {
            await Task.yield()
            try? await Task.sleep(for: .milliseconds(2))
        }
    }

    /// The row as a line of words: a tab by its id, a Group by its name in brackets.
    private func row(_ tabs: [WTTabSummary], _ groups: [WTGroupSummary]) -> [String] {
        StripItem.row(tabs: tabs, groups: groups).map { item in
            switch item {
            case let .tab(tab): "\(tab.tabID)"
            case let .group(group): "[\(group.name)]"
            }
        }
    }

    func testTheRowIsTheSitesInNoGroupThenEachGroupWithItsSitesThenTheOtherTabs() {
        let work = group(30, "Work")
        let house = group(31, "House", shut: true, sites: 2)
        let empty = group(32, "Later")
        // As the core lists them: a Site in no Group, Work's two, the one of House that is in
        // front (the other is left out, House being shut), and two tabs.
        let tabs = [
            tab(1, kept: true),
            tab(2, kept: true, group: 30),
            tab(3, kept: true, group: 30),
            tab(4, kept: true, group: 31),
            tab(5),
            tab(6),
        ]
        XCTAssertEqual(row(tabs, [work, house, empty]), ["1", "[Work]", "2", "3", "[House]", "4", "[Later]", "5", "6"])
        // No Groups: the tabs as they are listed. No tabs but one: the Groups before it.
        XCTAssertEqual(row(tabs, []), ["1", "2", "3", "4", "5", "6"])
        XCTAssertEqual(row([tab(5)], [work, house]), ["[Work]", "[House]", "5"])
        // A Group that is shut and has nothing shown is drawn all the same, where it stands.
        XCTAssertEqual(row([tab(1, kept: true), tab(5)], [house]), ["1", "[House]", "5"])
        // A tab that names a Group the list does not have is drawn as one in none: never lost.
        XCTAssertEqual(row([tab(7, kept: true, group: 99), tab(2, kept: true, group: 30), tab(5)], [work]), ["7", "[Work]", "2", "5"])
        // Every tab is drawn once.
        XCTAssertEqual(StripItem.row(tabs: tabs, groups: [work, house, empty]).count, tabs.count + 3)
        // Sites and Groups only: a Group with nothing to show is after the Sites in no Group.
        XCTAssertEqual(row([tab(1, kept: true)], [house]), ["1", "[House]"])
        // The tabs are never put in another order than the core's, whatever it is: a shell
        // places the chips, and nothing else.
        XCTAssertEqual(row([tab(5), tab(1, kept: true)], []), ["5", "1"])
        XCTAssertEqual(
            row([tab(5), tab(2, kept: true, group: 30), tab(1, kept: true)], [work, house]),
            ["[Work]", "[House]", "5", "2", "1"]
        )
        XCTAssertEqual(row([tab(4, kept: true, group: 31), tab(2, kept: true, group: 30)], [work, house]), ["[Work]", "[House]", "4", "2"])
    }

    func testAChipShowsItsNameAndShutHowManySitesItHolds() {
        let english = Locale(identifier: "en")
        XCTAssertEqual(GroupChip.shown(group(30, "Work", sites: 3), locale: english), "Work")
        XCTAssertEqual(GroupChip.shown(group(30, "Work", shut: true, sites: 3), locale: english), "Work (3)")
        XCTAssertEqual(GroupChip.shown(group(30, "Work", shut: true), locale: english), "Work (0)")
        XCTAssertEqual(GroupChip.said(of: group(30, "Work", sites: 3), locale: english), "Group, open")
        XCTAssertEqual(GroupChip.said(of: group(30, "Work", shut: true, sites: 3), locale: english), "Group, shut, Sites: 3")
    }

    func testAGroupIsShutOpenedAndRemovedAndATabIsMovedByAskingTheCore() async {
        let (model, core) = model()
        await model.open()
        let work = group(30, "Work")
        let house = group(31, "House", shut: true, sites: 1)
        core.emit(snapshot(tabs: [tab(2, kept: true, group: 30), tab(5), tab(6, host: "")], groups: [work, house]), subscription: 3)
        await settle { model.groups.count == 2 }
        XCTAssertEqual(model.groups.map(\.name), ["Work", "House"])
        XCTAssertEqual(model.groupOfTheTabInFront, 30)

        // A press on a chip asks for the other of what the core last said.
        model.shutOrOpen(work)
        model.shutOrOpen(house)
        XCTAssertEqual(asked(core), ["shut 30", "open 31"])
        // A tab is moved, and taken out, by asking; which tab can be is the core's to say, and
        // the model only spares the asking for a tab that has nothing to keep.
        XCTAssertTrue(model.canBeGrouped(model.tabs[0]))
        XCTAssertTrue(model.canBeGrouped(model.tabs[1]))
        XCTAssertFalse(model.canBeGrouped(model.tabs[2]), "an empty tab has nothing to keep")
        model.move(5, toGroup: 31)
        model.moveTheTabInFront(toGroup: 0)
        model.removeGroup(31)
        XCTAssertEqual(asked(core), ["shut 30", "open 31", "move 5 to 31", "move 2 to 0", "remove 31"])

        // A private window has no Groups (037): nothing of it can be put in one.
        core.emit(snapshot(tabs: [tab(2)], private: true), subscription: 3)
        await settle { model.isPrivate }
        XCTAssertFalse(model.canBeGrouped(model.tabs[0]))
        XCTAssertFalse(model.canGroupTheTabInFront)
        model.moveTheTabInFront(toGroup: 30)
        model.askForANewGroupForTheTabInFront()
        XCTAssertNil(model.groupAsked)
        XCTAssertEqual(asked(core).count, 5, "nothing more was asked")
    }

    func testANewGroupIsMadeColouredAndGivenTheTab() async {
        let (model, core) = model()
        await model.open()
        core.emit(snapshot(tabs: [tab(2), tab(5)]), subscription: 3)
        await settle { model.tabs.count == 2 }

        // The page opens for a tab, and the colours to choose among are asked for.
        model.askForANewGroup(for: 5)
        XCTAssertEqual(model.groupAsked, .new(tab: 5))
        await settle { !model.spaceColours.isEmpty }
        XCTAssertEqual(model.spaceColours, ["red", "blue"])

        // Its button: the Group is made by the name as it was typed, coloured, and the tab
        // put in it; and the page closes.
        await model.finishGroupPage(name: " Work ", colour: "blue")
        XCTAssertEqual(asked(core), ["make ' Work ' in 1", "colour 30 'blue'", "move 5 to 30"])
        XCTAssertNil(model.groupAsked)
        XCTAssertNil(model.groupRefusal)

        // With no colour chosen none is asked for.
        model.askForANewGroupForTheTabInFront()
        XCTAssertEqual(model.groupAsked, .new(tab: 2))
        await model.finishGroupPage(name: "House", colour: "")
        XCTAssertEqual(Array(asked(core).dropFirst(3)), ["make 'House' in 1", "move 2 to 30"])
    }

    func testAGroupTheCoreWillNotMakeLeavesThePageOpenWithItsReason() async {
        let (model, core) = model(refusing: ["make": "group.too_many"])
        await model.open()
        core.emit(snapshot(tabs: [tab(2)]), subscription: 3)
        await settle { model.tabs.count == 1 }

        model.askForANewGroup(for: 2)
        await model.finishGroupPage(name: "Work", colour: "red")
        XCTAssertEqual(model.groupRefusal, "group.too_many")
        XCTAssertEqual(model.groupAsked, .new(tab: 2), "the page stays")
        XCTAssertEqual(asked(core), ["make 'Work' in 1"], "nothing was coloured or moved")
        XCTAssertEqual(GroupPage.words(for: "group.too_many", new: true), "There are as many Groups as there can be.")
        XCTAssertEqual(GroupPage.words(for: "group.no_such_code", new: true), "The Group couldn’t be made.")
        XCTAssertEqual(GroupPage.words(for: "group.no_such_code", new: false), "The Group couldn’t be changed.")

        // Opened again, the page says nothing of the earlier refusal.
        model.groupAsked = nil
        model.askForANewGroup(for: 2)
        XCTAssertNil(model.groupRefusal)
    }

    func testAGroupIsEditedByWhatChangedAndARefusedNameLeavesTheColour() async {
        let (model, core) = model()
        await model.open()
        let work = group(30, "Work", colour: "red")
        core.emit(snapshot(tabs: [tab(2, kept: true, group: 30)], groups: [work]), subscription: 3)
        await settle { model.groups.count == 1 }

        // Nothing changed: nothing is asked, and the page closes.
        model.askToEdit(work)
        XCTAssertEqual(model.groupAsked, .edit(work))
        await model.finishGroupPage(name: "Work", colour: "red")
        XCTAssertEqual(asked(core), [])
        XCTAssertNil(model.groupAsked)

        // The name alone; the colour alone, also to none; both, the name first.
        model.askToEdit(work)
        await model.finishGroupPage(name: "Office", colour: "red")
        model.askToEdit(work)
        await model.finishGroupPage(name: "Work", colour: "")
        model.askToEdit(work)
        await model.finishGroupPage(name: "Office", colour: "blue")
        XCTAssertEqual(asked(core), ["name 30 'Office'", "colour 30 ''", "name 30 'Office'", "colour 30 'blue'"])

        // A name the core will not take: the colour is not asked for, and the page stays.
        let (strict, strictCore) = self.model(refusing: ["rename": "group.name_empty"])
        await strict.open()
        strictCore.emit(snapshot(tabs: [tab(2, kept: true, group: 30)], groups: [work]), subscription: 3)
        await settle { strict.groups.count == 1 }
        strict.askToEdit(work)
        await strict.finishGroupPage(name: "  ", colour: "blue")
        XCTAssertEqual(asked(strictCore), ["name 30 '  '"])
        XCTAssertEqual(strict.groupRefusal, "group.name_empty")
        XCTAssertEqual(strict.groupAsked, .edit(work))
    }
}

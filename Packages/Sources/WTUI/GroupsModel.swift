public import WTContract

// Groups (041), the model's part: what the window asks of the core about the Groups of the row
// it shows, and the page that makes or edits one. The core decides everything here: which
// Groups there are, what is in them and what a shut one hides; a name is tidied by it, and what
// it refuses is said in its words.
import Foundation
import WTCoreClient

/// What the Group page was opened for: a new Group for a tab to be put in, or a Group there is.
public enum GroupAsk: Identifiable, Equatable, Sendable {
    case new(tab: UInt64)
    case edit(WTGroupSummary)

    public var id: String {
        switch self {
        case let .new(tab): "new-\(tab)"
        case let .edit(group): "edit-\(group.groupID)"
        }
    }
}

public extension BrowserModel {
    /// A tab can be put in a Group: it has something to keep, or is kept already, and the
    /// window is not a private one, which has no Groups (037).
    func canBeGrouped(_ tab: WTTabSummary) -> Bool {
        !isPrivate && (tab.kept || !tab.host.isEmpty)
    }

    /// The same for the tab in front: what the menu's items ask.
    var canGroupTheTabInFront: Bool {
        !isPrivate && canKeepOrLetGo
    }

    /// The Group the tab in front is in; 0 when it is in none.
    var groupOfTheTabInFront: UInt64 {
        tabs.first { $0.tabID == tab.tabID }?.groupID ?? 0
    }

    /// Shuts a Group, or opens it: the other of what the core last said of it.
    func shutOrOpen(_ group: WTGroupSummary) {
        var ask = WTShutGroup()
        ask.groupID = group.groupID
        ask.shut = !group.shut
        send(.shutGroup(ask))
    }

    /// Puts a tab in a Group; with no Group (0), takes a Site out of the one it is in. A tab
    /// that is no Site is kept by it: the core's rule.
    func move(_ tab: UInt64, toGroup group: UInt64) {
        var ask = WTMoveToGroup()
        ask.tabID = tab
        ask.groupID = group
        send(.moveToGroup(ask))
    }

    /// The same for the tab in front.
    func moveTheTabInFront(toGroup group: UInt64) {
        guard let frontTabID, canGroupTheTabInFront else { return }
        move(frontTabID, toGroup: group)
    }

    /// Takes a Group away. Its Sites stay: where is the core's to say.
    func removeGroup(_ id: UInt64) {
        var ask = WTRemoveGroup()
        ask.groupID = id
        send(.removeGroup(ask))
    }

    /// Opens the page that makes a Group, for a tab to be put in it.
    func askForANewGroup(for tab: UInt64) {
        openGroupPage(.new(tab: tab))
    }

    /// The same for the tab in front.
    func askForANewGroupForTheTabInFront() {
        guard let frontTabID, canGroupTheTabInFront else { return }
        askForANewGroup(for: frontTabID)
    }

    /// Opens the page that edits a Group.
    func askToEdit(_ group: WTGroupSummary) {
        openGroupPage(.edit(group))
    }

    private func openGroupPage(_ ask: GroupAsk) {
        groupRefusal = nil
        // The colours to choose among are the core's, which lists them with the Spaces (025).
        if spaceColours.isEmpty {
            askForSpaces()
        }
        groupAsked = ask
    }

    /// What the Group page asked for, when its button was pressed: a new Group with that name
    /// and colour, and the tab in it; or the name and the colour of the Group it edits, each
    /// when it is another. The page closes when the core has done it, and stays, with the
    /// core's reason on it, when it would not.
    func finishGroupPage(name: String, colour: String) async {
        guard let asked = groupAsked else { return }
        let done: Bool = switch asked {
        case let .new(tab):
            await makeGroup(named: name, colour: colour, for: tab)
        case let .edit(group):
            await editGroup(group, name: name, colour: colour)
        }
        if done, groupAsked == asked {
            groupAsked = nil
        }
    }

    /// Makes a Group in the row the window shows, gives it `colour` when one was chosen, and
    /// puts `tab` in it. False when the core would not make it: `groupRefusal` then has its
    /// code, and nothing has changed.
    func makeGroup(named name: String, colour: String, for tab: UInt64) async -> Bool {
        guard let windowID else { return false }
        var ask = WTMakeGroup()
        ask.windowID = windowID
        ask.name = name
        guard case let .groupMade(made)? = await answer(to: .makeGroup(ask), otherwise: "group.failed") else { return false }
        if !colour.isEmpty {
            var paint = WTColourGroup()
            paint.groupID = made.groupID
            paint.colour = colour
            send(.colourGroup(paint))
        }
        move(tab, toGroup: made.groupID)
        return true
    }

    /// Names a Group and gives it a colour, or none: each when it is not what the Group has.
    /// The name first; a name the core will not take leaves the colour as it was. False when
    /// the core would not: `groupRefusal` then has its code.
    func editGroup(_ group: WTGroupSummary, name: String, colour: String) async -> Bool {
        if name != group.name {
            var ask = WTRenameGroup()
            ask.groupID = group.groupID
            ask.name = name
            guard await answer(to: .renameGroup(ask), otherwise: "group.change_failed") != nil else { return false }
        }
        if colour != group.colour {
            var ask = WTColourGroup()
            ask.groupID = group.groupID
            ask.colour = colour
            guard await answer(to: .colourGroup(ask), otherwise: "group.change_failed") != nil else { return false }
        }
        return true
    }

    /// Asks the core and waits for what it says. Nothing when it refused: `groupRefusal` then
    /// has its code, or `otherwise` when it gave none.
    private func answer(to request: WTRequest.OneOf_Kind, otherwise: String) async -> WTResponse.OneOf_Kind? {
        groupRefusal = nil
        do {
            let answer = try await session.core.send(request).value
            // A core says no by throwing; one that answers with the error says the same.
            if case let .error(refused) = answer {
                groupRefusal = refused.code
                return nil
            }
            return answer
        } catch let CoreError.refused(code, _) {
            groupRefusal = code
        } catch {
            groupRefusal = otherwise
        }
        return nil
    }
}

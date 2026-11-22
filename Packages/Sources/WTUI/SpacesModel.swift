import WTContract

// Several Spaces (024, 025), the model's part: what the window asks of the core about its
// Spaces, and the words for a Space with no name of its own. The core decides everything here:
// the list is its answer, a name is tidied by it, and what it refuses is said in its words.
import Foundation
import WTCoreClient

public extension BrowserModel {
    /// Asks the core for the Spaces. Its answer is what the list then shows.
    func askForSpaces() {
        let answer = session.core.send(.querySpaces(WTQuerySpaces()))
        Task { [weak self] in
            guard case let .spacesPage(page) = try? await answer.value else { return }
            self?.spaces = page.spaces
            // The colours a Space can have (025), as the core lists them.
            self?.spaceColours = page.colours
        }
    }

    /// The window is to show another Space. The core's snapshot says when it does.
    func showSpace(_ id: UInt64) {
        guard let windowID else { return }
        var ask = WTSwitchSpace()
        ask.windowID = windowID
        ask.spaceID = id
        send(.switchSpace(ask))
    }

    /// Makes a Space, with logins of its own or those of the Space `sharing`, and shows it in
    /// the window. Its colour is `colour`, one of the core's; with none, the core gives it one
    /// that no Space has (025). False when the core would not: `spaceRefusal` then has its
    /// code, and nothing else has changed.
    func makeSpace(named name: String, sharingLoginsWith sharing: UInt64?, colour: String = "") async -> Bool {
        var ask = WTCreateSpace()
        ask.name = name
        ask.shareLoginsWith = sharing ?? 0
        ask.colour = colour
        spaceRefusal = nil
        do {
            switch try await session.core.send(.createSpace(ask)).value {
            case let .spaceCreated(made):
                showSpace(made.spaceID)
                return true
            // A core says no by throwing; one that answers with the error says the same.
            case let .error(refused):
                spaceRefusal = refused.code
            default:
                spaceRefusal = "space.failed"
            }
        } catch let CoreError.refused(code, _) {
            spaceRefusal = code
        } catch {
            spaceRefusal = "space.failed"
        }
        return false
    }

    /// Names the Space the window shows (024 §6.3). The name goes as it was typed. False when
    /// the core would not: `spaceRefusal` then has its code. The window says the new name when
    /// the core's snapshot does.
    func renameSpace(to name: String) async -> Bool {
        guard spaceID != 0 else { return false }
        var ask = WTRenameSpace()
        ask.spaceID = spaceID
        ask.name = name
        spaceRefusal = nil
        do {
            switch try await session.core.send(.renameSpace(ask)).value {
            case .ack:
                return true
            case let .error(refused):
                spaceRefusal = refused.code
            default:
                spaceRefusal = "space.rename_failed"
            }
        } catch let CoreError.refused(code, _) {
            spaceRefusal = code
        } catch {
            spaceRefusal = "space.rename_failed"
        }
        return false
    }

    /// Gives the Space the window shows a colour, one of the core's, or none (025). False
    /// when the core would not: `spaceRefusal` then has its code.
    func colourSpace(_ colour: String) async -> Bool {
        guard spaceID != 0 else { return false }
        var ask = WTColourSpace()
        ask.spaceID = spaceID
        ask.colour = colour
        spaceRefusal = nil
        do {
            switch try await session.core.send(.colourSpace(ask)).value {
            case .ack:
                return true
            case let .error(refused):
                spaceRefusal = refused.code
            default:
                spaceRefusal = "space.rename_failed"
            }
        } catch let CoreError.refused(code, _) {
            spaceRefusal = code
        } catch {
            spaceRefusal = "space.rename_failed"
        }
        return false
    }

    /// What the page that edits a Space asked for (024 §6.3, 025): a name, a colour, or both.
    /// The name first; a name the core will not take leaves the colour as it was.
    func editSpace(_ edit: SpaceEdit) async -> Bool {
        if let name = edit.name, await !renameSpace(to: name) {
            return false
        }
        if let colour = edit.colour, await !colourSpace(colour) {
            return false
        }
        return true
    }

    /// The list of Spaces is opened: it is asked for anew, and says nothing of an earlier
    /// refusal.
    func openSpaces() {
        spaceRefusal = nil
        askForSpaces()
    }
}

/// What the page that edits a Space asks for: a name, when one was typed that is not the one
/// the Space has; a colour, when another was chosen. What is `nil` stays as it is.
public struct SpaceEdit: Equatable, Sendable {
    public var name: String?
    public var colour: String?

    public init(name: String? = nil, colour: String? = nil) {
        self.name = name
        self.colour = colour
    }
}

/// A Space's name as it is shown: its own, or the shell's word for the Space the app made
/// while nobody has named it.
enum SpaceName {
    static func shown(_ name: String, locale: Locale = .current) -> String {
        name.isEmpty ? Words.spaceHome.string(locale) : name
    }

    /// The first letter of a name as it is shown, as a capital: what stands for the Space
    /// where there is no room for its name. A name that begins with a picture has the picture.
    static func initial(of shown: String) -> String {
        shown.first.map { String($0).uppercased() } ?? ""
    }
}

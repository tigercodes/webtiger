import WTContract

// Undo for what is done to the tree (044), the shell's part: the command that undoes the newest
// thing done to arrange the row a window shows, with words that say what that was. What can be
// undone, and what undoing it puts back, are the core's: it names the kind, and a shell has the
// words for it. Where it is asked from is the owner's choice (WT-D32): on a Mac the Edit menu
// and ⌘Z, when the keyboard is in no field with typing to undo; on a touch screen the menu of
// a tab, and of a Group.
import SwiftUI

public extension BrowserModel {
    /// There is something done to the row that can be undone: the core says what.
    var canUndoArrangement: Bool {
        !undoArrangementKind.isEmpty
    }

    /// Undoes the newest thing done to arrange the row the window shows. The core puts the
    /// row back; nothing is asked when it says there is nothing.
    func undoArrangement() {
        guard let windowID, canUndoArrangement else { return }
        var ask = WTUndoArrangement()
        ask.windowID = windowID
        send(.undoArrangement(ask))
    }
}

extension BrowserModel {
    /// The words of the command for what it would undo, by the core's name for it. A kind
    /// this shell has no words for is undone all the same: the command says "Undo".
    static func undoWords(for kind: String) -> Word {
        switch kind {
        case "site.kept": Words.undoSiteKept
        case "site.let_go": Words.undoSiteLetGo
        case "group.made": Words.undoGroupMade
        case "group.removed": Words.undoGroupRemoved
        case "group.renamed": Words.undoGroupRenamed
        case "group.coloured": Words.undoGroupColoured
        case "group.moved": Words.undoGroupMoved
        case "favourite.added": Words.undoFavouriteAdded
        case "favourite.removed": Words.undoFavouriteRemoved
        case "tab.moved": Words.undoTabMoved
        case "group.reordered": Words.undoGroupReordered
        default: Words.undoArrangement
        }
    }

    /// The command for a menu, while there is something to undo.
    var treeUndo: TreeUndo? {
        guard canUndoArrangement else { return nil }
        return TreeUndo(words: Self.undoWords(for: undoArrangementKind)) { [weak self] in
            self?.undoArrangement()
        }
    }
}

/// "Undo …" for a menu (044): what it says, and what it does.
struct TreeUndo {
    let words: Word
    let action: () -> Void
}

/// The command at the top of a menu, with a line under it, while there is something to undo.
struct TreeUndoItem: View {
    let undo: TreeUndo?

    var body: some View {
        if let undo {
            Button(action: undo.action) {
                Text(undo.words)
            }
            .accessibilityIdentifier("undo-arrangement")
            Divider()
        }
    }
}

// Undo (033): a tab that was closed comes back. The core keeps what was closed, says whether
// there is something to bring back, and brings it back where it was; the window asks.
import WTContract

public extension BrowserModel {
    /// A closed tab can be brought back in the Space the window shows: the core says so.
    var canReopenClosedTab: Bool {
        undoKind == "tab.closed"
    }

    /// Brings back the tab that was last closed in the Space the window shows. The core is
    /// asked to undo, and says what comes of it; nothing is asked while the window has been
    /// told there is nothing to bring back.
    func reopenClosedTab() {
        guard canReopenClosedTab, let windowID else { return }
        var undo = WTUndo()
        undo.windowID = windowID
        send(.undo(undo))
    }
}

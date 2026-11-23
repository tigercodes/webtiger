// The Mac's Undo, between typing and the tree (044, WT-D32). ⌘Z is one key: where the keyboard
// is in a field of words that has typing to undo, it undoes typing, as it does in every app;
// otherwise it undoes the newest thing done to arrange the row. Which of the two it is now is
// the system's to say: the thing that has the keyboard, and its own record of what was typed.
#if os(macOS)
    import AppKit
    import Observation

    @MainActor
    enum EditUndo {
        /// The keyboard is somewhere that has typing to undo: the thing that has it keeps a
        /// record of what was typed, and the record is not empty.
        static var hasTypingToUndo: Bool {
            NSApp.keyWindow?.firstResponder?.undoManager?.canUndo == true
        }

        /// Undoes typing where there is typing to undo, by the system's own word for it, which
        /// goes to whatever has the keyboard; otherwise does what it is given: the tree's
        /// undo, when there is one.
        static func undo(orElse tree: (() -> Void)?) {
            if hasTypingToUndo {
                NSApp.sendAction(Selector(("undo:")), to: nil, from: nil)
            } else {
                tree?()
            }
        }

        /// Does again what was undone where the keyboard is: the field's own, or the page's.
        static func redo() {
            NSApp.sendAction(Selector(("redo:")), to: nil, from: nil)
        }
    }

    /// Counts the times a menu has begun to be shown. A menu's item that reads it is worked
    /// out again as the menu opens: what the system knows and does not announce (where the
    /// keyboard is) is then read at the moment it is shown.
    @MainActor
    @Observable
    final class MenuClock {
        static let shared = MenuClock()
        private(set) var times = 0

        private init() {
            NotificationCenter.default
                .addObserver(forName: NSMenu.didBeginTrackingNotification, object: nil, queue: .main) { [weak self] _ in
                    MainActor.assumeIsolated {
                        self?.times += 1
                    }
                }
        }
    }
#endif

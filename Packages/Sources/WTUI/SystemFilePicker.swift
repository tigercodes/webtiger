// The system's file picker, for a page that asks for a file (019). Only the app itself,
// started for real, gives it to its engine: it is a dialog of the system's, which no test and
// no fake mode may put on the screen. On an iPhone WebKit shows a picker of its own.
#if os(macOS)
    public import WTWebKit
    import AppKit

    public extension FilePicker {
        /// The Mac's open panel, as a sheet of the window in front: files only, one or several
        /// as the page asks. Folders are never offered (019 D3).
        static let system = FilePicker { asked in
            let panel = NSOpenPanel()
            panel.canChooseFiles = true
            panel.canChooseDirectories = false
            panel.allowsMultipleSelection = asked.several
            guard let window = NSApp.keyWindow ?? NSApp.mainWindow else {
                // No window to hang it on: nothing is picked.
                return []
            }
            let answer = await panel.beginSheetModal(for: window)
            return answer == .OK ? panel.urls : []
        }
    }
#endif

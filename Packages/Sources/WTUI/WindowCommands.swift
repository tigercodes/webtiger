// The window's commands (034): what a browser window offers to whatever asks by key or by menu.
// On the Mac the menu bar has them, each with its key, grey when it can do nothing. On an iPad
// with a keyboard the keys are carried by buttons nobody sees.
//
// Nothing is decided here: an item calls what the window's own button calls, and the model
// asks the core.
public import SwiftUI
import WTContract

/// What a browser window offers: its model, which says what can be done now, and the few
/// things only the window itself can do.
@MainActor
public struct WindowCommands {
    let model: BrowserModel
    /// The keyboard goes to the address field, with the whole address selected.
    let openLocation: () -> Void
    /// The find bar opens, with the keyboard in it.
    let find: () -> Void
    /// The lists of what was visited and what was downloaded open.
    let showHistory: () -> Void
    /// The window closes, however many tabs it has.
    let closeWindow: () -> Void

    /// Closes the tab in front. On a window's last tab it closes the window, as it does
    /// everywhere on a Mac: the core's own rule is only that a window is never left empty
    /// (005, T1). A Site that is asleep is no tab to be left with (039): when nothing else
    /// would be left to show, the window closes too. (Otherwise ⌘W would go round for ever:
    /// the core opens an empty tab when a row's last Site is closed, and closing that tab
    /// would wake the Site again.) The Sites are kept, and are there when a window next opens.
    func closeTab() {
        let front = model.tab.tabID
        let somethingElseToShow = model.tabs.contains { $0.tabID != front && !($0.kept && $0.asleep) }
        if somethingElseToShow {
            model.closeTab(front)
        } else {
            closeWindow()
        }
    }
}

public extension FocusedValues {
    /// The commands of the browser window in front; none when no browser window is.
    @Entry var windowCommands: WindowCommands?
}

/// One command: its words, its key, whether it can be chosen now, and what it asks.
private struct Command: View {
    let words: Word
    let key: KeyEquivalent
    var modifiers: EventModifiers = .command
    let enabled: Bool
    let action: () -> Void

    init(
        _ words: Word,
        _ key: KeyEquivalent,
        _ modifiers: EventModifiers = .command,
        enabled: Bool,
        action: @escaping () -> Void
    ) {
        self.words = words
        self.key = key
        self.modifiers = modifiers
        self.enabled = enabled
        self.action = action
    }

    var body: some View {
        Button(action: action) { Text(words) }
            .keyboardShortcut(key, modifiers: modifiers)
            .disabled(!enabled)
    }
}

#if os(macOS)
    /// The Mac's menu bar, for the browser window in front (034). With none in front, every
    /// item is grey but the two that close: those close whatever window is there.
    public struct BrowserCommands: Commands {
        @FocusedValue(\.windowCommands) private var window

        public init() {}

        public var body: some Commands {
            CommandGroup(after: .newItem) { FileItems(window: window) }
            // In the place of the system's own Close, which had ⌘W.
            CommandGroup(replacing: .saveItem) { CloseItems(window: window) }
            CommandGroup(after: .pasteboard) { FindItems(window: window) }
            CommandGroup(before: .toolbar) { ViewItems(window: window) }
            CommandMenu(Text(Words.menuHistory)) { HistoryItems(window: window) }
            CommandGroup(after: .windowSize) { TabItems(window: window) }
        }

        /// The tabs are the browser's own: the system's way of making tabs of an app's
        /// windows is turned off, and with it the Window menu's items for those, which have
        /// the names and the keys of the browser's (035). Called once, as the app starts.
        public static func useTheBrowsersOwnTabs() {
            NSWindow.allowsAutomaticWindowTabbing = false
        }
    }

    /// From tab to tab (035): the core says which.
    private struct TabItems: View {
        let window: WindowCommands?

        var body: some View {
            let several = (window?.model.tabs.count ?? 0) > 1
            Divider()
            Command(Words.tabsNext, .tab, .control, enabled: several) { window?.model.stepTab(.next) }
            Command(Words.tabsPrevious, .tab, [.control, .shift], enabled: several) { window?.model.stepTab(.previous) }
        }
    }

    private struct FileItems: View {
        let window: WindowCommands?
        @Environment(\.openWindow) private var openWindow

        var body: some View {
            // A private window (037): there with any window in front, or none.
            Command(Words.privateNew, "n", [.command, .shift], enabled: true) { openWindow(id: BrowserWindowView.privateWindows) }
            Command(Words.tabsNew, "t", enabled: window != nil) { window?.model.openTab() }
            Command(Words.toolbarOpenLocation, "l", enabled: window != nil) { window?.openLocation() }
            Command(Words.tabsReopen, "t", [.command, .shift], enabled: window?.model.canReopenClosedTab == true) {
                window?.model.reopenClosedTab()
            }
            // One command with two names (039): a tab is kept as a Site, a Site is let go.
            Command(
                window?.model.tab.kept == true ? Words.sitesLetGo : Words.sitesKeep, "d",
                enabled: window?.model.canKeepOrLetGo == true
            ) {
                window?.model.keepOrLetGoTheTabInFront()
            }
        }
    }

    private struct CloseItems: View {
        let window: WindowCommands?

        var body: some View {
            Command(Words.tabsClose, "w", enabled: true) {
                if let window {
                    window.closeTab()
                } else {
                    closeWhatIsInFront()
                }
            }
            Command(Words.menuCloseWindow, "w", [.command, .shift], enabled: true) {
                if let window {
                    window.closeWindow()
                } else {
                    closeWhatIsInFront()
                }
            }
        }

        /// A window that is no browser window (the design catalogue, the page that says the
        /// app could not start) has no tabs: it closes.
        private func closeWhatIsInFront() {
            NSApp.keyWindow?.performClose(nil)
        }
    }

    private struct FindItems: View {
        let window: WindowCommands?

        var body: some View {
            Divider()
            Command(Words.findOpen, "f", enabled: window?.model.hasPage == true) { window?.find() }
            Command(Words.findNext, "g", enabled: window?.model.isFinding == true) { window?.model.findNext() }
            Command(Words.findPrevious, "g", [.command, .shift], enabled: window?.model.isFinding == true) {
                window?.model.findPrevious()
            }
        }
    }

    private struct ViewItems: View {
        let window: WindowCommands?

        var body: some View {
            let model = window?.model
            let zoom = model?.zoom
            Command(Words.toolbarReload, "r", enabled: model?.hasPage == true) { model?.perform(.reload) }
            Command(Words.toolbarStop, ".", enabled: model?.isLoading == true) { model?.perform(.stop) }
            Divider()
            Command(Words.zoomIn, "+", enabled: zoom?.canZoomIn == true) { model?.zoom(.in) }
            Command(Words.zoomOut, "-", enabled: zoom?.canZoomOut == true) { model?.zoom(.out) }
            Command(Words.zoomReset, "0", enabled: zoom != nil && zoom?.percent != 100) { model?.zoom(.reset) }
            Divider()
        }
    }

    private struct HistoryItems: View {
        let window: WindowCommands?

        var body: some View {
            let model = window?.model
            Command(Words.toolbarBack, "[", enabled: model?.tab.canGoBack == true) { model?.perform(.back) }
            Command(Words.toolbarForward, "]", enabled: model?.tab.canGoForward == true) { model?.perform(.forward) }
            Divider()
            Command(Words.historyOpen, "y", enabled: window != nil) { window?.showHistory() }
        }
    }
#endif

/// The keys that no menu carries, on buttons nobody sees.
///
/// On the Mac the menu bar has every key but one: ⌘= beside ⌘+ for Zoom In, the same key
/// without Shift (a menu item has one key). On an iPad with a keyboard, every key is here.
struct KeysNobodySees: View {
    let window: WindowCommands

    var body: some View {
        Group {
            Command(Words.zoomIn, "=", enabled: true) { window.model.zoom(.in) }
            #if os(macOS)
                // From tab to tab (035), as every browser on the Mac has it: ⇧⌘] and ⇧⌘[ beside
                // the menu's ⌃Tab and ⌃⇧Tab; ⌘1 to ⌘8 for the tab at that place; ⌘9 for the last.
                Command(Words.tabsNext, "]", [.command, .shift], enabled: true) { window.model.stepTab(.next) }
                Command(Words.tabsPrevious, "[", [.command, .shift], enabled: true) { window.model.stepTab(.previous) }
                ForEach(1 ..< 9, id: \.self) { place in
                    Command(Words.tabsList, KeyEquivalent(Character("\(place)")), enabled: true) { window.model.showTab(at: place) }
                }
                Command(Words.tabsList, "9", enabled: true) { window.model.stepTab(.last) }
            #endif
            #if !os(macOS)
                Command(Words.tabsNew, "t", enabled: true) { window.model.openTab() }
                Command(Words.toolbarOpenLocation, "l", enabled: true) { window.openLocation() }
                // ⇧⌘T, as in every browser: the tab that was last closed comes back (033).
                Command(Words.tabsReopen, "t", [.command, .shift], enabled: window.model.canReopenClosedTab) {
                    window.model.reopenClosedTab()
                }
                Command(Words.tabsClose, "w", enabled: true) { window.closeTab() }
                // ⌘D keeps the tab in front as a Site, or lets a Site go (039).
                Command(Words.sitesKeep, "d", enabled: window.model.canKeepOrLetGo) { window.model.keepOrLetGoTheTabInFront() }
                Command(Words.historyOpen, "y", enabled: true) { window.showHistory() }
                Command(Words.findOpen, "f", enabled: true) { window.find() }
                Command(Words.findNext, "g", enabled: true) { window.model.findNext() }
                Command(Words.findPrevious, "g", [.command, .shift], enabled: true) { window.model.findPrevious() }
                // Zoom (012): ⌘+, ⌘− and ⌘0, bar or no bar.
                Command(Words.zoomIn, "+", enabled: true) { window.model.zoom(.in) }
                Command(Words.zoomOut, "-", enabled: true) { window.model.zoom(.out) }
                Command(Words.zoomReset, "0", enabled: true) { window.model.zoom(.reset) }
            #endif
        }
        .opacity(0)
        .frame(width: 0, height: 0)
        .accessibilityHidden(true)
    }
}

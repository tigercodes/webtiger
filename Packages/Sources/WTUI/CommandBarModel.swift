// The command bar (048), the shell's part: the address field's list has Spaces and commands
// in it. With each question the window sends the commands it can do now, by a name and its
// words; the core says which match what is typed. A row that is chosen is done here: a Space
// is shown, a command is what its menu's item is. What is typed is never done as a command.
import SwiftUI
import WTContract

/// A command of the window's that needs nothing more than to be chosen (048 D6): the name the
/// core gives back when its words match, its words, and what it does.
struct BarCommand {
    let name: String
    let words: Word
    let action: @MainActor () -> Void
}

/// What only the window can do, as the command bar asks it of the model. The model holds
/// these, so none of them holds the model.
struct WindowDoes {
    /// The keyboard goes to the find bar, which the model has opened.
    var find: (@MainActor () -> Void)?
    /// The lists of what was visited and what was downloaded open.
    var showHistory: (@MainActor () -> Void)?
    /// The window closes.
    var closeWindow: (@MainActor () -> Void)?
    /// A private window opens. Nil where a window does not open one by itself (a touch
    /// screen, where it is in the list of Spaces).
    var openPrivateWindow: (@MainActor () -> Void)?
    /// The row of tabs runs from right to left: left is towards its end.
    var rightToLeft = false
}

extension BrowserModel {
    /// The window has something else to show when the tab in front is closed: a tab, or a
    /// Site that is awake. A Site that is asleep is no tab to be left with (039).
    var hasAnotherTabToShow: Bool {
        tabs.contains { $0.tabID != tab.tabID && !($0.kept && $0.asleep) }
    }

    /// The commands that can be done now, in the order their menus have them. One that can
    /// do nothing now is not among them (048 D4): a row that does nothing when it is chosen
    /// is worse than no row.
    func barCommands() -> [BarCommand] {
        guard windowID != nil else { return [] }
        var commands = [BarCommand(name: "tab.new", words: Words.tabsNew) { [weak self] in self?.openTab() }]
        if let open = windowDoes.openPrivateWindow {
            commands.append(BarCommand(name: "window.private", words: Words.privateNew, action: open))
        }
        if canReopenClosedTab {
            commands.append(BarCommand(name: "tab.reopen", words: Words.tabsReopen) { [weak self] in self?.reopenClosedTab() })
        }
        commands.append(BarCommand(name: "tab.close", words: Words.tabsClose) { [weak self] in self?.closeTheTabInFront() })
        commands += siteCommands() + viewCommands() + tabCommands()
        return commands
    }

    /// Closes the tab in front. On a Mac the window goes with its last tab, as everywhere
    /// there (034): where nothing else would be left to show, the window closes.
    func closeTheTabInFront() {
        if !hasAnotherTabToShow, let close = windowDoes.closeWindow {
            close()
        } else {
            closeTab(tab.tabID)
        }
    }

    /// What is done to the tab in front as a Site: kept or let go, a Favourite or none, sent
    /// home.
    private func siteCommands() -> [BarCommand] {
        var commands: [BarCommand] = []
        if canKeepOrLetGo {
            commands.append(BarCommand(
                name: tab.kept ? "site.let_go" : "site.keep", words: tab.kept ? Words.sitesLetGo : Words.sitesKeep
            ) { [weak self] in self?.keepOrLetGoTheTabInFront() })
        }
        if canFavouriteTheTabInFront {
            commands.append(BarCommand(
                name: tab.favourite ? "favourite.remove" : "favourite.add",
                words: tab.favourite ? Words.favouritesRemove : Words.favouritesAdd
            ) { [weak self] in self?.addOrRemoveTheTabInFrontAsAFavourite() })
        }
        if tab.away {
            commands.append(BarCommand(name: "site.home", words: Words.sitesReturnSite) { [weak self] in self?.sendHomeTheTabInFront() })
        }
        return commands
    }

    /// How the window is drawn, and what it shows of its page: the layout, find, zoom, and
    /// the lists.
    private func viewCommands() -> [BarCommand] {
        var commands: [BarCommand] = []
        if hasRoomForSidebar {
            commands.append(BarCommand(
                name: showsSidebar ? "layout.top" : "layout.side", words: showsSidebar ? Words.layoutTop : Words.layoutSide
            ) { [weak self] in self?.switchLayout() })
        }
        if hasPage, let find = windowDoes.find {
            commands.append(BarCommand(name: "page.find", words: Words.findOpen) { [weak self] in
                self?.openFind()
                find()
            })
        }
        if zoom != nil {
            commands.append(BarCommand(name: "page.zoom", words: Words.zoomOpen) { [weak self] in self?.openZoom() })
        }
        if let show = windowDoes.showHistory {
            commands.append(BarCommand(name: "history.show", words: Words.historyOpen, action: show))
        }
        return commands
    }

    /// The tab in front among the others: a place along the row, and the next and the one
    /// before.
    private func tabCommands() -> [BarCommand] {
        var commands: [BarCommand] = []
        if let front = frontTabInTheRow {
            for move in movesOfTheTabInFront(rightToLeft: windowDoes.rightToLeft) where canMove(front, move.step) {
                commands
                    .append(BarCommand(name: move.step == .back ? "tab.move_back" : "tab.move_forward", words: move.words) { [weak self] in
                        self?.moveTheTabInFront(move.step)
                    })
            }
        }
        if tabs.count > 1 {
            commands.append(BarCommand(name: "tab.next", words: Words.tabsNext) { [weak self] in self?.stepTab(.next) })
            commands.append(BarCommand(name: "tab.previous", words: Words.tabsPrevious) { [weak self] in self?.stepTab(.previous) })
        }
        return commands
    }

    /// Puts in a question what the command bar asks: that Spaces are shown (a private window
    /// has none, 037), and the commands that can be done now, which are kept until the next
    /// question so that the one chosen is the one that was offered.
    func askOfTheCommandBar(_ ask: inout WTSuggestAddresses) {
        ask.spaces = !isPrivate
        let commands = barCommands()
        commandsAsked = commands
        ask.commands = commands.map { command in
            var offered = WTOfferedCommand()
            offered.name = command.name
            offered.words = command.words.string()
            return offered
        }
    }

    /// Does what a row of the command bar says, when it is a Space or a command. False for
    /// any other row: a place, or a tab that is open.
    func chooseOfTheCommandBar(_ suggestion: WTAddressSuggestion) -> Bool {
        if suggestion.spaceID != 0 {
            isEditingAddress = false
            showSpace(suggestion.spaceID)
            return true
        }
        guard !suggestion.command.isEmpty else { return false }
        isEditingAddress = false
        // The command as it was when it was offered. One the window did not send is done
        // by nobody.
        commandsAsked.first { $0.name == suggestion.command }?.action()
        return true
    }
}

/// Tells the model what only the window can do, for the command bar (048): where the
/// keyboard goes for find, that the lists open, that the window closes, that a private one
/// opens, and which way the row runs. None of what it hands over holds the model.
struct OffersWindowCommands: ViewModifier {
    let model: BrowserModel
    let findFocused: FocusState<Bool>.Binding
    let showingHistory: Binding<Bool>
    @Environment(\.layoutDirection) private var direction
    #if os(macOS)
        @Environment(\.dismiss) private var dismiss
        @Environment(\.openWindow) private var openWindow
    #endif

    func body(content: Content) -> some View {
        content
            .onAppear { tell() }
            .onChange(of: direction) { _, _ in tell() }
            .onDisappear { model.windowDoes = WindowDoes() }
    }

    private func tell() {
        let (focus, history) = (findFocused, showingHistory)
        var does = WindowDoes(find: { focus.wrappedValue = true }, showHistory: { history.wrappedValue = true })
        does.rightToLeft = direction == .rightToLeft
        #if os(macOS)
            // A Mac's window goes with its last tab, and opens a private one by itself.
            let (dismiss, openWindow) = (dismiss, openWindow)
            does.closeWindow = { dismiss() }
            does.openPrivateWindow = { openWindow(id: BrowserWindowView.privateWindows) }
        #endif
        model.windowDoes = does
    }
}

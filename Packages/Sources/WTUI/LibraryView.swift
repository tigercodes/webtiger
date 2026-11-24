// The Library (057): what the button beside the tabs opens. The lists about the browser that
// are opened now and then, with a switch between them: History (008), Downloads (013),
// Permissions (014), the Archive (049), and what is awake (056). One sheet and one button, so
// that the row of tabs gains none.
import SwiftUI
import WTContract
import WTDesign

struct LibraryView: View {
    enum Part {
        case history
        case downloads
        case permissions
        case archive
        case awake
    }

    @State private var part: Part
    private let history: HistoryModel
    private let downloads: DownloadsModel
    private let permissions: PermissionsModel
    /// The Archive of the window's Space (049). None: the window has none (a private one),
    /// and the switch has no part for it.
    private let archive: ArchiveModel?
    /// What is awake (056). None: the list has no part in the switch (a catalogue's state of
    /// another list, a test's).
    private let awake: AwakeModel?
    /// The window's row: what is awake is asked for again when it changes.
    private let row: [WTTabSummary]
    private let open: (WTHistoryEntry) -> Void
    private let done: () -> Void

    /// `open` is called with a visit a person chose; `done` when they are finished, and when
    /// a tab they chose in the Archive is a tab again.
    init(
        showing part: Part = .history,
        history: HistoryModel,
        downloads: DownloadsModel,
        permissions: PermissionsModel,
        archive: ArchiveModel? = nil,
        awake: AwakeModel? = nil,
        row: [WTTabSummary] = [],
        open: @escaping (WTHistoryEntry) -> Void,
        done: @escaping () -> Void
    ) {
        _part = State(initialValue: part)
        self.history = history
        self.downloads = downloads
        self.permissions = permissions
        self.archive = archive
        self.awake = awake
        self.row = row
        self.open = open
        self.done = done
    }

    var body: some View {
        switch part {
        case .history: HistoryView(model: history, switcher: AnyView(switcher), open: open, done: done)
        case .downloads: DownloadsView(model: downloads, switcher: AnyView(switcher), done: done)
        case .permissions: PermissionsView(model: permissions, switcher: AnyView(switcher), done: done)
        case .archive:
            if let archive {
                ArchiveView(model: archive, switcher: AnyView(switcher), broughtBack: done, done: done)
            }
        case .awake:
            if let awake {
                AwakeView(model: awake, switcher: AnyView(switcher), watching: row, done: done)
            }
        }
    }

    /// A button for each list, of which the one for the list that is shown is marked: on a
    /// line of its own, above the list's own buttons (057). On a phone the names are more
    /// than a row is wide: the row is moved sideways to reach its end, so that no name is cut
    /// short or broken; and it opens with the name of the list that is shown in sight.
    ///
    /// One line, and not as many as the names need: tried (057 §6 D2), names that went on to
    /// further lines made the top so tall at the largest text sizes that the list under it
    /// had no room, and Apple's audit said so.
    @ViewBuilder private var switcher: some View {
        #if os(iOS)
            ScrollViewReader { row in
                ScrollView(.horizontal, showsIndicators: false) {
                    choices
                }
                .onAppear { row.scrollTo(part, anchor: .trailing) }
            }
        #else
            choices
        #endif
    }

    private var choices: some View {
        HStack(spacing: Tokens.Space.xs) {
            choice(Words.historyTitle, .history, identifier: "library-history")
            choice(Words.downloadsTitle, .downloads, identifier: "library-downloads")
            choice(Words.permissionsTitle, .permissions, identifier: "library-permissions")
            if archive != nil {
                choice(Words.archiveTitle, .archive, identifier: "library-archive")
            }
            if awake != nil {
                choice(Words.awakeTitle, .awake, identifier: "library-awake")
            }
        }
    }

    private func choice(_ title: Word, _ which: Part, identifier: String) -> some View {
        Button {
            part = which
        } label: {
            Text(title)
                .font(.headline)
                // On more lines where one is not enough, and never cut short.
                .fixedSize(horizontal: false, vertical: true)
                .frame(minHeight: BrowserWindowView.hitSize)
                .padding(.horizontal, Tokens.Space.s)
                .background(part == which ? Tokens.Palette.fieldFill : Color.clear, in: RoundedRectangle(cornerRadius: Tokens.Radius.field))
                .contentShape(Rectangle())
        }
        // Plain, so its words are in the primary colour (F14); which list is shown is said by
        // the fill, and to assistive technologies as "selected".
        .buttonStyle(.plain)
        .accessibilityAddTraits(part == which ? [.isSelected, .isHeader] : [])
        .accessibilityIdentifier(identifier)
        // What the row is moved to on a phone, to have the one shown in sight.
        .id(which)
    }
}

/// The top of a list in the sheet: its title, or the switch that stands in for it, and the
/// list's own buttons. In one row; or, `stacked`, with the buttons on a row of their own.
///
/// A switch has a row of its own (057). A switch and two buttons were too many for one row on
/// an iPhone from the start: Apple's audit said the switch's words could be cut short there
/// at larger text sizes. A row that changed its mind as the text grew (`ViewThatFits`) did no
/// better: the audit then said of every control in it that its text size could not be
/// changed. On a Mac four names and two buttons filled the row, and a fifth name did not fit.
/// So it is one or the other, by what is in it: a title beside the buttons, a switch above.
struct ListHeader<Leading: View, Buttons: View>: View {
    let stacked: Bool
    @ViewBuilder let leading: Leading
    @ViewBuilder let buttons: Buttons

    /// Whether a header with a switch in it is stacked: it is, on every system (057).
    static var stacksASwitch: Bool {
        true
    }

    var body: some View {
        if stacked {
            VStack(alignment: .leading, spacing: Tokens.Space.s) {
                leading
                HStack(spacing: Tokens.Space.m) {
                    Spacer(minLength: 0)
                    buttons
                }
            }
        } else {
            HStack(spacing: Tokens.Space.m) {
                leading
                Spacer(minLength: Tokens.Space.s)
                buttons
            }
        }
    }
}

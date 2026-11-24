// What the button beside the tabs opens: the History list (008) and, beside it, the Downloads
// list (013), the Permissions list (014) and the Archive (049), with a switch between them. The
// Library (W3) will be their home; until then they share a sheet, so that the row of tabs
// gains no button.
import SwiftUI
import WTContract
import WTDesign

struct LibraryView: View {
    enum Part {
        case history
        case downloads
        case permissions
        case archive
    }

    @State private var part: Part
    private let history: HistoryModel
    private let downloads: DownloadsModel
    private let permissions: PermissionsModel
    /// The Archive of the window's Space (049). None: the window has none (a private one),
    /// and the switch has no part for it.
    private let archive: ArchiveModel?
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
        open: @escaping (WTHistoryEntry) -> Void,
        done: @escaping () -> Void
    ) {
        _part = State(initialValue: part)
        self.history = history
        self.downloads = downloads
        self.permissions = permissions
        self.archive = archive
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
        }
    }

    /// A button for each list, of which the one for the list that is shown is marked: where a
    /// list's title would be. On a phone the four names are more than a row is wide: the row
    /// is moved sideways to reach its end, so that no name is cut short or broken; and it
    /// opens with the name of the list that is shown in sight.
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
/// A switch and two buttons are too many for one row on an iPhone: Apple's audit said the
/// switch's words could be cut short there at larger text sizes. A row that changed its mind
/// as the text grew (`ViewThatFits`) did no better: the audit then said of every control in it
/// that its text size could not be changed. So it is one or the other, by what is in it and
/// where.
struct ListHeader<Leading: View, Buttons: View>: View {
    let stacked: Bool
    @ViewBuilder let leading: Leading
    @ViewBuilder let buttons: Buttons

    /// Whether a header with a switch in it is stacked here: on an iPhone, and not on a Mac,
    /// whose sheet is wide enough for both.
    static var stacksASwitch: Bool {
        #if os(iOS)
            true
        #else
            false
        #endif
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

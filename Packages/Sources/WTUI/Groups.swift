// Groups (041) as the shell shows them: a Group's chip in the strip, where its Sites follow
// it, and the page that makes or edits one. Which Groups there are, what is in them and what
// a shut one hides are the core's; the colours are a Space's (025).
import SwiftUI
import WTContract
import WTDesign

/// What the strip draws, in order: a tab, or a Group's chip.
enum StripItem: Identifiable, Equatable {
    case tab(WTTabSummary)
    case group(WTGroupSummary)

    var id: String {
        switch self {
        case let .tab(tab): "tab-\(tab.tabID)"
        case let .group(group): "group-\(group.groupID)"
        }
    }

    /// The row as the core's two lists say it (041 §5): the tabs, in the core's order and never
    /// in another, with each Group's chip before the first tab that says it is in that Group.
    /// A Group with no tab to show (shut, or empty) stands after the Group before it; and the
    /// first of the Groups stands after the Sites that are in no Group, which is before the
    /// first tab that is no Site. A tab that names a Group the list does not have is drawn as
    /// one in none.
    static func row(tabs: [WTTabSummary], groups: [WTGroupSummary]) -> [StripItem] {
        var row: [StripItem] = []
        // How many of the Groups have their chip in the row.
        var drawn = 0
        func draw(through last: Int) {
            while drawn <= last, drawn < groups.count {
                row.append(.group(groups[drawn]))
                drawn += 1
            }
        }
        for tab in tabs {
            if let place = groups.firstIndex(where: { $0.groupID == tab.groupID }) {
                // A Group's first tab has the Group's chip before it, and before that the
                // chips of the Groups before it that have none yet.
                draw(through: place)
            } else if !tab.kept {
                // The first tab that is no Site has every chip there is before it.
                draw(through: groups.count - 1)
            }
            row.append(.tab(tab))
        }
        draw(through: groups.count - 1)
        return row
    }
}

/// A Group's chip in the strip: its mark, in its colour when it has one, and its name; shut,
/// how many Sites it holds. A press shuts or opens it, and its own menu edits or removes it.
struct GroupChip: View {
    let group: WTGroupSummary
    let toggle: () -> Void
    let edit: () -> Void
    let remove: () -> Void
    /// What was last done to the row can be undone from its menu (044).
    var undo: TreeUndo?
    @Environment(\.locale) private var locale
    /// As wide as a tab's name may be (005): a long name is cut to one line.
    @ScaledMetric private var widestName: CGFloat = 180

    var body: some View {
        pressed
            .accessibilityLabel(Text(verbatim: group.name))
            .accessibilityValue(Text(verbatim: Self.said(of: group, locale: locale)))
            .accessibilityIdentifier("group")
    }

    @ViewBuilder
    private var pressed: some View {
        #if os(macOS)
            // On a Mac a button takes a right-click for itself and no menu comes of it (033):
            // the chip is pressed as a gesture, and says that it is a button.
            label
                .contentShape(Rectangle())
                .onTapGesture(perform: toggle)
                .contextMenu { menu }
                .accessibilityElement(children: .ignore)
                .accessibilityAddTraits(.isButton)
                .accessibilityAction(.default, toggle)
        #else
            Button(action: toggle) { label }
                .buttonStyle(.plain)
                .contextMenu { menu }
        #endif
    }

    private var label: some View {
        HStack(spacing: Tokens.Space.xs) {
            // An outline while it is open, filled when it is shut: what is in it is put away.
            Image(systemName: group.shut ? "folder.fill" : "folder")
                .font(.caption)
                .foregroundStyle(Tokens.SpaceColour.paint(group.colour) ?? Color.primary)
                .accessibilityHidden(true)
            Text(verbatim: Self.shown(group, locale: locale))
                .lineLimit(1)
                .fontWeight(.medium)
        }
        .padding(.horizontal, Tokens.Space.s)
        .frame(maxWidth: widestName, minHeight: BrowserWindowView.hitSize, alignment: .leading)
        .fixedSize(horizontal: true, vertical: false)
        .contentShape(Rectangle())
        // A line round it: a Group is not a tab, and none of them is ever "in front".
        .overlay {
            RoundedRectangle(cornerRadius: Tokens.Radius.field).strokeBorder(Tokens.Palette.hairline, lineWidth: 1)
        }
    }

    @ViewBuilder
    private var menu: some View {
        // What was last done to the row, undone (044): at the top, while there is something.
        TreeUndoItem(undo: undo)
        Button(action: edit) { Text(Words.groupEdit) }
            .accessibilityIdentifier("group-edit")
        Button(action: remove) { Text(Words.groupRemove) }
            .accessibilityIdentifier("group-remove")
    }

    /// What the chip shows: the Group's name; shut, with how many Sites it holds.
    static func shown(_ group: WTGroupSummary, locale: Locale = .current) -> String {
        group.shut ? Words.groupShutLabel.filled(group.name, Int(group.sites), locale: locale) : group.name
    }

    /// What is said of a Group beside its name: that it is one, and whether it is open or
    /// shut; shut, how many Sites it holds.
    static func said(of group: WTGroupSummary, locale: Locale = .current) -> String {
        group.shut ? Words.groupSaidShut.filled(Int(group.sites), locale: locale) : Words.groupSaidOpen.string(locale)
    }
}

/// "Move to Group" for a tab (041): the Groups of the row by name and "No Group", one of which
/// is the tab's and is marked so; and "New Group…".
struct MoveToGroupItems: View {
    let groups: [WTGroupSummary]
    /// The Group the tab is in; 0 for none.
    let current: UInt64
    let move: (UInt64) -> Void
    let new: () -> Void

    var body: some View {
        if !groups.isEmpty {
            // A choice of one: the system marks the one that is the tab's, and says so.
            Picker(selection: Binding(get: { current }, set: { move($0) })) {
                ForEach(groups, id: \.groupID) { group in
                    Text(verbatim: group.name).tag(group.groupID)
                }
                Text(Words.groupsNone).tag(UInt64(0))
            } label: {
                Text(Words.groupsMove)
            }
            .pickerStyle(.inline)
            .labelsHidden()
            Divider()
        }
        Button(action: new) { Text(Words.groupsNew) }
    }
}

/// The Group page over a window: open while the model has something asked of it.
struct GroupSheet: ViewModifier {
    let model: BrowserModel

    func body(content: Content) -> some View {
        content
            .sheet(item: Binding(get: { model.groupAsked }, set: { model.groupAsked = $0 })) { asked in
                GroupPage(
                    asked: asked, colours: model.spaceColours, refusal: model.groupRefusal,
                    done: { name, colour in
                        Task { await model.finishGroupPage(name: name, colour: colour) }
                    },
                    cancel: { model.groupAsked = nil }
                )
                #if os(macOS)
                .frame(minWidth: 420, minHeight: 300)
                #endif
            }
    }
}

/// The page that makes a Group, or edits one: its name, its colour or none, and two buttons.
struct GroupPage: View {
    let asked: GroupAsk
    /// The colours to choose among: the core's, in its order.
    let colours: [String]
    /// Why the core would not, by its code.
    let refusal: String?
    /// The button that does what the page is for was pressed: the name and the colour.
    let done: (String, String) -> Void
    let cancel: () -> Void
    /// The page is a picture of itself (004): the keyboard is not put in its field.
    private let isPicture: Bool
    @State private var name: String
    @State private var colour: String
    @FocusState private var nameFocused: Bool

    init(
        asked: GroupAsk, colours: [String], refusal: String?, name: String? = nil,
        done: @escaping (String, String) -> Void, cancel: @escaping () -> Void
    ) {
        self.asked = asked
        self.colours = colours
        self.refusal = refusal
        self.done = done
        self.cancel = cancel
        isPicture = name != nil
        var has = (name: "", colour: "")
        if case let .edit(group) = asked {
            has = (group.name, group.colour)
        }
        _name = State(initialValue: name ?? has.name)
        _colour = State(initialValue: has.colour)
    }

    private var isNew: Bool {
        if case .new = asked {
            true
        } else {
            false
        }
    }

    var body: some View {
        // It scrolls: on a small screen, or at a large text size, it is taller than its sheet.
        ScrollView {
            page
        }
    }

    private var page: some View {
        VStack(alignment: .leading, spacing: Tokens.Space.l) {
            Text(isNew ? Words.groupNewTitle : Words.groupEditTitle)
                .font(.title2.weight(.semibold))
                .accessibilityAddTraits(.isHeader)
                .accessibilityIdentifier("group-title")
            VStack(alignment: .leading, spacing: Tokens.Space.xs) {
                Text(Words.groupName)
                    .font(.callout)
                TextField(text: $name) { Text(Words.groupName) }
                    .textFieldStyle(.roundedBorder)
                    .focused($nameFocused)
                    .onSubmit(submit)
                    .accessibilityIdentifier("group-name")
            }
            SpaceSwatches(colours: colours, offersNone: true, chosen: $colour)
            if let refusal {
                Text(verbatim: Self.words(for: refusal, new: isNew))
                    .fixedSize(horizontal: false, vertical: true)
                    .accessibilityIdentifier("group-refusal")
            }
            HStack(spacing: Tokens.Space.m) {
                Button(action: cancel) {
                    Text(Words.groupCancel)
                        .frame(minHeight: BrowserWindowView.hitSize)
                }
                .buttonStyle(.bordered)
                .tint(.primary)
                .accessibilityIdentifier("group-cancel")
                Button(action: submit) {
                    // The one that does something stands out by its weight (024).
                    Text(isNew ? Words.groupMake : Words.groupSave)
                        .fontWeight(.semibold)
                        .frame(minHeight: BrowserWindowView.hitSize)
                }
                .buttonStyle(.bordered)
                .tint(.primary)
                // A Group has a name. (The core says so too: this only spares the press.)
                .disabled(name.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                .accessibilityIdentifier("group-done")
            }
        }
        .padding(Tokens.Space.l)
        .frame(maxWidth: .infinity, alignment: .leading)
        // The name is what is asked first: the keyboard is in its field when the page opens.
        .onAppear {
            if !isPicture {
                nameFocused = true
            }
        }
    }

    private func submit() {
        guard !name.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return }
        done(name, colour)
    }

    /// The words for why a Group was not made, or not changed: the code's own, or what is
    /// said when no more is known. Never the code's text.
    static func words(for code: String, new: Bool) -> String {
        (Word(named: "error." + code) ?? (new ? Words.errorGroupFailed : Words.errorGroupChangeFailed)).string()
    }
}

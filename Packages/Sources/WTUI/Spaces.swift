// Several Spaces (024 §6.1): the chip that names the Space a window shows, the list of Spaces
// it opens, and the pages that make one and edit the one shown. What they ask of the core is in
// SpacesModel.swift.
import SwiftUI
import WTContract
import WTDesign

/// The chip that names the Space the window shows, and opens the list of Spaces. It stands
/// before the tabs and does not scroll with them.
///
/// In a window as narrow as a phone's the strip has no room for a name beside the tabs: with
/// one, a second tab is pushed out of reach. There the chip is the Space's initial, and its
/// name is what a screen reader says and what the list shows.
struct SpaceChip: View {
    let name: String
    /// The Space's colour, by the core's name for it (025); empty for none. A Space with one
    /// has its mark here: a dot before the name, or a bar under the initial.
    var colour = ""
    let action: () -> Void
    /// How wide the name may be: it grows with the text's size, and a long name is cut.
    @ScaledMetric private var widestName: CGFloat = 120
    @Environment(\.locale) private var locale
    @Environment(\.horizontalSizeClass) private var width

    var body: some View {
        let shown = SpaceName.shown(name, locale: locale)
        Button(action: action) {
            Group {
                if width == .compact {
                    Text(verbatim: SpaceName.initial(of: shown))
                        .fontWeight(.semibold)
                        .frame(minWidth: BrowserWindowView.hitSize, minHeight: BrowserWindowView.hitSize)
                        // The mark takes none of the strip's room: a bar under the initial.
                        .overlay(alignment: .bottom) {
                            if let paint = Tokens.SpaceColour.paint(colour) {
                                Capsule().fill(paint).frame(width: 18, height: 3).padding(.bottom, 6)
                            }
                        }
                } else {
                    HStack(spacing: Tokens.Space.xs) {
                        if Tokens.SpaceColour.paint(colour) != nil {
                            SpaceMark(colour: colour)
                        }
                        Text(verbatim: shown)
                            .lineLimit(1)
                            .fontWeight(.semibold)
                            .frame(maxWidth: widestName, alignment: .leading)
                            .fixedSize(horizontal: true, vertical: false)
                        Image(systemName: "chevron.down")
                            .font(.caption)
                            .accessibilityHidden(true)
                    }
                    .padding(.horizontal, Tokens.Space.s)
                }
            }
            .frame(minHeight: BrowserWindowView.hitSize)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .background(Tokens.Palette.fieldFill, in: RoundedRectangle(cornerRadius: Tokens.Radius.field))
        .accessibilityLabel(Text(verbatim: String(format: String(localized: "space.chip", bundle: .module, locale: locale), shown)))
        .accessibilityIdentifier("space-chip")
    }
}

/// The list of Spaces over a window: asked for as it opens, and closed when a Space is chosen
/// or made, or by "Done".
struct SpacesSheet: ViewModifier {
    let model: BrowserModel
    @Binding var isShown: Bool

    func body(content: Content) -> some View {
        content
            .sheet(isPresented: $isShown) {
                SpacesView(
                    spaces: model.spaces, shownID: model.spaceID, colours: model.spaceColours, refusal: model.spaceRefusal,
                    show: { space in
                        model.showSpace(space)
                        isShown = false
                    },
                    make: { name, sharing, colour in
                        Task {
                            // A Space the core would not make leaves the page as it is, with
                            // the core's reason on it.
                            if await model.makeSpace(named: name, sharingLoginsWith: sharing, colour: colour) {
                                isShown = false
                            }
                        }
                    },
                    edit: { asked in
                        Task {
                            if await model.editSpace(asked) {
                                isShown = false
                            }
                        }
                    },
                    done: { isShown = false }
                )
                #if os(macOS)
                .frame(minWidth: 480, minHeight: 420)
                #endif
            }
            .onChange(of: isShown) { _, shown in
                if shown {
                    model.openSpaces()
                }
            }
    }
}

/// The list of Spaces, the page that makes one, and the page that edits the one shown.
struct SpacesView: View {
    /// What the sheet shows.
    private enum Page {
        case list
        case new
        case rename
    }

    /// The Spaces as the core listed them, in their order. `nil` until it has.
    let spaces: [WTSpaceSummary]?
    /// The Space the window shows.
    let shownID: UInt64
    /// The colours a Space can have, as the core lists them (025).
    let colours: [String]
    /// Why the core would not make a Space, by its code. `nil` when it has refused nothing.
    let refusal: String?
    /// A Space was pressed: the window is to show it.
    let show: (UInt64) -> Void
    /// A Space is to be made: its name as typed, the Space whose logins it shares, if any, and
    /// its colour, or none for the core to choose.
    let make: (String, UInt64?, String) -> Void
    /// The Space the window shows is to have another name, another colour, or both.
    let edit: (SpaceEdit) -> Void
    let done: () -> Void

    @State private var page: Page
    @State private var name: String
    /// The colour chosen on a page, by the core's name for it; empty for none.
    @State private var colour: String
    @State private var shares: Bool
    @State private var shareWith: UInt64
    @FocusState private var nameFocused: Bool
    /// It is a picture of one of the pages (004): it takes no keyboard.
    private let isPicture: Bool
    @Environment(\.locale) private var locale

    /// `making` opens on the page that makes a Space, with `name` typed and, with `sharingWith`,
    /// that Space's logins chosen; `renaming` on the page that edits the Space shown: for a
    /// picture of either (004).
    init(
        spaces: [WTSpaceSummary]?,
        shownID: UInt64,
        colours: [String] = [],
        refusal: String? = nil,
        making: Bool = false,
        name: String = "",
        sharingWith: UInt64? = nil,
        renaming: Bool = false,
        show: @escaping (UInt64) -> Void,
        make: @escaping (String, UInt64?, String) -> Void,
        edit: @escaping (SpaceEdit) -> Void,
        done: @escaping () -> Void
    ) {
        self.spaces = spaces
        self.shownID = shownID
        self.colours = colours
        self.refusal = refusal
        self.show = show
        self.make = make
        self.edit = edit
        self.done = done
        isPicture = making || renaming
        let shown = (spaces ?? []).first { $0.spaceID == shownID }
        _page = State(initialValue: making ? .new : renaming ? .rename : .list)
        _name = State(initialValue: renaming ? shown?.name ?? "" : name)
        _colour = State(initialValue: renaming ? shown?.colour ?? "" : "")
        _shares = State(initialValue: sharingWith != nil)
        // Whose logins it would share, should that be chosen: those of the Space shown.
        _shareWith = State(initialValue: sharingWith ?? Self.owner(of: shownID, among: spaces ?? []))
    }

    var body: some View {
        ScrollView {
            switch page {
            case .new: making
            case .rename: renaming
            case .list: list
            }
        }
    }

    // MARK: The list

    private var list: some View {
        VStack(alignment: .leading, spacing: 0) {
            VStack(alignment: .leading, spacing: Tokens.Space.s) {
                Text("spaces.title", bundle: .module)
                    .font(.title2.weight(.semibold))
                    .accessibilityAddTraits(.isHeader)
                    .accessibilityIdentifier("spaces-title")
                Text("spaces.detail", bundle: .module)
                    .fixedSize(horizontal: false, vertical: true)
            }
            .padding(Tokens.Space.l)
            ForEach(spaces ?? [], id: \.spaceID) { space in
                Divider()
                row(space)
            }
            Divider()
            HStack(spacing: Tokens.Space.m) {
                Button {
                    // Whose logins a new Space would share, should the person choose to: those
                    // of the Space the window shows.
                    shareWith = Self.owner(of: shownID, among: spaces ?? [])
                    name = ""
                    colour = ""
                    page = .new
                } label: {
                    Text("spaces.new", bundle: .module)
                        .frame(minHeight: BrowserWindowView.hitSize)
                }
                .buttonStyle(.bordered)
                .tint(.primary)
                .accessibilityIdentifier("spaces-new")
                Button {
                    // The name the Space has: none, for the one the app made while nobody has
                    // named it. The shell's word for that one is not a name it was given.
                    name = shown?.name ?? ""
                    colour = shown?.colour ?? ""
                    page = .rename
                } label: {
                    Text("spaces.rename", bundle: .module)
                        .frame(minHeight: BrowserWindowView.hitSize)
                }
                .buttonStyle(.bordered)
                .tint(.primary)
                // There is none to name until the core has listed the Space the window shows.
                .disabled(shown == nil)
                .accessibilityIdentifier("spaces-rename")
                Button(action: done) {
                    Text("spaces.done", bundle: .module)
                        .frame(minHeight: BrowserWindowView.hitSize)
                }
                .buttonStyle(.bordered)
                .tint(.primary)
                .accessibilityIdentifier("spaces-done")
            }
            .padding(Tokens.Space.l)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    /// One Space: its name, whose logins it shares if it shares any, and that the window shows
    /// it when it does. Said in words: not by a colour alone.
    private func row(_ space: WTSpaceSummary) -> some View {
        let isShown = space.spaceID == shownID
        return Button {
            show(space.spaceID)
        } label: {
            HStack(spacing: Tokens.Space.m) {
                // Its colour (025): a mark, and a ring for a Space with none.
                SpaceMark(colour: space.colour)
                VStack(alignment: .leading, spacing: 0) {
                    Text(verbatim: SpaceName.shown(space.name, locale: locale))
                        .fontWeight(.semibold)
                    if let other = sharer(with: space) {
                        Text(verbatim: String(format: String(localized: "spaces.shares", bundle: .module, locale: locale), other))
                            .font(.callout)
                            .fixedSize(horizontal: false, vertical: true)
                    }
                }
                Spacer(minLength: Tokens.Space.m)
                if isShown {
                    Text("spaces.shown", bundle: .module)
                        .font(.callout)
                }
            }
            .frame(maxWidth: .infinity, minHeight: BrowserWindowView.hitSize, alignment: .leading)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .padding(.horizontal, Tokens.Space.l)
        .padding(.vertical, Tokens.Space.xs)
        .accessibilityAddTraits(isShown ? .isSelected : [])
        .accessibilityIdentifier("space-\(space.spaceID)")
    }

    /// The Space the window shows, as the core listed it.
    private var shown: WTSpaceSummary? {
        (spaces ?? []).first { $0.spaceID == shownID }
    }

    /// The name of another Space that has the same logins as `space`: the first there is.
    private func sharer(with space: WTSpaceSummary) -> String? {
        (spaces ?? [])
            .first { $0.logins == space.logins && $0.spaceID != space.spaceID }
            .map { SpaceName.shown($0.name, locale: locale) }
    }

    /// The Spaces a new one can share logins with: the first of each that has logins of its
    /// own. (Sharing with one that shares already is sharing with whoever it shares with.)
    private var owners: [WTSpaceSummary] {
        Self.owners(among: spaces ?? [])
    }

    private static func owners(among spaces: [WTSpaceSummary]) -> [WTSpaceSummary] {
        var seen = Set<UInt32>()
        return spaces.filter { seen.insert($0.logins).inserted }
    }

    /// The first Space with the logins of the Space `id`.
    private static func owner(of id: UInt64, among spaces: [WTSpaceSummary]) -> UInt64 {
        let logins = spaces.first { $0.spaceID == id }?.logins
        let owners = owners(among: spaces)
        return owners.first { $0.logins == logins }?.spaceID ?? owners.first?.spaceID ?? 0
    }

    // MARK: Making one

    private var making: some View {
        VStack(alignment: .leading, spacing: Tokens.Space.l) {
            Text("space.new.title", bundle: .module)
                .font(.title2.weight(.semibold))
                .accessibilityAddTraits(.isHeader)
                .accessibilityIdentifier("space-new-title")
            VStack(alignment: .leading, spacing: Tokens.Space.xs) {
                Text("space.new.name", bundle: .module)
                    .font(.callout)
                TextField(text: $name) { Text("space.new.name", bundle: .module) }
                    .textFieldStyle(.roundedBorder)
                    .focused($nameFocused)
                    .onSubmit(submit)
                    .accessibilityIdentifier("space-name")
            }
            VStack(alignment: .leading, spacing: Tokens.Space.s) {
                Text("space.new.logins", bundle: .module)
                    .font(.headline)
                    .accessibilityAddTraits(.isHeader)
                choice(
                    "space.new.own", detail: String(localized: "space.new.own.detail", bundle: .module, locale: locale),
                    chosen: !shares, identifier: "space-logins-own"
                ) { shares = false }
                choice(
                    "space.new.shared",
                    detail: String(format: String(localized: "space.new.shared.detail", bundle: .module, locale: locale), sharedName),
                    chosen: shares, identifier: "space-logins-shared"
                ) { shares = true }
                // Which Space, where there is more than one to choose among.
                if shares, owners.count > 1 {
                    Picker(selection: $shareWith) {
                        ForEach(owners, id: \.spaceID) { space in
                            Text(verbatim: SpaceName.shown(space.name, locale: locale)).tag(space.spaceID)
                        }
                    } label: {
                        Text("space.new.share_with", bundle: .module)
                    }
                    .accessibilityIdentifier("space-share-with")
                }
            }
            VStack(alignment: .leading, spacing: Tokens.Space.xs) {
                SpaceSwatches(colours: colours, offersNone: false, chosen: $colour)
                Text("space.automatic_colour", bundle: .module)
                    .font(.callout)
                    .fixedSize(horizontal: false, vertical: true)
            }
            buttons("space.new.make", identifier: "space-make", unknown: "error.space.failed", action: submit)
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

    /// What the core refused, in words, and a page's two buttons: back to the list, and the
    /// one that does what the page is for, which is not to be pressed while there is no name.
    /// (The core says so too: this only spares the press.)
    private func buttons(
        _ title: LocalizedStringKey, identifier: String, unknown: String, needsName: Bool = true, action: @escaping () -> Void
    ) -> some View {
        VStack(alignment: .leading, spacing: Tokens.Space.l) {
            if let refusal {
                Text(verbatim: Self.words(for: refusal, unknown: unknown))
                    .fixedSize(horizontal: false, vertical: true)
                    .accessibilityIdentifier("space-refusal")
            }
            HStack(spacing: Tokens.Space.m) {
                Button {
                    page = .list
                } label: {
                    Text("space.new.back", bundle: .module)
                        .frame(minHeight: BrowserWindowView.hitSize)
                }
                .buttonStyle(.bordered)
                .tint(.primary)
                .accessibilityIdentifier("space-back")
                Button(action: action) {
                    // The one that does something stands out by its weight. (White on the
                    // accent's blue, which a prominent button is, is too faint to read at this
                    // size: Apple's audit, on an iPhone.)
                    Text(title, bundle: .module)
                        .fontWeight(.semibold)
                        .frame(minHeight: BrowserWindowView.hitSize)
                }
                .buttonStyle(.bordered)
                .tint(.primary)
                .disabled(needsName && name.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                .accessibilityIdentifier(identifier)
            }
        }
    }

    // MARK: Naming the one shown

    private var renaming: some View {
        VStack(alignment: .leading, spacing: Tokens.Space.l) {
            Text("space.rename.title", bundle: .module)
                .font(.title2.weight(.semibold))
                .accessibilityAddTraits(.isHeader)
                .accessibilityIdentifier("space-rename-title")
            VStack(alignment: .leading, spacing: Tokens.Space.xs) {
                Text("space.new.name", bundle: .module)
                    .font(.callout)
                TextField(text: $name) { Text("space.new.name", bundle: .module) }
                    .textFieldStyle(.roundedBorder)
                    .focused($nameFocused)
                    .onSubmit(save)
                    .accessibilityIdentifier("space-name")
            }
            SpaceSwatches(colours: colours, offersNone: true, chosen: $colour)
            buttons(
                "space.rename.save", identifier: "space-save", unknown: "error.space.rename_failed",
                needsName: shown?.name.isEmpty == false,
                action: save
            )
        }
        .padding(Tokens.Space.l)
        .frame(maxWidth: .infinity, alignment: .leading)
        .onAppear {
            if !isPicture {
                nameFocused = true
            }
        }
    }

    /// The name of the Space a new one would share logins with.
    private var sharedName: String {
        let chosen = owners.first { $0.spaceID == shareWith } ?? owners.first
        return SpaceName.shown(chosen?.name ?? "", locale: locale)
    }

    private func submit() {
        make(name, shares ? (owners.first { $0.spaceID == shareWith } ?? owners.first)?.spaceID : nil, colour)
    }

    /// What the page that edits a Space asks for: the name, when one is typed that is not the
    /// one the Space has; the colour, when it is another. A Space nobody named may keep no
    /// name; one that has a name is not given none.
    private func save() {
        let typed = name.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !typed.isEmpty || shown?.name.isEmpty != false else { return }
        edit(SpaceEdit(
            name: typed.isEmpty || name == shown?.name ? nil : name,
            colour: colour == (shown?.colour ?? "") ? nil : colour
        ))
    }

    /// One of the two choices of logins: its name, what it means in a sentence, and whether
    /// it is the one chosen, which a mark says and a screen reader is told.
    private func choice(
        _ title: LocalizedStringKey, detail: String, chosen: Bool, identifier: String, choose: @escaping () -> Void
    ) -> some View {
        Button(action: choose) {
            HStack(alignment: .firstTextBaseline, spacing: Tokens.Space.s) {
                Image(systemName: chosen ? "largecircle.fill.circle" : "circle")
                    .accessibilityHidden(true)
                VStack(alignment: .leading, spacing: 0) {
                    Text(title, bundle: .module)
                        .fontWeight(.semibold)
                    Text(verbatim: detail)
                        .font(.callout)
                        .fixedSize(horizontal: false, vertical: true)
                }
            }
            .frame(maxWidth: .infinity, minHeight: BrowserWindowView.hitSize, alignment: .leading)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityAddTraits(chosen ? .isSelected : [])
        .accessibilityIdentifier(identifier)
    }

    /// The words for why a Space was not made, or not named: the code's own, or `unknown`,
    /// what is said when no more is known. Never the code's text.
    static func words(for code: String, unknown: String = "error.space.failed") -> String {
        let key = "error." + code
        let words = Bundle.module.localizedString(forKey: key, value: nil, table: nil)
        return words != key ? words : Bundle.module.localizedString(forKey: unknown, value: nil, table: nil)
    }
}

// The Archive (049): the tabs that were tidied out of this Space, newest first. One that is
// chosen is a tab again; one is forgotten, or all of them. The fourth of the Library's lists.
public import SwiftUI
import WTContract
import WTDesign

public struct ArchiveView: View {
    @State private var model: ArchiveModel
    private let switcher: AnyView?
    private let broughtBack: () -> Void
    private let done: () -> Void
    /// The question of whether to forget them all is up.
    @State private var askingToForgetAll = false
    @Environment(\.locale) private var locale
    @Environment(\.timeZone) private var timeZone
    @Environment(\.calendar) private var calendar

    /// `broughtBack` is called when a tab a person chose is a tab again; `done` when they are
    /// finished. With a `switcher`, it stands where the title would.
    public init(model: ArchiveModel, switcher: AnyView? = nil, broughtBack: @escaping () -> Void, done: @escaping () -> Void) {
        _model = State(initialValue: model)
        self.switcher = switcher
        self.broughtBack = broughtBack
        self.done = done
    }

    public var body: some View {
        VStack(spacing: 0) {
            header
                .padding(.bottom, Tokens.Space.s)
            Divider()
            content
        }
        .task { model.reload() }
        .confirmationDialog(
            Text(Words.archiveClearTitle),
            isPresented: $askingToForgetAll,
            titleVisibility: .visible
        ) {
            Button(role: .destructive) {
                model.forgetEverything()
            } label: {
                Text(Words.archiveClearEverything)
            }
            .accessibilityIdentifier("archive-clear-everything")
            Button(role: .cancel) {} label: {
                Text(Words.archiveClearCancel)
            }
        } message: {
            Text(Words.archiveClearDetail)
        }
    }

    private var header: some View {
        ListHeader(stacked: switcher != nil && ListHeader<AnyView, AnyView>.stacksASwitch) {
            if let switcher {
                switcher
            } else {
                Text(Words.archiveTitle)
                    .font(.headline)
                    .accessibilityAddTraits(.isHeader)
            }
        } buttons: {
            Button {
                askingToForgetAll = true
            } label: {
                Text(Words.archiveClear)
                    .frame(minHeight: BrowserWindowView.hitSize)
            }
            // Neutral, as every button with words is (F14).
            .buttonStyle(.bordered)
            .tint(.primary)
            // There is nothing to forget in an Archive that lists nothing.
            .disabled(model.entries.isEmpty)
            .accessibilityIdentifier("archive-clear")
            Button(action: done) {
                Text(Words.archiveDone)
                    .frame(minHeight: BrowserWindowView.hitSize)
            }
            .buttonStyle(.bordered)
            .tint(.primary)
            .keyboardShortcut(.cancelAction)
            .accessibilityIdentifier("archive-done")
        }
        .padding(.horizontal, Tokens.Space.m)
        .padding(.top, Tokens.Space.s)
    }

    @ViewBuilder private var content: some View {
        if !model.loaded {
            // Nothing is said until the core has answered: not that nothing was tidied.
            Spacer()
        } else if model.entries.isEmpty {
            // In the middle of the list's place; and moved up and down where the words are
            // taller than it, at the largest text sizes, so that none is cut short.
            GeometryReader { place in
                ScrollView {
                    VStack(spacing: Tokens.Space.s) {
                        Text(Words.archiveEmpty)
                            .multilineTextAlignment(.center)
                            .fixedSize(horizontal: false, vertical: true)
                            .accessibilityIdentifier("archive-nothing")
                        // How a Space comes to be tidied: nothing is, until a person says so.
                        Text(Words.archiveEmptyDetail)
                            .font(.callout)
                            .multilineTextAlignment(.center)
                            .fixedSize(horizontal: false, vertical: true)
                    }
                    .padding(Tokens.Space.l)
                    .frame(maxWidth: .infinity, minHeight: place.size.height)
                }
            }
        } else {
            ScrollView {
                LazyVStack(spacing: 0) {
                    ForEach(model.entries, id: \.id) { entry in
                        row(entry)
                        Divider().padding(.leading, Tokens.Space.m)
                    }
                    if model.more {
                        Button {
                            model.showOlder()
                        } label: {
                            Text(Words.archiveOlder)
                                .frame(maxWidth: .infinity, minHeight: BrowserWindowView.hitSize)
                        }
                        // Plain, so its words are in the primary colour: they must be read (F14).
                        .buttonStyle(.plain)
                        .accessibilityIdentifier("archive-older")
                    }
                }
            }
        }
    }

    private func row(_ entry: WTArchiveEntry) -> some View {
        HStack(spacing: Tokens.Space.s) {
            Button {
                Task {
                    if await model.bringBack(entry) {
                        broughtBack()
                    }
                }
            } label: {
                // Each on lines of its own, as a visit's are (008): the name, the address, and
                // when it was tidied.
                VStack(alignment: .leading, spacing: Tokens.Space.xs) {
                    // A page with no name is named by its address.
                    Text(entry.title.isEmpty ? entry.display : entry.title)
                        .lineLimit(2)
                    Group {
                        if !entry.title.isEmpty {
                            Text(entry.display)
                                .lineLimit(2)
                                .truncationMode(.middle)
                                .accessibilityIdentifier("archive-address")
                        }
                        Text(verbatim: Words.archiveTidied.filled(when(entry), locale: locale))
                    }
                    .font(.footnote)
                }
                .frame(maxWidth: .infinity, minHeight: BrowserWindowView.hitSize, alignment: .leading)
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)
            .accessibilityIdentifier("archive-entry")
            Button {
                model.forget(entry)
            } label: {
                Label { Text(Words.archiveForget) } icon: { Image(systemName: "trash") }
                    .labelStyle(.iconOnly)
                    .frame(minWidth: BrowserWindowView.hitSize, minHeight: BrowserWindowView.hitSize)
                    .contentShape(Rectangle())
            }
            .buttonStyle(.borderless)
            .accessibilityIdentifier("archive-forget")
        }
        .padding(.horizontal, Tokens.Space.m)
        .padding(.vertical, Tokens.Space.xs)
    }

    /// When a tab was tidied, in the person's own words for dates and their own time zone.
    private func when(_ entry: WTArchiveEntry) -> String {
        let date = Date(timeIntervalSince1970: Double(entry.tidiedAt) / 1000)
        var style = Date.FormatStyle(date: .abbreviated, time: .shortened, locale: locale, calendar: calendar)
        style.timeZone = timeZone
        return date.formatted(style)
    }
}

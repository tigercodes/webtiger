// The History list (008): the pages visited, newest first, found by words of their titles and
// addresses, opened again by choosing one, and forgotten one at a time or many at once.
public import SwiftUI
public import WTContract
import WTDesign

public struct HistoryView: View {
    @State private var model: HistoryModel
    private let switcher: AnyView?
    private let open: (WTHistoryEntry) -> Void
    private let done: () -> Void
    /// The question of which visits to forget is up.
    @State private var choosingWhatToForget = false
    @Environment(\.locale) private var locale
    @Environment(\.timeZone) private var timeZone
    @Environment(\.calendar) private var calendar

    /// `open` is called with the visit a person chose; `done` when they are finished. With a
    /// `switcher`, it stands where the title would: the list is then one of two a person can
    /// choose between (013).
    public init(model: HistoryModel, switcher: AnyView? = nil, open: @escaping (WTHistoryEntry) -> Void, done: @escaping () -> Void) {
        _model = State(initialValue: model)
        self.switcher = switcher
        self.open = open
        self.done = done
    }

    public var body: some View {
        VStack(spacing: 0) {
            header
            search
            Divider()
            content
        }
        .task { model.reload() }
        .confirmationDialog(
            Text("history.clear.title", bundle: .module),
            isPresented: $choosingWhatToForget,
            titleVisibility: .visible
        ) {
            Button {
                model.forget(since: Date.now.addingTimeInterval(-3600))
            } label: {
                Text("history.clear.hour", bundle: .module)
            }
            Button {
                model.forget(since: calendar.startOfDay(for: .now))
            } label: {
                Text("history.clear.today", bundle: .module)
            }
            Button(role: .destructive) {
                model.forgetEverything()
            } label: {
                Text("history.clear.everything", bundle: .module)
            }
            .accessibilityIdentifier("history-clear-everything")
            Button(role: .cancel) {} label: {
                Text("history.clear.cancel", bundle: .module)
            }
        } message: {
            Text("history.clear.detail", bundle: .module)
        }
    }

    private var header: some View {
        ListHeader(stacked: switcher != nil && ListHeader<AnyView, AnyView>.stacksASwitch) {
            if let switcher {
                switcher
            } else {
                Text("history.title", bundle: .module)
                    .font(.headline)
                    .accessibilityAddTraits(.isHeader)
            }
        } buttons: {
            // A button that asks, and not a menu: forgetting is chosen in two steps, and a
            // menu's button has no action of its own for the Mac's assistive technologies.
            Button {
                choosingWhatToForget = true
            } label: {
                Text("history.clear", bundle: .module)
                    .frame(minHeight: BrowserWindowView.hitSize)
            }
            // Neutral, as every button with words is: a tinted label does not reach the
            // contrast words need in both appearances (F14).
            .buttonStyle(.bordered)
            .tint(.primary)
            .accessibilityIdentifier("history-clear")
            Button(action: done) {
                Text("history.done", bundle: .module)
                    .frame(minHeight: BrowserWindowView.hitSize)
            }
            .buttonStyle(.bordered)
            .tint(.primary)
            .keyboardShortcut(.cancelAction)
            .accessibilityIdentifier("history-done")
        }
        .padding(.horizontal, Tokens.Space.m)
        .padding(.top, Tokens.Space.s)
    }

    private var search: some View {
        HStack(spacing: Tokens.Space.s) {
            Image(systemName: "magnifyingglass")
                .foregroundStyle(.secondary)
                .accessibilityHidden(true)
            TextField(text: $model.text) {
                Text("history.search", bundle: .module)
            }
            .textFieldStyle(.plain)
            .autocorrectionDisabled()
            #if !os(macOS)
                .textInputAutocapitalization(.never)
            #endif
                .accessibilityLabel(Text("history.search", bundle: .module))
                .accessibilityIdentifier("history-search")
        }
        .padding(.horizontal, Tokens.Space.m)
        .frame(minHeight: Tokens.Size.fieldHeight)
        .background(Tokens.Palette.fieldFill, in: RoundedRectangle(cornerRadius: Tokens.Radius.field))
        .padding(Tokens.Space.m)
    }

    @ViewBuilder private var content: some View {
        if !model.loaded {
            // Nothing is said until the core has answered: not that nothing was visited.
            Spacer()
        } else if model.entries.isEmpty {
            Spacer()
            Text(model.text.isEmpty ? "history.empty" : "history.none_found", bundle: .module)
                .multilineTextAlignment(.center)
                .padding(Tokens.Space.l)
                .accessibilityIdentifier("history-nothing")
            Spacer()
        } else {
            ScrollView {
                LazyVStack(spacing: 0) {
                    ForEach(model.entries, id: \.visitID) { entry in
                        row(entry)
                        Divider().padding(.leading, Tokens.Space.m)
                    }
                    if model.more {
                        Button {
                            model.showOlder()
                        } label: {
                            Text("history.older", bundle: .module)
                                .frame(maxWidth: .infinity, minHeight: BrowserWindowView.hitSize)
                        }
                        // Plain, so its words are in the primary colour: they must be read (F14).
                        .buttonStyle(.plain)
                        .accessibilityIdentifier("history-older")
                    }
                }
            }
        }
    }

    private func row(_ entry: WTHistoryEntry) -> some View {
        HStack(spacing: Tokens.Space.s) {
            Button {
                open(entry)
            } label: {
                // Each on lines of its own, so that at the largest text sizes none is cut
                // short by its neighbour: the name, the address, and when. (The address and the
                // time shared a line at first; Apple's audit said they would be clipped.)
                VStack(alignment: .leading, spacing: Tokens.Space.xs) {
                    // A page with no title is named by its address.
                    Text(entry.title.isEmpty ? entry.display : entry.title)
                        .lineLimit(2)
                    Group {
                        if !entry.title.isEmpty {
                            // An address may be any length: it is cut to two lines, in the
                            // middle, so that its site and its end are both there, and it is
                            // read out whole.
                            Text(entry.display)
                                .lineLimit(2)
                                .truncationMode(.middle)
                                .accessibilityIdentifier("history-address")
                        }
                        Text(when(entry))
                    }
                    .font(.footnote)
                    // The primary colour: these are words a person must be able to read (F14).
                }
                .frame(maxWidth: .infinity, minHeight: BrowserWindowView.hitSize, alignment: .leading)
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)
            .accessibilityIdentifier("history-entry")
            Button {
                model.forget(entry)
            } label: {
                Label { Text("history.forget", bundle: .module) } icon: { Image(systemName: "trash") }
                    .labelStyle(.iconOnly)
                    .frame(minWidth: BrowserWindowView.hitSize, minHeight: BrowserWindowView.hitSize)
                    .contentShape(Rectangle())
            }
            .buttonStyle(.borderless)
            .accessibilityIdentifier("history-forget")
        }
        .padding(.horizontal, Tokens.Space.m)
        .padding(.vertical, Tokens.Space.xs)
    }

    /// When a visit was, in the person's own words for dates and their own time zone.
    private func when(_ entry: WTHistoryEntry) -> String {
        let date = Date(timeIntervalSince1970: Double(entry.visitedAtMs) / 1000)
        var style = Date.FormatStyle(date: .abbreviated, time: .shortened, locale: locale, calendar: calendar)
        style.timeZone = timeZone
        return date.formatted(style)
    }
}

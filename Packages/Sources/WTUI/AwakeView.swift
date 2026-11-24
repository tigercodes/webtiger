// What is awake (056): every tab that has a page loaded, wherever it is, with why it is awake;
// and "Sleep All Others". The fifth of the Library's lists. It is asked for when it is opened,
// and again whenever the window's own row changes while it is open: there is no timer.
public import SwiftUI
public import WTContract
import WTDesign

public struct AwakeView: View {
    @State private var model: AwakeModel
    private let switcher: AnyView?
    private let watching: [WTTabSummary]
    private let done: () -> Void
    @Environment(\.locale) private var locale

    /// `watching`: the window's row, which the list is asked for again with. `done` is called
    /// when the person is finished. With a `switcher`, it stands where the title would.
    public init(model: AwakeModel, switcher: AnyView? = nil, watching: [WTTabSummary] = [], done: @escaping () -> Void) {
        _model = State(initialValue: model)
        self.switcher = switcher
        self.watching = watching
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
        .onChange(of: watching) { model.reload() }
    }

    private var header: some View {
        ListHeader(stacked: switcher != nil && ListHeader<AnyView, AnyView>.stacksASwitch) {
            if let switcher {
                switcher
            } else {
                Text(Words.awakeTitle)
                    .font(.headline)
                    .accessibilityAddTraits(.isHeader)
            }
        } buttons: {
            Button {
                model.sleepOthers()
            } label: {
                Text(Words.awakeSleepOthers)
                    .frame(minHeight: BrowserWindowView.hitSize)
            }
            // Neutral, as every button with words is (F14).
            .buttonStyle(.bordered)
            .tint(.primary)
            // Nothing that is listed could be put to sleep: there are no others.
            .disabled(!model.othersCouldSleep)
            .accessibilityIdentifier("awake-sleep-others")
            Button(action: done) {
                Text(Words.awakeDone)
                    .frame(minHeight: BrowserWindowView.hitSize)
            }
            .buttonStyle(.bordered)
            .tint(.primary)
            .keyboardShortcut(.cancelAction)
            .accessibilityIdentifier("awake-done")
        }
        .padding(.horizontal, Tokens.Space.m)
        .padding(.top, Tokens.Space.s)
    }

    @ViewBuilder private var content: some View {
        if !model.loaded {
            // Nothing is said until the core has answered: not that no tab is awake.
            Spacer()
        } else if model.tabs.isEmpty {
            // In the middle of the list's place; and moved up and down where the words are
            // taller than it, at the largest text sizes, so that none is cut short.
            GeometryReader { place in
                ScrollView {
                    VStack(spacing: Tokens.Space.s) {
                        Text(Words.awakeEmpty)
                            .multilineTextAlignment(.center)
                            .fixedSize(horizontal: false, vertical: true)
                            .accessibilityIdentifier("awake-nothing")
                        counts
                    }
                    .padding(Tokens.Space.l)
                    .frame(maxWidth: .infinity, minHeight: place.size.height)
                }
            }
        } else {
            ScrollView {
                LazyVStack(spacing: 0) {
                    ForEach(model.tabs, id: \.tabID) { tab in
                        row(tab)
                        Divider().padding(.leading, Tokens.Space.m)
                    }
                    counts
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(.horizontal, Tokens.Space.m)
                        .padding(.vertical, Tokens.Space.s)
                }
            }
        }
    }

    /// How many are asleep; and, in an ordinary window, how many are awake in private ones.
    private var counts: some View {
        VStack(alignment: .leading, spacing: Tokens.Space.xs) {
            Text(verbatim: Words.awakeAsleep.filled(model.asleep, locale: locale))
                .accessibilityIdentifier("awake-asleep")
            if model.privateTabs > 0 {
                Text(verbatim: Words.awakePrivate.filled(model.privateTabs, locale: locale))
                    .accessibilityIdentifier("awake-private")
            }
        }
        .font(.footnote)
    }

    /// A tab that is awake: its name, where it is, and why it is awake. Not a button: a tab
    /// is gone to where it is, in its row.
    private func row(_ tab: WTAwakeTab) -> some View {
        VStack(alignment: .leading, spacing: Tokens.Space.xs) {
            Text(verbatim: Self.name(tab, locale: locale))
                .lineLimit(2)
            Group {
                Text(verbatim: Self.place(tab, locale: locale))
                    .lineLimit(2)
                    .truncationMode(.middle)
                // Nothing, for a reason this shell has no words for: no empty line then.
                if !Self.why(tab, locale: locale).isEmpty {
                    Text(verbatim: Self.why(tab, locale: locale))
                        .accessibilityIdentifier("awake-why")
                }
            }
            .font(.footnote)
        }
        .frame(maxWidth: .infinity, minHeight: BrowserWindowView.hitSize, alignment: .leading)
        .padding(.horizontal, Tokens.Space.m)
        .padding(.vertical, Tokens.Space.xs)
        // One thing to a screen reader: the tab, where it is, and why it is awake.
        .accessibilityElement(children: .combine)
        .accessibilityIdentifier("awake-tab")
    }

    /// A tab's name: its page's title, or its site, or the words for a tab with no page.
    static func name(_ tab: WTAwakeTab, locale: Locale) -> String {
        if !tab.title.isEmpty {
            return tab.title
        }
        return tab.host.isEmpty ? Words.tabsUntitled.string(locale) : tab.host
    }

    /// Where a tab is: its Space by name, with the shell's own word for the Space the app
    /// made, or that it is a private window's; and its site, when its name is not that.
    static func place(_ tab: WTAwakeTab, locale: Locale) -> String {
        let space = if tab.privateWindow {
            Words.privateWindow.string(locale)
        } else {
            tab.spaceName.isEmpty ? Words.spaceHome.string(locale) : tab.spaceName
        }
        return tab.host.isEmpty || tab.title.isEmpty ? space : "\(space) · \(tab.host)"
    }

    /// Why a tab is awake, in words: that it is on screen, and each of the core's reasons, in
    /// the core's order. With nothing to keep it awake, that it will sleep by itself. (A tab
    /// whose only reasons are ones this shell has no words for is not said to be about to
    /// sleep: nothing is said of why.)
    static func why(_ tab: WTAwakeTab, locale: Locale) -> String {
        if !tab.inSight, tab.keepsAwake.isEmpty {
            return Words.awakeWhyWaiting.string(locale)
        }
        var said: [String] = tab.inSight ? [Words.awakeWhyInSight.string(locale)] : []
        said += tab.keepsAwake.compactMap { word(for: $0)?.string(locale) }
        return said.joined(separator: ", ")
    }

    /// The words for one of the core's reasons. A reason this shell has no words for (a newer
    /// core's) is not said, rather than said wrongly.
    static func word(for reason: WTKeepAwake) -> Word? {
        switch reason {
        case .sound: Words.awakeWhySound
        case .capture: Words.awakeWhyCapture
        case .typed: Words.awakeWhyTyped
        case .download: Words.awakeWhyDownload
        case .question: Words.awakeWhyQuestion
        case .loading: Words.awakeWhyLoading
        case .unspecified, .UNRECOGNIZED: nil
        }
    }
}

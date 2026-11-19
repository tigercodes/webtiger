// A browser window: its tabs in a strip, a toolbar, a progress line, and the page of the tab
// in front (005). The strip is one plain row on every size of window; the sidebar and the tab
// strip proper arrive in W2.
public import SwiftUI
import WTContract
import WTDesign

public struct BrowserWindowView: View {
    @State private var model: BrowserModel
    private let session: BrowserSession
    @FocusState private var addressFocused: Bool
    @State private var addressSelection: TextSelection?
    /// The History list is open over the window (008).
    @State private var showingHistory = false
    @FocusState private var findFocused: Bool
    /// What the find bar opens on, once the window has opened: for a picture of the bar.
    private let finding: String?
    @Environment(\.dismiss) private var dismiss

    public init(session: BrowserSession) {
        self.init(session: session, finding: nil)
    }

    /// A window whose find bar is open on `finding`: the design catalogue's (004).
    init(session: BrowserSession, finding: String?) {
        self.session = session
        self.finding = finding
        _model = State(initialValue: BrowserModel(session: session, finding: finding))
    }

    public var body: some View {
        VStack(spacing: 0) {
            TabStrip(model: model, onFind: { model.openFind() }, onHistory: { showingHistory = true })
            toolbar
            if model.isFinding {
                findBar
            }
            progress
            content
        }
        .background { tabShortcuts }
        .task { await model.open() }
        .onDisappear { model.close() }
        // The find bar takes the keyboard when it opens, unless it is only a picture of one.
        .onChange(of: model.isFinding) { _, isFinding in
            if finding == nil {
                findFocused = isFinding
            }
        }
        .sheet(isPresented: $showingHistory) { history }
        // Another tab came in front. An empty one is given the keyboard in its address field:
        // there is nothing else to do in an empty tab. One with a page keeps the keyboard out
        // of the field. (The window's first snapshot names its first tab, and is no switch.)
        .onChange(of: model.tab.tabID) { before, _ in
            if before != 0 {
                addressFocused = !model.hasPage
            }
        }
        // A window that opens on a page gives the page the keyboard. A new window's first
        // responder is its address field; with nothing typed in it, the field lets go when the
        // page arrives. An empty tab keeps the keyboard in the field, ready for an address.
        .onChange(of: model.hasPage) { _, hasPage in
            if hasPage, addressFocused, !model.hasTypedAddress {
                addressFocused = false
            }
        }
        #if os(macOS)
        .navigationTitle(windowTitle)
        #endif
    }

    /// The History list, over the window. A visit chosen from it loads in the tab in front.
    private var history: some View {
        HistoryView(
            model: HistoryModel(core: session.core, windowID: model.windowID ?? 0),
            open: { entry in
                showingHistory = false
                model.load(entry.url)
            },
            done: { showingHistory = false }
        )
        #if os(macOS)
        .frame(minWidth: 440, idealWidth: 520, minHeight: 420, idealHeight: 560)
        #endif
    }

    private var windowTitle: String {
        if !model.tab.title.isEmpty {
            return model.tab.title
        }
        if !model.tab.address.host.isEmpty {
            return model.tab.address.host
        }
        return String(localized: "app.name", bundle: .module)
    }

    private var toolbar: some View {
        HStack(spacing: Tokens.Space.s) {
            toolbarButton("toolbar.back", symbol: "chevron.backward", identifier: "back", enabled: model.tab.canGoBack) {
                model.perform(.back)
            }
            toolbarButton("toolbar.forward", symbol: "chevron.forward", identifier: "forward", enabled: model.tab.canGoForward) {
                model.perform(.forward)
            }
            if model.isLoading {
                toolbarButton("toolbar.stop", symbol: "xmark", identifier: "stop", enabled: true) { model.perform(.stop) }
            } else {
                toolbarButton("toolbar.reload", symbol: "arrow.clockwise", identifier: "reload", enabled: model.hasPage) {
                    model.perform(.reload)
                }
            }
            addressField
        }
        .background { openLocationShortcut }
        .padding(.horizontal, Tokens.Space.m)
        .frame(minHeight: Tokens.Size.toolbarHeight)
    }

    /// The find bar (010): a field, what was found, the match before and the next, and Done.
    private var findBar: some View {
        HStack(spacing: Tokens.Space.s) {
            HStack(spacing: Tokens.Space.xs) {
                Image(systemName: "magnifyingglass")
                    .foregroundStyle(.secondary)
                    .accessibilityHidden(true)
                TextField(text: $model.findText) {
                    Text("find.field", bundle: .module)
                }
                .textFieldStyle(.plain)
                .focused($findFocused)
                .autocorrectionDisabled()
                #if os(iOS)
                    .textInputAutocapitalization(.never)
                    .submitLabel(.search)
                #endif
                    // Return goes to the next match, and the keyboard stays in the field.
                    .onSubmit {
                        model.findNext()
                        findFocused = true
                    }
                    .accessibilityLabel(Text("find.field", bundle: .module))
                    .accessibilityIdentifier("find-field")
                findOutcome
            }
            .padding(.horizontal, Tokens.Space.s)
            .padding(.vertical, Tokens.Space.xs)
            .frame(minHeight: Tokens.Size.fieldHeight)
            .background(Tokens.Palette.fieldFill, in: RoundedRectangle(cornerRadius: Tokens.Radius.field))
            toolbarButton("find.previous", symbol: "chevron.up", identifier: "find-previous", enabled: !model.findText.isEmpty) {
                model.findPrevious()
            }
            toolbarButton("find.next", symbol: "chevron.down", identifier: "find-next", enabled: !model.findText.isEmpty) {
                model.findNext()
            }
            Button {
                model.closeFind()
            } label: {
                Text("find.done", bundle: .module)
            }
            // Neutral, as every button with words is (F14).
            .buttonStyle(.bordered)
            .tint(.primary)
            .keyboardShortcut(.cancelAction)
            .accessibilityIdentifier("find-done")
        }
        .padding(.horizontal, Tokens.Space.m)
        .frame(minHeight: Tokens.Size.toolbarHeight)
    }

    /// What was found, in words: how many where the engine counts, and that there is none.
    @ViewBuilder private var findOutcome: some View {
        switch model.findOutcome {
        case .nothing:
            EmptyView()
        case .notFound:
            Text("find.none", bundle: .module)
                .lineLimit(1)
                .fixedSize()
                .accessibilityIdentifier("find-outcome")
        case let .match(current, total):
            Text(String(format: String(localized: "find.count", bundle: .module), current, total))
                .monospacedDigit()
                .lineLimit(1)
                .fixedSize()
                .accessibilityIdentifier("find-outcome")
        }
    }

    private func toolbarButton(
        _ title: LocalizedStringKey,
        symbol: String,
        identifier: String,
        enabled: Bool,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            Label { Text(title, bundle: .module) } icon: { Image(systemName: symbol) }
                .labelStyle(.iconOnly)
                .frame(minWidth: Self.hitSize, minHeight: Self.hitSize)
                .contentShape(Rectangle())
        }
        .buttonStyle(.borderless)
        .disabled(!enabled)
        .accessibilityIdentifier(identifier)
    }

    /// ⌘T and ⌘W, as in every browser. The buttons draw nothing; they carry the shortcuts until
    /// the app has its menus (W2).
    private var tabShortcuts: some View {
        Group {
            Button {
                model.openTab()
            } label: {
                Text("tabs.new", bundle: .module)
            }
            .keyboardShortcut("t", modifiers: .command)
            Button {
                showingHistory = true
            } label: {
                Text("history.open", bundle: .module)
            }
            .keyboardShortcut("y", modifiers: .command)
            Button {
                model.openFind()
                findFocused = true
            } label: {
                Text("find.open", bundle: .module)
            }
            .keyboardShortcut("f", modifiers: .command)
            Button {
                model.findNext()
            } label: {
                Text("find.next", bundle: .module)
            }
            .keyboardShortcut("g", modifiers: .command)
            Button {
                model.findPrevious()
            } label: {
                Text("find.previous", bundle: .module)
            }
            .keyboardShortcut("g", modifiers: [.command, .shift])
            Button {
                // On a window's last tab ⌘W closes the window, as it does everywhere on a Mac.
                // The core's own rule is only that a window is never left empty (005, T1).
                if model.tabs.count > 1 {
                    model.closeTab(model.tab.tabID)
                } else {
                    dismiss()
                }
            } label: {
                Text("tabs.close", bundle: .module)
            }
            .keyboardShortcut("w", modifiers: .command)
        }
        .opacity(0)
        .frame(width: 0, height: 0)
        .accessibilityHidden(true)
    }

    /// ⌘L, as in every browser: the keyboard goes to the address field, whole address selected.
    /// The button draws nothing; it is here to carry the shortcut.
    private var openLocationShortcut: some View {
        Button {
            addressFocused = true
        } label: {
            Text("toolbar.open_location", bundle: .module)
        }
        .keyboardShortcut("l", modifiers: .command)
        .opacity(0)
        .frame(width: 0, height: 0)
        .accessibilityHidden(true)
    }

    /// The least a toolbar button measures: a finger's width on a touch screen, less under a pointer.
    static var hitSize: CGFloat {
        #if os(iOS)
            Tokens.Size.minimumHit
        #else
            Tokens.Size.pointerHit
        #endif
    }

    private var addressField: some View {
        HStack(spacing: Tokens.Space.xs) {
            securityMark
            TextField(text: $model.addressText, selection: $addressSelection) {
                Text("address.placeholder", bundle: .module)
            }
            .textFieldStyle(.plain)
            .focused($addressFocused)
            .autocorrectionDisabled()
            #if os(iOS)
                .textInputAutocapitalization(.never)
                .keyboardType(.webSearch)
                .submitLabel(.go)
            #endif
                .onSubmit {
                    model.submit()
                    addressFocused = false
                }
                .accessibilityLabel(Text("address.label", bundle: .module))
                .accessibilityIdentifier("address-field")
        }
        .padding(.horizontal, Tokens.Space.s)
        .padding(.vertical, Tokens.Space.xs)
        .frame(minHeight: Tokens.Size.fieldHeight)
        .background(Tokens.Palette.fieldFill, in: RoundedRectangle(cornerRadius: Tokens.Radius.field))
        .onChange(of: addressFocused) { _, focused in
            model.isEditingAddress = focused
            if focused {
                // As every browser does: the whole address, selected, so typing replaces it.
                // On the next turn of the run loop, once the field holds the whole address:
                // a selection set together with the text is dropped with the old text.
                Task { @MainActor in
                    let text = model.addressText
                    addressSelection = TextSelection(range: text.startIndex ..< text.endIndex)
                }
            }
        }
    }

    @ViewBuilder private var securityMark: some View {
        switch model.tab.address.security {
        case .secure:
            Image(systemName: "lock.fill")
                .foregroundStyle(Tokens.Palette.secure)
                .accessibilityLabel(Text("address.secure", bundle: .module))
                .accessibilityIdentifier("security-secure")
        case .insecure:
            Image(systemName: "exclamationmark.triangle.fill")
                .foregroundStyle(Tokens.Palette.insecure)
                .accessibilityLabel(Text("address.insecure", bundle: .module))
                .accessibilityIdentifier("security-insecure")
        case .unspecified, .local, .UNRECOGNIZED:
            Image(systemName: "magnifyingglass")
                .foregroundStyle(.secondary)
                .accessibilityHidden(true)
        }
    }

    private var progress: some View {
        GeometryReader { geometry in
            Rectangle()
                .fill(Tokens.Palette.accent)
                .frame(width: geometry.size.width * (model.isLoading ? max(model.tab.progress, 0.05) : 0))
                .animation(.easeOut(duration: 0.2), value: model.tab.progress)
        }
        .frame(height: Tokens.Size.progressHeight)
        .background(Tokens.Palette.hairline)
        .accessibilityHidden(true)
    }

    @ViewBuilder private var content: some View {
        if model.hasFailed {
            FailedPageView(code: model.tab.errorCode) { model.perform(.reload) }
        } else if model.hasPage {
            WebViewHost(engine: session.engine, webViewID: model.tab.webViewID)
        } else {
            EmptyTabView()
        }
    }
}

/// The window's tabs in a row: each by its name, the one in front marked, a button to close each
/// and one to open another (005 §5).
struct TabStrip: View {
    let model: BrowserModel
    /// Opens the find bar (010), and the History list (008): their buttons stand at the end of
    /// the row, beside the tabs and not among them, where they stay when the tabs scroll.
    var onFind: () -> Void = {}
    var onHistory: () -> Void = {}
    /// How wide a tab's name may be. It grows with the text's size, so that at the largest
    /// sizes a short name is still whole; a long title is cut to one line at any size.
    @ScaledMetric private var widestName: CGFloat = 180

    var body: some View {
        HStack(spacing: 0) {
            tabs
            Button(action: onFind) {
                Label { Text("find.open", bundle: .module) } icon: { Image(systemName: "magnifyingglass") }
                    .labelStyle(.iconOnly)
                    .frame(minWidth: BrowserWindowView.hitSize, minHeight: BrowserWindowView.hitSize)
                    .contentShape(Rectangle())
            }
            .buttonStyle(.borderless)
            // There is nothing to find in a tab with no page.
            .disabled(!model.hasPage)
            .accessibilityIdentifier("find")
            Button(action: onHistory) {
                Label { Text("history.open", bundle: .module) } icon: { Image(systemName: "clock.arrow.circlepath") }
                    .labelStyle(.iconOnly)
                    .frame(minWidth: BrowserWindowView.hitSize, minHeight: BrowserWindowView.hitSize)
                    .contentShape(Rectangle())
            }
            .buttonStyle(.borderless)
            .accessibilityIdentifier("history")
            .padding(.trailing, Tokens.Space.m)
        }
    }

    private var tabs: some View {
        ScrollViewReader { row in
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: Tokens.Space.xs) {
                    ForEach(model.tabs, id: \.tabID) { tab in
                        chip(tab).id(tab.tabID)
                    }
                    Button {
                        model.openTab()
                    } label: {
                        Label { Text("tabs.new", bundle: .module) } icon: { Image(systemName: "plus") }
                            .labelStyle(.iconOnly)
                            .frame(minWidth: BrowserWindowView.hitSize, minHeight: BrowserWindowView.hitSize)
                            .contentShape(Rectangle())
                    }
                    .buttonStyle(.borderless)
                    .accessibilityIdentifier("new-tab")
                }
                .padding(.horizontal, Tokens.Space.m)
            }
            // The tab that comes in front is brought into view.
            .onChange(of: model.tab.tabID) { _, front in
                row.scrollTo(front)
            }
        }
        .frame(minHeight: BrowserWindowView.hitSize + Tokens.Space.s)
        .accessibilityElement(children: .contain)
        .accessibilityLabel(Text("tabs.list", bundle: .module))
        .accessibilityIdentifier("tab-strip")
    }

    private func chip(_ tab: WTTabSummary) -> some View {
        let isFront = tab.tabID == model.tab.tabID
        return HStack(spacing: 0) {
            Button {
                model.activate(tab.tabID)
            } label: {
                HStack(spacing: Tokens.Space.xs) {
                    if tab.loading {
                        // A dot, not a spinner: a tab that is loading is marked, and stays still.
                        Circle()
                            .fill(Tokens.Palette.accent)
                            .frame(width: 6, height: 6)
                            .accessibilityHidden(true)
                    }
                    name(tab)
                        .lineLimit(1)
                        .fontWeight(isFront ? .semibold : .regular)
                }
                .padding(.leading, Tokens.Space.s)
                .frame(maxWidth: widestName, minHeight: BrowserWindowView.hitSize, alignment: .leading)
                .fixedSize(horizontal: true, vertical: false)
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)
            .accessibilityIdentifier("tab")
            .accessibilityAddTraits(isFront ? .isSelected : [])
            Button {
                model.closeTab(tab.tabID)
            } label: {
                Label { Text("tabs.close", bundle: .module) } icon: { Image(systemName: "xmark") }
                    .labelStyle(.iconOnly)
                    .font(.caption)
                    .frame(minWidth: BrowserWindowView.hitSize, minHeight: BrowserWindowView.hitSize)
                    .contentShape(Rectangle())
            }
            .buttonStyle(.borderless)
            .accessibilityIdentifier("close-tab")
        }
        .background(isFront ? Tokens.Palette.fieldFill : .clear, in: RoundedRectangle(cornerRadius: Tokens.Radius.field))
    }

    /// A tab's name: its page's title, or its site, or the words for a tab with no page.
    private func name(_ tab: WTTabSummary) -> Text {
        if !tab.title.isEmpty {
            Text(verbatim: tab.title)
        } else if !tab.host.isEmpty {
            Text(verbatim: tab.host)
        } else {
            Text("tabs.untitled", bundle: .module)
        }
    }
}

/// Words in the middle of the space they are given, which scroll when that space is too small
/// for them. At the largest text sizes on a narrow screen they do not fit, and words that do
/// not fit are otherwise cut short: "The site's address c…" tells nobody why a page failed.
struct CenteredWords<Content: View>: View {
    @ViewBuilder let content: Content

    var body: some View {
        GeometryReader { space in
            ScrollView {
                content
                    .frame(maxWidth: .infinity, minHeight: space.size.height)
            }
            .scrollBounceBehavior(.basedOnSize)
        }
    }
}

/// A tab with no page yet.
struct EmptyTabView: View {
    var body: some View {
        CenteredWords {
            VStack(spacing: Tokens.Space.s) {
                Text("tab.empty.title", bundle: .module)
                    .font(.largeTitle.weight(.semibold))
                Text("tab.empty.hint", bundle: .module)
            }
            .multilineTextAlignment(.center)
            .padding(Tokens.Space.xl)
            .accessibilityElement(children: .combine)
            .accessibilityIdentifier("empty-tab")
        }
    }
}

/// A page that failed to load. The core gave a code; the words are the shell's.
struct FailedPageView: View {
    let code: String
    let retry: () -> Void

    var body: some View {
        CenteredWords {
            VStack(spacing: Tokens.Space.m) {
                Image(systemName: "exclamationmark.icloud")
                    .font(.system(size: 44))
                    .foregroundStyle(.secondary)
                    .accessibilityHidden(true)
                Text("error.title", bundle: .module)
                    .font(.title2.weight(.semibold))
                Text(Self.words(for: code))
                // A neutral button with its words in the primary colour: readable in both
                // appearances, where a tinted label is not.
                Button(action: retry) { Text("error.retry", bundle: .module) }
                    .buttonStyle(.bordered)
                    .tint(.primary)
                    .accessibilityIdentifier("retry")
            }
            .multilineTextAlignment(.center)
            .padding(Tokens.Space.xl)
            // A container that is an element of its own: without this the identifier is handed
            // down to the children and replaces theirs.
            .accessibilityElement(children: .contain)
            .accessibilityIdentifier("failed-page")
        }
    }

    /// The words for a core error code: the string keyed `error.<code>`, or the general ones
    /// when the code has none of its own.
    static func words(for code: String) -> String {
        let key = "error.\(code)"
        let words = Bundle.module.localizedString(forKey: key, value: nil, table: nil)
        if words != key {
            return words
        }
        return Bundle.module.localizedString(forKey: "error.net.failed", value: nil, table: nil)
    }
}

/// Shown instead of a window when the core would not start.
public struct StartFailedView: View {
    let code: String

    public init(code: String) {
        self.code = code
    }

    public var body: some View {
        CenteredWords {
            VStack(spacing: Tokens.Space.m) {
                Text("error.start.title", bundle: .module)
                    .font(.title2.weight(.semibold))
                Text(String(format: Bundle.module.localizedString(forKey: "error.start.detail", value: nil, table: nil), code))
                    .textSelection(.enabled)
            }
            .multilineTextAlignment(.center)
            .padding(Tokens.Space.xl)
            .accessibilityElement(children: .contain)
            .accessibilityIdentifier("start-failed")
        }
    }
}

// A browser window: its tabs, a toolbar, a progress line, and the page of the tab in front
// (005). The tabs are in a strip along the top, or, where the core says so and there is one to
// draw, in a sidebar down the side (043): the same tabs, and the same page beside them.
public import SwiftUI
#if !os(macOS)
    import QuickLook
#endif
import WTContract
import WTDesign

public struct BrowserWindowView: View {
    @State private var model: BrowserModel
    private let session: BrowserSession
    @FocusState private var addressFocused: Bool
    @State private var addressSelection: TextSelection?
    /// The History list is open over the window (008).
    @State private var showingHistory = false
    /// The list of Spaces is open over the window (024).
    @State private var showingSpaces = false
    /// The page behind a private window's chip (037) is open.
    @State private var showingPrivate = false
    /// Where the app shows one window at a time: the way to the private window, for an
    /// ordinary one; and a private window's ways out. None on a Mac.
    private var door: PrivateDoor?
    private var ways: PrivateWays?
    /// The window is closed in the core when this view goes: so for a window that is a view
    /// of its own. Not where the app keeps the window's model and draws it when it is shown.
    private var closesWithItsView = true
    @FocusState private var findFocused: Bool
    /// What the find bar is open on from the start: for a picture of the bar.
    private let finding: String?
    @Environment(\.dismiss) private var dismiss
    #if !os(macOS)
        /// How wide the window is, as the system classes it: a sidebar has room in a wide
        /// one (043).
        @Environment(\.horizontalSizeClass) private var widthClass
    #endif

    public init(session: BrowserSession) {
        self.init(session: session, finding: nil)
    }

    /// A private window (037): asked of the core as one, and saying so.
    public init(privateIn session: BrowserSession) {
        self.init(session: session, finding: nil)
        model.opensPrivate = true
    }

    /// A window where the app shows one at a time (037 §5.3), drawn over a model that
    /// outlives the drawing: the window is not closed in the core when it is no longer drawn.
    /// An ordinary one is given the way to the private window; a private one its ways out.
    init(model: BrowserModel, session: BrowserSession, door: PrivateDoor?, ways: PrivateWays?) {
        self.session = session
        finding = nil
        _model = State(initialValue: model)
        self.door = door
        self.ways = ways
        closesWithItsView = false
    }

    /// A window as a picture of one (004): its find bar open on `finding`, its address field
    /// holding `typing` with places `offered` for it, or its zoom bar open; and told where
    /// searches go from the start, as `searching` says. `warned`: with the warning before
    /// going on past a certificate open (026).
    init(
        session: BrowserSession,
        finding: String?,
        typing: String? = nil,
        offered: [WTAddressSuggestion] = [],
        zooming: Bool = false,
        searching: WTSearchSettings? = nil,
        warned: Bool = false
    ) {
        self.session = session
        self.finding = finding
        let model = BrowserModel(
            session: session, finding: finding, typing: typing, offered: offered, zooming: zooming, searching: searching
        )
        model.isWarnedOfCertificate = warned
        _model = State(initialValue: model)
    }

    public var body: some View {
        // The page keeps its place in the window whichever way the tabs are laid out: a
        // switch draws the tabs somewhere else, and nothing of the page again (043).
        HStack(spacing: 0) {
            if model.showsSidebar {
                Sidebar(
                    model: model, onFind: { model.openFind() }, onZoom: { model.openZoom() },
                    onHistory: { showingHistory = true }, onSpaces: openSpaces,
                    toolbar: { sideToolbar }
                )
                .frame(width: Sidebar<EmptyView>.width)
                Divider()
            }
            VStack(spacing: 0) {
                if !model.showsSidebar {
                    TabStrip(
                        model: model, onFind: { model.openFind() }, onZoom: { model.openZoom() },
                        onHistory: { showingHistory = true }, onSpaces: openSpaces
                    )
                    toolbar
                }
                // What the tab asks the person (014): in the tab, under the toolbar, and not
                // over the window. It takes the keyboard from nothing by appearing.
                if let asked = model.prompt {
                    PromptBar(prompt: asked) { model.answerPrompt(allow: $0, text: $1, silence: $2, secret: $3) }
                        // Each prompt is a bar of its own: a field starts from what its page offers.
                        .id(asked.promptID)
                }
                if model.isFinding {
                    findBar
                }
                if model.isZooming {
                    ZoomBar(model: model)
                }
                progress
                content
                    // What is offered for what is typed (011): under the toolbar, over the page.
                    .overlay(alignment: .top) {
                        if !model.suggestions.isEmpty {
                            SuggestionList(model: model) { addressFocused = false }
                        }
                    }
            }
        }
        #if !os(macOS)
        // A window says whether it is wide enough for a sidebar, and again when it changes:
        // an iPad's window made narrow has the strip, and made wide again what it had.
        .onChange(of: widthClass, initial: true) { _, width in
            model.hasRoomForSidebar = width == .regular
        }
        #endif
        .background { KeysNobodySees(window: commands) }
        // The menu bar speaks for the window in front (034).
        .focusedSceneValue(\.windowCommands, commands)
        .task { await model.open() }
        .onDisappear {
            if closesWithItsView {
                model.close()
            }
        }
        .modifier(OutsideLinks(model: model))
        // The find bar takes the keyboard when it opens, unless it is only a picture of one.
        .onChange(of: model.isFinding) { _, isFinding in
            if finding == nil {
                findFocused = isFinding
            }
        }
        .sheet(isPresented: $showingHistory) { history }
        .modifier(SpacesSheet(model: model, isShown: $showingSpaces, door: door))
        // The page that makes a Group or edits one (041).
        .modifier(GroupSheet(model: model))
        // On a Mac a private window is a window of its own: closing it is its way out.
        .modifier(PrivateSheet(isShown: $showingPrivate, ways: ways ?? PrivateWays(close: { dismiss() })))
        // Where searches go (020): over the window. At the first run it is there before the
        // window is used, and stays until an engine is chosen.
        .sheet(isPresented: Binding(get: { model.asksWhereSearchesGo }, set: {
            if !$0 {
                model.keepSearch()
            }
        })) { searchEngines }
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

    /// The list of search engines (020). At the first run it is offered once, with the default
    /// to keep: its button keeps it, and so does swiping the list away.
    @ViewBuilder private var searchEngines: some View {
        if let settings = model.search {
            SearchEnginesView(settings: settings, choose: { model.chooseSearchEngine($0) }, keep: { model.keepSearch() })
            #if os(macOS)
                .frame(minWidth: 480, minHeight: 420)
            #endif
        }
    }

    /// The History list, the Downloads list (013) and the Permissions list (014), over the
    /// window. A visit chosen loads in the tab in front.
    private var history: some View {
        LibraryView(
            history: HistoryModel(core: session.core, windowID: model.windowID ?? 0),
            downloads: DownloadsModel(core: session.core, windowID: model.windowID ?? 0),
            permissions: PermissionsModel(core: session.core, windowID: model.windowID ?? 0),
            open: { entry in
                showingHistory = false
                model.load(entry.url)
            },
            done: { showingHistory = false }
        )
        #if os(macOS)
        // Wide enough for the switch's three names on one line beside the list's buttons.
        .frame(minWidth: 480, idealWidth: 520, minHeight: 420, idealHeight: 560)
        #else
        // A downloaded file a person opened (030): the system's own preview, over the list.
        // It is there while the session's opener has a file to show, and takes the file
        // away when the person has done with it.
        .quickLookPreview(Bindable(session.preview).file)
        #endif
    }

    private var windowTitle: String {
        if !model.tab.title.isEmpty {
            return model.tab.title
        }
        if !model.tab.address.host.isEmpty {
            return model.tab.address.host
        }
        return Words.appName.string()
    }

    /// The chip that names the Space was pressed: the list of Spaces opens. A private window
    /// has no Space: its chip opens the page that says what it is (037).
    private func openSpaces() {
        if model.isPrivate {
            showingPrivate = true
        } else {
            showingSpaces = true
        }
    }

    private var toolbar: some View {
        HStack(spacing: Tokens.Space.s) {
            navigationButtons
            addressField
        }
        .padding(.horizontal, Tokens.Space.m)
        .frame(minHeight: Tokens.Size.toolbarHeight)
    }

    /// The toolbar as the sidebar has it (043): the buttons, and under them the address
    /// field, as wide as the sidebar.
    private var sideToolbar: some View {
        VStack(alignment: .leading, spacing: Tokens.Space.xs) {
            HStack(spacing: Tokens.Space.s) {
                navigationButtons
            }
            addressField
        }
        .padding(.horizontal, Tokens.Space.m)
        .padding(.vertical, Tokens.Space.xs)
    }

    /// Back, forward, and reload or stop.
    @ViewBuilder private var navigationButtons: some View {
        toolbarButton(Words.toolbarBack, symbol: "chevron.backward", identifier: "back", enabled: model.tab.canGoBack) {
            model.perform(.back)
        }
        toolbarButton(Words.toolbarForward, symbol: "chevron.forward", identifier: "forward", enabled: model.tab.canGoForward) {
            model.perform(.forward)
        }
        if model.isLoading {
            toolbarButton(Words.toolbarStop, symbol: "xmark", identifier: "stop", enabled: true) { model.perform(.stop) }
        } else {
            toolbarButton(Words.toolbarReload, symbol: "arrow.clockwise", identifier: "reload", enabled: model.hasPage) {
                model.perform(.reload)
            }
        }
    }

    /// The find bar (010): a field, what was found, the match before and the next, and Done.
    private var findBar: some View {
        HStack(spacing: Tokens.Space.s) {
            HStack(spacing: Tokens.Space.xs) {
                Image(systemName: "magnifyingglass")
                    .foregroundStyle(.secondary)
                    .accessibilityHidden(true)
                TextField(text: $model.findText) {
                    Text(Words.findField)
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
                    .accessibilityLabel(Text(Words.findField))
                    .accessibilityIdentifier("find-field")
                findOutcome
            }
            .padding(.horizontal, Tokens.Space.s)
            .padding(.vertical, Tokens.Space.xs)
            .frame(minHeight: Tokens.Size.fieldHeight)
            .background(Tokens.Palette.fieldFill, in: RoundedRectangle(cornerRadius: Tokens.Radius.field))
            toolbarButton(Words.findPrevious, symbol: "chevron.up", identifier: "find-previous", enabled: !model.findText.isEmpty) {
                model.findPrevious()
            }
            toolbarButton(Words.findNext, symbol: "chevron.down", identifier: "find-next", enabled: !model.findText.isEmpty) {
                model.findNext()
            }
            Button {
                model.closeFind()
            } label: {
                Text(Words.findDone)
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
            Text(Words.findNone)
                .lineLimit(1)
                .fixedSize()
                .accessibilityIdentifier("find-outcome")
        case let .match(current, total):
            Text(Words.findCount.filled(current, total))
                .monospacedDigit()
                .lineLimit(1)
                .fixedSize()
                .accessibilityIdentifier("find-outcome")
        }
    }

    private func toolbarButton(
        _ title: Word,
        symbol: String,
        identifier: String,
        enabled: Bool,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            Label { Text(title) } icon: { Image(systemName: symbol) }
                .labelStyle(.iconOnly)
                .frame(minWidth: Self.hitSize, minHeight: Self.hitSize)
                .contentShape(Rectangle())
        }
        .buttonStyle(.borderless)
        .disabled(!enabled)
        .accessibilityIdentifier(identifier)
    }

    /// What this window offers by key and by menu (034): its model, and what only the window
    /// can do.
    private var commands: WindowCommands {
        WindowCommands(
            model: model,
            openLocation: { addressFocused = true },
            find: {
                model.openFind()
                findFocused = true
            },
            showHistory: { showingHistory = true },
            closeWindow: { dismiss() }
        )
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
                Text(Words.addressPlaceholder)
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
                // The arrow keys move through what is offered; Escape puts the list away and
                // leaves what was typed.
                .onKeyPress(.downArrow) {
                    guard !model.suggestions.isEmpty else { return .ignored }
                    model.markNextSuggestion()
                    return .handled
                }
                .onKeyPress(.upArrow) {
                    guard !model.suggestions.isEmpty else { return .ignored }
                    model.markPreviousSuggestion()
                    return .handled
                }
                .onKeyPress(.escape) {
                    guard !model.suggestions.isEmpty else { return .ignored }
                    model.putSuggestionsAway()
                    return .handled
                }
                .accessibilityLabel(Text(Words.addressLabel))
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
        switch model.fieldSecurity {
        case .secure:
            Image(systemName: "lock.fill")
                .foregroundStyle(Tokens.Palette.secure)
                .accessibilityLabel(Text(Words.addressSecure))
                .accessibilityIdentifier("security-secure")
        case .insecure:
            Image(systemName: "exclamationmark.triangle.fill")
                .foregroundStyle(Tokens.Palette.insecure)
                .accessibilityLabel(Text(Words.addressInsecure))
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
            FailedPageView(code: model.tab.errorCode, goingOn: model.goingOn) { model.perform(.reload) }
        } else if model.hasPage {
            WebViewHost(engine: session.engine, webViewID: model.tab.webViewID, pageWaits: model.pageWaits)
        } else {
            EmptyTabView(searchesGoTo: model.searchEngineName, changeSearch: model.changeSearch, isPrivate: model.isPrivate)
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
    /// The name of the engine searches go to (020). `nil` until the core has said: nothing is
    /// then said of it.
    var searchesGoTo: String?
    var changeSearch: () -> Void = {}
    /// The tab is a private window's (037): it says what private means, and what it does not.
    var isPrivate = false
    @Environment(\.locale) private var locale

    var body: some View {
        CenteredWords {
            VStack(spacing: Tokens.Space.m) {
                VStack(spacing: Tokens.Space.s) {
                    Text(Words.tabEmptyTitle)
                        .font(.largeTitle.weight(.semibold))
                    Text(Words.tabEmptyHint)
                }
                .accessibilityElement(children: .combine)
                .accessibilityIdentifier("empty-tab")
                if isPrivate {
                    PrivateWords()
                }
                // Where what is typed goes when it is not an address, said where it is about
                // to happen, with the way to change it.
                if let searchesGoTo {
                    Text(Self.goesTo(searchesGoTo, locale: locale))
                        .accessibilityIdentifier("search-goes-to")
                    Button(action: changeSearch) {
                        Text(Words.searchChange)
                            .frame(minHeight: BrowserWindowView.hitSize)
                    }
                    .buttonStyle(.bordered)
                    .tint(.primary)
                    .accessibilityIdentifier("search-change")
                }
            }
            .multilineTextAlignment(.center)
            .padding(Tokens.Space.xl)
        }
    }

    /// The sentence that says where searches go.
    static func goesTo(_ engine: String, locale: Locale = .current) -> String {
        Words.searchGoesTo.filled(engine, locale: locale)
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
                Text(Words.errorStartTitle)
                    .font(.title2.weight(.semibold))
                Text(Words.errorStartDetail.filled(code))
                    .textSelection(.enabled)
            }
            .multilineTextAlignment(.center)
            .padding(Tokens.Space.xl)
            .accessibilityElement(children: .contain)
            .accessibilityIdentifier("start-failed")
        }
    }
}

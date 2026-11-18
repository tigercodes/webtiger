// A browser window: a toolbar, a progress line, and the page. W0 has one tab; the sidebar
// and the tab strip arrive in W2.
public import SwiftUI
import WTContract
import WTDesign

public struct BrowserWindowView: View {
    @State private var model: BrowserModel
    private let session: BrowserSession
    @FocusState private var addressFocused: Bool
    @State private var addressSelection: TextSelection?

    public init(session: BrowserSession) {
        self.session = session
        _model = State(initialValue: BrowserModel(session: session))
    }

    public var body: some View {
        VStack(spacing: 0) {
            toolbar
            progress
            content
        }
        .task { await model.open() }
        .onDisappear { model.close() }
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
    private static var hitSize: CGFloat {
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

/// A tab with no page yet.
struct EmptyTabView: View {
    var body: some View {
        VStack(spacing: Tokens.Space.s) {
            Text("tab.empty.title", bundle: .module)
                .font(.largeTitle.weight(.semibold))
            Text("tab.empty.hint", bundle: .module)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .accessibilityElement(children: .combine)
        .accessibilityIdentifier("empty-tab")
    }
}

/// A page that failed to load. The core gave a code; the words are the shell's.
struct FailedPageView: View {
    let code: String
    let retry: () -> Void

    var body: some View {
        VStack(spacing: Tokens.Space.m) {
            Image(systemName: "exclamationmark.icloud")
                .font(.system(size: 44))
                .foregroundStyle(.secondary)
                .accessibilityHidden(true)
            Text("error.title", bundle: .module)
                .font(.title2.weight(.semibold))
            Text(Self.words(for: code))
                .multilineTextAlignment(.center)
            // A neutral button with its words in the primary colour: readable in both
            // appearances, where a tinted label is not.
            Button(action: retry) { Text("error.retry", bundle: .module) }
                .buttonStyle(.bordered)
                .tint(.primary)
                .accessibilityIdentifier("retry")
        }
        .padding(Tokens.Space.xl)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        // A container that is an element of its own: without this the identifier is handed
        // down to the children and replaces theirs.
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier("failed-page")
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
        VStack(spacing: Tokens.Space.m) {
            Text("error.start.title", bundle: .module)
                .font(.title2.weight(.semibold))
            Text(String(format: Bundle.module.localizedString(forKey: "error.start.detail", value: nil, table: nil), code))
                .textSelection(.enabled)
        }
        .padding(Tokens.Space.xl)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier("start-failed")
    }
}

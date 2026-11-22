// The design catalogue (004): every design token, and the window in each of its states. A
// designer opens it (`WEBTIGER_CATALOGUE=1`); the snapshot tests draw the same views.
public import SwiftUI
import WTDesign

public struct CatalogueView: View {
    public init() {}

    public var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: Tokens.Space.xl) {
                Text(Words.catalogueTitle)
                    .font(.largeTitle.weight(.semibold))
                    .accessibilityAddTraits(.isHeader)
                CatalogueTokens()
                heading(Words.catalogueStates)
                ForEach(CatalogueState.allCases) { state in
                    VStack(alignment: .leading, spacing: Tokens.Space.s) {
                        Text(state.title)
                            .font(.headline)
                        CatalogueWindow(state: state)
                            .frame(height: CatalogueWindow.height)
                            .clipShape(RoundedRectangle(cornerRadius: Tokens.Radius.tile))
                            .overlay(RoundedRectangle(cornerRadius: Tokens.Radius.tile).strokeBorder(Tokens.Palette.hairline))
                    }
                }
                ForEach(CatalogueHistory.allCases) { state in
                    VStack(alignment: .leading, spacing: Tokens.Space.s) {
                        Text(state.title)
                            .font(.headline)
                        CatalogueHistoryList(state: state)
                            .frame(height: CatalogueWindow.height)
                            .clipShape(RoundedRectangle(cornerRadius: Tokens.Radius.tile))
                            .overlay(RoundedRectangle(cornerRadius: Tokens.Radius.tile).strokeBorder(Tokens.Palette.hairline))
                    }
                }
                ForEach(CatalogueDownloads.allCases) { state in
                    VStack(alignment: .leading, spacing: Tokens.Space.s) {
                        Text(state.title)
                            .font(.headline)
                        CatalogueDownloadsList(state: state)
                            .frame(height: CatalogueWindow.height)
                            .clipShape(RoundedRectangle(cornerRadius: Tokens.Radius.tile))
                            .overlay(RoundedRectangle(cornerRadius: Tokens.Radius.tile).strokeBorder(Tokens.Palette.hairline))
                    }
                }
                ForEach(CataloguePermissions.allCases) { state in
                    VStack(alignment: .leading, spacing: Tokens.Space.s) {
                        Text(state.title)
                            .font(.headline)
                        CataloguePermissionsList(state: state)
                            .frame(height: CatalogueWindow.height)
                            .clipShape(RoundedRectangle(cornerRadius: Tokens.Radius.tile))
                            .overlay(RoundedRectangle(cornerRadius: Tokens.Radius.tile).strokeBorder(Tokens.Palette.hairline))
                    }
                }
                ForEach(CatalogueSearch.allCases) { state in
                    VStack(alignment: .leading, spacing: Tokens.Space.s) {
                        Text(state.title)
                            .font(.headline)
                        CatalogueSearchEngines(state: state)
                            .frame(height: CatalogueWindow.height)
                            .clipShape(RoundedRectangle(cornerRadius: Tokens.Radius.tile))
                            .overlay(RoundedRectangle(cornerRadius: Tokens.Radius.tile).strokeBorder(Tokens.Palette.hairline))
                    }
                }
                ForEach(CatalogueSpaces.allCases) { state in
                    VStack(alignment: .leading, spacing: Tokens.Space.s) {
                        Text(state.title)
                            .font(.headline)
                        CatalogueSpacesList(state: state)
                            .frame(height: CatalogueWindow.height)
                            .clipShape(RoundedRectangle(cornerRadius: Tokens.Radius.tile))
                            .overlay(RoundedRectangle(cornerRadius: Tokens.Radius.tile).strokeBorder(Tokens.Palette.hairline))
                    }
                }
                ForEach(CatalogueGroups.allCases) { state in
                    VStack(alignment: .leading, spacing: Tokens.Space.s) {
                        Text(state.title)
                            .font(.headline)
                        CatalogueGroupPage(state: state)
                            .frame(height: CatalogueWindow.height)
                            .clipShape(RoundedRectangle(cornerRadius: Tokens.Radius.tile))
                            .overlay(RoundedRectangle(cornerRadius: Tokens.Radius.tile).strokeBorder(Tokens.Palette.hairline))
                    }
                }
                heading(Words.catalogueStartFailed)
                StartFailedView(code: "store.open")
                    .frame(height: CatalogueWindow.height)
                    .overlay(RoundedRectangle(cornerRadius: Tokens.Radius.tile).strokeBorder(Tokens.Palette.hairline))
            }
            .padding(Tokens.Space.xl)
            .frame(maxWidth: 720, alignment: .leading)
            .frame(maxWidth: .infinity)
        }
        .accessibilityIdentifier("catalogue")
    }

    private func heading(_ title: Word) -> some View {
        Text(title)
            .font(.title2.weight(.semibold))
            .accessibilityAddTraits(.isHeader)
    }
}

/// The window itself in one state: `BrowserWindowView` over a scripted core that publishes it.
struct CatalogueWindow: View {
    /// Tall enough for the failed page, the tallest of the states.
    static let height: CGFloat = 300

    @State private var session: BrowserSession
    private let state: CatalogueState

    init(state: CatalogueState) {
        _session = State(initialValue: state.session())
        self.state = state
    }

    var body: some View {
        // Every window is told where searches go (020), as a core tells it: an empty tab
        // says so.
        BrowserWindowView(
            session: session, finding: state.finding, typing: state.typing, offered: state.offered, zooming: state.zooming,
            searching: CatalogueSearch.inUse, warned: state.warned
        )
    }
}

/// The History list in one state: `HistoryView` over a scripted core that answers with it. Its
/// times are shown as one place and one language would show them, so a picture of it is the
/// same wherever it is drawn.
struct CatalogueHistoryList: View {
    @State private var model: HistoryModel

    init(state: CatalogueHistory) {
        _model = State(initialValue: state.model())
    }

    var body: some View {
        HistoryView(model: model, open: { _ in }, done: {})
            .environment(\.timeZone, Self.calendar.timeZone)
            .environment(\.calendar, Self.calendar)
            .environment(\.locale, Locale(identifier: "en_GB"))
    }

    private static let calendar: Calendar = {
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = TimeZone(identifier: "UTC") ?? .gmt
        calendar.locale = Locale(identifier: "en_GB")
        return calendar
    }()
}

/// Every design token, by name: the colours as swatches, the measures drawn to scale.
struct CatalogueTokens: View {
    var body: some View {
        VStack(alignment: .leading, spacing: Tokens.Space.l) {
            section(Words.catalogueColours) {
                ForEach(Tokens.Catalogue.colors, id: \.name) { token in
                    row(token.name) {
                        RoundedRectangle(cornerRadius: Tokens.Radius.field)
                            .fill(token.color)
                            .overlay(RoundedRectangle(cornerRadius: Tokens.Radius.field).strokeBorder(Tokens.Palette.hairline))
                            .frame(width: 96, height: 28)
                    }
                }
            }
            section(Words.catalogueSpace) {
                ForEach(Tokens.Catalogue.space, id: \.name) { token in
                    row(token.name, value: token.value) {
                        Rectangle().fill(Tokens.Palette.accent).frame(width: token.value, height: 12)
                    }
                }
            }
            section(Words.catalogueRadius) {
                ForEach(Tokens.Catalogue.radius, id: \.name) { token in
                    row(token.name, value: token.value) {
                        RoundedRectangle(cornerRadius: token.value)
                            .fill(Tokens.Palette.fieldFill)
                            .overlay(RoundedRectangle(cornerRadius: token.value).strokeBorder(Tokens.Palette.hairline))
                            .frame(width: 96, height: 28)
                    }
                }
            }
            section(Words.catalogueSize) {
                ForEach(Tokens.Catalogue.size, id: \.name) { token in
                    row(token.name, value: token.value) {
                        Rectangle().fill(Tokens.Palette.accent).frame(width: 12, height: token.value)
                    }
                }
            }
        }
        .accessibilityIdentifier("catalogue-tokens")
    }

    private func section(_ title: Word, @ViewBuilder rows: () -> some View) -> some View {
        VStack(alignment: .leading, spacing: Tokens.Space.s) {
            Text(title)
                .font(.title2.weight(.semibold))
                .accessibilityAddTraits(.isHeader)
            rows()
        }
    }

    /// One token: its name, its value when it is a measure, and the token itself drawn.
    private func row(_ name: String, value: CGFloat? = nil, @ViewBuilder drawn: () -> some View) -> some View {
        HStack(spacing: Tokens.Space.m) {
            HStack(spacing: Tokens.Space.m) {
                // A token's name is its identifier in Design/tokens.json: not words, so not translated.
                Text(verbatim: name)
                    .font(.body.monospaced())
                    .frame(width: 140, alignment: .leading)
                // In the primary colour, as every word a person must read is (001 §9, F14).
                Text(verbatim: value.map { "\(Int($0))" } ?? "")
                    .font(.body.monospacedDigit())
                    .frame(width: 36, alignment: .trailing)
            }
            // The words are the row's accessibility element; the drawing is not part of it. A
            // faint token (a hairline) is drawn faintly on purpose, and is not text to be read.
            .accessibilityElement(children: .combine)
            drawn()
                .accessibilityHidden(true)
        }
    }
}

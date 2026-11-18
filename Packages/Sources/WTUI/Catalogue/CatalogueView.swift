// The design catalogue (004): every design token, and the window in each of its states. A
// designer opens it (`WEBTIGER_CATALOGUE=1`); the snapshot tests draw the same views.
public import SwiftUI
import WTDesign

public struct CatalogueView: View {
    public init() {}

    public var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: Tokens.Space.xl) {
                Text("catalogue.title", bundle: .module)
                    .font(.largeTitle.weight(.semibold))
                    .accessibilityAddTraits(.isHeader)
                CatalogueTokens()
                heading("catalogue.states")
                ForEach(CatalogueState.allCases) { state in
                    VStack(alignment: .leading, spacing: Tokens.Space.s) {
                        Text(state.title, bundle: .module)
                            .font(.headline)
                        CatalogueWindow(state: state)
                            .frame(height: CatalogueWindow.height)
                            .clipShape(RoundedRectangle(cornerRadius: Tokens.Radius.tile))
                            .overlay(RoundedRectangle(cornerRadius: Tokens.Radius.tile).strokeBorder(Tokens.Palette.hairline))
                    }
                }
                heading("catalogue.start_failed")
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

    private func heading(_ title: LocalizedStringKey) -> some View {
        Text(title, bundle: .module)
            .font(.title2.weight(.semibold))
            .accessibilityAddTraits(.isHeader)
    }
}

/// The window itself in one state: `BrowserWindowView` over a scripted core that publishes it.
struct CatalogueWindow: View {
    /// Tall enough for the failed page, the tallest of the states.
    static let height: CGFloat = 300

    @State private var session: BrowserSession

    init(state: CatalogueState) {
        _session = State(initialValue: state.session())
    }

    var body: some View {
        BrowserWindowView(session: session)
    }
}

/// Every design token, by name: the colours as swatches, the measures drawn to scale.
struct CatalogueTokens: View {
    var body: some View {
        VStack(alignment: .leading, spacing: Tokens.Space.l) {
            section("catalogue.colours") {
                ForEach(Tokens.Catalogue.colors, id: \.name) { token in
                    row(token.name) {
                        RoundedRectangle(cornerRadius: Tokens.Radius.field)
                            .fill(token.color)
                            .overlay(RoundedRectangle(cornerRadius: Tokens.Radius.field).strokeBorder(Tokens.Palette.hairline))
                            .frame(width: 96, height: 28)
                    }
                }
            }
            section("catalogue.space") {
                ForEach(Tokens.Catalogue.space, id: \.name) { token in
                    row(token.name, value: token.value) {
                        Rectangle().fill(Tokens.Palette.accent).frame(width: token.value, height: 12)
                    }
                }
            }
            section("catalogue.radius") {
                ForEach(Tokens.Catalogue.radius, id: \.name) { token in
                    row(token.name, value: token.value) {
                        RoundedRectangle(cornerRadius: token.value)
                            .fill(Tokens.Palette.fieldFill)
                            .overlay(RoundedRectangle(cornerRadius: token.value).strokeBorder(Tokens.Palette.hairline))
                            .frame(width: 96, height: 28)
                    }
                }
            }
            section("catalogue.size") {
                ForEach(Tokens.Catalogue.size, id: \.name) { token in
                    row(token.name, value: token.value) {
                        Rectangle().fill(Tokens.Palette.accent).frame(width: 12, height: token.value)
                    }
                }
            }
        }
        .accessibilityIdentifier("catalogue-tokens")
    }

    private func section(_ title: LocalizedStringKey, @ViewBuilder rows: () -> some View) -> some View {
        VStack(alignment: .leading, spacing: Tokens.Space.s) {
            Text(title, bundle: .module)
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

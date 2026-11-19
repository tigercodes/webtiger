// The places offered for what is typed in the address field (011 §5): under the toolbar, over
// the page.
import SwiftUI
import WTContract
import WTDesign

/// Each place by its title and its address. Choosing one goes there; the field keeps the
/// keyboard until then.
struct SuggestionList: View {
    let model: BrowserModel
    /// A place was chosen: the address field gives up the keyboard.
    let onChoose: () -> Void

    var body: some View {
        VStack(spacing: 0) {
            ForEach(Array(model.suggestions.enumerated()), id: \.offset) { index, suggestion in
                Button {
                    model.choose(suggestion)
                    onChoose()
                } label: {
                    row(suggestion)
                        .background(index == model.markedSuggestion ? Tokens.Palette.fieldFill : Color.clear)
                        .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
                .accessibilityAddTraits(index == model.markedSuggestion ? .isSelected : [])
                .accessibilityIdentifier("suggestion")
                if index + 1 < model.suggestions.count {
                    Divider().padding(.leading, Tokens.Space.m)
                }
            }
        }
        .background(.background)
        .overlay(alignment: .bottom) { Rectangle().fill(Tokens.Palette.hairline).frame(height: 1) }
        .accessibilityElement(children: .contain)
        .accessibilityLabel(Text("suggestions.list", bundle: .module))
        .accessibilityIdentifier("suggestions")
    }

    private func row(_ suggestion: WTAddressSuggestion) -> some View {
        VStack(alignment: .leading, spacing: Tokens.Space.xs) {
            // A place with no title is named by its address.
            Text(suggestion.title.isEmpty ? suggestion.display : suggestion.title)
                .lineLimit(1)
            if !suggestion.title.isEmpty {
                Text(suggestion.display)
                    .font(.footnote)
                    .lineLimit(1)
                    .truncationMode(.middle)
                    .accessibilityIdentifier("suggestion-address")
            }
        }
        .frame(maxWidth: .infinity, minHeight: BrowserWindowView.hitSize, alignment: .leading)
        .padding(.horizontal, Tokens.Space.m)
        .padding(.vertical, Tokens.Space.xs)
    }
}

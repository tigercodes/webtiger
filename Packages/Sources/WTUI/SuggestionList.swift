// The places offered for what is typed in the address field (011 §5): under the toolbar, over
// the page.
import SwiftUI
import WTContract
import WTDesign

/// Each place by its title and its address. Choosing one goes there; the field keeps the
/// keyboard until then. A tab that is open is a row of the same list, which says so (038).
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
        .accessibilityLabel(Text(Words.suggestionsList))
        .accessibilityIdentifier("suggestions")
    }

    /// What a row is, when it is no place: said at its end.
    static func kind(of suggestion: WTAddressSuggestion) -> Word? {
        if suggestion.tabID != 0 {
            Words.suggestionsSwitchTab
        } else if suggestion.spaceID != 0 {
            Words.suggestionsSpace
        } else if !suggestion.command.isEmpty {
            Words.suggestionsCommand
        } else {
            nil
        }
    }

    private func row(_ suggestion: WTAddressSuggestion) -> some View {
        HStack(spacing: Tokens.Space.m) {
            VStack(alignment: .leading, spacing: Tokens.Space.xs) {
                // A place with no title is named by its address.
                Text(suggestion.title.isEmpty ? suggestion.display : suggestion.title)
                    .lineLimit(1)
                    .accessibilityIdentifier("suggestion-title")
                // (A Space and a command have a name and no address.)
                if !suggestion.title.isEmpty, !suggestion.display.isEmpty {
                    Text(suggestion.display)
                        .font(.footnote)
                        .lineLimit(1)
                        .truncationMode(.middle)
                        .accessibilityIdentifier("suggestion-address")
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            if let kind = Self.kind(of: suggestion) {
                // A tab that is open (038): choosing the row brings it in front. A Space
                // and a command (048): choosing the row shows the one and does the other.
                // Said in words, whole, whatever the title has to give up for it; and in the
                // text's own colour: small and grey, the iPhone's audit found it too faint
                // to read.
                Text(kind)
                    .font(.footnote)
                    .lineLimit(1)
                    .fixedSize()
                    .accessibilityIdentifier("suggestion-switch")
            }
        }
        .frame(maxWidth: .infinity, minHeight: BrowserWindowView.hitSize, alignment: .leading)
        .padding(.horizontal, Tokens.Space.m)
        .padding(.vertical, Tokens.Space.xs)
    }
}

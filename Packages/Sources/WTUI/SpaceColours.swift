// A Space's colour as the shell shows it (025 §6): a mark of its paint beside the Space's name,
// and the colours to choose among where a Space is made or edited. The colours are the core's,
// by its names for them; the paints are the design tokens', and the words the strings'.
import SwiftUI
import WTDesign

enum SpaceColour {
    /// The word for a colour ("Blue"): what a screen reader says of its swatch, and what is
    /// written beside the heading when it is chosen. Nothing for a name this build has no word
    /// for: a code's own text is never shown.
    static func word(_ name: String, locale: Locale = .current) -> String {
        Word(named: "space.colour." + name)?.string(locale) ?? ""
    }
}

/// The mark of a Space's colour: a dot of its paint. A Space with none, or with a colour this
/// build has no paint for, has a ring where the dot would be, so that names stay in line.
struct SpaceMark: View {
    let colour: String
    var size: CGFloat = 10

    var body: some View {
        Group {
            if let paint = Tokens.SpaceColour.paint(colour) {
                Circle().fill(paint)
            } else {
                Circle().strokeBorder(Tokens.Palette.hairline, lineWidth: 1.5)
            }
        }
        .frame(width: size, height: size)
        .accessibilityHidden(true)
    }
}

/// The colours to choose among, each a swatch to press: `colours` are the core's, in its
/// order; one this build has no paint for is left out. `chosen` is the one chosen, or empty.
/// With `offersNone` there is a choice of no colour too.
///
/// The one chosen is ringed, and named in words beside the heading: it is not told by its
/// colour alone.
struct SpaceSwatches: View {
    let colours: [String]
    let offersNone: Bool
    @Binding var chosen: String
    @Environment(\.locale) private var locale

    private var offered: [String] {
        colours.filter { Tokens.SpaceColour.paint($0) != nil }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: Tokens.Space.s) {
            heading
                .font(.headline)
                .accessibilityAddTraits(.isHeader)
                .accessibilityIdentifier("space-colour")
            LazyVGrid(columns: [GridItem(.adaptive(minimum: BrowserWindowView.hitSize), spacing: Tokens.Space.xs)], alignment: .leading) {
                if offersNone {
                    swatch("", word: Words.spaceNoColour.string(locale))
                }
                ForEach(offered, id: \.self) { colour in
                    swatch(colour, word: SpaceColour.word(colour, locale: locale))
                }
            }
        }
    }

    private var heading: Text {
        let word = SpaceColour.word(chosen, locale: locale)
        return word.isEmpty
            ? Text(Words.spaceColour)
            : Text(verbatim: Words.spaceChosenColour.filled(word, locale: locale))
    }

    private func swatch(_ colour: String, word: String) -> some View {
        let isChosen = colour == chosen
        return Button {
            chosen = colour
        } label: {
            SpaceMark(colour: colour, size: 22)
                .padding(Tokens.Space.xs)
                // The ring of the one that is chosen, in the colour of the words.
                .overlay {
                    if isChosen {
                        Circle().strokeBorder(.primary, lineWidth: 2)
                    }
                }
                .frame(minWidth: BrowserWindowView.hitSize, minHeight: BrowserWindowView.hitSize)
                .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityLabel(Text(verbatim: word))
        .accessibilityAddTraits(isChosen ? .isSelected : [])
        .accessibilityIdentifier("space-colour-\(colour.isEmpty ? "none" : colour)")
    }
}

// A Space's icon as the shell shows it (050): the icons to choose among where a Space is made
// or edited, and what is asked of the core when one is given. The icons are the core's, each an
// emoji: it is what is drawn, by the system's own font, and what a screen reader names. There
// is no paint for one and no word.
import SwiftUI
import WTContract
import WTCoreClient
import WTDesign

/// The icons to choose among, each a button to press: `icons` are the core's, in its order.
/// `chosen` is the one chosen, or empty; the first choice is none. The one chosen is ringed,
/// and said to be selected. With no icons to offer (a core that lists none) nothing is drawn.
struct SpaceIcons: View {
    let icons: [String]
    @Binding var chosen: String

    /// What there is to choose among: none, the first, and then the core's icons in its
    /// order. Nothing where the core lists none: there is no choice to make.
    static func choices(among icons: [String]) -> [String] {
        icons.isEmpty ? [] : [""] + icons
    }

    var body: some View {
        let choices = Self.choices(among: icons)
        if !choices.isEmpty {
            VStack(alignment: .leading, spacing: Tokens.Space.s) {
                Text(Words.spaceIcon)
                    .font(.headline)
                    .accessibilityAddTraits(.isHeader)
                    .accessibilityIdentifier("space-icon")
                LazyVGrid(
                    columns: [GridItem(.adaptive(minimum: BrowserWindowView.hitSize), spacing: Tokens.Space.xs)],
                    alignment: .leading
                ) {
                    // Each by its place among the choices: none is nought, and the core's
                    // first is one.
                    ForEach(Array(choices.enumerated()), id: \.offset) { number, icon in
                        choice(icon, number: number)
                    }
                }
            }
        }
    }

    /// One choice: an icon, or none. It is known to a test by its place in the core's list,
    /// from one: an emoji is no name for a button.
    private func choice(_ icon: String, number: Int) -> some View {
        let isChosen = icon == chosen
        return Button {
            chosen = icon
        } label: {
            Group {
                if icon.isEmpty {
                    Image(systemName: "circle.slash")
                } else {
                    Text(verbatim: icon)
                }
            }
            .font(.title2)
            .frame(minWidth: BrowserWindowView.hitSize, minHeight: BrowserWindowView.hitSize)
            // The ring of the one that is chosen, in the colour of the words.
            .overlay {
                if isChosen {
                    RoundedRectangle(cornerRadius: Tokens.Radius.field).strokeBorder(.primary, lineWidth: 2)
                }
            }
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityLabel(icon.isEmpty ? Text(Words.spaceNoIcon) : Text(verbatim: icon))
        .accessibilityAddTraits(isChosen ? .isSelected : [])
        .accessibilityIdentifier(icon.isEmpty ? "space-icon-none" : "space-icon-\(number)")
    }
}

/// A Space's icon where it stands beside a name or in a name's place: the emoji, drawn and
/// not read out. (What is read is the Space's name: the icon is not its name, 050 D5.)
struct SpaceIconMark: View {
    let icon: String

    var body: some View {
        Text(verbatim: icon)
            .accessibilityHidden(true)
            .accessibilityIdentifier("space-icon-mark")
    }
}

public extension BrowserModel {
    /// Gives the Space the window shows an icon, one of the core's, or none (050). False
    /// when the core would not: `spaceRefusal` then has its code.
    func iconSpace(_ icon: String) async -> Bool {
        guard spaceID != 0 else { return false }
        var ask = WTIconSpace()
        ask.spaceID = spaceID
        ask.icon = icon
        spaceRefusal = nil
        do {
            switch try await session.core.send(.iconSpace(ask)).value {
            case .ack:
                return true
            case let .error(refused):
                spaceRefusal = refused.code
            default:
                spaceRefusal = "space.rename_failed"
            }
        } catch let CoreError.refused(code, _) {
            spaceRefusal = code
        } catch {
            spaceRefusal = "space.rename_failed"
        }
        return false
    }
}

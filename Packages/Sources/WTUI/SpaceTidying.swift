// A Space's time (049): after how long out of sight its tabs are tidied into its Archive, or
// never. The times are the core's, in seconds; the words are the strings', by the number of
// seconds. The shell times nothing: it says which time was chosen, and the core tidies.
import SwiftUI
import WTContract
import WTCoreClient
import WTDesign

enum TidyTime {
    /// The words for a time ("After 12 hours"; "Never" for none). Nothing for a time this
    /// build has no words for: a number of seconds is never shown in their place.
    static func words(_ seconds: UInt32, locale: Locale = .current) -> String {
        Word(named: "space.tidy.\(seconds)")?.string(locale) ?? ""
    }
}

/// The times to choose among, each a row to press: `times` are the core's, in its order; one
/// this build has no words for is left out. The one chosen is marked, and said to be selected.
/// With fewer than two to choose among (a core that lists none) nothing is drawn.
struct SpaceTidyTimes: View {
    let times: [UInt32]
    @Binding var chosen: UInt32
    @Environment(\.locale) private var locale

    private var offered: [UInt32] {
        times.filter { !TidyTime.words($0, locale: locale).isEmpty }
    }

    var body: some View {
        if offered.count > 1 {
            VStack(alignment: .leading, spacing: Tokens.Space.s) {
                Text(Words.spaceTidy)
                    .font(.headline)
                    .accessibilityAddTraits(.isHeader)
                    .accessibilityIdentifier("space-tidy")
                Text(Words.spaceTidyDetail)
                    .font(.callout)
                    .fixedSize(horizontal: false, vertical: true)
                ForEach(offered, id: \.self) { time in
                    row(time)
                }
            }
        }
    }

    private func row(_ time: UInt32) -> some View {
        let isChosen = time == chosen
        return Button {
            chosen = time
        } label: {
            HStack(alignment: .firstTextBaseline, spacing: Tokens.Space.s) {
                Image(systemName: isChosen ? "largecircle.fill.circle" : "circle")
                    .accessibilityHidden(true)
                Text(verbatim: TidyTime.words(time, locale: locale))
                    .fixedSize(horizontal: false, vertical: true)
            }
            .frame(maxWidth: .infinity, minHeight: BrowserWindowView.hitSize, alignment: .leading)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityAddTraits(isChosen ? .isSelected : [])
        .accessibilityIdentifier("space-tidy-\(time)")
    }
}

public extension BrowserModel {
    /// Gives the Space the window shows its time, one of the core's, or none (049). False
    /// when the core would not: `spaceRefusal` then has its code.
    func tidySpace(after seconds: UInt32) async -> Bool {
        guard spaceID != 0 else { return false }
        var ask = WTSetSpaceTidying()
        ask.spaceID = spaceID
        ask.afterSeconds = seconds
        spaceRefusal = nil
        do {
            switch try await session.core.send(.setSpaceTidying(ask)).value {
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

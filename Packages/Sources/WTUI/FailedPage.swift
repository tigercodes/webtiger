// A page that failed to load, and, where the core says the load may be gone on with, the warning
// before going on past a certificate that is not trusted (026 §6).
import SwiftUI
import WTDesign

/// What a failed page offers when the core says its load may be gone on with (026): the host
/// whose certificate was not trusted, whether the warning is open, and the two presses there
/// are. Nothing goes on by one press: the first shows the warning, and only the button under
/// the warning goes on.
struct GoingOn {
    let host: String
    let isWarned: Bool
    let advanced: () -> Void
    let goOn: () -> Void
}

/// A page that failed to load. The core gave a code; the words are the shell's.
struct FailedPageView: View {
    let code: String
    var goingOn: GoingOn?
    let retry: () -> Void

    var body: some View {
        CenteredWords {
            VStack(spacing: Tokens.Space.m) {
                Image(systemName: "exclamationmark.icloud")
                    .font(.system(size: 44))
                    .foregroundStyle(.secondary)
                    .accessibilityHidden(true)
                Text(Words.errorTitle)
                    .font(.title2.weight(.semibold))
                Text(Self.words(for: code))
                // A neutral button with its words in the primary colour: readable in both
                // appearances, where a tinted label is not.
                Button(action: retry) { Text(Words.errorRetry) }
                    .buttonStyle(.bordered)
                    .tint(.primary)
                    .accessibilityIdentifier("retry")
                if let goingOn {
                    if goingOn.isWarned {
                        CertificateWarning(goingOn: goingOn)
                    } else {
                        Button(action: goingOn.advanced) { Text(Words.certificateAdvanced) }
                            .buttonStyle(.bordered)
                            .tint(.primary)
                            .accessibilityIdentifier("certificate-advanced")
                    }
                }
            }
            .multilineTextAlignment(.center)
            .padding(Tokens.Space.xl)
            // A container that is an element of its own: without this the identifier is handed
            // down to the children and replaces theirs.
            .accessibilityElement(children: .contain)
            .accessibilityIdentifier("failed-page")
        }
    }

    /// The words for a core error code: the string keyed `error.<code>`. A code with none of
    /// its own has those of the code above it (`net.tls` for `net.tls.date`, 021 §3), so that
    /// a newer core's finer code is never worse than the one it refines; and only then the
    /// general ones.
    static func words(for code: String) -> String {
        var parts = code.split(separator: ".").map(String.init)
        while parts.count >= 2 {
            if let words = Word(named: "error." + parts.joined(separator: ".")) {
                return words.string()
            }
            parts.removeLast()
        }
        return Words.errorNetFailed.string()
    }
}

/// The warning before going on (026 §6): what cannot be confirmed and what could come of it,
/// when going on is reasonable all the same, and under it the one button that goes on, which
/// says that it is not safe.
private struct CertificateWarning: View {
    let goingOn: GoingOn
    @Environment(\.locale) private var locale

    var body: some View {
        VStack(spacing: Tokens.Space.s) {
            Text(Words.certificateWarningTitle)
                .font(.headline)
                .accessibilityAddTraits(.isHeader)
            Text(verbatim: words(Words.certificateWarning))
                .accessibilityIdentifier("certificate-warning-words")
            Text(Words.certificateWarningOnlyIf)
            // Not the page's main button, and not coloured as one: going on is allowed, and is
            // not what the page suggests.
            Button(action: goingOn.goOn) { Text(verbatim: words(Words.certificateProceed)) }
                .buttonStyle(.bordered)
                .tint(.primary)
                .accessibilityIdentifier("certificate-go-on")
        }
        .padding(Tokens.Space.l)
        .frame(maxWidth: 520)
        .background(Tokens.Palette.fieldFill, in: RoundedRectangle(cornerRadius: Tokens.Radius.tile))
        .overlay(RoundedRectangle(cornerRadius: Tokens.Radius.tile).strokeBorder(Tokens.Palette.hairline))
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier("certificate-warning")
    }

    /// Words with the host's name in them.
    private func words(_ key: Word) -> String {
        key.filled(goingOn.host, locale: locale)
    }
}

// A server asks who is there (017 §6): the bar a tab shows for a name and a password. Every
// word in it is the app's or the core's: the server's own are not shown. What is typed is here
// only until it is sent: it is in no snapshot and no saved state.
import SwiftUI
import WTContract
import WTDesign

/// `answer` is told what the person chose: to sign in or not, and with the first the name and
/// the password; Cancel passes on nothing that was typed. Whether the answer came too soon to
/// be the person's is the core's to say (014 §3): the bar stays until the core takes it away.
///
/// No button is the default, and the bar takes the keyboard from nothing by appearing. In the
/// name's field Return goes to the password; in the password's, it signs in.
struct SignInBar: View {
    let signIn: WTSignInPrompt
    let answer: (Bool, String, String) -> Void
    @State private var name = ""
    @State private var password = ""
    @FocusState private var typing: Field?
    @Environment(\.locale) private var locale

    private enum Field {
        case name, password
    }

    var body: some View {
        let title = Self.title(signIn.site, locale: locale)
        VStack(alignment: .leading, spacing: Tokens.Space.xs) {
            // Who gets what is typed, which only the core can write.
            Text(title)
                .font(.headline)
                .fixedSize(horizontal: false, vertical: true)
                .accessibilityIdentifier("prompt-site")
            if signIn.again {
                line("prompt.sign_in.again", identifier: "prompt-again")
            }
            // Said unless the core says the connection is private: a shell told nothing warns.
            if !signIn.confidential {
                line("prompt.sign_in.not_private", identifier: "prompt-not-private")
            }
            TextField(text: $name) {
                Text("prompt.sign_in.name", bundle: .module)
            }
            .textFieldStyle(.roundedBorder)
            .autocorrectionDisabled()
            #if os(iOS)
                .textInputAutocapitalization(.never)
            #endif
                .focused($typing, equals: .name)
                .onSubmit { typing = .password }
                .accessibilityLabel(Text("prompt.sign_in.name", bundle: .module))
                .accessibilityIdentifier("prompt-name")
            // Drawn as dots, and a password to the system and to assistive technologies.
            SecureField(text: $password) {
                Text("prompt.sign_in.password", bundle: .module)
            }
            .textFieldStyle(.roundedBorder)
            .focused($typing, equals: .password)
            .onSubmit { answer(true, name, password) }
            .accessibilityLabel(Text("prompt.sign_in.password", bundle: .module))
            .accessibilityIdentifier("prompt-password")
            HStack(spacing: Tokens.Space.m) {
                Spacer(minLength: 0)
                button("prompt.cancel", identifier: "prompt-cancel") { answer(false, "", "") }
                button("prompt.sign_in.go", identifier: "prompt-sign-in") { answer(true, name, password) }
            }
        }
        .padding(.horizontal, Tokens.Space.m)
        .padding(.vertical, Tokens.Space.s)
        .background(Tokens.Palette.fieldFill)
        // Said to a screen reader when it appears: a load that waits says nothing by itself.
        .onAppear { AccessibilityNotification.Announcement(title).post() }
    }

    /// One of the bar's sentences, on as many lines as it needs.
    private func line(_ words: LocalizedStringKey, identifier: String) -> some View {
        Text(words, bundle: .module)
            .fixedSize(horizontal: false, vertical: true)
            .accessibilityIdentifier(identifier)
    }

    /// Neutral, as every button with words is (F14); its words on more lines where one is
    /// not enough, and never cut short.
    private func button(_ title: LocalizedStringKey, identifier: String, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            Text(title, bundle: .module)
                .fixedSize(horizontal: false, vertical: true)
                .frame(minHeight: BrowserWindowView.hitSize)
        }
        .buttonStyle(.bordered)
        .tint(.primary)
        .accessibilityIdentifier(identifier)
    }

    /// The bar's first line: the server that gets what is typed.
    static func title(_ site: String, locale: Locale = .current) -> String {
        String(format: String(localized: "prompt.sign_in", bundle: .module, locale: locale), site)
    }
}

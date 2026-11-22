// What a page says, or asks (016 §6): the bar a tab shows for `alert`, `confirm` and `prompt`.
// The site's name, in the core's words, on a line of its own; under it the page's words, which
// are the page's and are shown as such; for a prompt, a field; and the answers.
import SwiftUI
import WTContract
import WTDesign

/// `answer` is told what the person chose: OK or not, the text of a prompt's field with OK,
/// and whether the page's messages are to be stopped. Whether the answer came too soon to be
/// the person's is the core's to say (014 §3): the bar stays until the core takes it away.
///
/// No button is the default, and the bar takes the keyboard from nothing by appearing; in a
/// prompt's field, which the person has to go to themselves, Return is OK.
struct DialogBar: View {
    let dialog: WTDialogPrompt
    let answer: (Bool, String, Bool) -> Void
    /// What is in the field: the text the page offers, until the person changes it.
    @State private var text: String
    @Environment(\.locale) private var locale

    init(dialog: WTDialogPrompt, answer: @escaping (Bool, String, Bool) -> Void) {
        self.dialog = dialog
        self.answer = answer
        _text = State(initialValue: dialog.defaultText)
    }

    var body: some View {
        let says = Self.says(dialog.site, locale: locale)
        VStack(alignment: .leading, spacing: Tokens.Space.xs) {
            // Who says it, which only the core can write.
            Text(says)
                .font(.headline)
                .fixedSize(horizontal: false, vertical: true)
                .accessibilityIdentifier("prompt-site")
            // The page's words: as many lines as they need, up to a few; beyond that they
            // are moved up and down, and the answers stay where they are.
            ScrollView {
                Text(dialog.message)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .fixedSize(horizontal: false, vertical: true)
                    .accessibilityIdentifier("prompt-words")
            }
            .frame(maxHeight: Self.mostWords)
            .fixedSize(horizontal: false, vertical: true)
            if dialog.kind == .prompt {
                TextField(text: $text) {
                    Text(Words.promptAnswer)
                }
                .textFieldStyle(.roundedBorder)
                .onSubmit { answer(true, text, false) }
                .accessibilityLabel(Text(Words.promptAnswer))
                .accessibilityIdentifier("prompt-field")
            }
            // A page that said something before can be stopped: on a row of its own, apart
            // from the answers to what it says now.
            if dialog.maySilence {
                button(Words.promptSilence, identifier: "prompt-silence") { answer(false, "", true) }
            }
            HStack(spacing: Tokens.Space.m) {
                Spacer(minLength: 0)
                // Something said is only acknowledged: there is nothing to refuse.
                if dialog.kind != .alert {
                    button(Words.promptCancel, identifier: "prompt-cancel") { answer(false, "", false) }
                }
                button(Words.promptOk, identifier: "prompt-ok") { answer(true, dialog.kind == .prompt ? text : "", false) }
            }
        }
        .padding(.horizontal, Tokens.Space.m)
        .padding(.vertical, Tokens.Space.s)
        .background(Tokens.Palette.fieldFill)
        // Said to a screen reader when it appears: a page that waits says nothing by itself.
        .onAppear { AccessibilityNotification.Announcement("\(says) \(dialog.message)").post() }
    }

    /// Neutral, as every button with words is (F14); its words on more lines where one is
    /// not enough, and never cut short.
    private func button(_ title: Word, identifier: String, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            Text(title)
                .fixedSize(horizontal: false, vertical: true)
                .frame(minHeight: BrowserWindowView.hitSize)
        }
        .buttonStyle(.bordered)
        .tint(.primary)
        .accessibilityIdentifier(identifier)
    }

    /// How tall the page's words are let be before they scroll: about six lines.
    static let mostWords: CGFloat = 132

    /// The bar's first line: who says it.
    static func says(_ site: String, locale: Locale = .current) -> String {
        Words.promptSays.filled(site, locale: locale)
    }
}

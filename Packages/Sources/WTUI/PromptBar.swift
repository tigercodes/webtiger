public import WTContract

// What a tab asks the person (014 §7): a bar under the toolbar, in the tab that asks. A
// sentence that names who asks, in the core's words for the site, and what for; and the two
// answers. Nothing in it is the page's.
import SwiftUI
import WTDesign

public extension BrowserModel {
    /// What the tab in front asks: the core's. `nil` when it asks nothing; and for a prompt
    /// this build has no words for, which is not put before a person as two buttons and no
    /// question: its page waits.
    var prompt: WTPromptView? {
        guard tab.hasPrompt else { return nil }
        switch tab.prompt.kind {
        case .download: return tab.prompt
        case let .permission(permission): return PromptBar.wanted(permission).isEmpty ? nil : tab.prompt
        // What a page says or asks (016): a later contract's kind has no bar here.
        case let .dialog(dialog): return [.alert, .confirm, .prompt].contains(dialog.kind) ? tab.prompt : nil
        default: return nil
        }
    }

    /// The person's answer to the prompt the tab shows. What it lets the page do, what is
    /// remembered, and whether it came too soon after the prompt appeared to be the person's
    /// answer to it, are the core's: a refusal leaves the prompt where it is. For what a page
    /// says or asks (016), `text` is what a prompt's field held, and `silence` stops the
    /// page's messages.
    func answerPrompt(allow: Bool, text: String = "", silence: Bool = false) {
        guard let frontTabID, let prompt else { return }
        var answer = WTAnswerPrompt()
        answer.tabID = frontTabID
        answer.promptID = prompt.promptID
        answer.allow = allow
        answer.text = text
        answer.silence = silence
        send(.answerPrompt(answer))
    }
}

/// Neither answer is given by a key: Return and Escape are the page's and the field's, and a
/// press on one of the buttons is the only way.
struct PromptBar: View {
    let prompt: WTPromptView
    /// Allowed or not; and for what a page says or asks, a prompt's text and whether the
    /// page's messages are to be stopped.
    let answer: (Bool, String, Bool) -> Void
    @Environment(\.locale) private var locale

    var body: some View {
        // What a page says or asks has a bar of its own (016).
        if case let .dialog(said)? = prompt.kind {
            DialogBar(dialog: said, answer: answer)
        } else {
            asking
        }
    }

    @ViewBuilder private var asking: some View {
        let words = Self.words(prompt, locale: locale)
        Group {
            // The sentence on a line of its own where the window is a phone's; beside the
            // buttons on a Mac, whose window has room for both.
            if Self.isStacked {
                VStack(alignment: .leading, spacing: Tokens.Space.xs) {
                    sentence(words)
                    HStack(spacing: Tokens.Space.m) {
                        Spacer(minLength: 0)
                        buttons
                    }
                }
            } else {
                HStack(spacing: Tokens.Space.m) {
                    sentence(words)
                    Spacer(minLength: Tokens.Space.m)
                    buttons
                }
            }
        }
        .padding(.horizontal, Tokens.Space.m)
        .padding(.vertical, Tokens.Space.s)
        .frame(minHeight: Tokens.Size.toolbarHeight)
        .background(Tokens.Palette.fieldFill)
        // Said to a screen reader when it appears: a page that waits says nothing by itself.
        .onAppear { AccessibilityNotification.Announcement(words).post() }
    }

    private func sentence(_ words: String) -> some View {
        Text(words)
            .fixedSize(horizontal: false, vertical: true)
            .accessibilityIdentifier("prompt-words")
    }

    @ViewBuilder private var buttons: some View {
        // Neutral, as every button with words is (F14).
        Button {
            answer(false, "", false)
        } label: {
            Text("prompt.refuse", bundle: .module)
                // On more lines where one is not enough, and never cut short.
                .fixedSize(horizontal: false, vertical: true)
                .frame(minHeight: BrowserWindowView.hitSize)
        }
        .buttonStyle(.bordered)
        .tint(.primary)
        .accessibilityIdentifier("prompt-refuse")
        Button {
            answer(true, "", false)
        } label: {
            Text("prompt.allow", bundle: .module)
                .fixedSize(horizontal: false, vertical: true)
                .frame(minHeight: BrowserWindowView.hitSize)
        }
        .buttonStyle(.bordered)
        .tint(.primary)
        .accessibilityIdentifier("prompt-allow")
    }

    static var isStacked: Bool {
        #if os(iOS)
            true
        #else
            false
        #endif
    }

    /// What a page wants, each in its own words. A kind this build has no words for is left
    /// out.
    static func wanted(_ permission: WTPermissionPrompt) -> [String.LocalizationValue] {
        permission.kinds.compactMap { kind in
            switch kind {
            case .camera: "prompt.kind.camera"
            case .microphone: "prompt.kind.microphone"
            case .location: "prompt.kind.location"
            default: nil
            }
        }
    }

    /// A prompt's sentence: who asks, as the core names it, and what for, joined as the
    /// person's language joins a list.
    static func words(_ prompt: WTPromptView, locale: Locale = .current) -> String {
        // A download a site was not yet allowed (015): the site and the file, both in the
        // core's words. The name is in quotation marks, so that where it begins and ends
        // can be seen.
        if case let .download(download)? = prompt.kind {
            return String(format: String(localized: "prompt.download", bundle: .module, locale: locale), download.site, download.fileName)
        }
        guard case let .permission(permission)? = prompt.kind else { return "" }
        let names = wanted(permission).map { String(localized: $0, bundle: .module, locale: locale) }
        let list = ListFormatter()
        list.locale = locale
        let pattern = String(localized: "prompt.permission", bundle: .module, locale: locale)
        return String(format: pattern, permission.site, list.string(from: names) ?? names.joined(separator: ", "))
    }
}

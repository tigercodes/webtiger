// A page says something, or asks (016): `alert`, `confirm` and `prompt`, on WebKit. Left
// alone WebKit drops all three (an alert returns at once, a confirm is false, a prompt is
// nothing). The adapter reports what the page said and waits: the core says what the page is
// answered. It decides nothing, and shows nothing.
//
// "Leave site?" is not among what WebKit asks an app through its public delegate, so a page
// never holds a person here (016 §2).
import WebKit
import WTContract

/// The dialogs of one engine's pages that wait for the core's word.
@MainActor
final class Dialogs {
    private struct Waiting {
        let webViewID: UInt64
        let answer: (Bool, String) -> Void
    }

    private let report: @MainActor (UInt64, WTEngineEvent.OneOf_Kind) -> Void
    /// How many dialogs this engine has named. Each has the next number, and none is used
    /// twice: the ids are the shell's own (engine.proto, `DialogRequested`).
    private var named: UInt64 = 0
    private var waiting: [UInt64: Waiting] = [:]

    init(report: @escaping @MainActor (UInt64, WTEngineEvent.OneOf_Kind) -> Void) {
        self.report = report
    }

    /// A page of `webViewID` says `message`, or asks, as `origin`. It is reported, and the
    /// page's script waits until the core has said: whether it was accepted, and for a prompt
    /// the text.
    func ask(
        webViewID: UInt64,
        origin: String,
        kind: WTDialogKind,
        message: String,
        defaultText: String = ""
    ) async -> (accepted: Bool, text: String) {
        named += 1
        let id = named
        var asked = WTDialogRequested()
        asked.requestID = id
        asked.origin = origin
        asked.kind = kind
        asked.message = message
        asked.defaultText = defaultText
        return await withCheckedContinuation { continuation in
            waiting[id] = Waiting(webViewID: webViewID) { continuation.resume(returning: ($0, $1)) }
            report(webViewID, .dialogRequested(asked))
        }
    }

    /// The core's word on a dialog. One that no longer waits has no page to answer.
    func answer(_ id: UInt64, accepted: Bool, text: String) {
        waiting.removeValue(forKey: id)?.answer(accepted, text)
    }

    /// A web view is going: what its pages said is dismissed. WebKit must be given its
    /// answer: one it never gets is the app's fault, and it says so by stopping it.
    func drop(webViewID: UInt64) {
        for (id, dialog) in waiting where dialog.webViewID == webViewID {
            waiting[id] = nil
            dialog.answer(false, "")
        }
    }
}

/// What WebKit asks the page's delegate when a page says something or asks (the rest of the
/// delegate is beside the page, in WebKitEngine.swift).
extension Page {
    func webView(_: WKWebView, runJavaScriptAlertPanelWithMessage message: String, initiatedByFrame frame: WKFrameInfo) async {
        _ = await says(frame, .alert, message)
    }

    func webView(_: WKWebView, runJavaScriptConfirmPanelWithMessage message: String, initiatedByFrame frame: WKFrameInfo) async -> Bool {
        await says(frame, .confirm, message).accepted
    }

    func webView(
        _: WKWebView,
        runJavaScriptTextInputPanelWithPrompt prompt: String,
        defaultText: String?,
        initiatedByFrame frame: WKFrameInfo
    ) async -> String? {
        let answer = await says(frame, .prompt, prompt, offering: defaultText ?? "")
        return answer.accepted ? answer.text : nil
    }

    private func says(
        _ frame: WKFrameInfo,
        _ kind: WTDialogKind,
        _ message: String,
        offering: String = ""
    ) async -> (accepted: Bool, text: String) {
        guard let engine else { return (false, "") }
        let who = frame.securityOrigin
        let origin = Permissions.origin(scheme: who.protocol, host: who.host, port: who.port)
        return await engine.dialogs.ask(webViewID: id, origin: origin, kind: kind, message: message, defaultText: offering)
    }
}

// A page asks for a file (019): `<input type="file">`. WebKit on a Mac picks nothing by
// itself: it asks the page's delegate, and left alone the button does nothing. The adapter
// asks the engine's file picker and gives WebKit what was picked. On an iPhone WebKit shows a
// picker of its own, and the adapter has no part in it.
//
// The picker is a seam (019 §3): an engine picks nothing, and shows nothing of the system's,
// unless the app itself gave it the system's picker.
public import Foundation
import WebKit

/// What a page asks of a file picker.
public struct FilesAsked: Sendable, Equatable {
    /// Several files may be chosen, and not only one.
    public let several: Bool

    public init(several: Bool) {
        self.several = several
    }
}

/// How an engine picks files for a page: the files' addresses, or none when nothing is chosen.
public struct FilePicker: Sendable {
    let pick: @MainActor @Sendable (FilesAsked) async -> [URL]

    public init(pick: @escaping @MainActor @Sendable (FilesAsked) async -> [URL]) {
        self.pick = pick
    }

    /// Picks nothing, and shows nothing. What every engine has unless it is given another: so
    /// every engine a test makes, and fake mode's. The system's file picker is a dialog of the
    /// system's, and nothing but the app itself, started for real, may bring that up.
    public static let none = FilePicker { _ in [] }
}

/// The web views of one engine whose pages have a picker open.
@MainActor
final class Files {
    /// For each such web view, which opening of the picker it is.
    private var open: [UInt64: UInt64] = [:]
    private var opened: UInt64 = 0

    /// Picks files for a page of `webViewID` with `picker`. Nil when nothing is to be given:
    /// nothing was picked; a picker is open for that web view already; or the web view went
    /// while the picker was open, and what was then picked goes nowhere. A page that may have
    /// one file is given no more than the first.
    func choose(for webViewID: UInt64, asked: FilesAsked, with picker: FilePicker) async -> [URL]? {
        guard open[webViewID] == nil else { return nil }
        opened += 1
        let mine = opened
        open[webViewID] = mine
        let picked = await picker.pick(asked)
        guard open[webViewID] == mine else { return nil }
        open[webViewID] = nil
        let given = asked.several ? picked : Array(picked.prefix(1))
        return given.isEmpty ? nil : given
    }

    /// A web view is going: what is picked for it from now goes nowhere.
    func drop(webViewID: UInt64) {
        open[webViewID] = nil
    }
}

#if os(macOS)
    /// What WebKit asks the page's delegate when a page's file input is pressed (the rest of
    /// the delegate is beside the page, in WebKitEngine.swift). Folders are never offered:
    /// what a page may be given is files the person chose one by one.
    extension Page {
        func webView(_: WKWebView, runOpenPanelWith parameters: WKOpenPanelParameters, initiatedByFrame _: WKFrameInfo) async -> [URL]? {
            guard let engine else { return nil }
            let asked = FilesAsked(several: parameters.allowsMultipleSelection)
            return await engine.files.choose(for: id, asked: asked, with: engine.filePicker)
        }
    }
#endif

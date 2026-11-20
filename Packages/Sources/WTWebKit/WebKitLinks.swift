// Where a page's navigations are decided, and a link the person asks for in a new tab (018):
// by ⌘-click, which WebKit reports with the navigation; by the menu an iPhone shows for a long
// press on a link, which is the shell's; and by WebKit's own menu on a Mac, whose item for it
// is given the shell's words (WebKitMenus.swift). The adapter reports the wish; whether a tab
// opens is the core's.
public import Foundation
import WebKit
import WTContract

/// A link's menu, and a picture's: the shell's words, and where "Copy Link" copies to.
public struct LinkMenu {
    /// "Open in New Tab", as this system says it.
    public var openInNewTab: String
    /// "Open Image in New Tab": the Mac's item for a picture, which WebKit names for a window.
    public var openImageInNewTab: String
    /// "Download Linked File" (022), in a menu that is the shell's.
    public var downloadLinkedFile: String
    /// "Copy Link".
    public var copyLink: String
    /// Puts a link's address where it can be pasted from.
    public var copy: @MainActor (URL) -> Void

    public init(
        openInNewTab: String,
        openImageInNewTab: String,
        downloadLinkedFile: String,
        copyLink: String,
        copy: @escaping @MainActor (URL) -> Void
    ) {
        self.openInNewTab = openInNewTab
        self.openImageInNewTab = openImageInNewTab
        self.downloadLinkedFile = downloadLinkedFile
        self.copyLink = copyLink
        self.copy = copy
    }
}

public extension WebKitEngine {
    /// The person asked for a link of a web view's page in a new tab. The core is told:
    /// whether a tab opens is its to say.
    func openLink(webViewID: UInt64, url: URL, background: Bool) {
        var asked = WTLinkOpenRequested()
        asked.url = url.absoluteString
        asked.background = background
        asked.userGesture = true
        report(webViewID, .linkOpenRequested(asked))
    }
}

/// What WebKit asks the page's delegate about where the page goes (the rest of the delegate
/// is beside the page, in WebKitEngine.swift).
extension Page {
    func webView(_ webView: WKWebView, decidePolicyFor action: WKNavigationAction) async -> WKNavigationActionPolicy {
        // A navigation with no target frame opens a new window; it is decided as a top-level one.
        let mainFrame = action.targetFrame?.isMainFrame ?? true
        let userGesture = action.navigationType == .linkActivated || action.navigationType == .formSubmitted
        let verdict = engine?.verdict(for: action.request.url, webViewID: id, mainFrame: mainFrame, userGesture: userGesture)
        if verdict == .external {
            // An address for the system, not for a browser: refused here like any other, and
            // reported first, for a frame as for the page. Whether it is handed to the system
            // is the core's to say (007); the adapter opens nothing.
            var asked = WTOutsideLinkRequested()
            asked.url = action.request.url?.absoluteString ?? ""
            asked.userGesture = userGesture
            engine?.report(id, .outsideLinkRequested(asked))
        }
        switch verdict {
        case .allow:
            // A link of the page pressed with ⌘ held is the person asking for it in a new
            // tab (018): behind, or with ⇧ in front. The page stays where it is, and nothing
            // of its own was going anywhere, so nothing is said to have ended.
            if mainFrame, action.targetFrame != nil, action.navigationType == .linkActivated, action.modifierFlags.contains(.command),
               let link = action.request.url {
                engine?.openLink(webViewID: id, url: link, background: !action.modifierFlags.contains(.shift))
                return .cancel
            }
            // A link that says "download" is one (013), once the core has allowed its address.
            return action.shouldPerformDownload ? .download : .allow
        case .cancel, .external, nil:
            // A refused navigation with no target frame was for a new window: nothing of this
            // web view's own was going anywhere, and nothing is said about it.
            if mainFrame, action.targetFrame != nil {
                // WebKit reports nothing about a navigation refused here, so the adapter says
                // it ended: otherwise a tab the user sent somewhere would wait for ever.
                var failed = WTNavigationFailed()
                failed.code = "net.cancelled"
                failed.url = action.request.url?.absoluteString ?? ""
                // A refusal that leaves the web view loading nothing is said to be one: a
                // window a page opened then has nothing to show (006). With a load on its
                // way, it is that load's end that will say.
                refusedWhileProvisional = provisional
                failed.refused = !provisional && !webView.isLoading
                engine?.report(id, .navigationFailed(failed))
            }
            return .cancel
        }
    }
}

#if os(iOS)
    /// What a long press on a link shows on an iPhone or an iPad: the shell's menu, when it has
    /// one. No preview: a preview loads a page nobody chose to load. Anything that is not a
    /// link, and every link when the shell has no menu, is left to WebKit.
    extension Page {
        func webView(_: WKWebView, contextMenuConfigurationFor element: WKContextMenuElementInfo) async -> UIContextMenuConfiguration? {
            guard let engine, let menu = engine.linkMenu, let link = element.linkURL else { return nil }
            let webViewID = id
            return UIContextMenuConfiguration(identifier: nil, previewProvider: nil) { _ in
                UIMenu(children: [
                    UIAction(title: menu.openInNewTab, image: UIImage(systemName: "plus.square.on.square")) { _ in
                        engine.openLink(webViewID: webViewID, url: link, background: true)
                    },
                    // What the link leads to, saved (022): a download the person asked for.
                    UIAction(title: menu.downloadLinkedFile, image: UIImage(systemName: "arrow.down.circle")) { _ in
                        engine.saveFile(webViewID: webViewID, url: link)
                    },
                    UIAction(title: menu.copyLink, image: UIImage(systemName: "doc.on.doc")) { _ in
                        menu.copy(link)
                    },
                ])
            }
        }
    }

    /// The web view the adapter makes: WebKit's own.
    typealias MenuWebView = WKWebView
#endif

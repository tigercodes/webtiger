// Where "Copy Link" puts an address (018), and the words of a link's menu. The system's
// clipboard is the person's own: in fake mode, and in every session a test makes, a link is
// copied to a clipboard of the app's own instead, so that no test changes what somebody had
// copied. (An iPhone simulator shares its clipboard with the Mac it runs on.)
public import Foundation
#if os(macOS)
    import AppKit
#else
    import UIKit
#endif
import WTWebKit

public struct LinkClipboard: Sendable {
    let copy: @MainActor @Sendable (URL) -> Void

    public init(copy: @escaping @MainActor @Sendable (URL) -> Void) {
        self.copy = copy
    }

    /// The system's clipboard.
    public static let system = LinkClipboard { link in
        #if os(macOS)
            NSPasteboard.general.clearContents()
            NSPasteboard.general.setString(link.absoluteString, forType: .string)
        #else
            UIPasteboard.general.url = link
        #endif
    }

    /// A clipboard of the app's own, by a name nothing else pastes from.
    public static let ownOfTheApp = LinkClipboard { link in
        #if os(macOS)
            let own = NSPasteboard(name: NSPasteboard.Name(ownName))
            own.clearContents()
            own.setString(link.absoluteString, forType: .string)
        #else
            UIPasteboard(name: UIPasteboard.Name(ownName), create: true)?.url = link
        #endif
    }

    static let ownName = "com.tigercodes.webtiger.links"
}

extension LinkMenu {
    /// The menu a session gives its engine: the shell's words for this system, and where it
    /// copies to. On a Mac the words stand beside WebKit's own ("Copy Link"), which name the
    /// link each time; an iPhone's menu is all the shell's.
    @MainActor
    static func words(copyingTo clipboard: LinkClipboard) -> LinkMenu {
        #if os(macOS)
            let open = Words.linkOpenLinkNewTab.string()
        #else
            let open = Words.linkOpenNewTab.string()
        #endif
        return LinkMenu(
            openInNewTab: open,
            openImageInNewTab: Words.linkOpenImageNewTab.string(),
            downloadLinkedFile: Words.linkDownload.string(),
            copyLink: Words.linkCopy.string(),
            copy: clipboard.copy
        )
    }
}

// A link's menu (018), the shell's part on Apple's systems: the words a session gives its
// engine, and where "Copy Link" copies to.
#if os(macOS)
    import AppKit
#else
    import UIKit
#endif
import WTContract
import WTCoreClient
@testable import WTUI
import WTWebKit
import XCTest

@MainActor
final class LinkMenuTests: XCTestCase {
    /// What was copied, kept where a clipboard's closure can put it.
    private final class Copied: @unchecked Sendable {
        private let lock = NSLock()
        private var links: [URL] = []

        func add(_ link: URL) {
            lock.withLock { links.append(link) }
        }

        var all: [URL] {
            lock.withLock { links }
        }
    }

    func testASessionGivesItsEngineTheMenusWordsAndWhereToCopy() throws {
        let copied = Copied()
        let engine = WebKitEngine()
        XCTAssertNil(engine.linkMenu, "an engine by itself shows WebKit's own menu")

        let session = BrowserSession(core: ScriptedCore { _ in .ack(WTAck()) }, engine: engine, clipboard: LinkClipboard { copied.add($0) })

        let menu = try XCTUnwrap(session.engine.linkMenu)
        #if os(macOS)
            // Beside WebKit's own items, which name the link each time.
            XCTAssertEqual(menu.openInNewTab, "Open Link in New Tab")
        #else
            XCTAssertEqual(menu.openInNewTab, "Open in New Tab")
        #endif
        XCTAssertEqual(menu.copyLink, "Copy Link")
        // A picture's item on a Mac, and what a link's menu saves (022).
        XCTAssertEqual(menu.openImageInNewTab, "Open Image in New Tab")
        XCTAssertEqual(menu.downloadLinkedFile, "Download Linked File")
        let link = try XCTUnwrap(URL(string: "https://example.com/next?page=2"))
        menu.copy(link)
        XCTAssertEqual(copied.all, [link])
    }

    func testTheAppsOwnClipboardIsNotTheSystems() throws {
        let link = try XCTUnwrap(URL(string: "https://example.com/next?page=2"))
        #if os(macOS)
            // How often the system's clipboard has changed: looked at, never read.
            let before = NSPasteboard.general.changeCount

            LinkClipboard.ownOfTheApp.copy(link)

            XCTAssertEqual(NSPasteboard.general.changeCount, before, "what somebody had copied is still theirs")
            XCTAssertEqual(NSPasteboard(name: NSPasteboard.Name(LinkClipboard.ownName)).string(forType: .string), link.absoluteString)
        #else
            let before = UIPasteboard.general.changeCount

            LinkClipboard.ownOfTheApp.copy(link)

            XCTAssertEqual(UIPasteboard.general.changeCount, before, "what somebody had copied is still theirs")
            XCTAssertEqual(UIPasteboard(name: UIPasteboard.Name(LinkClipboard.ownName), create: false)?.url, link)
        #endif
    }
}

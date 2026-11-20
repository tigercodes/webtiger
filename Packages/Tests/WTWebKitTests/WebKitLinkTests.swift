// The adapter suite's WebKit run, for a link the person asks for in a new tab (018): a click
// with ⌘ held, against the test site's page of links in a real web view; what a plain click
// still does; what WebKit says of a click a page makes by itself; and the Mac's menu item.
import WebKit
import WTContract
import WTCoreClient
@testable import WTWebKit
import XCTest

@MainActor
final class WebKitLinkTests: AdapterTestCase {
    /// A web view on the page of links.
    private func page(_ id: UInt64 = 1) async {
        await create(id)
        await load(id, site.url("/links"))
        await wait(id, "the page to load") { self.finished($0) && self.hasTitle("Links")($0) }
    }

    /// The links a web view's page was reported as asked for in a new tab, in order.
    private func asked(_ id: UInt64 = 1) -> [WTLinkOpenRequested] {
        reports(id).compactMap {
            if case let .linkOpenRequested(asked) = $0 {
                asked
            } else {
                nil
            }
        }
    }

    /// The addresses a web view's pages have committed at, in order.
    private func committed(_ id: UInt64 = 1) -> [String] {
        reports(id).compactMap {
            if case let .navigationCommitted(page) = $0 {
                page.url
            } else {
                nil
            }
        }
    }

    func testAPlainClickOnALinkStillFollowsIt() async {
        await page()

        await run("document.getElementById('words').click()", in: 1)

        await wait(1, "the link's page") { self.hasTitle("Landed")($0) }
        XCTAssertEqual(asked().count, 0, "nobody asked for a new tab")
    }

    func testALinkAskedForInANewTabIsReportedAsThePersonsDoing() async throws {
        await create(1)
        let link = try XCTUnwrap(URL(string: "https://example.com/next?page=2"))

        engine.openLink(webViewID: 1, url: link, background: true)
        engine.openLink(webViewID: 1, url: link, background: false)

        XCTAssertEqual(asked().map(\.url), ["https://example.com/next?page=2", "https://example.com/next?page=2"])
        XCTAssertEqual(asked().map(\.background), [true, false], "behind, or in front, as was asked")
        XCTAssertEqual(asked().map(\.userGesture), [true, true])
    }

    #if os(macOS)
        /// Puts a web view in a window that is not on the screen, so that it can be clicked.
        private func window(for id: UInt64) throws -> NSWindow {
            let webView = try XCTUnwrap(engine.webView(for: id))
            let window = NSWindow(
                contentRect: NSRect(x: 0, y: 0, width: 480, height: 360),
                styleMask: [.borderless],
                backing: .buffered,
                defer: false
            )
            window.isReleasedWhenClosed = false
            window.contentView = webView
            return window
        }

        /// Clicks the middle of an element of the page with the mouse, as the system gives a
        /// click to a window: with the keys that are held.
        private func click(_ element: String, in id: UInt64, of window: NSWindow, holding keys: NSEvent.ModifierFlags) async throws {
            let webView = try XCTUnwrap(engine.webView(for: id))
            let middle = await run(
                "(() => { const r = document.getElementById('\(element)').getBoundingClientRect(); "
                    + "return [r.x + r.width / 2, r.y + r.height / 2]; })()",
                in: id
            )
            let place = try XCTUnwrap(middle as? [Double])
            // The page counts down from its top; a window counts up from its bottom.
            let inWebView = NSPoint(x: place[0], y: webView.isFlipped ? place[1] : Double(webView.bounds.height) - place[1])
            let point = webView.convert(inWebView, to: nil)
            for (kind, number) in [(NSEvent.EventType.leftMouseDown, 1), (.leftMouseUp, 2)] {
                let event = try XCTUnwrap(NSEvent.mouseEvent(
                    with: kind, location: point, modifierFlags: keys, timestamp: ProcessInfo.processInfo.systemUptime,
                    windowNumber: window.windowNumber, context: nil, eventNumber: number, clickCount: 1,
                    pressure: kind == .leftMouseDown ? 1 : 0
                ))
                if kind == .leftMouseDown {
                    webView.mouseDown(with: event)
                } else {
                    webView.mouseUp(with: event)
                }
            }
        }

        func testAClickWithCommandHeldIsReportedAndThePageStays() async throws {
            await page()
            let window = try window(for: 1)

            try await click("words", in: 1, of: window, holding: .command)

            await wait(1, "the click to be reported") { _ in !self.asked().isEmpty }
            XCTAssertEqual(asked().map(\.url), [site.url("/landed")])
            XCTAssertEqual(asked().map(\.background), [true], "behind the page being read")
            XCTAssertEqual(asked().map(\.userGesture), [true])
            // The page is where it was, and nothing is said to have ended there.
            try? await Task.sleep(for: .milliseconds(500))
            XCTAssertEqual(committed(), [site.url("/links")])
            XCTAssertFalse(names(reports(1)).contains { $0.hasPrefix("failed") }, "\(names(reports(1)))")

            // With ⇧ as well: in front.
            try await click("words", in: 1, of: window, holding: [.command, .shift])
            await wait(1, "the second click to be reported") { _ in self.asked().count == 2 }
            XCTAssertEqual(asked().map(\.background), [true, false])
            window.close()
        }

        func testAClickWithNoKeyHeldFollowsTheLink() async throws {
            await page()
            let window = try window(for: 1)

            try await click("words", in: 1, of: window, holding: [])

            await wait(1, "the link's page") { self.hasTitle("Landed")($0) }
            XCTAssertEqual(asked().count, 0)
            window.close()
        }

        func testTheMacsOwnItemForALinkInANewWindowIsGivenTheShellsWords() async throws {
            await create(1)
            let webView = try XCTUnwrap(engine.webView(for: 1) as? MenuWebView)
            func menu() -> NSMenu {
                let menu = NSMenu()
                for (title, identifier) in [
                    ("Open Link", "WKMenuItemIdentifierOpenLink"),
                    ("Open Link in New Window", MenuWebView.opensLinkInNewWindow),
                    ("Copy Link", "WKMenuItemIdentifierCopyLink"),
                ] {
                    let item = NSMenuItem(title: title, action: nil, keyEquivalent: "")
                    item.identifier = NSUserInterfaceItemIdentifier(identifier)
                    menu.addItem(item)
                }
                return menu
            }
            let event = try XCTUnwrap(NSEvent.mouseEvent(
                with: .rightMouseDown, location: .zero, modifierFlags: [], timestamp: 0, windowNumber: 0, context: nil, eventNumber: 0,
                clickCount: 1, pressure: 1
            ))

            // With no words of the shell's, the menu is WebKit's as it is.
            let untouched = menu()
            webView.willOpenMenu(untouched, with: event)
            XCTAssertEqual(untouched.items.map(\.title), ["Open Link", "Open Link in New Window", "Copy Link"])

            engine.linkMenu = LinkMenu(openInNewTab: "Open Link in New Tab", copyLink: "Copy Link") { _ in }
            let named = menu()
            webView.willOpenMenu(named, with: event)
            XCTAssertEqual(named.items.map(\.title), ["Open Link", "Open Link in New Tab", "Copy Link"], "that item, and no other")
        }
    #endif

    func testAClickAPageMakesItselfWithTheCommandKeySaidToBeHeldIsOnlyAClick() async {
        await page()

        // No finger and no mouse: the page's own script, which says ⌘ was held.
        await run(
            "document.getElementById('words').dispatchEvent("
                + "new MouseEvent('click', { metaKey: true, bubbles: true, cancelable: true, view: window }))",
            in: 1
        )

        // Seen here: WebKit follows the link in its own tab, and says nothing of a key. A page
        // cannot ask for a tab this way.
        await wait(1, "the link's page") { self.hasTitle("Landed")($0) }
        XCTAssertEqual(asked().count, 0)
    }
}

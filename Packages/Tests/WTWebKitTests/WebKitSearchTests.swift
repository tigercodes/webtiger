// The adapter suite's WebKit run, for a search for words that are selected (023): a search is
// reported for the core to make. And on a Mac, where the menu is WebKit's: what the adapter's
// script says was selected at a right-click, against a real web view with the mouse's own
// events, and never what is typed in a password's field; and what the adapter makes of
// WebKit's item that searches. No menu is opened here: the page of these tests keeps WebKit's
// from opening, and the menu the adapter is handed is one the test made. No search is made:
// the adapter only reports.
import WebKit
import WTContract
import WTCoreClient
@testable import WTWebKit
import XCTest

@MainActor
final class WebKitSearchTests: AdapterTestCase {
    /// The searches the adapter has reported for a web view, in order.
    private func searches(_ id: UInt64 = 1) -> [WTSearchRequested] {
        reports(id).compactMap {
            if case let .searchRequested(asked) = $0 {
                asked
            } else {
                nil
            }
        }
    }

    func testASearchForSelectedWordsIsReportedAsThePersonsDoing() async {
        await create(1)
        await load(1, site.url("/words"))
        await wait(1, "the page to load") { self.finished($0) && self.hasTitle("Words")($0) }

        engine.search(webViewID: 1, words: "  tigers in\nthe wild ")

        // The words as they were selected: the core tidies them. And nothing else comes of it
        // here: the page is where it was.
        XCTAssertEqual(searches().map(\.text), ["  tigers in\nthe wild "])
        XCTAssertEqual(searches().map(\.userGesture), [true])
        XCTAssertEqual(engine.webView(for: 1)?.url?.absoluteString, site.url("/words"))
    }

    #if os(macOS)
        /// What the adapter's script said the last right-click in a web view was on, once it
        /// has said.
        private func pressed(in id: UInt64 = 1) async throws -> Pressed? {
            let webView = try XCTUnwrap(engine.webView(for: id))
            for _ in 0 ..< 500 {
                if let pressed = engine.pressed.take(for: webView) {
                    return pressed
                }
                try? await Task.sleep(for: .milliseconds(10))
            }
            return nil
        }

        /// A menu as a test makes it: its items by the names WebKit gives them, with the
        /// lines between.
        private func menu(_ items: [(String, String)?]) -> NSMenu {
            let menu = NSMenu()
            for named in items {
                guard let (title, identifier) = named else {
                    menu.addItem(.separator())
                    continue
                }
                let item = NSMenuItem(title: title, action: nil, keyEquivalent: "")
                item.identifier = NSUserInterfaceItemIdentifier(identifier)
                menu.addItem(item)
            }
            return menu
        }

        /// The shell's words before it knows where searches go.
        private var words: LinkMenu {
            LinkMenu(
                openInNewTab: "Open Link in New Tab", openImageInNewTab: "Open Image in New Tab",
                downloadLinkedFile: "Download Linked File", copyLink: "Copy Link"
            ) { _ in }
        }

        /// The shell's words once it knows where searches go.
        private var wordsWithASearch: LinkMenu {
            var words = words
            words.searchWords = "Search with DuckDuckGo"
            return words
        }

        /// WebKit's menu for words that are selected, as a test makes it.
        private func webKitsMenuOfWords() -> NSMenu {
            menu([
                ("Look Up “tigers”", "WKMenuItemIdentifierLookUp"),
                nil,
                ("Search with Google", MenuWebView.searchesWeb),
                nil,
                ("Copy", "WKMenuItemIdentifierCopy"),
            ])
        }

        /// Selects what is in an element of the page of words, and right-clicks it. What the
        /// adapter's script then said.
        private func rightClickSelected(_ element: String, of window: NSWindow) async throws -> Pressed? {
            await run("choose('\(element)')", in: 1)
            try await rightClick(element, in: 1, of: window)
            return try await pressed()
        }

        func testARightClickOnSelectedWordsTellsTheAdapterTheWords() async throws {
            await create(1)
            await load(1, site.url("/words?menu=own"))
            await wait(1, "the page to load") { self.finished($0) && self.hasTitle("Words")($0) }
            let window = try window(for: 1)

            // A word of the page, selected: the word.
            let onWord = try await rightClickSelected("word", of: window)
            XCTAssertEqual(onWord, Pressed(picture: nil, link: nil, words: "tigers"))

            // What is typed in a password's field is nobody's to read: selected, and
            // right-clicked, it is told as nothing.
            let onSecret = try await rightClickSelected("secret", of: window)
            XCTAssertEqual(onSecret, Pressed(picture: nil, link: nil, words: ""))
            let typed = await run("document.getElementById('secret').value", in: 1)
            XCTAssertEqual(typed as? String, "hunter2", "there was something there to read")

            // The same for a password's field the script cannot see is one: it is inside a
            // part of the page that is closed to scripts.
            let onClosed = try await rightClickSelected("closed", of: window)
            XCTAssertEqual(onClosed, Pressed(picture: nil, link: nil, words: ""))

            // Words typed in an ordinary field are words like any other.
            let onField = try await rightClickSelected("field", of: window)
            XCTAssertEqual(onField, Pressed(picture: nil, link: nil, words: "typed words"))
            window.close()
        }

        func testTheMacsSearchForSelectedWordsIsTheShellsAndTellsTheAdapterTheWords() {
            var searched: [String] = []
            let saves: @MainActor (URL) -> Void = { _ in XCTFail("nothing to save") }

            // Words are selected, and the shell knows where searches go: in place of WebKit's
            // item, which names the system's engine, stands the shell's, under WebKit's name.
            let menu = webKitsMenuOfWords()
            MenuWebView.arrange(
                menu, pressed: Pressed(picture: nil, link: nil, words: "tigers in the wild"), words: wordsWithASearch, save: saves
            ) { searched.append($0) }
            XCTAssertEqual(menu.items.map(\.title), ["Look Up “tigers”", "", "Search with DuckDuckGo", "", "Copy"])
            XCTAssertEqual(menu.items[2].identifier?.rawValue, MenuWebView.searchesWeb)
            XCTAssertEqual(searched, [], "nothing is searched for until the item is chosen")
            menu.performActionForItem(at: 2)
            XCTAssertEqual(searched, ["tigers in the wild"])
            // No other item is the adapter's.
            XCTAssertNil(menu.items[0].target)
            XCTAssertNil(menu.items[4].target)
        }

        func testASearchThatWouldGoElsewhereOrForNothingIsTakenOut() {
            let saves: @MainActor (URL) -> Void = { _ in XCTFail("nothing to save") }
            let searches: @MainActor (String) -> Void = { _ in XCTFail("nothing to search for") }
            let expected = ["Look Up “tigers”", "", "Copy"]

            // The shell does not know where searches go yet: WebKit's item, which goes to the
            // system's engine in another app, is not left in its place.
            let unnamed = webKitsMenuOfWords()
            MenuWebView.arrange(
                unnamed,
                pressed: Pressed(picture: nil, link: nil, words: "tigers"),
                words: words,
                save: saves,
                search: searches
            )
            XCTAssertEqual(unnamed.items.map(\.title), expected)
            // With no words of the shell's at all.
            let wordless = webKitsMenuOfWords()
            MenuWebView.arrange(
                wordless,
                pressed: Pressed(picture: nil, link: nil, words: "tigers"),
                words: nil,
                save: saves,
                search: searches
            )
            XCTAssertEqual(wordless.items.map(\.title), expected)

            // No words were said to be selected, or nothing but white space; or nothing was
            // said of the click at all.
            for pressed in [Pressed(picture: nil, link: nil, words: ""), Pressed(picture: nil, link: nil, words: " \n\t "), nil] {
                let menu = webKitsMenuOfWords()
                MenuWebView.arrange(menu, pressed: pressed, words: wordsWithASearch, save: saves, search: searches)
                XCTAssertEqual(menu.items.map(\.title), expected, "\(String(describing: pressed))")
            }
        }

        func testWhatAMenuSearchesForIsWhatWasSelectedAtTheClickThatOpenedIt() async throws {
            await create(1)
            await load(1, site.url("/words?menu=own"))
            await wait(1, "the page to load") { self.finished($0) && self.hasTitle("Words")($0) }
            let window = try window(for: 1)
            let webView = try XCTUnwrap(engine.webView(for: 1) as? MenuWebView)
            engine.linkMenu = wordsWithASearch
            let event = try XCTUnwrap(NSEvent.mouseEvent(
                with: .rightMouseDown, location: .zero, modifierFlags: [], timestamp: 0, windowNumber: 0, context: nil, eventNumber: 0,
                clickCount: 1, pressure: 1
            ))

            // A right-click on the selected word, with the mouse; then the menu WebKit would
            // open for it, handed to the web view as WebKit hands its own.
            await run("choose('word')", in: 1)
            try await rightClick("word", in: 1, of: window)
            await wait(1, "the page to have had the click") { self.hasTitle("own menu")($0) }
            try? await Task.sleep(for: .milliseconds(200))
            let opened = webKitsMenuOfWords()
            webView.willOpenMenu(opened, with: event)

            XCTAssertEqual(opened.items.map(\.title), ["Look Up “tigers”", "", "Search with DuckDuckGo", "", "Copy"])
            opened.performActionForItem(at: 2)
            XCTAssertEqual(searches().map(\.text), ["tigers"])
            XCTAssertEqual(searches().map(\.userGesture), [true])
            window.close()
        }
    #endif
}

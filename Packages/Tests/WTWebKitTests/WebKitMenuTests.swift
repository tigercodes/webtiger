// The adapter suite's WebKit run, for what a menu saves (022): a file the person chose is a
// download of their own, fetched in the page's session. And on a Mac, where the menu is
// WebKit's: what the adapter's script says a right-click was on, against a real web view with
// the mouse's own events; that a page cannot say so itself; and what the adapter makes of
// WebKit's items. No menu is opened here: the page of these tests keeps WebKit's from
// opening, and the menu the adapter is handed is one the test made. (A search for words that
// are selected is in WebKitSearchTests.swift.)
import WebKit
import WTContract
import WTCoreClient
@testable import WTWebKit
import XCTest

@MainActor
final class WebKitMenuTests: AdapterTestCase {
    /// A web view on the page of pictures; `own` for the one that keeps the engine's menu shut.
    private func page(_ id: UInt64 = 1, ownMenu: Bool = false) async {
        await create(id)
        await load(id, site.url(ownMenu ? "/pictures?menu=own" : "/pictures"))
        await wait(id, "the page to load") { self.finished($0) && self.hasTitle("Pictures")($0) }
    }

    private var picture: URL {
        get throws { try XCTUnwrap(URL(string: site.url("/tiger.svg"))) }
    }

    /// The downloads the adapter has reported for a web view, in order.
    private func asked(_ id: UInt64 = 1) -> [WTDownloadRequested] {
        reports(id).compactMap {
            if case let .downloadRequested(asked) = $0 {
                asked
            } else {
                nil
            }
        }
    }

    /// Waits for one more download to be reported than `before`. (The first download of a run
    /// takes WebKit on an iPhone simulator a while to get going: five minutes of patience.)
    private func nextAsked(after before: Int, in id: UInt64 = 1) async -> WTDownloadRequested {
        for _ in 0 ..< Patience.steps * 5 where asked(id).count == before {
            try? await Task.sleep(for: .milliseconds(10))
        }
        XCTAssertGreaterThan(asked(id).count, before, "a download was reported")
        return asked(id).last ?? WTDownloadRequested()
    }

    /// The core's word that a download starts, under `name`; waits for its file to be whole.
    private func save(_ download: WTDownloadRequested, as name: String, in id: UInt64 = 1) async -> String? {
        var start = WTDownloadStart()
        start.downloadID = download.downloadID
        start.fileName = name
        _ = await engine.answer(.downloadStart(start))
        await wait(id, "the download to finish") { reports in
            reports.contains {
                if case .downloadFinished = $0 {
                    true
                } else {
                    false
                }
            }
        }
        return try? String(contentsOf: engine.downloadsFolder.appendingPathComponent(name), encoding: .utf8)
    }

    func testAFileChosenInAMenuIsADownloadOfThePersonsOwn() async throws {
        await page()

        try engine.saveFile(webViewID: 1, url: picture)
        let download = await nextAsked(after: 0)

        XCTAssertTrue(download.userAsked, "the person chose it: no page asked")
        XCTAssertEqual(download.url, site.url("/tiger.svg"))
        XCTAssertEqual(download.suggestedFilename, "tiger.svg")
        XCTAssertEqual(download.mimeType, "image/svg+xml")
        XCTAssertNotEqual(download.downloadID, 0, "named by the shell")
        // Saved like any other, once the core has said under what name.
        let saved = await save(download, as: "A tiger.svg")
        XCTAssertEqual(saved?.hasPrefix("<svg"), true, "the picture itself: \(saved ?? "nothing")")
        // The page is where it was, and nothing of its own ended.
        XCTAssertEqual(engine.webView(for: 1)?.url?.absoluteString, site.url("/pictures"))
        XCTAssertFalse(names(reports(1)).contains { $0.hasPrefix("failed(") }, "\(names(reports(1)))")
    }

    func testADownloadAPageStartsIsNotSaidToBeThePersons() async {
        await create(1)
        await load(1, site.url("/downloads"))
        await wait(1, "the page of links") { self.finished($0) && self.hasTitle("Downloads")($0) }

        await run("document.getElementById('attachment').click()", in: 1)
        let download = await nextAsked(after: 0)

        XCTAssertEqual(download.suggestedFilename, "notes.txt")
        XCTAssertFalse(download.userAsked)
    }

    func testAFileChosenIsFetchedAsItsLoginSet() async throws {
        // Signed in, in this web view's login set; then a file that says who asked for it.
        await create(1)
        await load(1, site.url("/set?v=ada"))
        await wait(1, "the cookie to be set") { self.finished($0) && self.hasTitle("Set")($0) }
        await load(1, site.url("/pictures"))
        await wait(1, "the page of pictures") { self.hasTitle("Pictures")($0) }

        try engine.saveFile(webViewID: 1, url: XCTUnwrap(URL(string: site.url("/whose-file"))))
        let mine = await save(nextAsked(after: 0), as: "who.txt")
        XCTAssertEqual(mine, "who=ada", "WebKit fetches in the page's own session")

        // Another login set's web view is nobody there.
        await page(2)
        try engine.saveFile(webViewID: 2, url: XCTUnwrap(URL(string: site.url("/whose-file"))))
        let theirs = await save(nextAsked(after: 0, in: 2), as: "who else.txt", in: 2)
        XCTAssertEqual(theirs, "who=none")
    }

    func testAFileChosenThatIsSomewhereElseIsNotFetchedThereAsThisSite() async throws {
        // Signed in at this site, in this web view's login set.
        await create(1)
        await load(1, site.url("/set?v=ada"))
        await wait(1, "the cookie to be set") { self.finished($0) && self.hasTitle("Set")($0) }

        // A file that says who asked for it, at an address of this site that sends whoever
        // fetches it on to another site (the server by its other name): what this site knows
        // the person by is not sent there.
        try engine.saveFile(webViewID: 1, url: XCTUnwrap(URL(string: site.url("/to-whose-file?port=\(site.port)"))))
        let download = await nextAsked(after: 0)
        XCTAssertEqual(download.url, "http://localhost:\(site.port)/whose-file", "reported as where the file is")
        XCTAssertTrue(download.userAsked)
        let elsewhere = await save(download, as: "elsewhere.txt")
        XCTAssertEqual(elsewhere, "who=none")
    }

    func testOnlyAnAddressOfTheWebIsFetchedAndOnlyForAWebViewThereIs() async throws {
        await page()

        for address in ["data:text/plain,made%20by%20the%20page", "file:///etc/hosts", "javascript:void(0)", "blob:\(site.url("/1"))"] {
            try engine.saveFile(webViewID: 1, url: XCTUnwrap(URL(string: address)))
        }
        try engine.saveFile(webViewID: 99, url: picture)

        // A real one after them, to have waited long enough: it is the only one reported.
        try engine.saveFile(webViewID: 1, url: picture)
        let download = await nextAsked(after: 0)
        try? await Task.sleep(for: .milliseconds(300))
        XCTAssertEqual(asked().map(\.url), [site.url("/tiger.svg")])
        XCTAssertEqual(download.url, site.url("/tiger.svg"))
        XCTAssertEqual(asked(99).count, 0)
    }

    func testAFileChosenThatCannotBeFetchedIsListedAndSaidToHaveFailed() async throws {
        await page()

        // A server that is not there: WebKit never gets as far as asking where the file goes.
        let nowhere = "http://127.0.0.1:\(WebKitAdapterTests.unusedPort())/gone.png"
        try engine.saveFile(webViewID: 1, url: XCTUnwrap(URL(string: nowhere)))
        let download = await nextAsked(after: 0)

        // The person asked for it: it is reported all the same, for the core to list.
        XCTAssertTrue(download.userAsked)
        XCTAssertEqual(download.url, nowhere)
        XCTAssertEqual(download.suggestedFilename, "", "no site suggested a name")
        XCTAssertFalse(names(reports(1)).contains { $0.hasPrefix("download failed") }, "not until the core has listed it")

        var start = WTDownloadStart()
        start.downloadID = download.downloadID
        start.fileName = "gone.png"
        _ = await engine.answer(.downloadStart(start))
        await wait(1, "the download to be said to have failed") { reports in
            reports.contains {
                if case .downloadFailed = $0 {
                    true
                } else {
                    false
                }
            }
        }
        let failed = reports(1).compactMap { report -> WTDownloadFailed? in
            if case let .downloadFailed(failed) = report {
                failed
            } else {
                nil
            }
        }
        XCTAssertEqual(failed.map(\.downloadID), [download.downloadID])
        XCTAssertEqual(failed.map(\.code), ["download.network"])
        let kept = (try? FileManager.default.contentsOfDirectory(atPath: engine.downloadsFolder.path)) ?? []
        XCTAssertEqual(kept, [], "nothing was saved")
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

        /// WebKit's menu for a picture that is a link, as a test makes it: its items by the
        /// names WebKit gives them, with the lines between.
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

        private func webKitsMenu() -> NSMenu {
            menu([
                ("Open Link in New Window", MenuWebView.opensLinkInNewWindow),
                ("Download Linked File", MenuWebView.downloadsLinkedFile),
                ("Copy Link", "WKMenuItemIdentifierCopyLink"),
                nil,
                ("Open Image in New Window", MenuWebView.opensImageInNewWindow),
                ("Download Image", MenuWebView.downloadsImage),
                nil,
                ("Copy Image", "WKMenuItemIdentifierCopyImage"),
            ])
        }

        private var words: LinkMenu {
            LinkMenu(
                openInNewTab: "Open Link in New Tab", openImageInNewTab: "Open Image in New Tab",
                downloadLinkedFile: "Download Linked File", copyLink: "Copy Link"
            ) { _ in }
        }

        func testARightClickTellsTheAdapterWhatItWasOn() async throws {
            await page(ownMenu: true)
            let window = try window(for: 1)
            let landed = try XCTUnwrap(URL(string: site.url("/landed?from=picture")))

            // A picture: its address, and no link.
            try await rightClick("plain", in: 1, of: window)
            let onPicture = try await pressed()
            XCTAssertEqual(onPicture, try Pressed(picture: picture, link: nil))
            // The page had the click too, and kept the menu shut: nothing waits for a choice.
            await wait(1, "the page to have had the click") { self.hasTitle("own menu")($0) }

            // A picture that is a link: both.
            try await rightClick("linked", in: 1, of: window)
            let onLinkedPicture = try await pressed()
            XCTAssertEqual(onLinkedPicture, try Pressed(picture: picture, link: landed))

            // Words: said, with neither.
            try await rightClick("words", in: 1, of: window)
            let onWords = try await pressed()
            XCTAssertEqual(onWords, Pressed(picture: nil, link: nil))

            // What was said is said once: the next menu is not given an older click's picture.
            let webView = try XCTUnwrap(engine.webView(for: 1))
            XCTAssertNil(engine.pressed.take(for: webView))

            // And it holds for a moment only: a menu that opens later was not that click's.
            try await rightClick("plain", in: 1, of: window)
            try? await Task.sleep(for: .milliseconds(300))
            XCTAssertNil(engine.pressed.take(for: webView, now: Date().addingTimeInterval(PressedReports.fresh + 1)))
            XCTAssertNil(engine.pressed.take(for: webView), "and it is not kept for another")
            // (That the click was heard at all: the next one is.)
            try await rightClick("plain", in: 1, of: window)
            let heard = try await pressed()
            XCTAssertEqual(heard, try Pressed(picture: picture, link: nil))
            window.close()
        }

        func testAClickAPageMakesItselfTellsTheAdapterNothing() async throws {
            await page(ownMenu: true)
            let webView = try XCTUnwrap(engine.webView(for: 1))

            // No mouse: the page's own script, which says the picture was right-clicked.
            await run(
                "document.getElementById('plain').dispatchEvent("
                    + "new MouseEvent('contextmenu', { bubbles: true, cancelable: true, view: window, button: 2 }))",
                in: 1
            )
            // The page's own listener had it: the event went all the way.
            await wait(1, "the page's listener to have had it") { self.hasTitle("own menu")($0) }
            try? await Task.sleep(for: .milliseconds(200))

            XCTAssertNil(engine.pressed.take(for: webView), "a page cannot say what was pressed")
            // Nor can it call what the adapter's script calls: that is in a world of its own.
            let reachable = await run(
                "typeof (window.webkit && window.webkit.messageHandlers && window.webkit.messageHandlers.pressed)",
                in: 1
            )
            XCTAssertEqual(reachable as? String, "undefined")
        }

        func testWhatAMenuSavesIsTheClickThatOpenedIt() async throws {
            await page(ownMenu: true)
            let window = try window(for: 1)
            let webView = try XCTUnwrap(engine.webView(for: 1) as? MenuWebView)
            let event = try XCTUnwrap(NSEvent.mouseEvent(
                with: .rightMouseDown, location: .zero, modifierFlags: [], timestamp: 0, windowNumber: 0, context: nil, eventNumber: 0,
                clickCount: 1, pressure: 1
            ))

            // A right-click on the picture, with the mouse; then the menu WebKit would open
            // for it, handed to the web view as WebKit hands its own.
            try await rightClick("plain", in: 1, of: window)
            await wait(1, "the page to have had the click") { self.hasTitle("own menu")($0) }
            try? await Task.sleep(for: .milliseconds(200))
            let opened = webKitsMenu()
            webView.willOpenMenu(opened, with: event)

            // The picture can be saved; there was no link, so nothing offers to save one.
            XCTAssertEqual(
                opened.items.map(\.title),
                ["Open Link in New Window", "Copy Link", "", "Open Image in New Window", "Download Image", "", "Copy Image"]
            )
            let item = try XCTUnwrap(opened.items.first { $0.identifier?.rawValue == MenuWebView.downloadsImage })
            opened.performActionForItem(at: opened.index(of: item))
            let download = await nextAsked(after: 0)
            XCTAssertEqual(download.url, site.url("/tiger.svg"))
            XCTAssertTrue(download.userAsked)

            // A menu that opens with nothing said of its click offers to save nothing.
            let unsaid = webKitsMenu()
            webView.willOpenMenu(unsaid, with: event)
            XCTAssertEqual(
                unsaid.items.map(\.title),
                ["Open Link in New Window", "Copy Link", "", "Open Image in New Window", "", "Copy Image"]
            )
            window.close()
        }

        func testTheMacsItemsAreGivenTheShellsWordsAndTheirWork() throws {
            let landed = try XCTUnwrap(URL(string: site.url("/landed")))
            var saved: [URL] = []

            // Everything known: the shell's words on the two that open a tab, and the two that
            // save are the adapter's own, under WebKit's words and names.
            let whole = webKitsMenu()
            try MenuWebView.arrange(whole, pressed: Pressed(picture: picture, link: landed), words: words) { saved.append($0) }
            XCTAssertEqual(
                whole.items.map(\.title),
                [
                    "Open Link in New Tab", "Download Linked File", "Copy Link", "",
                    "Open Image in New Tab", "Download Image", "", "Copy Image",
                ]
            )
            XCTAssertEqual(whole.items[1].identifier?.rawValue, MenuWebView.downloadsLinkedFile)
            XCTAssertEqual(whole.items[5].identifier?.rawValue, MenuWebView.downloadsImage)
            XCTAssertEqual(saved, [], "nothing is saved until an item is chosen")
            whole.performActionForItem(at: 5)
            XCTAssertEqual(saved, try [picture])
            whole.performActionForItem(at: 1)
            XCTAssertEqual(saved, try [picture, landed])
            // No other item is the adapter's.
            for index in [0, 2, 4, 7] {
                XCTAssertNil(whole.items[index].target, "\(whole.items[index].title)")
            }

            // With no words of the shell's, WebKit's stay; the items that save still do.
            saved = []
            let unworded = webKitsMenu()
            try MenuWebView.arrange(unworded, pressed: Pressed(picture: picture, link: nil), words: nil) { saved.append($0) }
            XCTAssertEqual(
                unworded.items.map(\.title),
                ["Open Link in New Window", "Copy Link", "", "Open Image in New Window", "Download Image", "", "Copy Image"]
            )
            unworded.performActionForItem(at: 4)
            XCTAssertEqual(saved, try [picture])
        }

        func testAnItemThatSavesWithNothingToSaveIsTakenOutAndLeavesNoLineBehind() {
            // Nothing was said of the click: neither item is shown, and no line stands at an
            // end of the menu or beside another.
            let menu = menu([
                ("Download Image", MenuWebView.downloadsImage),
                nil,
                ("Download Linked File", MenuWebView.downloadsLinkedFile),
                nil,
                ("Copy Image", "WKMenuItemIdentifierCopyImage"),
                nil,
                ("Download Image", MenuWebView.downloadsImage),
            ])
            MenuWebView.arrange(menu, pressed: nil, words: words) { _ in XCTFail("nothing to save") }
            XCTAssertEqual(menu.items.map(\.title), ["Copy Image"])
        }
    #endif
}

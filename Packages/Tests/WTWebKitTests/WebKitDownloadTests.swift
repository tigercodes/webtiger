// The adapter suite's WebKit run, for downloads (013): what the adapter reports when a page
// asks for one, and what it saves, where and under what name, once the core has said. The
// files go to a folder of the test's own engine, never to anybody's Downloads.
import WebKit
import WTContract
import WTCoreClient
import WTWebKit
import XCTest

@MainActor
final class WebKitDownloadTests: AdapterTestCase {
    /// A web view on the page of links that each start a download a different way.
    private func page(_ id: UInt64 = 1) async {
        await create(id)
        await load(id, site.url("/downloads"))
        await wait(id, "the page to load") { self.finished($0) && self.hasTitle("Downloads")($0) }
    }

    /// The downloads the adapter has reported a web view's page asking for, in order.
    private func asked(_ id: UInt64 = 1) -> [WTDownloadRequested] {
        reports(id).compactMap {
            if case let .downloadRequested(asked) = $0 {
                asked
            } else {
                nil
            }
        }
    }

    /// The names the adapter has said downloads finished under, in order.
    private func finished(_ id: UInt64 = 1) -> [String] {
        reports(id).compactMap {
            if case let .downloadFinished(done) = $0 {
                done.fileName
            } else {
                nil
            }
        }
    }

    /// Clicks a link of the page, and waits for the download it starts to be reported.
    private func click(_ link: String, in id: UInt64 = 1) async -> WTDownloadRequested {
        let before = asked(id).count
        await run("document.getElementById('\(link)').click()", in: id)
        // The first download of a run takes WebKit on an iPhone simulator a while to get going
        // (23 s was measured here, then 7, then under one): five minutes, where a minute is
        // patience enough for anything else.
        for _ in 0 ..< Patience.steps * 5 where asked(id).count == before {
            try? await Task.sleep(for: .milliseconds(10))
        }
        XCTAssertGreaterThan(asked(id).count, before, "a download was asked for")
        return asked(id).last ?? WTDownloadRequested()
    }

    /// The core's word that a download starts, under `name`.
    @discardableResult
    private func start(_ download: UInt64, as name: String) async -> WTHostResponse.OneOf_Kind {
        var start = WTDownloadStart()
        start.downloadID = download
        start.fileName = name
        return await engine.answer(.downloadStart(start))
    }

    /// Starts a download and waits for its file to be whole. The name it finished under.
    private func save(_ download: WTDownloadRequested, as name: String, in id: UInt64 = 1) async -> String {
        let before = finished(id).count
        await start(download.downloadID, as: name)
        await wait(id, "the download to finish") { _ in self.finished(id).count > before }
        return finished(id).last ?? ""
    }

    private func cancel(_ download: UInt64) async -> WTHostResponse.OneOf_Kind {
        var cancel = WTDownloadCancel()
        cancel.downloadID = download
        return await engine.answer(.downloadCancel(cancel))
    }

    /// What is in the file of that name in the engine's folder. Nil when there is none.
    private func saved(_ name: String) -> String? {
        try? String(contentsOf: engine.downloadsFolder.appendingPathComponent(name), encoding: .utf8)
    }

    /// The names of everything in the engine's folder.
    private func folder() -> [String] {
        ((try? FileManager.default.contentsOfDirectory(atPath: engine.downloadsFolder.path)) ?? []).sorted()
    }

    func testAnAttachmentIsReportedAndSavedUnderTheNameTheCoreGives() async {
        await page()

        let download = await click("attachment")

        XCTAssertEqual(download.url, site.url("/notes"))
        XCTAssertEqual(download.suggestedFilename, "notes.txt", "what the site suggests is passed on as it is")
        XCTAssertEqual(download.mimeType, "text/plain")
        XCTAssertEqual(download.expectedBytes, 26)
        XCTAssertNotEqual(download.downloadID, 0, "named by the shell")
        XCTAssertEqual(folder(), [], "nothing is saved until the core says under what name")

        let name = await save(download, as: "My notes.txt")

        XCTAssertEqual(name, "My notes.txt")
        XCTAssertEqual(saved("My notes.txt"), "Twelve notes, to be saved.")
        XCTAssertEqual(folder(), ["My notes.txt"])
        // The page that asked is the page on screen: the load that became a download stopped.
        XCTAssertEqual(engine.webView(for: 1)?.url?.absoluteString, site.url("/downloads"))
        XCTAssertFalse(names(reports(1)).contains { $0.hasPrefix("failed(") && $0 != "failed(net.cancelled)" })
    }

    func testAKindNothingShowsIsADownload() async {
        await page()

        let download = await click("kind")

        XCTAssertEqual(download.url, site.url("/data.bin"))
        // The kind is what WebKit makes of it. The site says "application/octet-stream"; a Mac
        // passes that on, and an iPhone, going by the address's ending, says
        // "application/macbinary". Neither is a kind it shows.
        XCTAssertTrue(download.mimeType.hasPrefix("application/"), download.mimeType)
        XCTAssertEqual(download.expectedBytes, 16)
        let name = await save(download, as: "data.bin")
        XCTAssertEqual(saved(name), "0123456789abcdef")
    }

    func testALinkThatSaysDownloadIsOne() async {
        await page()

        // A page a browser would show: the link says it is to be saved, and under what name.
        let download = await click("says")

        XCTAssertEqual(download.url, site.url("/landed"))
        XCTAssertEqual(download.suggestedFilename, "kept page.html")
        let name = await save(download, as: "kept page.html")
        XCTAssertTrue(saved(name)?.contains("<title>Landed</title>") ?? false)
        XCTAssertEqual(engine.webView(for: 1)?.url?.absoluteString, site.url("/downloads"), "and the web view went nowhere")
    }

    func testADownloadTheCoreDropsSavesNothing() async {
        await page()
        let download = await click("attachment")

        guard case .ack = await cancel(download.downloadID) else { return XCTFail("dropping is acknowledged") }
        try? await Task.sleep(for: .milliseconds(300))

        XCTAssertEqual(folder(), [])
        XCTAssertEqual(finished(), [])
        // It is no longer waiting for a word: one that comes now is for nothing.
        guard case let .error(error) = await start(download.downloadID, as: "late.txt") else { return XCTFail("expected a refusal") }
        XCTAssertEqual(error.code, "download.unknown")
        XCTAssertEqual(folder(), [])
    }

    func testANameThatIsTakenGainsANumber() async throws {
        await page()
        try FileManager.default.createDirectory(at: engine.downloadsFolder, withIntermediateDirectories: true)
        try Data("mine".utf8).write(to: engine.downloadsFolder.appendingPathComponent("notes.txt"))

        let second = await save(click("attachment"), as: "notes.txt")
        let third = await save(click("attachment"), as: "notes.txt")

        XCTAssertEqual([second, third], ["notes (2).txt", "notes (3).txt"], "the adapter says what it used")
        XCTAssertEqual(saved("notes.txt"), "mine", "what was there is as it was")
        XCTAssertEqual(saved("notes (2).txt"), "Twelve notes, to be saved.")
        XCTAssertEqual(folder(), ["notes (2).txt", "notes (3).txt", "notes.txt"])
    }

    func testANameIsHeldToANameWhateverItSays() async {
        await page()
        // The site's own suggestion has folders in it. It is the core that makes the name
        // (013 §4); a core that gave a path all the same is held to its last part.
        let download = await click("sneaky")
        XCTAssertEqual(folder(), [])

        let name = await save(download, as: "../../escaped.txt")

        XCTAssertEqual(name, "escaped.txt")
        XCTAssertEqual(folder(), ["escaped.txt"])
        let above = engine.downloadsFolder.deletingLastPathComponent().deletingLastPathComponent().appendingPathComponent("escaped.txt")
        XCTAssertFalse(FileManager.default.fileExists(atPath: above.path), "nothing is written outside the folder")
    }

    func testADownloadIsFetchedAsItsLoginSet() async {
        // Signed in, in this web view's login set; then a file that says who asked for it.
        await create(1)
        await load(1, site.url("/set?v=ada"))
        await wait(1, "the cookie to be set") { self.finished($0) && self.hasTitle("Set")($0) }
        await load(1, site.url("/downloads"))
        await wait(1, "the page of links") { self.hasTitle("Downloads")($0) }

        let name = await save(click("whose"), as: "who.txt")

        XCTAssertEqual(saved(name), "who=ada", "WebKit fetches in the page's own session")
        // Another login set's web view is nobody there.
        await page(2)
        let other = await save(click("whose", in: 2), as: "who.txt", in: 2)
        XCTAssertEqual(saved(other), "who=none")
    }

    /// Cookies stay with their site (027), for a download a page starts: a link of this site
    /// leads to a file that is at another site (the server by its other name). The person is
    /// nobody there, and is not made somebody by what this site knows them by; signed in
    /// there too, as somebody else, the file is fetched as that site knows them. WebKit moves
    /// the bytes itself, and this is its own doing: the shell sends no cookie anywhere.
    func testADownloadAPageStartsThatIsAtAnotherSiteIsFetchedThereAsThatSiteKnowsThePerson() async {
        await create(1)
        await show(site.url("/set?v=ada"), titled: "Set")
        await show(site.url("/downloads-elsewhere?port=\(site.port)"), titled: "Downloads elsewhere")

        let download = await click("elsewhere")

        XCTAssertEqual(download.url, "http://localhost:\(site.port)/whose-file", "reported as where the file is")
        let elsewhere = await save(download, as: "elsewhere.txt")
        XCTAssertEqual(saved(elsewhere), "who=none", "what this site knows the person by was not sent to the other")

        // Signed in at the other site as somebody else, in the same login set.
        await show("http://localhost:\(site.port)/set?v=grace", titled: "Set")
        await show(site.url("/downloads-elsewhere?port=\(site.port)"), titled: "Downloads elsewhere")
        let theirs = await save(click("elsewhere"), as: "theirs.txt")
        XCTAssertEqual(saved(theirs), "who=grace", "fetched there as that site knows the person")
        // And this site still knows them as it did.
        await load(1, site.url("/whoami"))
        await wait(1, "who this site says is signed in") { self.hasTitle("who=ada")($0) }
    }

    /// Loads an address in web view 1 and waits for its page to be whole: this load's own end,
    /// and its title. A page's links are not there to be clicked when its title has come: only
    /// when it has finished.
    private func show(_ address: String, titled title: String) async {
        let ended = finishes(reports(1))
        await load(1, address)
        await wait(1, "the page \(title) to be whole") { self.finishes($0) > ended && self.lastTitle(1) == title }
    }

    /// How many loads have been reported finished.
    private func finishes(_ reports: [WTEngineEvent.OneOf_Kind]) -> Int {
        reports.count(where: {
            if case .navigationFinished = $0 {
                true
            } else {
                false
            }
        })
    }

    func testAWordAboutADownloadThereIsNot() async {
        guard case let .error(error) = await start(99, as: "nothing.txt") else { return XCTFail("expected a refusal") }
        XCTAssertEqual(error.code, "download.unknown")
        guard case .ack = await cancel(99) else { return XCTFail("dropping what is not there is acknowledged") }
    }
}

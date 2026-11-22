// Favourites (042), against a scripted core: what the window asks of the core to make a tab a
// Favourite or to stop it being one, when it does not ask at all, and what a tile shows and
// says.
import WTContract
import WTCoreClient
@testable import WTUI
import WTWebKit
import XCTest

@MainActor
final class FavouriteModelTests: XCTestCase {
    /// A model over a core that opens window 1 with tab 2 and says Ack to the rest.
    private func model() -> (BrowserModel, ScriptedCore) {
        let core = ScriptedCore { request in
            switch request {
            case .openWindow:
                var opened = WTWindowOpened()
                opened.windowID = 1
                opened.tabID = 2
                return .windowOpened(opened)
            case .subscribe:
                var subscribed = WTSubscribed()
                subscribed.subscriptionID = 3
                return .subscribed(subscribed)
            default:
                return .ack(WTAck())
            }
        }
        return (BrowserModel(session: BrowserSession(core: core, engine: WebKitEngine())), core)
    }

    private func tab(
        _ id: UInt64, kept: Bool = false, favourite: Bool = false, asleep: Bool = false, away: Bool = false,
        title: String = "", host: String = "example.com", siteName: String = ""
    ) -> WTTabSummary {
        var summary = WTTabSummary()
        summary.tabID = id
        summary.kept = kept || favourite
        summary.favourite = favourite
        summary.asleep = asleep
        summary.away = away
        summary.title = title
        summary.host = host
        summary.siteName = siteName
        return summary
    }

    /// The window as the core says it is: tab 2 in front, with a page or without, a Favourite
    /// or not; these tabs; and whether the row is full.
    private func snapshot(
        page: Bool = true, favourite: Bool = false, tabs: [WTTabSummary], full: Bool = false, private isPrivate: Bool = false
    ) -> WTWindowSnapshot {
        var said = WTWindowSnapshot()
        said.windowID = 1
        said.tab.tabID = 2
        said.tab.webViewID = page ? 4 : 0
        said.tab.kept = favourite
        said.tab.favourite = favourite
        said.tabs = tabs
        said.favouritesFull = full
        said.privateWindow = isPrivate
        return said
    }

    /// What the core was asked about Favourites, in order: "add 2", "remove 5".
    private func asked(_ core: ScriptedCore) -> [String] {
        core.requests.compactMap { request -> String? in
            if case let .setFavourite(set) = request {
                "\(set.favourite ? "add" : "remove") \(set.tabID)"
            } else {
                nil
            }
        }
    }

    /// Lets the session's event task run.
    private func settle(until done: () -> Bool) async {
        for _ in 0 ..< 200 where !done() {
            await Task.yield()
            try? await Task.sleep(for: .milliseconds(2))
        }
    }

    func testATabIsMadeAFavouriteOrStopsBeingOneByAskingTheCore() async {
        let (model, core) = model()
        // Before the window is open, and for a tab with no page: nothing to keep.
        XCTAssertFalse(model.canFavouriteTheTabInFront)
        model.addOrRemoveTheTabInFrontAsAFavourite()
        await model.open()
        core.emit(snapshot(page: false, tabs: [tab(2, host: "")]), subscription: 3)
        await settle { model.tabs.count == 1 }
        XCTAssertFalse(model.canFavouriteTheTabInFront)
        XCTAssertFalse(model.canFavourite(model.tabs[0]), "an empty tab has nothing to keep")
        model.addOrRemoveTheTabInFrontAsAFavourite()
        XCTAssertEqual(asked(core), [])

        // A page: the tab in front, and a tab of the strip, can be made one; the core is
        // asked, and decides the rest.
        core.emit(snapshot(tabs: [tab(2), tab(5)]), subscription: 3)
        await settle { model.tabs.count == 2 }
        XCTAssertTrue(model.canFavouriteTheTabInFront)
        XCTAssertTrue(model.canFavourite(model.tabs[1]))
        model.addOrRemoveTheTabInFrontAsAFavourite()
        model.setFavourite(5, true)
        XCTAssertEqual(asked(core), ["add 2", "add 5"])

        // The core says it is one: the same command stops it being one.
        core.emit(snapshot(favourite: true, tabs: [tab(2, favourite: true), tab(5, favourite: true)]), subscription: 3)
        await settle { model.tab.favourite }
        XCTAssertTrue(model.canFavouriteTheTabInFront)
        model.addOrRemoveTheTabInFrontAsAFavourite()
        model.setFavourite(5, false)
        XCTAssertEqual(asked(core), ["add 2", "add 5", "remove 2", "remove 5"])
    }

    func testNoMoreIsOfferedWhenTheCoreSaysTheRowIsFullOrTheWindowIsPrivate() async {
        let (model, core) = model()
        await model.open()
        // The row is full, by the core's word: a tab that is none cannot be made one; one
        // that is can stop being one.
        core.emit(snapshot(tabs: [tab(5, favourite: true), tab(2), tab(6)], full: true), subscription: 3)
        await settle { model.favouritesFull }
        XCTAssertFalse(model.canFavouriteTheTabInFront)
        XCTAssertFalse(model.canFavourite(model.tabs[2]))
        XCTAssertTrue(model.canFavourite(model.tabs[0]), "a Favourite can stop being one")
        model.addOrRemoveTheTabInFrontAsAFavourite()
        XCTAssertEqual(asked(core), [], "nothing is asked for a tab that cannot be made one")
        // With room again, it can.
        core.emit(snapshot(tabs: [tab(5, favourite: true), tab(2), tab(6)]), subscription: 3)
        await settle { !model.favouritesFull }
        XCTAssertTrue(model.canFavouriteTheTabInFront)
        // The row full, and the tab in front one of its Favourites: it can stop being one,
        // and the command asks for that.
        core.emit(snapshot(favourite: true, tabs: [tab(2, favourite: true), tab(6)], full: true), subscription: 3)
        await settle { model.favouritesFull && model.tab.favourite }
        XCTAssertTrue(model.canFavouriteTheTabInFront, "a Favourite in front can stop being one, however full the row")
        model.addOrRemoveTheTabInFrontAsAFavourite()
        XCTAssertEqual(asked(core), ["remove 2"])

        // A private window has no Favourites (037).
        core.emit(snapshot(tabs: [tab(2)], private: true), subscription: 3)
        await settle { model.isPrivate }
        XCTAssertFalse(model.canFavouriteTheTabInFront)
        XCTAssertFalse(model.canFavourite(model.tabs[0]))
        model.addOrRemoveTheTabInFrontAsAFavourite()
        XCTAssertEqual(asked(core), ["remove 2"], "nothing more was asked")
    }

    func testATileShowsALetterAndSaysTheWholeName() {
        let english = Locale(identifier: "en")
        // The first letter of the name it was kept with, as a capital: not of where it has
        // wandered to.
        let wandered = tab(1, favourite: true, away: true, title: "Re: lunch", host: "mail.example.com", siteName: "mail")
        XCTAssertEqual(TabStrip.letter(of: wandered, locale: english), "M")
        XCTAssertEqual(TabStrip.shownName(of: wandered, locale: english), "Re: lunch")
        // Kept with no name: of the name it shows, its title or its site.
        XCTAssertEqual(TabStrip.letter(of: tab(1, favourite: true, title: "Board"), locale: english), "B")
        XCTAssertEqual(TabStrip.letter(of: tab(1, favourite: true, host: "news.example.com"), locale: english), "N")
        XCTAssertEqual(TabStrip.shownName(of: tab(1, favourite: true, host: "news.example.com"), locale: english), "news.example.com")
        // A name that begins with a picture has the picture.
        XCTAssertEqual(TabStrip.letter(of: tab(1, favourite: true, siteName: "🐯 Tiger"), locale: english), "🐯")

        // What is said of it beside its name: that it is a Favourite, which is a Site and
        // is not said to be both; that it is away; that it is asleep.
        XCTAssertEqual(TabStrip.said(of: tab(1, favourite: true), locale: english), "Favourite")
        XCTAssertEqual(TabStrip.said(of: tab(1, favourite: true, asleep: true), locale: english), "Favourite, Asleep")
        XCTAssertEqual(TabStrip.said(of: wandered, locale: english), "Favourite, Away")
        XCTAssertEqual(TabStrip.said(of: tab(1, kept: true), locale: english), "Site")
    }
}

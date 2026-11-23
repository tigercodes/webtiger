// Sites' icons in a window (046), against a scripted core: which pictures the window asks the
// core for, how many to a question, what it does with what is no picture, and what it lets go
// of when it has kept many.
import ImageIO
import SwiftUI
import UniformTypeIdentifiers
import WTContract
import WTCoreClient
@testable import WTUI
import WTWebKit
import XCTest

@MainActor
final class IconModelTests: XCTestCase {
    /// A square picture of one colour, as a PNG.
    static func png(side: Int) -> Data {
        let context = CGContext(
            data: nil, width: side, height: side, bitsPerComponent: 8, bytesPerRow: side * 4,
            space: CGColorSpaceCreateDeviceRGB(), bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
        )
        context?.setFillColor(red: 0.1, green: 0.4, blue: 0.9, alpha: 1)
        context?.fill(CGRect(x: 0, y: 0, width: side, height: side))
        let written = NSMutableData()
        guard let image = context?.makeImage(),
              let destination = CGImageDestinationCreateWithData(written, UTType.png.identifier as CFString, 1, nil)
        else { return Data() }
        CGImageDestinationAddImage(destination, image, nil)
        return CGImageDestinationFinalize(destination) ? written as Data : Data()
    }

    /// A model over a core that opens window 1 with tab 2, and answers a question about icons
    /// with the pictures it has of the numbers asked.
    private func model(pictures: [UInt64: Data]) -> (BrowserModel, ScriptedCore) {
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
            case let .queryIcons(query):
                var page = WTIconsPage()
                page.icons = query.iconIds.compactMap { number in
                    pictures[number].map { png in
                        var icon = WTIcon()
                        icon.iconID = number
                        icon.png = png
                        return icon
                    }
                }
                return .iconsPage(page)
            default:
                return .ack(WTAck())
            }
        }
        return (BrowserModel(session: BrowserSession(core: core, engine: WebKitEngine())), core)
    }

    /// The window as the core says it is: tab 2 in front, and a tab for each of these icons'
    /// numbers (0 for a tab that has none).
    private func snapshot(icons: [UInt64]) -> WTWindowSnapshot {
        var said = WTWindowSnapshot()
        said.windowID = 1
        said.tab.tabID = 2
        said.tabs = icons.enumerated().map { place, number in
            var tab = WTTabSummary()
            tab.tabID = UInt64(place + 2)
            tab.host = "site\(place).example.com"
            tab.iconID = number
            return tab
        }
        return said
    }

    /// The numbers the core was asked the pictures of, a list for each question.
    private func asked(_ core: ScriptedCore) -> [[UInt64]] {
        core.requests.compactMap { request -> [UInt64]? in
            if case let .queryIcons(query) = request {
                query.iconIds
            } else {
                nil
            }
        }
    }

    /// Lets the session's event task run.
    private func settle(until done: () -> Bool) async {
        for _ in 0 ..< 400 where !done() {
            await Task.yield()
            try? await Task.sleep(for: .milliseconds(2))
        }
    }

    func testTheWindowAsksOnceForEachPictureItHasNotGot() async {
        let (model, core) = model(pictures: [7: Self.png(side: 32), 9: Self.png(side: 64)])
        await model.open()
        // Three tabs, two with icons, one of them twice: one question, each number once.
        core.emit(snapshot(icons: [9, 0, 7, 9]), subscription: 3)
        await settle { model.icons.count == 2 }
        XCTAssertEqual(asked(core), [[7, 9]])
        XCTAssertEqual(model.icons[7]?.width, 32)
        XCTAssertEqual(model.icons[9]?.width, 64)
        XCTAssertNotNil(model.icon(of: model.tabs[0]))
        XCTAssertNil(model.icon(of: model.tabs[1]), "a tab with no icon has none")

        // Told the same again, and with one of them gone: nothing is asked.
        core.emit(snapshot(icons: [9, 0, 7, 9]), subscription: 3)
        core.emit(snapshot(icons: [7]), subscription: 3)
        await settle { model.tabs.count == 1 }
        XCTAssertEqual(asked(core), [[7, 9]])
        // A tab comes to have another: that one is asked for, alone.
        core.emit(snapshot(icons: [7, 11]), subscription: 3)
        await settle { asked(core).count == 2 }
        XCTAssertEqual(asked(core), [[7, 9], [11]])
    }

    func testWhatIsNoPictureIsNotDrawnAndNotAskedForAgain() async {
        // 1: a picture. 2: one the core has none of. 3: words. 4: a picture larger than the
        // core keeps. 5: the first bytes of a PNG and nothing after.
        let pictures: [UInt64: Data] = [
            1: Self.png(side: 16),
            3: Data("<!doctype html>".utf8),
            4: Self.png(side: IconPictures.pixels + 1),
            5: Data([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]),
        ]
        let (model, core) = model(pictures: pictures)
        await model.open()
        core.emit(snapshot(icons: [1, 2, 3, 4, 5]), subscription: 3)
        await settle { model.icons.count == 1 }
        XCTAssertEqual(model.icons.keys.sorted(), [1])
        XCTAssertEqual(asked(core), [[1, 2, 3, 4, 5]])
        core.emit(snapshot(icons: [1, 2, 3, 4, 5, 0]), subscription: 3)
        await settle { model.tabs.count == 6 }
        XCTAssertEqual(asked(core), [[1, 2, 3, 4, 5]], "what was no picture is not asked for again")
        // A picture as large as the core keeps is one.
        XCTAssertNotNil(IconPictures.picture(Self.png(side: IconPictures.pixels)))
        XCTAssertNil(IconPictures.picture(Self.png(side: IconPictures.pixels + 1)))
        XCTAssertNil(IconPictures.picture(Data()))
    }

    func testNoMoreIsAskedInOneQuestionThanAnAnswerCarries() async {
        let numbers = (1 ... 150).map(UInt64.init)
        let picture = Self.png(side: 8)
        let (model, core) = model(pictures: Dictionary(uniqueKeysWithValues: numbers.map { ($0, picture) }))
        await model.open()
        core.emit(snapshot(icons: numbers), subscription: 3)
        await settle { model.icons.count == 150 }
        XCTAssertEqual(asked(core).map(\.count), [64, 64, 22])
        XCTAssertEqual(asked(core).flatMap(\.self), numbers, "each number once, in order")
        XCTAssertEqual(model.icons.count, 150)
    }

    func testAWindowThatHasKeptManyLetsGoOfThoseItsTabsDoNotShow() async {
        // (A window keeps 512 beyond those it shows.)
        let numbers = Array(1 ... UInt64(552))
        let picture = Self.png(side: 8)
        let (model, core) = model(pictures: Dictionary(uniqueKeysWithValues: numbers.map { ($0, picture) }))
        await model.open()
        // Fewer than it keeps: tabs that go leave their pictures, and one that comes back is
        // not asked for again.
        core.emit(snapshot(icons: Array(numbers.prefix(100))), subscription: 3)
        await settle { model.icons.count == 100 }
        core.emit(snapshot(icons: [1]), subscription: 3)
        await settle { model.tabs.count == 1 }
        XCTAssertEqual(model.icons.count, 100)
        core.emit(snapshot(icons: [1, 50]), subscription: 3)
        await settle { model.tabs.count == 2 }
        XCTAssertEqual(asked(core).flatMap(\.self).count { $0 == 50 }, 1)

        // More than it keeps: at the next word from the core it keeps those its tabs show.
        core.emit(snapshot(icons: numbers), subscription: 3)
        await settle { model.icons.count == numbers.count }
        XCTAssertEqual(model.icons.count, 552)
        core.emit(snapshot(icons: [1, 2, 3]), subscription: 3)
        await settle { model.icons.count == 3 }
        XCTAssertEqual(model.icons.keys.sorted(), [1, 2, 3])
        // And asks again for one it then shows.
        core.emit(snapshot(icons: [1, 2, 3, 50]), subscription: 3)
        await settle { model.icons.count == 4 }
        XCTAssertEqual(asked(core).flatMap(\.self).count { $0 == 50 }, 2)
        XCTAssertEqual(asked(core).flatMap(\.self).count { $0 == 1 }, 1, "one it kept is not asked for again")
    }

    // MARK: Drawn

    /// How many pixels of a picture are a colour, to within a little.
    private func count(_ colour: [Int], in picture: Picture?) -> Int {
        guard let picture else { return -1 }
        var found = 0
        for index in stride(from: 0, to: picture.pixels.count, by: 4)
            where (0 ..< 3).allSatisfy({ abs(Int(picture.pixels[index + $0]) - colour[$0]) <= 12 }) {
            found += 1
        }
        return found
    }

    /// The colours of the catalogue's three icons, which nothing else in a window is.
    private static let orange = [0xE0, 0x6A, 0x10]
    private static let blue = [0x10, 0x5A, 0xE0]
    private static let green = [0x2E, 0x9E, 0x55]

    /// The pictures' own check lets a change as small as an icon by (a hundredth of a
    /// picture may differ), so that an icon is drawn is counted here: by its colour.
    func testEachIconIsDrawnInItsRowAndOnItsTile() async {
        func drawn(_ state: CatalogueState, wide: Bool = false) async -> Picture? {
            // (Tall enough for every row down the side, on a touch screen too, where a row is taller.)
            let size = wide ? CGSize(width: 720, height: 900) : CGSize(width: 480, height: CatalogueWindow.height)
            return await Snapshot.Stage(CatalogueWindow(state: state), size: size, dark: false).settledPicture()
        }
        // A window whose tabs have no icons has nothing of their colours.
        let none = await drawn(.sites)
        XCTAssertLessThan(count(Self.orange, in: none), 20)
        XCTAssertLessThan(count(Self.blue, in: none), 20)
        XCTAssertLessThan(count(Self.green, in: none), 20)
        XCTAssertGreaterThanOrEqual(count(Self.orange, in: none), 0, "the window was drawn")

        // The tabs along the top: a Favourite's tile is its icon, the larger; a Site has its
        // icon before its name, the smaller; the tab with none has none.
        let strip = await drawn(.favourites)
        let tile = count(Self.orange, in: strip)
        let row = count(Self.blue, in: strip)
        XCTAssertGreaterThan(tile, 1000, "a tile's icon, 24 points a side")
        XCTAssertGreaterThan(row, 400, "a tab's icon, 16 points a side")
        XCTAssertGreaterThan(tile, row * 2, "a tile's is the larger")
        XCTAssertLessThan(count(Self.green, in: strip), 20)

        // Down the side: the same, and a tab that is no Site has its icon too. The sleeping
        // Site's icon is fainter, and is not counted as its colour: one green, not two.
        let side = await drawn(.sidebar, wide: true)
        XCTAssertGreaterThan(count(Self.orange, in: side), 1000)
        XCTAssertGreaterThan(count(Self.blue, in: side), 400)
        XCTAssertGreaterThan(count(Self.green, in: side), 400)
        XCTAssertLessThan(count(Self.green, in: side), 1000, "the sleeping Site's is fainter")
    }

    func testAnIconIsDrawnBeforeATabsNameAndFainterWhenTheTabSleeps() async throws {
        let picture = try XCTUnwrap(IconPictures.picture(XCTUnwrap(CatalogueIcons.png(CatalogueIcons.blue))))
        var tab = WTTabSummary()
        tab.tabID = 5
        tab.title = "Docs"
        func drawn(_ tab: WTTabSummary, icon: CGImage?) async -> Picture? {
            let label = TabLabel(tab: tab, isFront: false, icon: icon)
            return await Snapshot.Stage(label, size: CGSize(width: 200, height: 40), dark: false).settledPicture()
        }
        let without = await drawn(tab, icon: nil)
        let with = await drawn(tab, icon: picture)
        XCTAssertEqual(count(Self.blue, in: without), 0)
        XCTAssertGreaterThan(count(Self.blue, in: with), 400)
        // Asleep: it is there, and is not its full colour.
        tab.asleep = true
        let bare = await drawn(tab, icon: nil)
        let faint = await drawn(tab, icon: picture)
        let asleepWithout = try XCTUnwrap(bare)
        let asleep = try XCTUnwrap(faint)
        XCTAssertEqual(count(Self.blue, in: asleep), 0, "a sleeping tab's icon is fainter")
        // And is drawn: there is as much that is blue as when it is awake, and none of it
        // in a sleeping tab with no icon.
        func bluish(_ picture: Picture) -> Int {
            stride(from: 0, to: picture.pixels.count, by: 4).count { index in
                let (red, green, blue) = (Int(picture.pixels[index]), Int(picture.pixels[index + 1]), Int(picture.pixels[index + 2]))
                return blue > red + 40 && blue > green + 20
            }
        }
        XCTAssertGreaterThan(bluish(asleep), 400)
        XCTAssertEqual(bluish(asleepWithout), 0)
    }
}

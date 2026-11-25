// A sleeping tab's picture in a window (059), against a scripted core: when the window asks
// the core for the picture its tab names, what it shows and until when, what it does with
// what is no picture, and where the picture is drawn.
import ImageIO
import SwiftUI
import UniformTypeIdentifiers
import WTContract
import WTCoreClient
@testable import WTUI
import WTWebKit
import XCTest

@MainActor
final class PagePictureTests: XCTestCase {
    /// A picture whose top half is one colour and whose bottom half another, as a JPEG (or,
    /// asked for one, as a PNG).
    static func picture(wide: Int, tall: Int, as type: UTType = .jpeg) -> Data {
        let context = CGContext(
            data: nil, width: wide, height: tall, bitsPerComponent: 8, bytesPerRow: 0,
            space: CGColorSpaceCreateDeviceRGB(), bitmapInfo: CGImageAlphaInfo.noneSkipLast.rawValue
        )
        // Quartz counts up from the bottom: the top half is the upper one.
        context?.setFillColor(red: 0.8, green: 0.1, blue: 0.1, alpha: 1)
        context?.fill(CGRect(x: 0, y: tall / 2, width: wide, height: tall - tall / 2))
        context?.setFillColor(red: 0.1, green: 0.2, blue: 0.8, alpha: 1)
        context?.fill(CGRect(x: 0, y: 0, width: wide, height: tall / 2))
        let written = NSMutableData()
        guard let image = context?.makeImage(),
              let destination = CGImageDestinationCreateWithData(written, type.identifier as CFString, 1, nil)
        else { return Data() }
        CGImageDestinationAddImage(destination, image, nil)
        return CGImageDestinationFinalize(destination) ? written as Data : Data()
    }

    /// A model over a core that opens window 1 with tab 2, and answers a question about
    /// pictures with those it has of the numbers asked.
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
            case let .queryPictures(query):
                var page = WTPicturesPage()
                page.pictures = query.pictureIds.compactMap { number in
                    pictures[number].map { jpeg in
                        var picture = WTPagePicture()
                        picture.pictureID = number
                        picture.jpeg = jpeg
                        return picture
                    }
                }
                return .picturesPage(page)
            default:
                return .ack(WTAck())
            }
        }
        return (BrowserModel(session: BrowserSession(core: core, engine: WebKitEngine())), core)
    }

    /// The tab in front as the core says it is: on its way, and naming this picture (0: none).
    private func tab(picture: UInt64, id: UInt64 = 2, progress: Double = 0.1) -> WTTabView {
        var tab = WTTabView()
        tab.tabID = id
        tab.webViewID = 4
        tab.loadState = .loading
        tab.progress = progress
        tab.pictureID = picture
        return tab
    }

    private func snapshot(picture: UInt64, tab id: UInt64 = 2, progress: Double = 0.1) -> WTWindowSnapshot {
        var said = WTWindowSnapshot()
        said.windowID = 1
        said.tab = tab(picture: picture, id: id, progress: progress)
        return said
    }

    /// The numbers the core was asked the pictures of, a list for each question.
    private func asked(_ core: ScriptedCore) -> [[UInt64]] {
        core.requests.compactMap { request -> [UInt64]? in
            if case let .queryPictures(query) = request {
                query.pictureIds
            } else {
                nil
            }
        }
    }

    private func settle(until done: () -> Bool) async {
        for _ in 0 ..< 400 where !done() {
            await Task.yield()
            try? await Task.sleep(for: .milliseconds(2))
        }
    }

    func testTheWindowAsksForThePictureItsTabNamesOnceAndShowsItUntilTheTabNamesNone() async {
        let (model, core) = model(pictures: [7: Self.picture(wide: 800, tall: 600), 9: Self.picture(wide: 400, tall: 900)])
        await model.open()
        // A tab with no picture: nothing is asked, and nothing shown.
        core.emit(snapshot(picture: 0), subscription: 3)
        await settle { model.tab.webViewID == 4 }
        XCTAssertEqual(asked(core), [])
        XCTAssertNil(model.pagePicture)

        // The tab in front names a picture: asked for, by its number alone, and shown.
        core.emit(snapshot(picture: 7), subscription: 3)
        await settle { model.pagePicture != nil }
        XCTAssertEqual(asked(core), [[7]])
        XCTAssertEqual([model.pagePicture?.width, model.pagePicture?.height], [800, 600])
        // Told again while the page comes (its progress moves): not asked again, and there still.
        core.emit(snapshot(picture: 7, progress: 0.6), subscription: 3)
        await settle { model.tab.progress == 0.6 }
        XCTAssertEqual(asked(core), [[7]])
        XCTAssertNotNil(model.pagePicture)

        // The page has arrived: the tab names none, and there is none.
        core.emit(snapshot(picture: 0, progress: 0.8), subscription: 3)
        await settle { model.tab.progress == 0.8 }
        XCTAssertNil(model.pagePicture)
        XCTAssertEqual(asked(core), [[7]])

        // Another tab in front, waking with its own; and the first again, on its way still:
        // each is asked for as it is named.
        core.emit(snapshot(picture: 9, tab: 5), subscription: 3)
        await settle { model.pagePicture?.width == 400 }
        XCTAssertEqual([model.pagePicture?.width, model.pagePicture?.height], [400, 900])
        core.emit(snapshot(picture: 0, tab: 6), subscription: 3)
        await settle { model.tab.tabID == 6 }
        XCTAssertNil(model.pagePicture)
        core.emit(snapshot(picture: 9, tab: 5), subscription: 3)
        await settle { model.pagePicture != nil }
        XCTAssertEqual(asked(core), [[7], [9], [9]])
        XCTAssertEqual(model.pagePicture?.width, 400)
    }

    func testWhatIsNoPictureIsNotDrawn() async {
        // 1: the core has none of it. 2: words. 3: a PNG. 4: a JPEG wider than the core keeps.
        // 5: the first bytes of a JPEG and nothing after. 6: a picture.
        let pictures: [UInt64: Data] = [
            2: Data("<!doctype html>".utf8),
            3: Self.picture(wide: 800, tall: 600, as: .png),
            4: Self.picture(wide: PagePictures.mostWide + 1, tall: 600),
            5: Data([0xFF, 0xD8, 0xFF]),
            6: Self.picture(wide: 320, tall: 240),
        ]
        let (model, core) = model(pictures: pictures)
        await model.open()
        for number in UInt64(1) ... 5 {
            core.emit(snapshot(picture: number), subscription: 3)
            await settle { asked(core).count == Int(number) }
            // Given the time a picture would take to come.
            try? await Task.sleep(for: .milliseconds(50))
            XCTAssertNil(model.pagePicture, "picture \(number) is none")
        }
        core.emit(snapshot(picture: 6), subscription: 3)
        await settle { model.pagePicture != nil }
        XCTAssertEqual(model.pagePicture?.width, 320, "and the next that is one is shown")

        // What is a picture to draw: a JPEG, no wider than the core keeps, and no taller than a window.
        XCTAssertNotNil(PagePictures.picture(Self.picture(wide: PagePictures.mostWide, tall: 2)))
        XCTAssertNil(PagePictures.picture(Self.picture(wide: PagePictures.mostWide + 1, tall: 2)))
        XCTAssertNotNil(PagePictures.picture(Self.picture(wide: 2, tall: PagePictures.mostTall)))
        XCTAssertNil(PagePictures.picture(Self.picture(wide: 2, tall: PagePictures.mostTall + 1)))
        XCTAssertNil(PagePictures.picture(Self.picture(wide: 80, tall: 60, as: .png)))
        XCTAssertNil(PagePictures.picture(Data()))
    }

    /// A core whose answers wait until the test lets each go, in whatever order it likes.
    private final class HeldCore: CoreClient, @unchecked Sendable {
        private let lock = NSLock()
        private var asked = 0
        private var waiting: [Int: CheckedContinuation<WTResponse.OneOf_Kind, any Error>] = [:]
        let events = AsyncStream<WTEvent.OneOf_Kind> { _ in }

        /// How many questions are waiting for their answers.
        var held: Int {
            lock.withLock { waiting.count }
        }

        func send(_: WTRequest.OneOf_Kind) -> CoreAnswer {
            let number = lock.withLock { () -> Int in
                asked += 1
                return asked
            }
            return Task {
                try await withCheckedThrowingContinuation { continuation in
                    lock.withLock { waiting[number] = continuation }
                }
            }
        }

        /// Answers the question that was asked `number`th, from one, with this picture under
        /// this number.
        func answer(_ number: Int, picture id: UInt64, _ jpeg: Data) {
            var picture = WTPagePicture()
            picture.pictureID = id
            picture.jpeg = jpeg
            var page = WTPicturesPage()
            page.pictures = [picture]
            lock.withLock { waiting.removeValue(forKey: number) }?.resume(returning: .picturesPage(page))
        }

        func report(_: WTEngineEvent) {}

        func navigationVerdict(for _: String, webViewID _: UInt64, mainFrame _: Bool, userGesture _: Bool) -> NavigationVerdict {
            .allow
        }

        func stop() {}
    }

    func testAnAnswerThatComesAfterTheTabNamesAnotherPictureOrNoneIsNotShown() async {
        let core = HeldCore()
        let model = BrowserModel(session: BrowserSession(core: core, engine: WebKitEngine()))
        let (small, large) = (Self.picture(wide: 320, tall: 240), Self.picture(wide: 800, tall: 600))

        // Named, and the page arrives before the picture does: the picture is nobody's.
        model.showPicture(of: tab(picture: 7))
        await settle { core.held == 1 }
        model.showPicture(of: tab(picture: 0))
        core.answer(1, picture: 7, small)
        try? await Task.sleep(for: .milliseconds(50))
        XCTAssertNil(model.pagePicture, "the page was there first")

        // One named and then another, and the first's answer comes: not shown. The second's is.
        model.showPicture(of: tab(picture: 7))
        model.showPicture(of: tab(picture: 9, id: 5))
        await settle { core.held == 2 }
        core.answer(2, picture: 7, small)
        try? await Task.sleep(for: .milliseconds(50))
        XCTAssertNil(model.pagePicture, "an older picture is not the one named now")
        core.answer(3, picture: 9, large)
        await settle { model.pagePicture != nil }
        XCTAssertEqual(model.pagePicture?.width, 800)

        // An answer that carries another picture than was asked for is not shown either.
        model.showPicture(of: tab(picture: 0))
        model.showPicture(of: tab(picture: 11))
        await settle { core.held == 1 }
        core.answer(4, picture: 12, small)
        try? await Task.sleep(for: .milliseconds(50))
        XCTAssertNil(model.pagePicture)
    }

    /// Whether a dot of a drawn view is nearly this colour: the place given as shares of the
    /// view's width and height, from its top left.
    private func isNearly(_ colour: [Int], _ drawn: Picture, across: Double, down: Double) -> Bool {
        let (x, y) = (Int(Double(drawn.width) * across), Int(Double(drawn.height) * down))
        let dot = (y * drawn.width + x) * 4
        return (0 ..< 3).allSatisfy { abs(Int(drawn.pixels[dot + $0]) - colour[$0]) < 40 }
    }

    func testThePictureIsDrawnFromTheTopAsWideAsItsPlaceInItsOwnShape() throws {
        let (red, blue, under) = ([204, 26, 26], [26, 51, 204], [26, 204, 26])
        let picture = try XCTUnwrap(PagePictures.picture(Self.picture(wide: 800, tall: 600)))
        // Over a place that is green, so that what is not the picture shows.
        func drawn(_ size: CGSize) throws -> Picture {
            let view = Color(red: 0.1, green: 0.8, blue: 0.1).overlay { PagePictureView(picture: picture) }
            return try XCTUnwrap(Snapshot.Stage(view, size: size, dark: false).picture())
        }
        // A place of the picture's shape: the picture fills it, its top half above its bottom.
        let same = try drawn(CGSize(width: 400, height: 300))
        XCTAssertTrue(isNearly(red, same, across: 0.5, down: 0.25))
        XCTAssertTrue(isNearly(blue, same, across: 0.5, down: 0.75))
        XCTAssertTrue(isNearly(red, same, across: 0.02, down: 0.02) && isNearly(blue, same, across: 0.98, down: 0.98), "to its corners")
        // A taller place: the picture is as wide, from the top, and what is under shows below it.
        let taller = try drawn(CGSize(width: 400, height: 600))
        XCTAssertTrue(isNearly(red, taller, across: 0.5, down: 0.125))
        XCTAssertTrue(isNearly(blue, taller, across: 0.5, down: 0.375))
        XCTAssertTrue(isNearly(under, taller, across: 0.5, down: 0.75), "the picture is not made taller than its shape")
        // A shorter place: as wide, from the top, and cut off at the place's end.
        let shorter = try drawn(CGSize(width: 400, height: 200))
        XCTAssertTrue(isNearly(red, shorter, across: 0.5, down: 0.5))
        XCTAssertTrue(isNearly(blue, shorter, across: 0.5, down: 0.9))
        // And no further: what is below the place is not drawn over.
        // (The place is the top 200 points of a green stage of 300, drawn over it.)
        let beside = Color(red: 0.1, green: 0.8, blue: 0.1).overlay(alignment: .top) {
            Color.clear.frame(height: 200).overlay { PagePictureView(picture: picture) }
        }
        let below = try XCTUnwrap(Snapshot.Stage(beside, size: CGSize(width: 400, height: 300), dark: false).picture())
        XCTAssertTrue(isNearly(blue, below, across: 0.5, down: 0.6), "the picture's bottom half, at the place's end")
        XCTAssertTrue(isNearly(under, below, across: 0.5, down: 0.8), "the picture ends where its place ends")
        // A wider place than the picture has dots: drawn larger, as wide as the place.
        let wider = try drawn(CGSize(width: 1000, height: 750))
        XCTAssertTrue(isNearly(red, wider, across: 0.98, down: 0.25) && isNearly(blue, wider, across: 0.98, down: 0.75))
    }
}

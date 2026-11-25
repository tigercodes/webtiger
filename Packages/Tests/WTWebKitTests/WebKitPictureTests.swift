// A sleeping tab's picture (059), against the suite's own pages. What WebKit can do, which
// was tried before anything was built on it (§5.1): whether a web view that is in no window,
// as a tab behind is, can be pictured at all; that the picture is of the page and not of
// nothing; how long it takes; and how large it is as a JPEG no wider than 800 pixels. And
// what the adapter answers when the core asks for one: a JPEG of the page, no wider and of
// no more bytes than asked, or none.
import CoreGraphics
import ImageIO
import UniformTypeIdentifiers
import WebKit
import WTContract
@testable import WTWebKit
import XCTest
#if canImport(AppKit)
    import AppKit
#else
    import UIKit
#endif

@MainActor
final class WebKitPictureTests: AdapterTestCase {
    /// The most bytes a picture may be, and the most pixels wide (059 D4).
    private static let mostBytes = 96 * 1024
    private static let mostWide = 800

    /// The share of a picture's pixels that are not the colour of its first: of a page on
    /// white, how much of it is drawn on.
    private func inked(_ image: CGImage) -> Double {
        let (width, height) = (image.width, image.height)
        var pixels = [UInt8](repeating: 0, count: width * height * 4)
        let drawn = pixels.withUnsafeMutableBytes { buffer -> Bool in
            guard let context = CGContext(
                data: buffer.baseAddress, width: width, height: height, bitsPerComponent: 8, bytesPerRow: width * 4,
                space: CGColorSpaceCreateDeviceRGB(), bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
            ) else { return false }
            context.draw(image, in: CGRect(x: 0, y: 0, width: width, height: height))
            return true
        }
        guard drawn, width * height > 0 else { return 0 }
        var other = 0
        for pixel in stride(from: 0, to: pixels.count, by: 4)
            where (0 ..< 3).contains(where: { abs(Int(pixels[pixel + $0]) - Int(pixels[$0])) > 24 }) {
            other += 1
        }
        return Double(other) / Double(width * height)
    }

    /// A picture no wider than so many pixels, and a picture as a JPEG: the adapter's own.
    private func fitted(_ image: CGImage, to most: Int) throws -> CGImage {
        try XCTUnwrap(WebKitPictures.fitted(image, to: most))
    }

    private func jpeg(_ image: CGImage) throws -> Data {
        try XCTUnwrap(WebKitPictures.jpeg(image, quality: 0.6))
    }

    /// A JPEG's bitmap, when the bytes are one.
    private func decoded(_ jpeg: Data) -> CGImage? {
        guard let source = CGImageSourceCreateWithData(jpeg as CFData, nil),
              CGImageSourceGetType(source) == UTType.jpeg.identifier as CFString else { return nil }
        return CGImageSourceCreateImageAtIndex(source, 0, nil)
    }

    /// A web view's picture, as the core asks for one. Nil when the adapter answers with
    /// anything but a picture.
    private func picture(of id: UInt64, mostWide: UInt32 = 800, mostBytes: UInt32 = 96 * 1024) async -> WTWebViewPicture? {
        var asked = WTWebViewSnapshot()
        asked.webViewID = id
        asked.maxWidth = mostWide
        asked.maxBytes = mostBytes
        guard case let .webViewPicture(picture) = await engine.answer(.webViewSnapshot(asked)) else { return nil }
        return picture
    }

    /// A web view of a window's size, in no window, at the suite's page of pictures and words.
    private func atThePictures(_ id: UInt64) async throws -> WKWebView {
        await create(id)
        let view = try XCTUnwrap(engine.webView(for: id))
        view.frame = CGRect(x: 0, y: 0, width: 800, height: 600)
        await load(id, site.url("/pictures"))
        await wait(id, "the page to finish") { self.finished($0) }
        XCTAssertNil(view.window, "in no window, as a tab that is behind is")
        return view
    }

    /// What WebKit hands back for a web view, as a bitmap: at the pixels WebKit chose.
    private func bitmap(of view: WKWebView, wide points: CGFloat) async throws -> CGImage {
        let configuration = WKSnapshotConfiguration()
        configuration.snapshotWidth = NSNumber(value: Double(points))
        let picture = try await view.takeSnapshot(configuration: configuration)
        #if canImport(AppKit)
            return try XCTUnwrap(picture.cgImage(forProposedRect: nil, context: nil, hints: nil))
        #else
            return try XCTUnwrap(picture.cgImage)
        #endif
    }

    func testAWebViewThatIsInNoWindowCanBePicturedAndThePictureIsOfItsPageAndSmall() async throws {
        let view = try await atThePictures(1)

        let began = Date()
        let whole = try await bitmap(of: view, wide: 800)
        let took = Date().timeIntervalSince(began)
        // WebKit's picture is as many pixels to a point as the screen has: three on a phone.
        // What is kept is no wider than 800 pixels.
        let image = try fitted(whole, to: Self.mostWide)
        let (large, small) = try (jpeg(whole), jpeg(image))

        // It is of the page: the page's pictures and words are drawn on it.
        let share = inked(image)
        XCTAssertGreaterThan(share, 0.005, "the picture is of the page, and not of nothing")
        XCTAssertLessThanOrEqual(image.width, Self.mostWide)
        XCTAssertEqual(Double(image.height) / Double(image.width), 0.75, accuracy: 0.01, "the shape the web view has")
        XCTAssertLessThan(small.count, Self.mostBytes / 2, "a page of pictures and words, as a JPEG no wider than 800 pixels")
        print(
            "WebKit's picture of a web view in no window: \(whole.width) by \(whole.height) pixels as taken "
                + "(\(large.count) bytes as a JPEG), \(image.width) by \(image.height) as kept (\(small.count) bytes), "
                + "taken in \(Int(took * 1000)) ms, \(Int(share * 1000)) thousandths drawn on"
        )
    }

    // The adapter

    func testTheAdapterAnswersWithAJPEGOfThePageNoWiderAndOfNoMoreBytesThanAsked() async throws {
        _ = try await atThePictures(1)

        // As the core asks: no wider than 800 pixels, of no more than 96 KB.
        let whole = try await XCTUnwrapAsync(picture(of: 1))
        let image = try XCTUnwrap(decoded(whole.jpeg), "the bytes are a JPEG")
        XCTAssertEqual([image.width, image.height], [Int(whole.width), Int(whole.height)], "and of the size the answer says")
        XCTAssertEqual(whole.width, 800, "as wide as it may be, where the web view is as wide")
        XCTAssertEqual(Double(whole.height) / Double(whole.width), 0.75, accuracy: 0.01, "in the web view's shape")
        XCTAssertLessThanOrEqual(whole.jpeg.count, 96 * 1024)
        XCTAssertGreaterThan(inked(image), 0.005, "of the page, and not of nothing")

        // Asked for a narrower one: as narrow, in the same shape, and of the page still.
        let narrow = try await XCTUnwrapAsync(picture(of: 1, mostWide: 200))
        let small = try XCTUnwrap(decoded(narrow.jpeg))
        XCTAssertEqual([small.width, small.height], [Int(narrow.width), Int(narrow.height)])
        XCTAssertTrue((190 ... 200).contains(narrow.width), "\(narrow.width) pixels wide")
        XCTAssertEqual(Double(narrow.height) / Double(narrow.width), 0.75, accuracy: 0.02)
        XCTAssertGreaterThan(inked(small), 0.005)
        XCTAssertLessThan(narrow.jpeg.count, whole.jpeg.count)

        // Asked for fewer bytes than the first try comes to: a picture all the same, made
        // smaller until it is under them. And for fewer than any picture has: none.
        let tight = try await XCTUnwrapAsync(picture(of: 1, mostBytes: UInt32(whole.jpeg.count - 1)))
        let tighter = try XCTUnwrap(decoded(tight.jpeg), "a picture, under the bytes asked for")
        XCTAssertLessThan(tight.jpeg.count, whole.jpeg.count)
        XCTAssertEqual(tight.width, whole.width, "the next step is as wide, and a poorer JPEG")
        XCTAssertEqual([tighter.width, tighter.height], [Int(tight.width), Int(tight.height)])
        XCTAssertGreaterThan(inked(tighter), 0.005)
        let none = try await XCTUnwrapAsync(picture(of: 1, mostBytes: 200))
        XCTAssertEqual([none.jpeg.count, Int(none.width), Int(none.height)], [0, 0, 0], "none that small: no bytes, and no size")
        for (wide, bytes) in [(UInt32(0), UInt32(96 * 1024)), (800, 0)] {
            let nothing = try await XCTUnwrapAsync(picture(of: 1, mostWide: wide, mostBytes: bytes))
            XCTAssertTrue(nothing.jpeg.isEmpty, "asked for a picture of no width, or of no bytes")
        }

        // The web view is as it was: its state is there to ask for, as the core does next.
        var save = WTWebViewSaveState()
        save.webViewID = 1
        guard case let .webViewState(state) = await engine.answer(.webViewSaveState(save)) else {
            return XCTFail("a state is answered with a state")
        }
        XCTAssertFalse(state.state.isEmpty)
        print(
            "The adapter's picture: \(whole.width) by \(whole.height), \(whole.jpeg.count) bytes; "
                + "under them, \(tight.width) wide, \(tight.jpeg.count)"
        )
    }

    /// A picture of which no two dots are alike: as large a JPEG as its size can be.
    private func noise(_ width: Int, _ height: Int) throws -> CGImage {
        var bytes = [UInt8](repeating: 255, count: width * height * 4)
        var seed: UInt64 = 12345
        for index in bytes.indices where index % 4 != 3 {
            seed = seed &* 6_364_136_223_846_793_005 &+ 1_442_695_040_888_963_407
            bytes[index] = UInt8(truncatingIfNeeded: seed >> 33)
        }
        let image = bytes.withUnsafeMutableBytes { buffer in
            CGContext(
                data: buffer.baseAddress, width: width, height: height, bitsPerComponent: 8, bytesPerRow: width * 4,
                space: CGColorSpaceCreateDeviceRGB(), bitmapInfo: CGImageAlphaInfo.noneSkipLast.rawValue
            )?.makeImage()
        }
        return try XCTUnwrap(image)
    }

    /// A picture that is white but for a dark bar across its top quarter.
    private func barred(_ width: Int, _ height: Int) throws -> CGImage {
        let context = try XCTUnwrap(CGContext(
            data: nil, width: width, height: height, bitsPerComponent: 8, bytesPerRow: 0,
            space: CGColorSpaceCreateDeviceRGB(), bitmapInfo: CGImageAlphaInfo.noneSkipLast.rawValue
        ))
        context.setFillColor(red: 1, green: 1, blue: 1, alpha: 1)
        context.fill(CGRect(x: 0, y: 0, width: width, height: height))
        // Quartz counts up from the bottom: the top quarter is the last.
        context.setFillColor(red: 0.1, green: 0.2, blue: 0.3, alpha: 1)
        context.fill(CGRect(x: 0, y: height - height / 4, width: width, height: height / 4))
        return try XCTUnwrap(context.makeImage())
    }

    /// How dark a dot of a picture is, from 0 for white to 1: counted from the picture's top.
    private func darkness(of image: CGImage, x: Int, y: Int) throws -> Double {
        var pixel = [UInt8](repeating: 0, count: 4)
        let drawn = pixel.withUnsafeMutableBytes { buffer -> Bool in
            guard let context = CGContext(
                data: buffer.baseAddress, width: 1, height: 1, bitsPerComponent: 8, bytesPerRow: 4,
                space: CGColorSpaceCreateDeviceRGB(), bitmapInfo: CGImageAlphaInfo.noneSkipLast.rawValue
            ) else { return false }
            // The one dot wanted is drawn at the context's only place.
            context.draw(image, in: CGRect(x: -x, y: -(image.height - 1 - y), width: image.width, height: image.height))
            return true
        }
        XCTAssertTrue(drawn)
        return 1 - Double(Int(pixel[0]) + Int(pixel[1]) + Int(pixel[2])) / 765
    }

    func testAPictureIsMadeToFitTheWholeOfItInItsShape() throws {
        // A phone's: three dots to a point, 2,400 wide. Made 800 wide, with its bar where it was.
        let large = try barred(2400, 1800)
        let fitted = try XCTUnwrap(WebKitPictures.fitted(large, to: 800))
        XCTAssertEqual([fitted.width, fitted.height], [800, 600])
        XCTAssertGreaterThan(try darkness(of: fitted, x: 400, y: 75), 0.6, "the bar, at the top")
        XCTAssertLessThan(try darkness(of: fitted, x: 400, y: 300), 0.1, "white under it")
        XCTAssertLessThan(try darkness(of: fitted, x: 400, y: 590), 0.1, "to the bottom")
        // One that fits is the one it was; one a dot too wide is made to.
        let same = try XCTUnwrap(WebKitPictures.fitted(fitted, to: 800))
        XCTAssertTrue(same === fitted)
        XCTAssertEqual(try XCTUnwrap(WebKitPictures.fitted(barred(801, 600), to: 800)).width, 800)
        // A tall one keeps its shape; and no picture is of no height.
        let tall = try XCTUnwrap(WebKitPictures.fitted(barred(1080, 2400), to: 800))
        XCTAssertEqual([tall.width, tall.height], [800, 1777])
        XCTAssertEqual(try XCTUnwrap(WebKitPictures.fitted(barred(2000, 1), to: 100)).height, 1)
    }

    func testAPictureOfTooManyBytesIsMadeSmallerAStepAtATimeAndIsNoneWhenNoStepFits() throws {
        let noise = try noise(800, 600)
        // With all the bytes it could want: the first step, the whole width at the best a
        // picture is made.
        let first = try XCTUnwrap(WebKitPictures.smallEnough(noise, mostBytes: .max))
        XCTAssertEqual([first.width, first.height], [800, 600])
        XCTAssertGreaterThan(first.jpeg.count, 96 * 1024, "noise is a large JPEG")
        XCTAssertNotNil(decoded(first.jpeg))
        // Asked for exactly the bytes it comes to: the same.
        XCTAssertEqual(try XCTUnwrap(WebKitPictures.smallEnough(noise, mostBytes: first.jpeg.count)).jpeg, first.jpeg)
        // A byte fewer than each step comes to is the next step: as wide and a poorer JPEG;
        // three quarters as wide; half as wide; half as wide and poorer still.
        var most = first.jpeg.count - 1
        var widths: [UInt32] = []
        for _ in 1 ..< WebKitPictures.steps.count {
            let next = try XCTUnwrap(WebKitPictures.smallEnough(noise, mostBytes: most), "a step fits under \(most) bytes")
            let image = try XCTUnwrap(decoded(next.jpeg))
            XCTAssertLessThanOrEqual(next.jpeg.count, most)
            XCTAssertEqual([image.width, image.height], [Int(next.width), Int(next.height)])
            XCTAssertEqual(Double(next.height) / Double(next.width), 0.75, accuracy: 0.01)
            widths.append(next.width)
            most = next.jpeg.count - 1
        }
        XCTAssertEqual(widths, [800, 600, 400, 400])
        XCTAssertNil(WebKitPictures.smallEnough(noise, mostBytes: most), "no step is as small as \(most) bytes")
        // The steps are the ones Android's adapter goes by.
        XCTAssertEqual(WebKitPictures.steps.map(\.width), [1, 1, 0.75, 0.5, 0.5])
        XCTAssertEqual(WebKitPictures.steps.map(\.quality), [0.6, 0.4, 0.4, 0.4, 0.25])
    }

    // A picture holds nothing up

    func testAPictureIsBegunInItsTurnAndAWebViewLetGoMeanwhileIsDrawnAllTheSame() async throws {
        _ = try await atThePictures(1)
        weak let held = engine.webView(for: 1)
        var asked = WTWebViewSnapshot()
        asked.webViewID = 1
        asked.maxWidth = 800
        asked.maxBytes = 96 * 1024
        // Begun now, of the web view as it is: what comes back is the wait for its answer.
        let later = try XCTUnwrap(engine.begin(.webViewSnapshot(asked)), "a picture is begun, and answered later")

        // As the core does next: the web view is let go. It is the tab's no longer at once.
        await destroy(1)
        XCTAssertNil(engine.webView(for: 1), "the engine has no such web view from now")
        XCTAssertEqual(engine.webViewCount, 0)
        // And what is asked after is answered meanwhile: another web view is made, and loads.
        await create(2)
        await load(2, site.url("/landed"))
        await wait(2, "a page in another web view") { self.finished($0) }

        // The picture comes all the same, and is of the page the web view had.
        guard case let .webViewPicture(picture) = await later() else { return XCTFail("the wait ends in a picture") }
        let image = try XCTUnwrap(decoded(picture.jpeg), "a JPEG of the web view that was let go")
        XCTAssertEqual(picture.width, 800)
        XCTAssertGreaterThan(inked(image), 0.005, "of its page, and not of nothing")
        // And the web view was let go when it was asked to be: nobody held it for its picture.
        for _ in 0 ..< 200 where held != nil {
            try await Task.sleep(for: .milliseconds(10))
        }
        XCTAssertNil(held, "the web view is gone")
    }

    func testOnlyAPictureOfAWebViewThereIsBegunAndEveryOtherRequestIsAnsweredInItsTurn() async throws {
        _ = try await atThePictures(1)
        var asked = WTWebViewSnapshot()
        asked.webViewID = 99
        asked.maxWidth = 800
        asked.maxBytes = 96 * 1024
        XCTAssertNil(engine.begin(.webViewSnapshot(asked)), "a web view that is not there: answered in its turn, with its error")
        var save = WTWebViewSaveState()
        save.webViewID = 1
        XCTAssertNil(engine.begin(.webViewSaveState(save)), "a state is answered in its turn")
        var destroy = WTWebViewDestroy()
        destroy.webViewID = 1
        XCTAssertNil(engine.begin(.webViewDestroy(destroy)))
        XCTAssertNotNil(engine.webView(for: 1), "and nothing was done by asking")
        // A picture begun and waited for, with nothing between: as when it is asked for whole.
        asked.webViewID = 1
        let later = try XCTUnwrap(engine.begin(.webViewSnapshot(asked)))
        guard case let .webViewPicture(picture) = await later() else { return XCTFail("a picture") }
        XCTAssertNotNil(decoded(picture.jpeg))
        XCTAssertNotNil(engine.webView(for: 1), "a web view that was not let go is there still")
    }

    func testTheEngineCountsTheWebViewsItMakes() async {
        let before = engine.made.count
        await create(1)
        XCTAssertEqual(engine.made.count, before + 1)
        await create(2)
        XCTAssertEqual(engine.made.count, before + 2)
        // One that could not be made is not counted: the same number again is refused.
        guard case .error = await create(2) else { return XCTFail("a web view that is there already is refused") }
        XCTAssertEqual(engine.made.count, before + 2)
        await destroy(1)
        XCTAssertEqual(engine.made.count, before + 2, "letting one go makes none")
    }

    func testAWebViewWithNoSizeOrThatIsNotThereHasNoPicture() async throws {
        // As the adapter makes a web view: with no size, until a window gives it one. A tab
        // that was opened behind and never looked at has drawn nothing.
        await create(1)
        await load(1, site.url("/pictures"))
        await wait(1, "the page to finish") { self.finished($0) }
        let unsized = try await XCTUnwrapAsync(picture(of: 1))
        XCTAssertEqual([unsized.jpeg.count, Int(unsized.width), Int(unsized.height)], [0, 0, 0])

        var asked = WTWebViewSnapshot()
        asked.webViewID = 99
        asked.maxWidth = 800
        asked.maxBytes = 96 * 1024
        guard case let .error(error) = await engine.answer(.webViewSnapshot(asked)) else {
            return XCTFail("a web view that is not there has an error for an answer")
        }
        XCTAssertEqual(error.code, "webview.unknown")
    }

    func testAPictureThatDoesNotComeInTimeIsNoneAndWhatComesLateIsNobodys() async throws {
        let view = try await atThePictures(1)
        // No time at all: the wait is over before WebKit can have said anything.
        let late = await WebKitPictures.picture(of: view, mostWide: 800, mostBytes: 96 * 1024, within: .zero)
        XCTAssertTrue(late.jpeg.isEmpty, "none in time")
        // WebKit says its piece afterwards, to nobody; and the next one asked for comes.
        try await Task.sleep(for: .milliseconds(600))
        let next = try await XCTUnwrapAsync(picture(of: 1))
        XCTAssertNotNil(decoded(next.jpeg))
    }
}

/// `XCTUnwrap`, for what was awaited.
private func XCTUnwrapAsync<T>(_ value: T?, file: StaticString = #filePath, line: UInt = #line) async throws -> T {
    try XCTUnwrap(value, file: file, line: line)
}

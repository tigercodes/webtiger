// What WebKit can do for a sleeping tab's picture (059 §5.1), tried against the suite's own
// pages before anything is built on it: whether a web view that is in no window, as a tab
// behind is, can be pictured at all; that the picture is of the page and not of nothing; how
// long it takes; and how large it is as a JPEG no wider than 800 pixels.
import CoreGraphics
import ImageIO
import WebKit
import WTContract
import WTWebKit
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

    /// A picture no wider than so many pixels: itself, or drawn again smaller.
    private func fitted(_ image: CGImage, to most: Int) throws -> CGImage {
        guard image.width > most else { return image }
        let height = max(1, image.height * most / image.width)
        let context = try XCTUnwrap(CGContext(
            data: nil, width: most, height: height, bitsPerComponent: 8, bytesPerRow: 0,
            space: CGColorSpaceCreateDeviceRGB(), bitmapInfo: CGImageAlphaInfo.noneSkipLast.rawValue
        ))
        context.interpolationQuality = .high
        context.draw(image, in: CGRect(x: 0, y: 0, width: most, height: height))
        return try XCTUnwrap(context.makeImage())
    }

    private func jpeg(_ image: CGImage) throws -> Data {
        let data = NSMutableData()
        let destination = try XCTUnwrap(CGImageDestinationCreateWithData(data, "public.jpeg" as CFString, 1, nil))
        CGImageDestinationAddImage(destination, image, [kCGImageDestinationLossyCompressionQuality: 0.6] as CFDictionary)
        XCTAssertTrue(CGImageDestinationFinalize(destination))
        return data as Data
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
        await create(1)
        let view = try XCTUnwrap(engine.webView(for: 1))
        view.frame = CGRect(x: 0, y: 0, width: 800, height: 600)
        await load(1, site.url("/pictures"))
        await wait(1, "the page to finish") { self.finished($0) }
        XCTAssertNil(view.window, "in no window, as a tab that is behind is")

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
}

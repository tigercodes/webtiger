// A web view's picture for the core (059): what WebKit draws of a web view as it is, whether
// or not it is on the screen, made no wider than the core asks and a JPEG of no more bytes
// than it asks. The core asks as a tab is put to sleep, and the picture stands where the
// page will be while the tab wakes. This file decides nothing about tabs: it draws.
import CoreGraphics
import Foundation
import ImageIO
import UniformTypeIdentifiers
import WebKit
import WTContract
#if canImport(AppKit)
    import AppKit
#else
    import UIKit
#endif

@MainActor
enum WebKitPictures {
    /// How long WebKit is waited for. A web view whose page's process is gone may never say:
    /// the core then has no picture, and the web view is let go without one. A web view that
    /// has been out of its window for a while is slow to draw the first time: three quarters
    /// of a second in the app on the iPhone simulator, where one in a test's hands took a
    /// quarter. Three seconds is more than any that was seen.
    static let patience: Duration = .seconds(3)

    /// How a picture is made smaller until it is of no more bytes than were asked for: how
    /// much of its width it keeps, and how good a JPEG it is. A plain page fits at the first;
    /// a page of photographs may need the later ones; one that fits at none has no picture.
    /// (Android's adapter goes by the same steps.)
    static let steps: [(width: Double, quality: Double)] = [(1, 0.6), (1, 0.4), (0.75, 0.4), (0.5, 0.4), (0.5, 0.25)]

    /// The picture of a web view as it is now, as the core asks for one: with no bytes when
    /// there is none to give, or none that small.
    static func picture(of webView: WKWebView, mostWide: Int, mostBytes: Int, within limit: Duration = patience) async -> WTWebViewPicture {
        await begin(of: webView, mostWide: mostWide, mostBytes: mostBytes, within: limit).picture()
    }

    /// Begins a web view's picture, now: WebKit is asked for it before this returns, of the
    /// web view as it is at this moment. What it gives back waits for the picture. The web
    /// view may be let go meanwhile: WebKit draws what it was asked for.
    static func begin(of webView: WKWebView, mostWide: Int, mostBytes: Int, within limit: Duration = patience) -> PictureInFlight {
        let flight = PictureInFlight(mostWide: mostWide, mostBytes: mostBytes)
        let size = webView.bounds.size
        // A web view that was never given a size has drawn nothing.
        guard mostWide > 0, size.width >= 1, size.height >= 1 else {
            flight.end(with: nil)
            return flight
        }
        // WebKit draws as many pixels to a point as the screen has. Asked for fewer points it
        // draws the picture small itself, and no large one is made on the way.
        let configuration = WKSnapshotConfiguration()
        configuration.snapshotWidth = NSNumber(value: Double(min(size.width, CGFloat(mostWide) / scale(of: webView))))
        flight.timer = Task { @MainActor in
            try? await Task.sleep(for: limit)
            flight.end(with: nil)
        }
        webView.takeSnapshot(with: configuration) { picture, _ in
            MainActor.assumeIsolated {
                #if canImport(AppKit)
                    flight.end(with: picture?.cgImage(forProposedRect: nil, context: nil, hints: nil))
                #else
                    flight.end(with: picture?.cgImage)
                #endif
            }
        }
        return flight
    }

    /// What WebKit drew, as the core is answered: made to fit, and small enough.
    fileprivate static func answer(_ drawn: CGImage?, mostWide: Int, mostBytes: Int) -> WTWebViewPicture {
        guard let drawn, let whole = fitted(drawn, to: mostWide) else { return WTWebViewPicture() }
        return smallEnough(whole, mostBytes: mostBytes) ?? WTWebViewPicture()
    }

    /// A picture as a JPEG of no more than so many bytes: as the first of the steps makes it
    /// that comes to no more. Nil when none does.
    static func smallEnough(_ whole: CGImage, mostBytes: Int) -> WTWebViewPicture? {
        for step in steps {
            let wide = max(1, Int((Double(whole.width) * step.width).rounded()))
            guard let image = fitted(whole, to: wide), let jpeg = jpeg(image, quality: step.quality) else { return nil }
            if jpeg.count <= mostBytes {
                var picture = WTWebViewPicture()
                picture.jpeg = jpeg
                picture.width = UInt32(image.width)
                picture.height = UInt32(image.height)
                return picture
            }
        }
        return nil
    }

    /// How many pixels WebKit is taken to draw to a point for this web view. A guess for a
    /// web view that is in no window: whatever WebKit then draws is made to fit afterwards.
    private static func scale(of webView: WKWebView) -> CGFloat {
        #if canImport(AppKit)
            max(1, webView.window?.backingScaleFactor ?? webView.layer?.contentsScale ?? 1)
        #else
            max(1, webView.traitCollection.displayScale)
        #endif
    }

    /// A picture no wider than so many pixels: itself, or drawn again smaller, in its shape.
    static func fitted(_ image: CGImage, to most: Int) -> CGImage? {
        guard image.width > most else { return image }
        let height = max(1, image.height * most / image.width)
        guard let context = CGContext(
            data: nil, width: most, height: height, bitsPerComponent: 8, bytesPerRow: 0,
            space: CGColorSpaceCreateDeviceRGB(), bitmapInfo: CGImageAlphaInfo.noneSkipLast.rawValue
        ) else { return nil }
        context.interpolationQuality = .high
        context.draw(image, in: CGRect(x: 0, y: 0, width: most, height: height))
        return context.makeImage()
    }

    /// A picture as a JPEG. Nothing is written into it but the picture.
    static func jpeg(_ image: CGImage, quality: Double) -> Data? {
        let data = NSMutableData()
        guard let destination = CGImageDestinationCreateWithData(data, UTType.jpeg.identifier as CFString, 1, nil) else { return nil }
        CGImageDestinationAddImage(destination, image, [kCGImageDestinationLossyCompressionQuality: quality] as CFDictionary)
        return CGImageDestinationFinalize(destination) ? data as Data : nil
    }
}

/// A web view's picture that WebKit has been asked for (059): ended once, by the picture or
/// by the time, whichever is first. What the other brings afterwards is nobody's.
@MainActor
final class PictureInFlight {
    private let mostWide: Int
    private let mostBytes: Int
    private var done = false
    private var drawn: CGImage?
    private var waiting: [CheckedContinuation<Void, Never>] = []
    fileprivate var timer: Task<Void, Never>?

    fileprivate init(mostWide: Int, mostBytes: Int) {
        self.mostWide = mostWide
        self.mostBytes = mostBytes
    }

    fileprivate func end(with image: CGImage?) {
        guard !done else { return }
        done = true
        drawn = image
        timer?.cancel()
        for continuation in waiting {
            continuation.resume()
        }
        waiting = []
    }

    /// The picture, when it has come or the time is up: with no bytes when there is none.
    func picture() async -> WTWebViewPicture {
        if !done {
            await withCheckedContinuation { waiting.append($0) }
        }
        return WebKitPictures.answer(drawn, mostWide: mostWide, mostBytes: mostBytes)
    }
}

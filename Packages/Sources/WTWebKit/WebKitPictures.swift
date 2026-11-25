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
    /// the core then has no picture, and the web view is let go without one.
    static let patience: Duration = .seconds(1)

    /// How a picture is made smaller until it is of no more bytes than were asked for: how
    /// much of its width it keeps, and how good a JPEG it is. A plain page fits at the first;
    /// a page of photographs may need the later ones; one that fits at none has no picture.
    /// (Android's adapter goes by the same steps.)
    static let steps: [(width: Double, quality: Double)] = [(1, 0.6), (1, 0.4), (0.75, 0.4), (0.5, 0.4), (0.5, 0.25)]

    /// The picture of a web view as it is now, as the core asks for one: with no bytes when
    /// there is none to give, or none that small.
    static func picture(of webView: WKWebView, mostWide: Int, mostBytes: Int, within limit: Duration = patience) async -> WTWebViewPicture {
        let none = WTWebViewPicture()
        let size = webView.bounds.size
        // A web view that was never given a size has drawn nothing.
        guard mostWide > 0, size.width >= 1, size.height >= 1 else { return none }
        // WebKit draws as many pixels to a point as the screen has. Asked for fewer points it
        // draws the picture small itself, and no large one is made on the way.
        let points = min(size.width, CGFloat(mostWide) / scale(of: webView))
        guard let drawn = await bitmap(of: webView, points: points, within: limit), let whole = fitted(drawn, to: mostWide) else {
            return none
        }
        return smallEnough(whole, mostBytes: mostBytes) ?? none
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

    /// What WebKit draws of the web view, so many points wide; or nothing, when WebKit has
    /// none or does not say within the time.
    private static func bitmap(of webView: WKWebView, points: CGFloat, within limit: Duration) async -> CGImage? {
        let configuration = WKSnapshotConfiguration()
        configuration.snapshotWidth = NSNumber(value: Double(points))
        let waiting = Waiting()
        await withCheckedContinuation { (continuation: CheckedContinuation<Void, Never>) in
            waiting.continuation = continuation
            waiting.timer = Task { @MainActor in
                try? await Task.sleep(for: limit)
                waiting.end(with: nil)
            }
            webView.takeSnapshot(with: configuration) { picture, _ in
                MainActor.assumeIsolated {
                    #if canImport(AppKit)
                        waiting.end(with: picture?.cgImage(forProposedRect: nil, context: nil, hints: nil))
                    #else
                        waiting.end(with: picture?.cgImage)
                    #endif
                }
            }
        }
        return waiting.image
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

/// The wait for WebKit's picture: ended once, by the picture or by the time, whichever is
/// first. What the other brings afterwards is nobody's.
@MainActor
private final class Waiting {
    var continuation: CheckedContinuation<Void, Never>?
    var timer: Task<Void, Never>?
    private(set) var image: CGImage?

    func end(with image: CGImage?) {
        guard let continuation else { return }
        self.continuation = nil
        self.image = image
        timer?.cancel()
        continuation.resume()
    }
}

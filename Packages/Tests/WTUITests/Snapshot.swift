// Snapshot testing in about a hundred lines (004 §4): draw a view into a picture, and compare
// it with the one checked in beside the tests. No library: this is all of it.
import CoreGraphics
import ImageIO
import SwiftUI
import UniformTypeIdentifiers
import XCTest
#if canImport(AppKit)
    import AppKit
#else
    import UIKit
#endif

/// A picture as plain numbers: four bytes a pixel, red first, in sRGB.
struct Picture: Equatable {
    /// A pixel differs when a channel differs by more than this, of 255 (004 §4.1).
    static let channelTolerance = 24
    /// Two pictures match when fewer than this share of their pixels differ.
    static let shareTolerance = 0.01

    let width: Int
    let height: Int
    var pixels: [UInt8]

    init?(_ image: CGImage) {
        width = image.width
        height = image.height
        pixels = [UInt8](repeating: 0, count: width * height * 4)
        guard let space = CGColorSpace(name: CGColorSpace.sRGB),
              let context = CGContext(
                  data: &pixels, width: width, height: height, bitsPerComponent: 8, bytesPerRow: width * 4, space: space,
                  bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
              )
        else { return nil }
        context.draw(image, in: CGRect(x: 0, y: 0, width: width, height: height))
    }

    var image: CGImage? {
        var copy = pixels
        guard let space = CGColorSpace(name: CGColorSpace.sRGB),
              let context = CGContext(
                  data: &copy, width: width, height: height, bitsPerComponent: 8, bytesPerRow: width * 4, space: space,
                  bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
              )
        else { return nil }
        return context.makeImage()
    }

    /// The share of pixels, from 0 to 1, that differ from `other`'s by more than `channel` in
    /// any channel; and a picture of where. 1 when the two are not the same size.
    func difference(from other: Picture, channel: Int = Picture.channelTolerance) -> (share: Double, picture: Picture) {
        guard width == other.width, height == other.height else { return (1, self) }
        var marked = self
        var differing = 0
        for index in stride(from: 0, to: pixels.count, by: 4) {
            let differs = (0 ..< 4).contains { abs(Int(pixels[index + $0]) - Int(other.pixels[index + $0])) > channel }
            if differs {
                differing += 1
                marked.pixels.replaceSubrange(index ..< index + 4, with: [255, 0, 255, 255])
            } else {
                // What is the same is faded, so what differs stands out.
                for offset in 0 ..< 3 {
                    marked.pixels[index + offset] = UInt8(128 + Int(pixels[index + offset]) / 2)
                }
            }
        }
        return (Double(differing) / Double(width * height), marked)
    }
}

@MainActor
enum Snapshot {
    /// The folder for this system's pictures: its platform and its major version.
    static var system: String {
        let version = ProcessInfo.processInfo.operatingSystemVersion.majorVersion
        #if os(macOS)
            return "macOS-\(version)"
        #else
            return "iOS-\(version)"
        #endif
    }

    static var isRecording: Bool {
        ProcessInfo.processInfo.environment["WEBTIGER_RECORD_SNAPSHOTS"] == "1"
    }

    private static func folder(_ file: StaticString) -> URL {
        URL(fileURLWithPath: "\(file)").deletingLastPathComponent().appending(path: "__Snapshots__").appending(path: system)
    }

    /// Whether this system has pictures to compare with. Without any, a suite skips: it does
    /// not pass.
    static func hasPictures(file: StaticString = #filePath) -> Bool {
        isRecording || FileManager.default.fileExists(atPath: folder(file).path(percentEncoded: false))
    }

    /// A view in a window that is never shown, to be drawn when it is ready: a view that waits
    /// for something (a model's first snapshot) is given the time, and then drawn.
    @MainActor
    final class Stage {
        private let size: CGSize
        #if canImport(AppKit)
            private let hosting: NSView
            private let window: NSWindow
        #else
            private let controller: UIViewController
            private let window: UIWindow?
        #endif

        /// Puts `view` on the stage at `size` points, in the light or the dark appearance.
        init(_ view: some View, size: CGSize, dark: Bool) {
            self.size = size
            // Nothing is animated: a picture is of where a view comes to rest, and a window on
            // no screen has no frames for an animation to run through.
            let content = view.frame(width: size.width, height: size.height)
                .environment(\.colorScheme, dark ? .dark : .light)
                .transaction { $0.disablesAnimations = true }
            #if canImport(AppKit)
                let hosting = NSHostingView(rootView: content.background(Color(nsColor: .windowBackgroundColor)))
                hosting.frame = CGRect(origin: .zero, size: size)
                window = NSWindow(contentRect: hosting.frame, styleMask: [.borderless], backing: .buffered, defer: false)
                window.appearance = NSAppearance(named: dark ? .darkAqua : .aqua)
                window.colorSpace = .sRGB
                window.contentView = hosting
                self.hosting = hosting
            #else
                let controller = UIHostingController(rootView: content.background(Color(uiColor: .systemBackground)))
                controller.overrideUserInterfaceStyle = dark ? .dark : .light
                // The picture is the view's own size, with nothing set aside for a status bar
                // that a window on no screen does not have.
                controller.safeAreaRegions = []
                controller.view.frame = CGRect(origin: .zero, size: size)
                // A view draws once it is in a window. The package's tests run in a process with
                // no scene to give a window, so the window has none: it is never on a screen,
                // and it is enough for the view to appear and be drawn.
                let scene = UIApplication.shared.connectedScenes.compactMap { $0 as? UIWindowScene }.first
                let window = scene.map { UIWindow(windowScene: $0) } ?? Self.windowWithoutAScene()
                window?.frame = controller.view.frame
                window?.rootViewController = controller
                window?.isHidden = false
                self.window = window
                self.controller = controller
            #endif
        }

        #if canImport(UIKit)
            /// A window that belongs to no scene. UIKit has deprecated making one, because such
            /// a window is never shown; that is what a test wants of it. It is made by its
            /// class's name so that the deprecation, a warning and so an error here, is not
            /// tripped by the one place that means it.
            private static func windowWithoutAScene() -> UIWindow? {
                (NSClassFromString("UIWindow") as? NSObject.Type)?.init() as? UIWindow
            }
        #endif

        /// What the stage shows now, at twice as many pixels as points. This draws what the app
        /// draws, text fields included, which SwiftUI's own `ImageRenderer` does not.
        func picture() -> Picture? {
            #if canImport(AppKit)
                hosting.layoutSubtreeIfNeeded()
                guard let bitmap = NSBitmapImageRep(
                    bitmapDataPlanes: nil, pixelsWide: Int(size.width) * 2, pixelsHigh: Int(size.height) * 2, bitsPerSample: 8,
                    samplesPerPixel: 4, hasAlpha: true, isPlanar: false, colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0
                ) else { return nil }
                bitmap.size = size
                hosting.cacheDisplay(in: hosting.bounds, to: bitmap)
                return bitmap.cgImage.flatMap(Picture.init)
            #else
                window?.layoutIfNeeded()
                let format = UIGraphicsImageRendererFormat()
                format.scale = 2
                format.preferredRange = .standard
                let image = UIGraphicsImageRenderer(size: size, format: format).image { context in
                    controller.view.layer.render(in: context.cgContext)
                }
                return image.cgImage.flatMap(Picture.init)
            #endif
        }

        /// The picture once it has stopped changing: drawn until two in a row are the same,
        /// for at most three seconds.
        func settledPicture() async -> Picture? {
            var last = picture()
            for _ in 0 ..< 30 {
                try? await Task.sleep(for: .milliseconds(100))
                let next = picture()
                if next == last {
                    return next
                }
                last = next
            }
            return last
        }
    }

    /// Compares a picture with the one checked in as `name`, or writes it when recording. A
    /// picture that is not there yet is written, and the test fails: a new one is looked at
    /// before it is trusted.
    static func verify(_ picture: Picture?, named name: String, file: StaticString = #filePath, line: UInt = #line) {
        guard let picture else { return XCTFail("\(name) could not be drawn", file: file, line: line) }
        let folder = folder(file)
        let reference = folder.appending(path: "\(name).png")
        let known = load(reference)
        if isRecording || known == nil {
            try? FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
            write(picture, to: reference)
            if !isRecording {
                XCTFail(
                    "\(name) had no picture; one was written to \(system)/\(name).png: look at it, then commit it",
                    file: file,
                    line: line
                )
            }
            return
        }
        guard let known else { return }
        let (share, marked) = picture.difference(from: known)
        guard share >= Picture.shareTolerance else { return }
        write(picture, to: folder.appending(path: "\(name).actual.png"))
        write(marked, to: folder.appending(path: "\(name).diff.png"))
        let percent = String(format: "%.2f", share * 100)
        XCTFail(
            "\(name) is not the picture checked in: \(percent) % of its pixels differ (see \(system)/\(name).actual.png and .diff.png)",
            file: file, line: line
        )
    }

    private static func load(_ url: URL) -> Picture? {
        guard let source = CGImageSourceCreateWithURL(url as CFURL, nil),
              let image = CGImageSourceCreateImageAtIndex(source, 0, nil)
        else { return nil }
        return Picture(image)
    }

    private static func write(_ picture: Picture, to url: URL) {
        guard let image = picture.image,
              let destination = CGImageDestinationCreateWithURL(url as CFURL, UTType.png.identifier as CFString, 1, nil)
        else { return }
        CGImageDestinationAddImage(destination, image, nil)
        CGImageDestinationFinalize(destination)
    }
}

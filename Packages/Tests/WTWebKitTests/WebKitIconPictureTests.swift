// What a server sends as an icon, made the picture the core is handed (046 §5a); and which
// addresses a fetch may ask. Neither needs a web view or a site: the pictures are made here.
import ImageIO
import UniformTypeIdentifiers
@testable import WTWebKit
import XCTest

/// Pictures a test makes for itself.
enum TestPictures {
    /// A picture of one colour, or of noise in its colours and in how far it is seen
    /// through, which no PNG makes small.
    static func image(wide: Int, tall: Int, noise: Bool = false) -> CGImage? {
        guard noise else {
            let context = CGContext(
                data: nil, width: wide, height: tall, bitsPerComponent: 8, bytesPerRow: wide * 4,
                space: CGColorSpaceCreateDeviceRGB(), bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
            )
            context?.setFillColor(red: 0.1, green: 0.4, blue: 0.9, alpha: 1)
            context?.fill(CGRect(x: 0, y: 0, width: wide, height: tall))
            return context?.makeImage()
        }
        var seed: UInt32 = 12345
        let bytes = (0 ..< wide * tall * 4).map { _ in
            seed = seed &* 1_664_525 &+ 1_013_904_223
            return UInt8(truncatingIfNeeded: seed >> 16)
        }
        return CGDataProvider(data: Data(bytes) as CFData).flatMap {
            CGImage(
                width: wide, height: tall, bitsPerComponent: 8, bitsPerPixel: 32, bytesPerRow: wide * 4,
                space: CGColorSpaceCreateDeviceRGB(), bitmapInfo: CGBitmapInfo(rawValue: CGImageAlphaInfo.last.rawValue),
                provider: $0, decode: nil, shouldInterpolate: false, intent: .defaultIntent
            )
        }
    }

    /// A picture as a file of a kind the system writes.
    static func file(wide: Int, tall: Int, as kind: UTType, noise: Bool = false) -> Data {
        let written = NSMutableData()
        guard let image = image(wide: wide, tall: tall, noise: noise),
              let destination = CGImageDestinationCreateWithData(written, kind.identifier as CFString, 1, nil)
        else { return Data() }
        CGImageDestinationAddImage(destination, image, nil)
        return CGImageDestinationFinalize(destination) ? written as Data : Data()
    }

    /// An `.ico` of square pictures of these sizes, each kept in it as a PNG.
    static func ico(sides: [Int]) -> Data {
        let pictures = sides.map { file(wide: $0, tall: $0, as: .png) }
        var written = Data([0, 0, 1, 0, UInt8(sides.count), 0])
        var place = 6 + 16 * sides.count
        for (side, picture) in zip(sides, pictures) {
            written.append(contentsOf: [UInt8(side % 256), UInt8(side % 256), 0, 0, 1, 0, 32, 0])
            written.append(contentsOf: little(picture.count))
            written.append(contentsOf: little(place))
            place += picture.count
        }
        pictures.forEach { written.append($0) }
        return written
    }

    private static func little(_ number: Int) -> [UInt8] {
        (0 ..< 4).map { UInt8((number >> ($0 * 8)) & 0xFF) }
    }

    /// How wide and how tall a picture's file says it is; and what kind of file it is.
    static func size(of file: Data?) -> [Int] {
        guard let file, let source = CGImageSourceCreateWithData(file as CFData, nil),
              let said = CGImageSourceCopyPropertiesAtIndex(source, 0, nil) as? [CFString: Any]
        else { return [] }
        return [said[kCGImagePropertyPixelWidth] as? Int ?? 0, said[kCGImagePropertyPixelHeight] as? Int ?? 0]
    }
}

final class WebKitIconPictureTests: XCTestCase {
    func testAPictureIsKnownByItsOwnFirstBytes() {
        XCTAssertEqual(IconPicture.kind(of: TestPictures.file(wide: 8, tall: 8, as: .png)), .png)
        XCTAssertEqual(IconPicture.kind(of: TestPictures.file(wide: 8, tall: 8, as: .jpeg)), .jpeg)
        XCTAssertEqual(IconPicture.kind(of: TestPictures.file(wide: 8, tall: 8, as: .gif)), .gif)
        XCTAssertEqual(IconPicture.kind(of: TestPictures.ico(sides: [16])), .ico)
        // What is none of the four: words, a drawing in SVG, kinds that are not decoded, nothing.
        XCTAssertNil(IconPicture.kind(of: Data("<!doctype html><title>Not found</title>".utf8)))
        XCTAssertNil(IconPicture.kind(of: Data("<svg xmlns=\"http://www.w3.org/2000/svg\"/>".utf8)))
        XCTAssertNil(IconPicture.kind(of: Data("RIFF\u{0}\u{0}\u{0}\u{0}WEBPVP8 ".utf8)))
        XCTAssertNil(IconPicture.kind(of: TestPictures.file(wide: 8, tall: 8, as: .tiff)))
        XCTAssertNil(IconPicture.kind(of: Data([0x89, 0x50, 0x4E])))
        XCTAssertNil(IconPicture.kind(of: Data()))
    }

    func testEachOfTheFourKindsIsHandedOverAsASmallPNGInItsOwnShape() {
        // A small picture is handed over at its own size.
        let small = IconPicture.png(from: TestPictures.file(wide: 32, tall: 32, as: .png))
        XCTAssertEqual(small.flatMap(IconPicture.kind(of:)), .png)
        XCTAssertEqual(TestPictures.size(of: small), [32, 32])
        // A large one is made 128 a side at most, in its own shape; whatever kind it came as,
        // it leaves as a PNG.
        let photograph = IconPicture.png(from: TestPictures.file(wide: 512, tall: 256, as: .jpeg))
        XCTAssertEqual(photograph.flatMap(IconPicture.kind(of:)), .png)
        XCTAssertEqual(TestPictures.size(of: photograph), [128, 64])
        let moving = IconPicture.png(from: TestPictures.file(wide: 16, tall: 16, as: .gif))
        XCTAssertEqual(moving.flatMap(IconPicture.kind(of:)), .png)
        XCTAssertEqual(TestPictures.size(of: moving), [16, 16])
        // An .ico holds several sizes: the largest is the one taken, wherever it is in the file.
        let several = IconPicture.png(from: TestPictures.ico(sides: [16, 48, 32]))
        XCTAssertEqual(several.flatMap(IconPicture.kind(of:)), .png)
        XCTAssertEqual(TestPictures.size(of: several), [48, 48])
    }

    func testWhatIsNoPictureOrTooLargeAPictureIsNothing() {
        XCTAssertNil(IconPicture.png(from: Data("<!doctype html><title>Not found</title>".utf8)))
        XCTAssertNil(IconPicture.png(from: TestPictures.file(wide: 8, tall: 8, as: .tiff)), "a kind that is not decoded")
        // It begins as a PNG and is not one.
        XCTAssertNil(IconPicture.png(from: Data([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] + Array(repeating: UInt8(7), count: 64))))
        XCTAssertNil(IconPicture.png(from: Data([0, 0, 1, 0, 1, 0] + Array(repeating: UInt8(7), count: 64))))
        // A picture wider or taller than is decoded at all; and one just within.
        XCTAssertNil(IconPicture.png(from: TestPictures.file(wide: IconPicture.mostPixels + 1, tall: 4, as: .png)))
        XCTAssertNil(IconPicture.png(from: TestPictures.file(wide: 4, tall: IconPicture.mostPixels + 1, as: .png)))
        XCTAssertEqual(
            TestPictures.size(of: IconPicture.png(from: TestPictures.file(wide: IconPicture.mostPixels, tall: 4, as: .png))).first,
            128
        )
        // An .ico whose only picture is too large has none to take: the system's decoder
        // refuses the file. With a smaller one before it, that one is taken.
        XCTAssertNil(IconPicture.png(from: TestPictures.ico(sides: [IconPicture.mostPixels + 8])))
        XCTAssertEqual(TestPictures.size(of: IconPicture.png(from: TestPictures.ico(sides: [32, IconPicture.mostPixels + 8]))), [32, 32])
    }

    func testAPictureWhosePNGIsMoreThanTheCoreKeepsIsMadeSmallerOrIsNothing() throws {
        // Noise, at the largest size that is handed over as it is: a PNG of it is four bytes
        // for each of its pixels, and more.
        let noise = TestPictures.file(wide: 128, tall: 128, as: .png, noise: true)
        let whole = try XCTUnwrap(IconPicture.png(from: noise, mostBytes: .max))
        XCTAssertEqual(TestPictures.size(of: whole), [128, 128])
        XCTAssertGreaterThan(whole.count, IconPicture.mostBytes, "noise at 128 a side is more than the core keeps")
        // So it is handed over smaller: as the largest of the sizes tried that is few enough bytes.
        let kept = try XCTUnwrap(IconPicture.png(from: noise))
        XCTAssertEqual(TestPictures.size(of: kept), [96, 96])
        XCTAssertLessThanOrEqual(kept.count, IconPicture.mostBytes)
        // Exactly as many bytes as are kept is few enough; one fewer allowed, and it is smaller still.
        XCTAssertEqual(IconPicture.png(from: noise, mostBytes: kept.count), kept)
        XCTAssertEqual(TestPictures.size(of: IconPicture.png(from: noise, mostBytes: kept.count - 1)), [64, 64])
        // With no size few enough, nothing.
        XCTAssertNil(IconPicture.png(from: noise, mostBytes: 100))
    }

    private func asked(_ address: String, _ reach: WebKitEngine.IconFetching) -> Bool {
        URL(string: address).map { IconFetch.allows($0, reach) } ?? false
    }

    func testOnlyThisDeviceAndHTTPSAreAskedAndNeverAsSomebody() {
        let own = [
            "http://127.0.0.1:8080/icon.png", "http://localhost/icon.png", "http://dev.localhost:3000/icon.png",
            "http://[::1]:8080/icon.png", "http://127.9.9.9/icon.png", "https://localhost/icon.png",
        ]
        let secure = ["https://example.com/icon.png", "HTTPS://Example.com/icon.png", "https://files.example.net:8443/a/b.ico"]
        let never = [
            "http://example.com/icon.png", "http://192.168.1.10/icon.png", "http://10.0.0.1/icon.png",
            "http://localhost.example.com/icon.png", "http://notlocalhost/icon.png", "http://127.0.0.1.example.com/icon.png",
            "http://1270.0.0.1/icon.png", "http://127.0.0.256/icon.png", "http://127.0.0.1.5/icon.png", "http://[2001:db8::1]/icon.png",
            "ftp://example.com/icon.png", "file:///tmp/icon.png", "data:image/png;base64,AAAA", "about:blank",
            "javascript:alert(1)", "https:///icon.png",
            // An address that names somebody: a fetch has no login.
            "https://ada:secret@example.com/icon.png", "https://ada@example.com/icon.png", "http://ada:secret@127.0.0.1/icon.png",
        ]
        for address in own {
            XCTAssertTrue(asked(address, .web), address)
            XCTAssertTrue(asked(address, .thisDevice), address)
            XCTAssertFalse(asked(address, .never), address)
        }
        for address in secure {
            XCTAssertTrue(asked(address, .web), address)
            XCTAssertFalse(asked(address, .thisDevice), "\(address): a suite's engine asks nothing beyond this device")
            XCTAssertFalse(asked(address, .never), address)
        }
        for address in never {
            XCTAssertFalse(asked(address, .web), address)
            XCTAssertFalse(asked(address, .thisDevice), address)
            XCTAssertFalse(asked(address, .never), address)
        }
    }
}

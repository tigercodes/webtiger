// A sleeping tab's picture (059), the window's part: the picture the core says stands where
// the page of the tab in front will be, asked for by its number and drawn there until the
// core says none. When there is one, and for how long, are the core's: nothing here decides.
import CoreGraphics
import ImageIO
import SwiftUI
import WTContract

/// The pictures of pages as they were.
enum PagePictures {
    /// The widest a picture the core keeps is, in pixels; and the tallest this window draws.
    static let mostWide = 800
    static let mostTall = 8192

    /// The picture the core's bytes are, to be drawn. Nil for bytes that are no JPEG, and for
    /// a picture wider than the core keeps or taller than any window: its size is read before
    /// any of it is decoded.
    static func picture(_ jpeg: Data) -> CGImage? {
        // Only bytes that begin as a JPEG does are handed to the system at all: it reads many
        // kinds of picture, and none of the others is asked of it here.
        guard jpeg.starts(with: [0xFF, 0xD8, 0xFF]),
              let source = CGImageSourceCreateWithData(jpeg as CFData, [kCGImageSourceTypeIdentifierHint: "public.jpeg"] as CFDictionary),
              let said = CGImageSourceCopyPropertiesAtIndex(source, 0, nil) as? [CFString: Any],
              let wide = said[kCGImagePropertyPixelWidth] as? Int, let tall = said[kCGImagePropertyPixelHeight] as? Int,
              (1 ... mostWide).contains(wide), (1 ... mostTall).contains(tall)
        else { return nil }
        return CGImageSourceCreateImageAtIndex(source, 0, nil)
    }
}

extension BrowserModel {
    /// Shows the picture the tab in front says stands where its page will be: asked of the
    /// core once for each number, and shown when it comes if the tab says that number still.
    /// When the tab says none there is none, at once.
    func showPicture(of tab: WTTabView) {
        let wanted = tab.pictureID
        guard wanted != pictureWanted else { return }
        pictureWanted = wanted
        pagePicture = nil
        guard wanted != 0 else { return }
        var query = WTQueryPictures()
        query.pictureIds = [wanted]
        let answer = session.core.send(.queryPictures(query))
        Task { [weak self] in
            guard case let .picturesPage(page) = try? await answer.value, let self, pictureWanted == wanted,
                  let jpeg = page.pictures.first(where: { $0.pictureID == wanted })?.jpeg else { return }
            pagePicture = PagePictures.picture(jpeg)
        }
    }
}

/// A page as it was, drawn where the page will be (059): as wide as its place, from the top,
/// in its own shape, and cut off where its place ends. A place that is taller than the
/// picture shows what is under it below. It is a picture and nothing else: a press goes
/// through it, and a screen reader is told what it is.
struct PagePictureView: View {
    let picture: CGImage

    var body: some View {
        GeometryReader { place in
            Image(decorative: picture, scale: 1)
                .resizable()
                .interpolation(.high)
                .frame(width: place.size.width, height: place.size.width * CGFloat(picture.height) / CGFloat(max(picture.width, 1)))
        }
        .clipped()
        .allowsHitTesting(false)
        .accessibilityElement()
        .accessibilityLabel(Text(Words.pagePicture))
        .accessibilityAddTraits(.isImage)
        .accessibilityIdentifier("page-picture")
    }
}

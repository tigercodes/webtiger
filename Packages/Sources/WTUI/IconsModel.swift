// Sites' icons (046), the window's part: the pictures of the numbers the core gives tabs'
// icons, asked for once and kept while the window is open. Which tab has which icon, and
// whether a picture is an icon at all, are the core's.
import CoreGraphics
import ImageIO
import SwiftUI
import WTContract

/// The pictures of a window's icons.
enum IconPictures {
    /// The most pictures asked for in one question: what an answer carries.
    static let asked = 64
    /// The most pictures a window keeps beyond those its tabs show now.
    static let kept = 512
    /// The widest and the tallest a picture the core keeps is, in pixels.
    static let pixels = 256

    /// The picture the core's bytes are, to be drawn. Nil for bytes that are no PNG, and for
    /// a picture larger than the core keeps: its size is read before any of it is decoded.
    static func picture(_ png: Data) -> CGImage? {
        guard png.starts(with: [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]),
              let source = CGImageSourceCreateWithData(png as CFData, [kCGImageSourceTypeIdentifierHint: "public.png"] as CFDictionary),
              let said = CGImageSourceCopyPropertiesAtIndex(source, 0, nil) as? [CFString: Any],
              let wide = said[kCGImagePropertyPixelWidth] as? Int, let tall = said[kCGImagePropertyPixelHeight] as? Int,
              (1 ... pixels).contains(wide), (1 ... pixels).contains(tall)
        else { return nil }
        return CGImageSourceCreateImageAtIndex(source, 0, nil)
    }
}

extension BrowserModel {
    /// The picture of a tab's icon, when the window has it.
    func icon(of tab: WTTabSummary) -> CGImage? {
        icons[tab.iconID]
    }

    /// Asks the core for the pictures of the tabs' icons that this window has not got: once
    /// for each number, and no more to a question than an answer carries. A window that has
    /// kept many keeps only those its tabs show now.
    func askForIcons(_ tabs: [WTTabSummary]) {
        let shown = Set(tabs.map(\.iconID))
        if icons.count > IconPictures.kept {
            icons = icons.filter { shown.contains($0.key) }
            iconsAsked.formIntersection(shown)
        }
        let wanted = shown.filter { $0 != 0 && iconsAsked.insert($0).inserted }.sorted()
        for first in stride(from: 0, to: wanted.count, by: IconPictures.asked) {
            var query = WTQueryIcons()
            query.iconIds = Array(wanted[first ..< min(first + IconPictures.asked, wanted.count)])
            let answer = session.core.send(.queryIcons(query))
            Task { [weak self] in
                guard case let .iconsPage(page) = try? await answer.value, let self else { return }
                let pictures = page.icons.compactMap { icon in IconPictures.picture(icon.png).map { (icon.iconID, $0) } }
                if !pictures.isEmpty {
                    icons.merge(pictures) { _, new in new }
                }
            }
        }
    }
}

/// A site's icon as it is drawn: before a tab's name, or as a Favourite's tile. It says
/// nothing to a screen reader: the tab's name is said, and the icon adds no word to it.
struct SiteIcon: View {
    /// How large it is before a tab's name, and as a tile, in points.
    static let inARow: CGFloat = 16
    static let onATile: CGFloat = 24

    let picture: CGImage
    let side: CGFloat
    /// Asleep, a tab has no page: its icon is fainter, as its name is.
    let asleep: Bool

    var body: some View {
        Image(decorative: picture, scale: 1)
            .resizable()
            .interpolation(.high)
            .scaledToFit()
            .frame(width: side, height: side)
            .opacity(asleep ? 0.5 : 1)
            .accessibilityHidden(true)
    }
}

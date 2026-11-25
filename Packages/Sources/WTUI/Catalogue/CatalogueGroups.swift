// Groups (041) in the design catalogue: the row of a window that has them, and the page that
// makes a Group or edits one.
import ImageIO
import SwiftUI
import WTContract

enum CatalogueGroups: String, CaseIterable, Identifiable {
    case new
    case editRefused = "edit-refused"

    var id: String {
        rawValue
    }

    /// The state's name in the catalogue.
    var title: Word {
        switch self {
        case .new: Words.catalogueStateGroupNew
        case .editRefused: Words.catalogueStateGroupEdit
        }
    }

    /// What a core would list: a Group with a colour, open; and one with none, shut, that
    /// holds three Sites.
    static let groups: [WTGroupSummary] = [
        group(30, "Work", colour: "blue", sites: 1),
        group(31, "House", shut: true, sites: 3),
    ]

    /// The row beside them, as a core would list it: the open Group's Site, which is in
    /// front; none of the shut Group's; and a tab.
    static func row(front: WTTabSummary) -> [WTTabSummary] {
        var first = front
        first.groupID = 30
        return [first, tab(22, "News", "news.example.com")]
    }

    /// A row with Favourites (042), as a core would list it: the one in front, one asleep,
    /// a Site that is none, and a tab.
    static func favourites(front: WTTabSummary) -> [WTTabSummary] {
        var calendar = tab(23, "Calendar", "calendar.example.com")
        calendar.kept = true
        calendar.favourite = true
        calendar.asleep = true
        calendar.siteName = "Calendar"
        var wiki = tab(24, "Wiki", "wiki.example.com")
        wiki.kept = true
        wiki.siteName = "Wiki"
        wiki.iconID = CatalogueIcons.blue
        // The one in front has its icon (046), and is a tile of it; the one asleep has none,
        // and is a letter.
        var first = front
        first.iconID = CatalogueIcons.orange
        return [first, calendar, wiki, tab(22, "News", "news.example.com")]
    }

    /// A row of every kind of thing, for the sidebar (043), as a core would list it: two
    /// Favourites, one asleep; a Site in no Group; the open Group's two, the one in front
    /// and one asleep; none of the shut Group's; and two tabs, one still loading.
    static func sidebar(front: WTTabSummary) -> [WTTabSummary] {
        var docs = tab(25, "Docs", "docs.example.com")
        docs.kept = true
        docs.siteName = "Docs"
        docs.iconID = CatalogueIcons.blue
        var first = front
        first.groupID = 30
        var board = tab(21, "Board", "board.example.com")
        board.kept = true
        board.asleep = true
        board.groupID = 30
        board.iconID = CatalogueIcons.green
        var loading = tab(26, "", "example.org")
        loading.loading = true
        var mail = tab(20, "Mail", "mail.example.com")
        mail.kept = true
        mail.favourite = true
        mail.siteName = "Mail"
        mail.iconID = CatalogueIcons.orange
        let calendar = favourites(front: mail)[1]
        var news = tab(22, "News", "news.example.com")
        news.iconID = CatalogueIcons.green
        return [mail, calendar, docs, first, board, news, loading]
    }

    private static func group(_ id: UInt64, _ name: String, colour: String = "", shut: Bool = false, sites: UInt32 = 0) -> WTGroupSummary {
        var group = WTGroupSummary()
        group.groupID = id
        group.name = name
        group.colour = colour
        group.shut = shut
        group.sites = sites
        return group
    }

    private static func tab(_ id: UInt64, _ title: String, _ host: String) -> WTTabSummary {
        var tab = WTTabSummary()
        tab.tabID = id
        tab.title = title
        tab.host = host
        return tab
    }
}

/// The page of a Group in one state: making one, with a name typed; or editing one that has
/// a colour, which the core would not change.
struct CatalogueGroupPage: View {
    let state: CatalogueGroups

    var body: some View {
        GroupPage(
            asked: state == .new ? .new(tab: 2) : .edit(CatalogueGroups.groups[0]),
            colours: CatalogueSpaces.colours,
            refusal: state == .editRefused ? "group.unknown" : nil,
            name: state == .new ? "Reading" : "Work",
            done: { _, _ in }, cancel: {}
        )
    }
}

/// The phone's switcher (047), over the row the sidebar's state has: every kind of thing a
/// row holds.
struct CatalogueSwitcher: View {
    @State private var model = BrowserModel(session: CatalogueState.sidebar.session())

    var body: some View {
        Switcher(model: model, onDone: {}, onSpaces: {})
            .task { await model.open() }
    }
}

/// The icons the catalogue's sites have (046): each a plain shape of one colour, made here, as
/// a core would hand a picture over.
enum CatalogueIcons {
    static let orange: UInt64 = 1
    static let blue: UInt64 = 2
    static let green: UInt64 = 3

    /// The picture of a number, as a PNG; nil for a number that is none of these.
    static func png(_ number: UInt64) -> Data? {
        let colour: [CGFloat]
        switch number {
        case orange: colour = [0xE0, 0x6A, 0x10]
        case blue: colour = [0x10, 0x5A, 0xE0]
        case green: colour = [0x2E, 0x9E, 0x55]
        default: return nil
        }
        let context = CGContext(
            data: nil, width: 64, height: 64, bitsPerComponent: 8, bytesPerRow: 256,
            space: CGColorSpaceCreateDeviceRGB(), bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
        )
        context?.setFillColor(red: colour[0] / 255, green: colour[1] / 255, blue: colour[2] / 255, alpha: 1)
        context?.addPath(CGPath(roundedRect: CGRect(x: 4, y: 4, width: 56, height: 56), cornerWidth: 14, cornerHeight: 14, transform: nil))
        context?.fillPath()
        let written = NSMutableData()
        guard let image = context?.makeImage(),
              let destination = CGImageDestinationCreateWithData(written, "public.png" as CFString, 1, nil)
        else { return nil }
        CGImageDestinationAddImage(destination, image, nil)
        return CGImageDestinationFinalize(destination) ? written as Data : nil
    }

    /// What a core answers when it is asked for these numbers' pictures.
    static func page(_ numbers: [UInt64]) -> WTIconsPage {
        var page = WTIconsPage()
        page.icons = numbers.compactMap { number in
            png(number).map { picture in
                var icon = WTIcon()
                icon.iconID = number
                icon.png = picture
                return icon
            }
        }
        return page
    }
}

/// The picture of a page as it was (059), which the catalogue's waking tab is shown with: a
/// page of sorts, made here, as a core would hand a picture over.
enum CataloguePictures {
    static let page: UInt64 = 7

    /// A page of sorts, 800 by 600: a heading's bar, lines of words as bars, and a picture's
    /// block; as a JPEG.
    static func jpeg() -> Data? {
        let (wide, tall) = (800, 600)
        guard let context = CGContext(
            data: nil, width: wide, height: tall, bitsPerComponent: 8, bytesPerRow: 0,
            space: CGColorSpaceCreateDeviceRGB(), bitmapInfo: CGImageAlphaInfo.noneSkipLast.rawValue
        ) else { return nil }
        context.setFillColor(red: 1, green: 1, blue: 1, alpha: 1)
        context.fill(CGRect(x: 0, y: 0, width: wide, height: tall))
        // Quartz counts up from the bottom: `down` is how far from the top a bar begins.
        func bar(_ down: Int, _ left: Int, _ width: Int, _ height: Int, _ grey: CGFloat) {
            context.setFillColor(red: grey, green: grey, blue: grey, alpha: 1)
            context.fill(CGRect(x: left, y: tall - down - height, width: width, height: height))
        }
        bar(48, 60, 420, 36, 0.15)
        for (line, width) in [680, 640, 660, 420].enumerated() {
            bar(120 + line * 30, 60, width, 12, 0.55)
        }
        context.setFillColor(red: 0.88, green: 0.42, blue: 0.06, alpha: 1)
        context.fill(CGRect(x: 60, y: tall - 270 - 200, width: 320, height: 200))
        for (line, width) in [300, 280, 300, 180].enumerated() {
            bar(280 + line * 30, 420, width, 12, 0.55)
        }
        let written = NSMutableData()
        guard let image = context.makeImage(),
              let destination = CGImageDestinationCreateWithData(written, "public.jpeg" as CFString, 1, nil)
        else { return nil }
        CGImageDestinationAddImage(destination, image, [kCGImageDestinationLossyCompressionQuality: 0.8] as CFDictionary)
        return CGImageDestinationFinalize(destination) ? written as Data : nil
    }

    /// What a core answers when asked for pictures: the one there is, when it is asked for.
    static func answer(_ numbers: [UInt64]) -> WTPicturesPage {
        var answer = WTPicturesPage()
        if numbers.contains(page), let jpeg = jpeg() {
            var picture = WTPagePicture()
            picture.pictureID = page
            picture.jpeg = jpeg
            picture.width = 800
            picture.height = 600
            answer.pictures = [picture]
        }
        return answer
    }
}

// The list of Spaces (024) in the design catalogue: the list, the page that makes one with
// logins of its own or another Space's, and a Space the core would not make.
import SwiftUI
import WTContract

enum CatalogueSpaces: String, CaseIterable, Identifiable {
    case list
    case new
    case newShared = "new-shared"
    case refused
    case rename

    var id: String {
        rawValue
    }

    /// The state's name in the catalogue.
    var title: Word {
        switch self {
        case .list: Words.catalogueStateSpaces
        case .new: Words.catalogueStateSpacesNew
        case .newShared: Words.catalogueStateSpacesNewShared
        case .refused: Words.catalogueStateSpacesRefused
        case .rename: Words.catalogueStateSpacesRename
        }
    }

    /// What a core would list: the app's own Space, which nobody has named; "Work", with logins
    /// of its own; and two that share Work's, one with a name as long as a name can be.
    static let spaces: [WTSpaceSummary] = [
        space(1, "", logins: 1, colour: ""),
        space(2, "Work", logins: 2, colour: "blue", tidyAfter: 86400, icon: "\u{1F4BC}"),
        space(3, "Project Tiger", logins: 2, colour: "green", icon: "\u{1F331}"),
        space(4, "A Space with a name that is as long as a name is let be", logins: 4, colour: "pink"),
    ]

    /// The colours a core lists (025), in its order.
    static let colours = ["red", "orange", "yellow", "green", "teal", "blue", "purple", "pink", "grey"]

    /// The times a core lists (049), in its order: never, twelve hours, a day, a week, thirty
    /// days.
    static let tidyTimes: [UInt32] = [0, 43200, 86400, 604_800, 2_592_000]

    /// The icons a core lists (050), in its order, by their code points: house, briefcase,
    /// books, backpack, shopping bags, money bag, aeroplane, puzzle piece, guitar, popcorn,
    /// newspaper, speech balloon, hammer and spanner, test tube, seedling, heart, star, lock,
    /// globe, camera, pizza, running shoe, dog, light bulb.
    static let icons = [
        "\u{1F3E0}", "\u{1F4BC}", "\u{1F4DA}", "\u{1F392}", "\u{1F6CD}\u{FE0F}", "\u{1F4B0}", "\u{2708}\u{FE0F}", "\u{1F9E9}",
        "\u{1F3B8}", "\u{1F37F}", "\u{1F4F0}", "\u{1F4AC}", "\u{1F6E0}\u{FE0F}", "\u{1F9EA}", "\u{1F331}", "\u{2764}\u{FE0F}",
        "\u{2B50}", "\u{1F512}", "\u{1F30D}", "\u{1F4F7}", "\u{1F355}", "\u{1F45F}", "\u{1F436}", "\u{1F4A1}",
    ]

    private static func space(
        _ id: UInt64, _ name: String, logins: UInt32, colour: String, tidyAfter: UInt32 = 0, icon: String = ""
    ) -> WTSpaceSummary {
        var space = WTSpaceSummary()
        space.spaceID = id
        space.name = name
        space.logins = logins
        space.colour = colour
        space.tidyAfterSeconds = tidyAfter
        space.icon = icon
        return space
    }
}

/// The list of Spaces in one state.
struct CatalogueSpacesList: View {
    let state: CatalogueSpaces

    var body: some View {
        SpacesView(
            spaces: CatalogueSpaces.spaces,
            shownID: 2,
            colours: CatalogueSpaces.colours,
            icons: CatalogueSpaces.icons,
            tidyTimes: CatalogueSpaces.tidyTimes,
            refusal: state == .refused ? "space.too_many" : nil,
            making: [.new, .newShared, .refused].contains(state),
            name: "Travel",
            sharingWith: state == .newShared ? 2 : nil,
            renaming: state == .rename,
            show: { _ in }, make: { _, _, _, _ in }, edit: { _ in }, done: {}
        )
    }
}

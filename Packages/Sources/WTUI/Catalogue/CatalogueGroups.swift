// Groups (041) in the design catalogue: the row of a window that has them, and the page that
// makes a Group or edits one.
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

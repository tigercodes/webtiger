// The Archive (049) in the design catalogue: the Library with its fourth list shown, with
// tidied tabs in it, and with none. What a core would answer, with no core behind it.
import SwiftUI
import WTContract
import WTCoreClient

enum CatalogueArchive: String, CaseIterable, Identifiable {
    case list
    case empty

    var id: String {
        rawValue
    }

    /// The state's name in the catalogue.
    var title: Word {
        switch self {
        case .list: Words.catalogueStateArchiveList
        case .empty: Words.catalogueStateArchiveEmpty
        }
    }

    /// What a core would answer for the state: two tabs tidied at one look, one whose name is
    /// longer than the list is wide; one with no name, tidied half a day before; and one of
    /// last week. The addresses are the documentation's own, and the times are fixed: midday
    /// on the first of January 2026, and before.
    private var page: WTArchivePage {
        var page = WTArchivePage()
        guard self == .list else { return page }
        page.entries = [
            Self.tidied(4, "Example Domain", "https://example.com/", hoursAgo: 0),
            Self.tidied(
                3,
                "A page whose title is longer than the list is wide, and then goes on",
                "https://example.org/a/long/path",
                hoursAgo: 0
            ),
            Self.tidied(2, "", "https://example.net/plain.txt", hoursAgo: 12),
            Self.tidied(1, "Last week’s page", "http://example.com/last-week", hoursAgo: 24 * 7),
        ]
        page.more = true
        return page
    }

    /// A tab tidied so many hours before midday on the first of January 2026, with its
    /// address as the address field shows one: without its scheme when that is https.
    private static func tidied(_ id: Int64, _ title: String, _ url: String, hoursAgo: Int64) -> WTArchiveEntry {
        var entry = WTArchiveEntry()
        entry.id = id
        entry.title = title
        entry.url = url
        entry.display = url.hasPrefix("https://") ? String(url.dropFirst(8)).trimmingCharacters(in: ["/"]) : url
        entry.tidiedAt = 1_767_268_800_000 - hoursAgo * 3_600_000
        return entry
    }

    /// A core that answers with this state, and with nothing for the Library's other lists.
    func core() -> ScriptedCore {
        let answer = page
        return ScriptedCore { request in
            if case .queryArchive = request {
                return .archivePage(answer)
            }
            return .ack(WTAck())
        }
    }
}

/// The Library with the Archive shown, in one state. Its times are shown as one place and one
/// language would show them, so a picture of it is the same wherever it is drawn.
struct CatalogueArchiveList: View {
    private let core: ScriptedCore

    init(state: CatalogueArchive) {
        core = state.core()
    }

    var body: some View {
        LibraryView(
            showing: .archive,
            history: HistoryModel(core: core, windowID: 1),
            downloads: DownloadsModel(core: core, windowID: 1),
            permissions: PermissionsModel(core: core, windowID: 1),
            archive: ArchiveModel(core: core, windowID: 1),
            open: { _ in }, done: {}
        )
        .environment(\.timeZone, Self.calendar.timeZone)
        .environment(\.calendar, Self.calendar)
        .environment(\.locale, Locale(identifier: "en_GB"))
    }

    private static let calendar: Calendar = {
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = TimeZone(identifier: "UTC") ?? .gmt
        calendar.locale = Locale(identifier: "en_GB")
        return calendar
    }()
}

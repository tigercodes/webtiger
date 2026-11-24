// What is awake (056) in the design catalogue: the Library (057) with its fifth list shown,
// with tabs in it, and with none. What a core would answer, with no core behind it.
import SwiftUI
import WTContract
import WTCoreClient

enum CatalogueAwake: String, CaseIterable, Identifiable {
    case list
    case empty

    var id: String {
        rawValue
    }

    /// The state's name in the catalogue.
    var title: Word {
        switch self {
        case .list: Words.catalogueStateAwakeList
        case .empty: Words.catalogueStateAwakeEmpty
        }
    }

    /// What a core would answer for the state. With tabs: the one on screen, which is also
    /// on its way; one behind it with nothing to keep it awake; one that plays a sound and
    /// was typed into, whose name is longer than the list is wide; one in another Space that
    /// asks a question; one with no name, downloading; and a Site in a Space with a long
    /// name, using the camera. Two more are awake in private windows, and twelve are asleep.
    /// With none: no tab is awake, and forty are asleep.
    private var page: WTAwakePage {
        var page = WTAwakePage()
        guard self == .list else {
            page.asleep = 40
            return page
        }
        page.tabs = [
            Self.tab(1, "Example Domain", "example.com", space: "", inSight: true, [.loading]),
            Self.tab(2, "A Story", "example.org", space: "", []),
            Self.tab(
                3, "A page whose title is longer than the list is wide, and then goes on", "music.example.com", space: "",
                [.sound, .typed]
            ),
            Self.tab(4, "Calendar", "calendar.example.com", space: "Work", [.question]),
            Self.tab(5, "", "files.example.net", space: "Work", [.download]),
            Self.tab(6, "Meeting", "meet.example.com", space: "A Space with a long name of its own", kept: true, [.capture]),
        ]
        page.privateTabs = 2
        page.asleep = 12
        return page
    }

    private static func tab(
        _ id: UInt64, _ title: String, _ host: String, space: String, inSight: Bool = false, kept: Bool = false,
        _ reasons: [WTKeepAwake]
    ) -> WTAwakeTab {
        var tab = WTAwakeTab()
        tab.tabID = id
        tab.windowID = 1
        tab.spaceID = space.isEmpty ? 1 : 2
        tab.spaceName = space
        tab.title = title
        tab.host = host
        tab.inSight = inSight
        tab.kept = kept
        tab.keepsAwake = reasons
        return tab
    }

    /// A core that answers with this state, and with nothing for the Library's other lists.
    func core() -> ScriptedCore {
        let answer = page
        return ScriptedCore { request in
            if case .queryAwake = request {
                return .awakePage(answer)
            }
            return .ack(WTAck())
        }
    }
}

/// The Library with what is awake shown, in one state.
struct CatalogueAwakeList: View {
    private let core: ScriptedCore

    init(state: CatalogueAwake) {
        core = state.core()
    }

    var body: some View {
        LibraryView(
            showing: .awake,
            history: HistoryModel(core: core, windowID: 1),
            downloads: DownloadsModel(core: core, windowID: 1),
            permissions: PermissionsModel(core: core, windowID: 1),
            archive: ArchiveModel(core: core, windowID: 1),
            awake: AwakeModel(core: core, windowID: 1),
            open: { _ in }, done: {}
        )
        .environment(\.locale, Locale(identifier: "en_GB"))
    }
}

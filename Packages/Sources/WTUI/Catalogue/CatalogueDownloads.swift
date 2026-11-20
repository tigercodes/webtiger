// The Downloads list's states, for the design catalogue (004) and its pictures: what a core
// would answer, with no core and no file behind it.
import SwiftUI
import WTContract
import WTCoreClient

enum CatalogueDownloads: String, CaseIterable, Identifiable {
    case list
    case empty

    var id: String {
        rawValue
    }

    /// The state's name in the catalogue.
    var title: LocalizedStringKey {
        switch self {
        case .list: "catalogue.state.downloads_list"
        case .empty: "catalogue.state.downloads_empty"
        }
    }

    /// What a core would answer for the state: one download coming with its size known, one
    /// that failed, one that finished, one coming whose size is not known, and one whose name
    /// is longer than the list is wide and that was stopped. The addresses are the
    /// documentation's own.
    private var page: WTDownloadsPage {
        var page = WTDownloadsPage()
        guard self == .list else { return page }
        page.downloads = [
            Self.download(5, "Annual report 2026.pdf", "example.com", .coming, received: 1_200_000, total: 4_800_000),
            Self.download(4, "notes.txt", "example.com", .failed, code: "download.network"),
            Self.download(3, "holiday-photos.zip", "example.net", .finished, total: 12_400_000),
            Self.download(2, "stream.bin", "example.org", .coming, received: 300_000),
            Self.download(
                1,
                "A file whose name is longer than the list is wide, and then goes on for a while.tar.gz",
                "downloads.example.com",
                .failed,
                code: "download.cancelled"
            ),
        ]
        page.more = true
        return page
    }

    private static func download(
        _ id: UInt64,
        _ name: String,
        _ host: String,
        _ state: WTDownloadState,
        received: UInt64 = 0,
        total: UInt64 = 0,
        code: String = ""
    ) -> WTDownloadEntry {
        var entry = WTDownloadEntry()
        entry.id = id
        entry.fileName = name
        entry.host = host
        entry.url = "https://\(host)/files/\(id)"
        entry.state = state
        entry.receivedBytes = received
        entry.totalBytes = total
        entry.errorCode = code
        return entry
    }

    /// A model over a core that answers with this state, and nothing else.
    @MainActor
    func model() -> DownloadsModel {
        let answer = page
        let core = ScriptedCore { request in
            if case .queryDownloads = request {
                return .downloadsPage(answer)
            }
            return .ack(WTAck())
        }
        let model = DownloadsModel(core: core, windowID: 1)
        model.reload()
        return model
    }
}

/// The Downloads list in one state.
struct CatalogueDownloadsList: View {
    @State private var model: DownloadsModel

    init(state: CatalogueDownloads) {
        _model = State(initialValue: state.model())
    }

    var body: some View {
        DownloadsView(model: model, done: {})
            .environment(\.locale, Locale(identifier: "en_GB"))
    }
}

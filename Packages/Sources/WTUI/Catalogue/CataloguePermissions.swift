// The Permissions list's states, for the design catalogue (004) and its pictures: what a core
// would answer, with no core behind it.
import SwiftUI
import WTContract
import WTCoreClient

enum CataloguePermissions: String, CaseIterable, Identifiable {
    case list
    case empty

    var id: String {
        rawValue
    }

    /// The state's name in the catalogue.
    var title: Word {
        switch self {
        case .list: Words.catalogueStatePermissionsList
        case .empty: Words.catalogueStatePermissionsEmpty
        }
    }

    /// What a core would answer for the state: a site with a name longer than the list is
    /// wide, one that is not over https, and one with two things. The addresses are the
    /// documentation's own.
    private var page: WTPermissionsPage {
        var page = WTPermissionsPage()
        guard self == .list else { return page }
        page.sites = [
            Self.site("a-site-whose-name-is-longer-than-the-list-is-wide-and-then-goes-on.example.com", scheme: "https", [.camera]),
            Self.site("127.0.0.1:8080", scheme: "http", [.location, .downloads]),
            Self.site("meet.example.com", scheme: "https", [.camera, .microphone]),
        ]
        return page
    }

    private static func site(_ host: String, scheme: String, _ kinds: [WTPermissionKind]) -> WTSitePermissions {
        var site = WTSitePermissions()
        site.origin = "\(scheme)://\(host)"
        // As the address field shows it: with its scheme when that is not https.
        site.site = scheme == "https" ? host : site.origin
        site.kinds = kinds
        return site
    }

    /// A model over a core that answers with this state, and nothing else.
    @MainActor
    func model() -> PermissionsModel {
        let answer = page
        let core = ScriptedCore { request in
            if case .queryPermissions = request {
                return .permissionsPage(answer)
            }
            return .ack(WTAck())
        }
        let model = PermissionsModel(core: core, windowID: 1)
        model.reload()
        return model
    }
}

/// The Permissions list in one state.
struct CataloguePermissionsList: View {
    @State private var model: PermissionsModel

    init(state: CataloguePermissions) {
        _model = State(initialValue: state.model())
    }

    var body: some View {
        PermissionsView(model: model, done: {})
            .environment(\.locale, Locale(identifier: "en_GB"))
    }
}

// The list of search engines (020) in the design catalogue: the first run's question, and the
// same list opened again to change the engine in use.
import SwiftUI
import WTContract

enum CatalogueSearch: String, CaseIterable, Identifiable {
    case question
    case change

    var id: String {
        rawValue
    }

    /// The state's name in the catalogue.
    var title: LocalizedStringKey {
        switch self {
        case .question: "catalogue.state.search_question"
        case .change: "catalogue.state.search_change"
        }
    }

    /// What a core would say of where searches go in this state: at the first run nobody has
    /// chosen; opened again, one engine is in use.
    var settings: WTSearchSettings {
        var settings = Self.inUse
        switch self {
        case .question:
            settings.chosen = false
        case .change:
            settings.chosenID = "ecosia"
        }
        return settings
    }

    /// What a core says in every other picture: an engine is in use, by somebody's choice.
    static let inUse: WTSearchSettings = {
        var settings = WTSearchSettings()
        settings.engines = [
            ("bing", "Bing", "www.bing.com"),
            ("brave", "Brave Search", "search.brave.com"),
            ("duckduckgo", "DuckDuckGo", "duckduckgo.com"),
            ("ecosia", "Ecosia", "www.ecosia.org"),
        ]
        .map { id, name, host in
            var engine = WTSearchEngine()
            engine.id = id
            engine.name = name
            engine.host = host
            return engine
        }
        settings.chosenID = "duckduckgo"
        settings.chosen = true
        return settings
    }()
}

/// The list of search engines in one state.
struct CatalogueSearchEngines: View {
    let state: CatalogueSearch

    var body: some View {
        SearchEnginesView(settings: state.settings, choose: { _ in }, keep: {})
    }
}

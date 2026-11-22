public import WTContract

// Where searches go (020): the list of search engines a person chooses among. It is the first
// run's question, over the window before the window is used; and the same list again when an
// empty tab's "Change…" is pressed, with the engine in use marked.
import SwiftUI
import WTDesign

public extension BrowserModel {
    /// The list of engines is before the person: because nobody has chosen yet (the first
    /// run's choice, offered once with the default to keep), or because they asked to change.
    /// Not until the core has said where searches go.
    var asksWhereSearchesGo: Bool {
        guard let search else { return false }
        return (!search.chosen && !hasKeptDefaultSearch) || isChangingSearch
    }

    /// The name of the engine searches go to now. `nil` until the core has said.
    var searchEngineName: String? {
        guard let search else { return nil }
        return search.engines.first { $0.id == search.chosenID }?.name
    }

    /// Asks the core where searches go. Its answer is what the window then shows.
    func askWhereSearchesGo() {
        let answer = session.core.send(.querySearch(WTQuerySearch()))
        Task { [weak self] in
            guard case let .searchSettings(settings) = try? await answer.value else { return }
            self?.told(whereSearchesGo: settings)
        }
    }

    /// The core has said where searches go: the window shows it, and the menu of selected
    /// words names the engine a search for them goes to (023). The menu's words are the
    /// session's, which every model of it shares; so they change here, on the core's word,
    /// and never because a model was made: a window's view is made again many times, each
    /// time with a model that is thrown away, and one that knows nothing yet must not take
    /// the words away.
    private func told(whereSearchesGo settings: WTSearchSettings) {
        search = settings
        session.engine.linkMenu?.searchWords = searchEngineName.map {
            Words.selectionSearchWith.filled($0)
        }
    }

    /// The person's choice of an engine. The list goes when the core has taken it; a choice
    /// it refuses leaves the list as it is.
    func chooseSearchEngine(_ id: String) {
        var choice = WTChooseSearchEngine()
        choice.id = id
        let answer = session.core.send(.chooseSearchEngine(choice))
        Task { [weak self] in
            guard case .ack = try? await answer.value, let self else { return }
            isChangingSearch = false
            // The core's own word on what is now in use, and by whose choice.
            askWhereSearchesGo()
        }
    }

    /// An empty tab's "Change…".
    func changeSearch() {
        isChangingSearch = true
    }

    /// The list is closed with nothing changed. At a first run, when nobody has chosen, that
    /// is keeping the default: it is chosen, so that the choice is not offered again.
    func keepSearch() {
        if let search, !search.chosen {
            hasKeptDefaultSearch = true
            chooseSearchEngine(search.chosenID)
        }
        isChangingSearch = false
    }
}

/// `choose` is told the engine that was pressed, by the core's name for it. `keep` closes the
/// list with nothing changed: at the first run that keeps the default, which the button names.
struct SearchEnginesView: View {
    let settings: WTSearchSettings
    let choose: (String) -> Void
    let keep: () -> Void
    @Environment(\.locale) private var locale

    /// The name of the engine searches go to now: the default, while nobody has chosen.
    private var inUseName: String {
        settings.engines.first { $0.id == settings.chosenID }?.name ?? ""
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                VStack(alignment: .leading, spacing: Tokens.Space.s) {
                    Text(Words.searchQuestionTitle)
                        .font(.title2.weight(.semibold))
                        .fixedSize(horizontal: false, vertical: true)
                        .accessibilityAddTraits(.isHeader)
                        .accessibilityIdentifier("search-question")
                    // In the colour of the heading: a quieter grey on a sheet is too faint to
                    // read at this size (Apple's audit, on an iPhone).
                    Text(Words.searchQuestionDetail)
                        .fixedSize(horizontal: false, vertical: true)
                }
                .padding(Tokens.Space.l)
                ForEach(settings.engines, id: \.id) { engine in
                    Divider()
                    row(engine)
                }
                Divider()
                Button(action: keep) {
                    // "Cancel" where one is in use by somebody's choice; at the first run,
                    // "Keep DuckDuckGo": the default, by its name.
                    Group {
                        if settings.chosen {
                            Text(Words.searchKeep)
                        } else {
                            Text(verbatim: Words.searchKeepDefault.filled(inUseName, locale: locale))
                        }
                    }
                    .frame(minHeight: BrowserWindowView.hitSize)
                }
                .buttonStyle(.bordered)
                .tint(.primary)
                .padding(Tokens.Space.l)
                .accessibilityIdentifier("search-keep")
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        }
    }

    /// One engine: its name, where the words are sent, and that it is in use when it is.
    private func row(_ engine: WTSearchEngine) -> some View {
        // The engine searches go to is marked, in words: "In use" where somebody chose it,
        // "Default" at the first run, while nobody has.
        let inUse = engine.id == settings.chosenID
        return Button {
            choose(engine.id)
        } label: {
            HStack(spacing: Tokens.Space.m) {
                VStack(alignment: .leading, spacing: 0) {
                    // The name stands out by its weight, not by the address being fainter.
                    Text(engine.name)
                        .fontWeight(.semibold)
                    Text(engine.host)
                        .font(.callout)
                }
                Spacer(minLength: Tokens.Space.m)
                if inUse {
                    Text(settings.chosen ? Words.searchInUse : Words.searchDefault)
                        .font(.callout)
                }
            }
            .frame(maxWidth: .infinity, minHeight: BrowserWindowView.hitSize, alignment: .leading)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .padding(.horizontal, Tokens.Space.l)
        .padding(.vertical, Tokens.Space.xs)
        .accessibilityIdentifier("search-engine-\(engine.id)")
    }
}

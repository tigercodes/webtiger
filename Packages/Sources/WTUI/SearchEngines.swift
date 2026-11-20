public import WTContract

// Where searches go (020): the list of search engines a person chooses among. It is the first
// run's question, over the window before the window is used; and the same list again when an
// empty tab's "Change…" is pressed, with the engine in use marked.
import SwiftUI
import WTDesign

public extension BrowserModel {
    /// The list of engines is before the person: because nobody has chosen yet (the first
    /// run's question, which stays until one is chosen), or because they asked to change.
    /// Not until the core has said where searches go.
    var asksWhereSearchesGo: Bool {
        guard let search else { return false }
        return !search.chosen || isChangingSearch
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
            self?.search = settings
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

    /// The list is closed with nothing changed.
    func keepSearch() {
        isChangingSearch = false
    }
}

/// `choose` is told the engine that was pressed, by the core's name for it. `keep` closes the
/// list with nothing changed: it is `nil` at the first run, when one has to be chosen.
struct SearchEnginesView: View {
    let settings: WTSearchSettings
    let choose: (String) -> Void
    let keep: (() -> Void)?

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                VStack(alignment: .leading, spacing: Tokens.Space.s) {
                    Text("search.question.title", bundle: .module)
                        .font(.title2.weight(.semibold))
                        .fixedSize(horizontal: false, vertical: true)
                        .accessibilityAddTraits(.isHeader)
                        .accessibilityIdentifier("search-question")
                    // In the colour of the heading: a quieter grey on a sheet is too faint to
                    // read at this size (Apple's audit, on an iPhone).
                    Text("search.question.detail", bundle: .module)
                        .fixedSize(horizontal: false, vertical: true)
                }
                .padding(Tokens.Space.l)
                ForEach(settings.engines, id: \.id) { engine in
                    Divider()
                    row(engine)
                }
                Divider()
                if let keep {
                    Button(action: keep) {
                        Text("search.keep", bundle: .module)
                            .frame(minHeight: BrowserWindowView.hitSize)
                    }
                    .buttonStyle(.bordered)
                    .tint(.primary)
                    .padding(Tokens.Space.l)
                    .accessibilityIdentifier("search-keep")
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        }
    }

    /// One engine: its name, where the words are sent, and that it is in use when it is.
    private func row(_ engine: WTSearchEngine) -> some View {
        // Marked only where somebody chose it: at the first run none is anybody's choice.
        let inUse = settings.chosen && engine.id == settings.chosenID
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
                    Text("search.in_use", bundle: .module)
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

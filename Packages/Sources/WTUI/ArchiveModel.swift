// The Archive's model (049): the tabs that were tidied out of the Space a window shows, newest
// first, a page at a time; brought back, and forgotten. The shell keeps no archive of its own:
// it asks.
public import Observation
public import WTContract
public import WTCoreClient

@MainActor
@Observable
public final class ArchiveModel {
    /// The tidied tabs shown, newest first.
    public private(set) var entries: [WTArchiveEntry] = []
    /// There are older ones than these to show.
    public private(set) var more = false
    /// The core has answered at least once. Until then the list says nothing at all, rather
    /// than that nothing was tidied.
    public private(set) var loaded = false

    private let core: any CoreClient
    private let windowID: UInt64
    /// Counts what was asked: an answer to anything but the last question is out of date.
    private var asked = 0

    /// The Archive of a window's Space.
    public init(core: any CoreClient, windowID: UInt64) {
        self.core = core
        self.windowID = windowID
    }

    /// Asks for the newest of the Archive.
    public func reload() {
        ask(before: 0)
    }

    /// Asks for the ones older than the last one shown.
    public func showOlder() {
        guard more, let last = entries.last else { return }
        ask(before: last.id)
    }

    /// Brings a tidied tab back: the core makes it a tab again, in front. It leaves the list
    /// at once. False when the core would not (the Archive has it no longer: another window
    /// brought it back, or forgot it): the list is then asked for again.
    public func bringBack(_ entry: WTArchiveEntry) async -> Bool {
        var restore = WTRestoreArchived()
        restore.windowID = windowID
        restore.entryID = entry.id
        let answer = core.send(.restoreArchived(restore))
        entries.removeAll { $0.id == entry.id }
        if case .tabOpened = try? await answer.value {
            return true
        }
        reload()
        return false
    }

    /// Forgets one tidied tab. It leaves the list at once; the store forgets it for good.
    public func forget(_ entry: WTArchiveEntry) {
        var forget = WTForgetArchived()
        forget.windowID = windowID
        forget.entryID = entry.id
        _ = core.send(.forgetArchived(forget))
        entries.removeAll { $0.id == entry.id }
    }

    /// Forgets all of the Space's Archive, and asks again for what is left: nothing.
    public func forgetEverything() {
        var forget = WTForgetArchived()
        forget.windowID = windowID
        forget.all = true
        _ = core.send(.forgetArchived(forget))
        reload()
    }

    private func ask(before: Int64) {
        asked += 1
        let question = asked
        var query = WTQueryArchive()
        query.windowID = windowID
        query.before = before
        // Sent now, after whatever was sent before it: a question that follows a forgetting is
        // answered with what is left.
        let answer = core.send(.queryArchive(query))
        Task { [weak self] in
            guard case let .archivePage(page) = try? await answer.value, let self, question == asked else { return }
            entries = before == 0 ? page.entries : entries + page.entries
            more = page.more
            loaded = true
        }
    }
}

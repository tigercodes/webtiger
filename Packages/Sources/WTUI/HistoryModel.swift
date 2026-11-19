// The History list's model (008): what the core's store says was visited, a page at a time,
// found by words, and forgotten. The shell keeps no history of its own: it asks.
public import Foundation
public import Observation
public import WTContract
public import WTCoreClient

@MainActor
@Observable
public final class HistoryModel {
    /// The visits shown, newest first.
    public private(set) var entries: [WTHistoryEntry] = []
    /// There are older visits than these to show.
    public private(set) var more = false
    /// The core has answered at least once. Until then the list says nothing at all, rather
    /// than that nothing was visited.
    public private(set) var loaded = false
    /// The words to look for. Changing them asks again.
    public var text = "" {
        didSet {
            if text != oldValue {
                reload()
            }
        }
    }

    private let core: any CoreClient
    private let windowID: UInt64
    /// Counts what was asked: an answer to anything but the last question is out of date.
    private var asked = 0

    /// The history of a window's Space.
    public init(core: any CoreClient, windowID: UInt64) {
        self.core = core
        self.windowID = windowID
    }

    /// Asks for the newest visits that match the words.
    public func reload() {
        ask(before: 0)
    }

    /// Asks for the visits older than the last one shown.
    public func showOlder() {
        guard more, let last = entries.last else { return }
        ask(before: last.visitID)
    }

    /// Forgets one visit. It leaves the list at once; the store forgets it for good.
    public func forget(_ entry: WTHistoryEntry) {
        var forget = WTForgetHistory()
        forget.windowID = windowID
        forget.visitID = entry.visitID
        _ = core.send(.forgetHistory(forget))
        entries.removeAll { $0.visitID == entry.visitID }
    }

    /// Forgets every visit made at or after a time, and asks again for what is left.
    public func forget(since time: Date) {
        var forget = WTForgetHistory()
        forget.windowID = windowID
        forget.sinceMs = Int64((time.timeIntervalSince1970 * 1000).rounded(.down))
        _ = core.send(.forgetHistory(forget))
        reload()
    }

    /// Forgets every visit.
    public func forgetEverything() {
        var forget = WTForgetHistory()
        forget.windowID = windowID
        forget.everything = true
        _ = core.send(.forgetHistory(forget))
        reload()
    }

    private func ask(before: UInt64) {
        asked += 1
        let question = asked
        var query = WTQueryHistory()
        query.windowID = windowID
        query.text = text
        query.beforeVisitID = before
        // Sent now, after whatever was sent before it: a question that follows a forgetting is
        // answered with what is left.
        let answer = core.send(.queryHistory(query))
        Task { [weak self] in
            guard case let .historyPage(page) = try? await answer.value, let self, question == asked else { return }
            entries = before == 0 ? page.entries : entries + page.entries
            more = page.more
            loaded = true
        }
    }
}

public import Observation
public import WTContract
public import WTCoreClient

// The Downloads list's model (013): what the core says was downloaded, a page at a time, with
// how far each one that is coming has got. The shell keeps no list of its own: it asks.
import Foundation

@MainActor
@Observable
public final class DownloadsModel {
    /// The downloads shown, newest first.
    public private(set) var entries: [WTDownloadEntry] = []
    /// There are older downloads than these to show.
    public private(set) var more = false
    /// The core has answered at least once. Until then the list says nothing at all, rather
    /// than that nothing was downloaded.
    public private(set) var loaded = false

    /// Why a file a person pressed did not open, by its download: the core's code (030). Said
    /// under the download until it is pressed again.
    public private(set) var refusals: [UInt64: String] = [:]

    private let core: any CoreClient
    private let windowID: UInt64
    /// Counts what was asked: an answer to anything but the last question is out of date.
    private var asked = 0

    /// The downloads of a window's Space.
    public init(core: any CoreClient, windowID: UInt64) {
        self.core = core
        self.windowID = windowID
    }

    /// Something in the list is still coming.
    public var isComing: Bool {
        entries.contains { $0.state == .coming }
    }

    /// Asks for the list again, as far down as it is shown.
    public func reload() {
        ask(before: 0)
    }

    /// Asks for the downloads older than the last one shown.
    public func showOlder() {
        guard more, let last = entries.last else { return }
        ask(before: last.id)
    }

    /// Opens a finished download's file with whatever the system has for it (030). The core
    /// is asked: whether it may be opened, and what came of it, are the core's to say. A
    /// download with no whole file is not asked about.
    public func open(_ entry: WTDownloadEntry) async {
        guard entry.state == .finished else { return }
        refusals[entry.id] = nil
        var open = WTOpenDownload()
        open.windowID = windowID
        open.downloadID = entry.id
        do {
            // A core says no by throwing; one that answers with the error says the same.
            if case let .error(refused) = try await core.send(.openDownload(open)).value {
                refusals[entry.id] = refused.code
            }
        } catch let CoreError.refused(code, _) {
            refusals[entry.id] = code
        } catch {
            // The core is gone, or did not answer: the file did not open.
            refusals[entry.id] = ""
        }
    }

    /// Asks again every second for as long as the list is shown: how far a download has got
    /// is not pushed, and one that starts while the list is open belongs in it.
    public func watch() async {
        reload()
        while !Task.isCancelled {
            try? await Task.sleep(for: .seconds(1))
            if !Task.isCancelled {
                reload()
            }
        }
    }

    /// Stops a download that is coming. What the core then says of it is what the list shows.
    public func stop(_ entry: WTDownloadEntry) {
        var cancel = WTCancelDownload()
        cancel.downloadID = entry.id
        _ = core.send(.cancelDownload(cancel))
        reload()
    }

    /// Takes one download off the list. It leaves at once; its file stays where it is.
    public func forget(_ entry: WTDownloadEntry) {
        var forget = WTForgetDownloads()
        forget.windowID = windowID
        forget.downloadID = entry.id
        _ = core.send(.forgetDownloads(forget))
        entries.removeAll { $0.id == entry.id }
    }

    /// Takes every download off the list.
    public func forgetEverything() {
        var forget = WTForgetDownloads()
        forget.windowID = windowID
        forget.everything = true
        _ = core.send(.forgetDownloads(forget))
        reload()
    }

    private func ask(before: UInt64) {
        asked += 1
        let question = asked
        var query = WTQueryDownloads()
        query.windowID = windowID
        query.beforeID = before
        // A list that has been taken further down is asked for whole again, so that what it
        // shows of the older ones is as fresh as the rest.
        if before == 0, entries.count > 100 {
            query.limit = UInt32(min(entries.count, 500))
        }
        // Sent now, after whatever was sent before it: a question that follows a forgetting is
        // answered with what is left.
        let answer = core.send(.queryDownloads(query))
        Task { [weak self] in
            guard case let .downloadsPage(page) = try? await answer.value, let self, question == asked else { return }
            entries = before == 0 ? page.downloads : entries + page.downloads
            more = page.more
            loaded = true
        }
    }
}

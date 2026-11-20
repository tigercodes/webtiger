// Downloads (013 §6), on WebKit. WebKit moves the bytes, in the page's own session: its
// cookies, the answer to a form it posted, a file it made itself. The adapter says a download
// was asked for, waits for the core's word, saves the file under the name the core made, and
// says how it goes. It decides nothing, and never writes a name a page gave it.
import Foundation
import WebKit
import WTContract

@MainActor
final class Downloads: NSObject, WKDownloadDelegate {
    /// A download WebKit is fetching, and where.
    private struct Coming {
        let download: WKDownload
        let webView: UInt64
        let destination: URL
        var watching: NSKeyValueObservation?
        var lastSaid: Date = .distantPast
    }

    private let folder: URL
    private let report: @MainActor (UInt64, WTEngineEvent.OneOf_Kind) -> Void
    /// How many downloads this engine has named. Each has the next number, and none is used
    /// twice: the ids are the shell's own (engine.proto, `DownloadRequested`).
    private var named: UInt64 = 0
    /// Each download WebKit has handed over, by which one it is: its id, and the web view
    /// whose page asked for it.
    private var known: [ObjectIdentifier: (id: UInt64, webView: UInt64)] = [:]
    /// Downloads reported to the core and waiting for its word: resumed with the name to
    /// save under, or with nil to drop it.
    private var asked: [UInt64: CheckedContinuation<String?, Never>] = [:]
    private var coming: [UInt64: Coming] = [:]

    /// How often, at most, a download's progress is passed on.
    private static let pace: TimeInterval = 0.25

    init(folder: URL, report: @escaping @MainActor (UInt64, WTEngineEvent.OneOf_Kind) -> Void) {
        self.folder = folder
        self.report = report
    }

    /// WebKit turned a navigation into a download: it is this adapter's from here.
    func adopt(_ download: WKDownload, from webView: UInt64) {
        named += 1
        known[ObjectIdentifier(download)] = (named, webView)
        download.delegate = self
    }

    /// The core's word: the download starts, saved under `name`. False for a download that is
    /// not waiting for one.
    func start(_ id: UInt64, named name: String) -> Bool {
        guard let waiting = asked.removeValue(forKey: id) else { return false }
        waiting.resume(returning: name)
        return true
    }

    /// The core's word: the download is dropped. One that was waiting never starts; one that
    /// is coming stops, and what was saved of it goes.
    func cancel(_ id: UInt64) {
        if let waiting = asked.removeValue(forKey: id) {
            waiting.resume(returning: nil)
        } else if let stopping = coming[id] {
            stopping.download.cancel { [weak self] _ in
                Task { @MainActor in self?.ended(id, code: "download.cancelled") }
            }
        }
    }

    // MARK: WKDownloadDelegate

    /// WebKit asks where the file goes. That is the moment a download is reported: the core
    /// is told what the site suggests, and its answer is the name, or that there is to be no
    /// download.
    func download(_ download: WKDownload, decideDestinationUsing response: URLResponse, suggestedFilename: String) async -> URL? {
        guard let (id, webView) = known[ObjectIdentifier(download)] else { return nil }
        var asking = WTDownloadRequested()
        asking.downloadID = id
        asking.url = (response.url ?? download.originalRequest?.url)?.absoluteString ?? ""
        asking.suggestedFilename = suggestedFilename
        asking.mimeType = response.mimeType ?? ""
        asking.expectedBytes = response.expectedContentLength > 0 ? UInt64(response.expectedContentLength) : 0
        let name = await withCheckedContinuation { (waiting: CheckedContinuation<String?, Never>) in
            asked[id] = waiting
            report(webView, .downloadRequested(asking))
        }
        guard let name, let destination = place(for: name) else {
            known[ObjectIdentifier(download)] = nil
            return nil
        }
        coming[id] = Coming(download: download, webView: webView, destination: destination)
        watch(download, id)
        return destination
    }

    func downloadDidFinish(_ download: WKDownload) {
        guard let (id, _) = known.removeValue(forKey: ObjectIdentifier(download)),
              let finished = coming.removeValue(forKey: id) else { return }
        var done = WTDownloadFinished()
        done.downloadID = id
        done.fileName = finished.destination.lastPathComponent
        report(finished.webView, .downloadFinished(done))
    }

    func download(_ download: WKDownload, didFailWithError error: any Error, resumeData _: Data?) {
        guard let (id, _) = known[ObjectIdentifier(download)] else { return }
        ended(id, code: Self.code(for: error as NSError))
    }

    // MARK: How it goes

    /// A download ended without its file: what was saved of it goes, and the core is told why.
    /// One the adapter no longer has ended already.
    private func ended(_ id: UInt64, code: String) {
        guard let failed = coming.removeValue(forKey: id) else { return }
        known[ObjectIdentifier(failed.download)] = nil
        try? FileManager.default.removeItem(at: failed.destination)
        var gone = WTDownloadFailed()
        gone.downloadID = id
        gone.code = code
        report(failed.webView, .downloadFailed(gone))
    }

    /// Passes on how far a download has got, a few times a second at most.
    private func watch(_ download: WKDownload, _ id: UInt64) {
        coming[id]?.watching = download.progress.observe(\.completedUnitCount) { [weak self] progress, _ in
            let received = progress.completedUnitCount
            let expected = progress.totalUnitCount
            Task { @MainActor in self?.said(id, received: received, expected: expected) }
        }
    }

    private func said(_ id: UInt64, received: Int64, expected: Int64) {
        guard let download = coming[id], Date().timeIntervalSince(download.lastSaid) >= Self.pace else { return }
        coming[id]?.lastSaid = Date()
        var progress = WTDownloadProgress()
        progress.downloadID = id
        progress.receivedBytes = UInt64(max(received, 0))
        progress.expectedBytes = UInt64(max(expected, 0))
        report(download.webView, .downloadProgress(progress))
    }

    /// Where a file called `name` is saved: in the folder, under that name, or with " (2)" and
    /// so on when the name is taken by a file or by a download that is coming. The name is the
    /// core's and has no folder in it; it is held to that here all the same.
    private func place(for name: String) -> URL? {
        let name = (name as NSString).lastPathComponent
        guard !name.isEmpty, name != ".", name != ".." else { return nil }
        do {
            try FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
        } catch {
            return nil
        }
        let ending = (name as NSString).pathExtension
        let stem = (name as NSString).deletingPathExtension
        var place = folder.appendingPathComponent(name)
        var number = 2
        while FileManager.default.fileExists(atPath: place.path) || coming.values.contains(where: { $0.destination == place }) {
            guard number < 10000 else { return nil }
            place = folder.appendingPathComponent(ending.isEmpty ? "\(stem) (\(number))" : "\(stem) (\(number)).\(ending)")
            number += 1
        }
        return place
    }

    /// An error as one of the contract's codes. The error's own text never leaves the shell.
    private static func code(for error: NSError) -> String {
        switch error.domain {
        case NSURLErrorDomain:
            switch error.code {
            case NSURLErrorCancelled: "download.cancelled"
            case NSURLErrorCannotCreateFile, NSURLErrorCannotOpenFile, NSURLErrorCannotWriteToFile, NSURLErrorCannotCloseFile,
                 NSURLErrorCannotRemoveFile, NSURLErrorCannotMoveFile, NSURLErrorNoPermissionsToReadFile: "download.disk"
            default: "download.network"
            }
        case NSCocoaErrorDomain, NSPOSIXErrorDomain: "download.disk"
        default: "download.failed"
        }
    }
}

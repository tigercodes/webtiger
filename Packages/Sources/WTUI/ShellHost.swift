// What the core asks of the shell (000 §5.5), sorted by who answers: the System service is the
// shell's own, and everything else is a web engine's.
public import Foundation
public import WTContract
public import WTCoreClient
#if os(macOS)
    import AppKit
#else
    import UIKit
#endif

/// Opens an address with whatever the system has for it: the mail app for a letter, the
/// dialler for a number. True when the system took it; false when nothing on the device opens
/// it.
///
/// The shell opens only what the core asks it to (`OpenOutside`): which addresses may leave
/// the browser, and for whom, is decided there (007).
public struct OutsideOpener: Sendable {
    public let open: @MainActor @Sendable (URL) async -> Bool

    public init(open: @escaping @MainActor @Sendable (URL) async -> Bool) {
        self.open = open
    }

    /// The system's own. On a Mac, when nothing opens the address, the system says so itself
    /// in a dialog of its own, and offers to choose an app: that is what a person is told.
    public static let system = OutsideOpener { url in await openWithTheSystem(url, prompting: true) }

    /// The system's own, asked not to put anything of its own on the screen when nothing opens
    /// the address. A test's: the system's dialog stays on the screen of the machine a suite
    /// runs on until somebody answers it (007 §6).
    static let systemWithoutItsPrompt = OutsideOpener { url in await openWithTheSystem(url, prompting: false) }

    @MainActor private static func openWithTheSystem(_ url: URL, prompting: Bool) async -> Bool {
        #if os(macOS)
            let configuration = NSWorkspace.OpenConfiguration()
            configuration.promptsUserIfNeeded = prompting
            return await (try? NSWorkspace.shared.open(url, configuration: configuration)) != nil
        #else
            return await UIApplication.shared.open(url)
        #endif
    }

    /// Opens nothing, and says it did. Fake mode's: a UI suite, or a screenshot run, must not
    /// start the mail app of the machine it runs on.
    public static let nothing = OutsideOpener { _ in true }
}

/// What came of asking the system to open a file (030).
public enum FileOpened: Sendable {
    /// The system took it.
    case taken
    /// Nothing on the device opens files of its kind.
    case noHandler
    /// The file is not where it was saved.
    case missing
}

/// Opens a file that was downloaded, by the name it was saved under, with whatever the system
/// has for its kind (030).
///
/// The shell opens only what the core asks it to (`OpenFile`): which downloads may be opened
/// is decided there, on a person's press in the list.
public struct FileOpener: Sendable {
    public let open: @MainActor @Sendable (_ fileName: String, _ kind: String) async -> FileOpened

    public init(open: @escaping @MainActor @Sendable (_ fileName: String, _ kind: String) async -> FileOpened) {
        self.open = open
    }

    /// Opens nothing, and says it did. Fake mode's: a test must not start another app.
    public static let nothing = FileOpener { _, _ in .taken }

    /// The system's own, for a file in the folder this shell saves downloads in. On a Mac,
    /// when nothing opens the file, the system says so itself and offers to choose an app.
    public static func system(in folder: URL) -> FileOpener {
        FileOpener { name, _ in await openWithTheSystem(name, in: folder, prompting: true) }
    }

    /// The same, asked to put nothing of its own on the screen: a test's.
    static func systemWithoutItsPrompt(in folder: URL) -> FileOpener {
        FileOpener { name, _ in await openWithTheSystem(name, in: folder, prompting: false) }
    }

    /// The file of that name in the folder, when it is there and is a file. A name is a name:
    /// one that is a path, or that leads out of the folder, is no file of this shell's.
    static func file(named name: String, in folder: URL) -> URL? {
        guard !name.isEmpty, name == (name as NSString).lastPathComponent, name != ".", name != ".." else { return nil }
        let file = folder.appendingPathComponent(name, isDirectory: false)
        var isFolder: ObjCBool = false
        guard FileManager.default.fileExists(atPath: file.path, isDirectory: &isFolder), !isFolder.boolValue else { return nil }
        return file
    }

    @MainActor private static func openWithTheSystem(_ name: String, in folder: URL, prompting: Bool) async -> FileOpened {
        guard let file = file(named: name, in: folder) else { return .missing }
        #if os(macOS)
            let configuration = NSWorkspace.OpenConfiguration()
            configuration.promptsUserIfNeeded = prompting
            // The file and nothing else: the system chooses what opens it.
            return await (try? NSWorkspace.shared.open(file, configuration: configuration)) != nil ? .taken : .noHandler
        #else
            // An iPhone has no way to hand a file of an app's to whatever opens its kind: the
            // list does not offer it there yet (030 §5).
            _ = file
            _ = prompting
            return .noHandler
        #endif
    }
}

/// Answers the core's host requests: the System service here, the rest by the engine adapter.
@MainActor
public final class ShellHost: HostServices {
    private let engine: any HostServices
    private let opener: OutsideOpener
    private let files: FileOpener

    public init(engine: any HostServices, opener: OutsideOpener, files: FileOpener = .nothing) {
        self.engine = engine
        self.opener = opener
        self.files = files
    }

    public func answer(_ request: WTHostRequest.OneOf_Kind) async -> WTHostResponse.OneOf_Kind {
        switch request {
        case let .openOutside(open):
            // An address the system cannot even read is one nothing opens.
            guard let url = URL(string: open.url), url.scheme != nil, await opener.open(url) else {
                var error = WTError()
                error.code = "system.no_handler"
                return .error(error)
            }
            return .ack(WTAck())
        case let .openFile(open):
            // A downloaded file a person pressed in the list (030): the core said it may be.
            var error = WTError()
            switch await files.open(open.fileName, open.mimeType) {
            case .taken: return .ack(WTAck())
            case .noHandler: error.code = "system.no_handler"
            case .missing: error.code = "download.missing"
            }
            return .error(error)
        case .ping, .webViewCreate, .webViewLoad, .webViewCommand, .webViewDestroy, .webViewFind, .webViewSetZoom, .downloadStart,
             .downloadCancel, .permissionAnswer, .dialogAnswer, .signInAnswer, .webViewTrustCertificate:
            return await engine.answer(request)
        }
    }
}

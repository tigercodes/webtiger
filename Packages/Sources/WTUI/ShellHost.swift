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

/// Answers the core's host requests: the System service here, the rest by the engine adapter.
@MainActor
public final class ShellHost: HostServices {
    private let engine: any HostServices
    private let opener: OutsideOpener

    public init(engine: any HostServices, opener: OutsideOpener) {
        self.engine = engine
        self.opener = opener
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
        case .ping, .webViewCreate, .webViewLoad, .webViewCommand, .webViewDestroy, .webViewFind, .webViewSetZoom, .downloadStart,
             .downloadCancel, .permissionAnswer:
            return await engine.answer(request)
        }
    }
}

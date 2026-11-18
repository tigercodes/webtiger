// One core and one engine for the whole app. Windows ask it for their models.
public import Foundation
public import WTCoreClient
public import WTWebKit
import WTContract

/// The app's connection to the core: started once, shared by every window.
@MainActor
public final class BrowserSession {
    public let core: any CoreClient
    public let engine: WebKitEngine
    private var windows: [UInt64: @MainActor (WTWindowSnapshot) -> Void] = [:]
    private var listening: Task<Void, Never>?

    /// A session over a core that is already running. Tests pass a `ScriptedCore`.
    public init(core: any CoreClient, engine: WebKitEngine) {
        self.core = core
        self.engine = engine
        engine.attach(core)
        let events = core.events
        listening = Task { [weak self] in
            for await event in events {
                self?.receive(event)
            }
        }
    }

    #if WEBTIGER_CORE
        /// Starts the Rust core for this process.
        ///
        /// Fake mode, for UI suites and screenshots: `WEBTIGER_FAKE=1` keeps the store in memory
        /// and makes ids deterministic; `WEBTIGER_FAKE_START` names an address every new window
        /// opens.
        public static func live(environment: [String: String] = ProcessInfo.processInfo.environment) throws -> BrowserSession {
            let engine = WebKitEngine()
            var config = WTCoreConfig()
            #if os(macOS)
                config.platform = .macos
            #else
                config.platform = .ios
            #endif
            config.locale = Locale.current.identifier(.bcp47)
            config.appVersion = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "0"
            config.engines = [WebKitEngine.info]
            if environment["WEBTIGER_FAKE"] == "1" {
                config.testMode = .fake
                config.fake.startAddress = environment["WEBTIGER_FAKE_START"] ?? ""
            } else {
                config.testMode = .live
                config.dataDir = try dataDirectory().path(percentEncoded: false)
            }
            let core = try NativeCore.start(config: config, host: engine)
            return BrowserSession(core: core, engine: engine)
        }

        /// Application Support/WebTiger, made if it is not there.
        private static func dataDirectory() throws -> URL {
            let support = try FileManager.default.url(
                for: .applicationSupportDirectory,
                in: .userDomainMask,
                appropriateFor: nil,
                create: true
            )
            return support.appending(path: "WebTiger", directoryHint: .isDirectory)
        }
    #endif

    /// Opens a window in the core and delivers its snapshots to `update` until it is closed.
    func openWindow(update: @escaping @MainActor (WTWindowSnapshot) -> Void) async throws -> OpenedWindow {
        guard case let .windowOpened(opened) = try await core.request(.openWindow(WTOpenWindow())) else {
            throw CoreError.refused(code: "response.unexpected", field: "")
        }
        var subscribe = WTSubscribe()
        subscribe.window.windowID = opened.windowID
        guard case let .subscribed(subscribed) = try await core.request(.subscribe(subscribe)) else {
            throw CoreError.refused(code: "response.unexpected", field: "")
        }
        windows[subscribed.subscriptionID] = update
        return OpenedWindow(windowID: opened.windowID, tabID: opened.tabID, subscriptionID: subscribed.subscriptionID)
    }

    func closeWindow(_ window: OpenedWindow) {
        windows[window.subscriptionID] = nil
        var close = WTCloseWindow()
        close.windowID = window.windowID
        let core = core
        Task { try? await core.perform(.closeWindow(close)) }
    }

    private func receive(_ event: WTEvent.OneOf_Kind) {
        guard case let .snapshot(snapshot) = event, case let .window(window) = snapshot.model else { return }
        windows[snapshot.subscriptionID]?(window)
    }
}

struct OpenedWindow: Equatable {
    let windowID: UInt64
    let tabID: UInt64
    let subscriptionID: UInt64
}

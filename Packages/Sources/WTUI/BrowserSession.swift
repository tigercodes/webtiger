// One core and one engine for the whole app. Windows ask it for their models.
public import Foundation
public import WTCoreClient
public import WTWebKit
import WTContract
#if canImport(UIKit)
    import UIKit
#endif

/// The app's connection to the core: started once, shared by every window.
@MainActor
public final class BrowserSession {
    public let core: any CoreClient
    public let engine: WebKitEngine
    /// The file the system's preview shows over a window (030): an iPhone's and an iPad's way
    /// to open a download.
    public let preview: FilePreview
    private var windows: [UInt64: @MainActor (WTWindowSnapshot) -> Void] = [:]
    /// Windows being opened, and the snapshots that came for subscriptions not yet known to be
    /// theirs.
    private var opening = 0
    private var unclaimed: [UInt64: WTWindowSnapshot] = [:]
    private var listening: Task<Void, Never>?

    /// A session over a core that is already running. Tests pass a `ScriptedCore`. A link is
    /// copied to a clipboard of the app's own unless the session is given the system's (018).
    /// `preview`: what the session's opener of files shows a file in, where it has one (030).
    public init(core: any CoreClient, engine: WebKitEngine, clipboard: LinkClipboard = .ownOfTheApp, preview: FilePreview = FilePreview()) {
        self.core = core
        self.engine = engine
        self.preview = preview
        engine.linkMenu = .words(copyingTo: clipboard)
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
        /// Fake mode, for UI suites and screenshots: `WEBTIGER_FAKE=1` keeps the store in memory,
        /// and what pages keep too, and makes ids deterministic; `WEBTIGER_FAKE_START` names an
        /// address every new window opens. Nothing leaves the app in it: a link for the system
        /// opens no other app.
        ///
        /// A profile, for a test that must outlive a quit, or for running the app without
        /// touching one's own data: `WEBTIGER_PROFILE=<name>` keeps everything in a folder of
        /// that name, apart from the usual one; with `WEBTIGER_PROFILE_FRESH=1` the folder is
        /// emptied first. It is the real app otherwise: the store is on disk, and the tabs come
        /// back (009).
        public static func live(environment: [String: String] = ProcessInfo.processInfo.environment) throws -> BrowserSession {
            var config = WTCoreConfig()
            #if os(macOS)
                config.platform = .macos
            #else
                config.platform = .ios
            #endif
            config.locale = Locale.current.identifier(.bcp47)
            config.appVersion = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "0"
            config.engines = [WebKitEngine.info]
            (config.deviceKind, config.memoryMegabytes) = device()
            let opener: OutsideOpener
            // Where downloads are saved (013). In fake mode, a folder of the engine's own among
            // the temporary files: a UI suite writes nothing into anybody's Downloads.
            let downloads: URL?
            if environment["WEBTIGER_FAKE"] == "1" {
                config.testMode = .fake
                config.fake = fakeOptions(environment)
                opener = .nothing
                downloads = nil
            } else {
                config.testMode = .live
                let profile = Self.profileName(environment["WEBTIGER_PROFILE"])
                let folder = try dataDirectory(profile: profile)
                if profile != nil, environment["WEBTIGER_PROFILE_FRESH"] == "1" {
                    // Only ever a named profile's folder: never the usual one.
                    try? FileManager.default.removeItem(at: folder)
                }
                config.dataDir = folder.path(percentEncoded: false)
                opener = .system
                downloads = Self.downloadsFolder(profile: profile, data: folder)
            }
            let fake = config.testMode == .fake
            let engine = WebKitEngine(downloads: downloads, deviceAccess: deviceAccess(fake: fake), webData: webData(fake: fake))
            engine.iconFetching = iconFetching(fake: fake, environment: environment)
            #if os(macOS)
                // A page's file input opens the system's picker (019). Not in fake mode: its
                // engine picks nothing, and shows nothing of the system's.
                if config.testMode != .fake {
                    engine.filePicker = .system
                }
            #endif
            let preview = FilePreview()
            let files = filesOpener(fake: config.testMode == .fake, folder: engine.downloadsFolder, preview: preview)
            let core = try NativeCore.start(config: config, host: ShellHost(engine: engine, opener: opener, files: files))
            // In fake mode nothing of the person's is touched: not their clipboard either.
            return BrowserSession(
                core: core, engine: engine, clipboard: config.testMode == .fake ? .ownOfTheApp : .system, preview: preview
            )
        }

        /// Fake mode's knobs, as a suite set them in what the app was started with.
        static func fakeOptions(_ environment: [String: String]) -> WTFakeOptions {
            var fake = WTFakeOptions()
            fake.startAddress = environment["WEBTIGER_FAKE_START"] ?? ""
            // A first run, in which nobody has chosen where searches go (020).
            fake.firstRun = environment["WEBTIGER_FAKE_FIRST_RUN"] == "1"
            // How far the core's clock moves each time it is read, in seconds, for a suite
            // that must see a day pass (049). Without it: a second.
            fake.secondsARead = UInt32(environment["WEBTIGER_FAKE_CLOCK_STEP"] ?? "") ?? 0
            // How long a tab is out of sight before the core puts it to sleep, in seconds,
            // for a suite that must see one sleep (053). Without it: the usual half hour.
            fake.sleepAfterSeconds = UInt32(environment["WEBTIGER_FAKE_SLEEP_AFTER"] ?? "") ?? 0
            // The most tabs that are alive at once, for a suite that must see the limit met
            // (054). Without it fake mode has none, whatever it runs on.
            fake.liveTabs = UInt32(environment["WEBTIGER_FAKE_LIVE_TABS"] ?? "") ?? 0
            return fake
        }

        /// What the app runs on (054): the core works out from it how many tabs may be alive
        /// at once. A Mac is a desktop, an iPad a tablet and an iPhone a phone; the memory is
        /// the device's own, in megabytes. Said once, as the app starts.
        static func device() -> (kind: WTDeviceKind, megabytes: UInt32) {
            let megabytes = UInt32(clamping: ProcessInfo.processInfo.physicalMemory / (1024 * 1024))
            #if os(macOS)
                return (.desktop, megabytes)
            #else
                return (kind(of: UIDevice.current.userInterfaceIdiom), megabytes)
            #endif
        }

        #if !os(macOS)
            /// An iPad is a tablet; anything else the app runs on here is taken for a phone.
            static func kind(of idiom: UIUserInterfaceIdiom) -> WTDeviceKind {
                idiom == .pad ? .tablet : .phone
            }
        #endif

        /// What opens a downloaded file from where the app saves (030). On a Mac the system
        /// opens it with whatever it has for its kind; on an iPhone and an iPad the system's
        /// preview shows it. Not in fake mode: no test starts another app, and none shows
        /// anything of the system's.
        static func filesOpener(fake: Bool, folder: URL, preview: FilePreview) -> FileOpener {
            if fake {
                return .nothing
            }
            #if os(macOS)
                return .system(in: folder)
            #else
                return .preview(in: folder, showing: preview)
            #endif
        }

        /// Whether a page is let have what the core grants it of the device's (014). Not in
        /// fake mode: letting a page start a capture makes the system put its own dialog on
        /// the screen, which a UI suite must never do.
        static func deviceAccess(fake: Bool) -> DeviceAccess {
            fake ? .never : .asTheCoreSays
        }

        /// Where what pages keep is kept (their cookies, their storage). In fake mode in memory,
        /// as the core's store is: nothing of one run is there in the next, so that a suite's
        /// test finds nothing another left (the owner's, WT-D35). Otherwise on disk.
        static func webData(fake: Bool) -> WebKitEngine.WebData {
            fake ? .inMemory : .kept
        }

        /// Whether the app fetches a page's icon itself (046 §5a), and from where. Started for
        /// real, from the web. In fake mode from nowhere, unless a suite says so
        /// (`WEBTIGER_FAKE_ICONS=1`): and then from this device only, where its site is.
        static func iconFetching(fake: Bool, environment: [String: String]) -> WebKitEngine.IconFetching {
            guard fake else { return .web }
            return environment["WEBTIGER_FAKE_ICONS"] == "1" ? .thisDevice : .never
        }

        /// Where the app saves downloads (013): the system's place for them. A profile's go in
        /// its own folder, with the rest of its data and apart from the person's own.
        static func downloadsFolder(profile: String?, data: URL) -> URL? {
            profile == nil ? WebKitEngine.systemDownloads : data.appending(path: "Downloads", directoryHint: .isDirectory)
        }

        /// Application Support/WebTiger, made if it is not there; a profile's data is in
        /// Application Support/WebTiger Profiles/<name>.
        private static func dataDirectory(profile: String?) throws -> URL {
            let support = try FileManager.default.url(
                for: .applicationSupportDirectory,
                in: .userDomainMask,
                appropriateFor: nil,
                create: true
            )
            guard let profile else { return support.appending(path: "WebTiger", directoryHint: .isDirectory) }
            return support.appending(path: "WebTiger Profiles", directoryHint: .isDirectory)
                .appending(path: profile, directoryHint: .isDirectory)
        }

        /// A profile's name as a folder's: letters, digits and hyphens, at most forty. Anything
        /// else is no profile at all, and the usual data is used: a name is never a path.
        static func profileName(_ text: String?) -> String? {
            guard let text, !text.isEmpty, text.count <= 40 else { return nil }
            let allowed = text.unicodeScalars.allSatisfy { scalar in
                scalar.isASCII && (scalar.properties.isAlphabetic || scalar.properties.numericType != nil || scalar == "-")
            }
            return allowed ? text : nil
        }
    #endif

    /// Opens a window in the core and delivers its snapshots to `update` until it is closed.
    func openWindow(private asPrivate: Bool = false, update: @escaping @MainActor (WTWindowSnapshot) -> Void) async throws -> OpenedWindow {
        opening += 1
        defer {
            opening -= 1
            if opening == 0 {
                unclaimed.removeAll()
            }
        }
        // A private window (037) is asked for as one: its logins are its own, and nothing it
        // visits is kept.
        var open = WTOpenWindow()
        open.privateWindow = asPrivate
        guard case let .windowOpened(opened) = try await core.request(.openWindow(open)) else {
            throw CoreError.refused(code: "response.unexpected", field: "")
        }
        var subscribe = WTSubscribe()
        subscribe.window.windowID = opened.windowID
        guard case let .subscribed(subscribed) = try await core.request(.subscribe(subscribe)) else {
            throw CoreError.refused(code: "response.unexpected", field: "")
        }
        windows[subscribed.subscriptionID] = update
        // The first snapshot follows the answer at once, and may have been taken off the events
        // before this was told its subscription's id.
        if let first = unclaimed.removeValue(forKey: subscribed.subscriptionID) {
            update(first)
        }
        return OpenedWindow(windowID: opened.windowID, tabID: opened.tabID, subscriptionID: subscribed.subscriptionID)
    }

    func closeWindow(_ window: OpenedWindow) {
        windows[window.subscriptionID] = nil
        var close = WTCloseWindow()
        close.windowID = window.windowID
        _ = core.send(.closeWindow(close))
    }

    private func receive(_ event: WTEvent.OneOf_Kind) {
        guard case let .snapshot(snapshot) = event, case let .window(window) = snapshot.model else { return }
        if let update = windows[snapshot.subscriptionID] {
            update(window)
        } else if opening > 0 {
            unclaimed[snapshot.subscriptionID] = window
        }
    }
}

struct OpenedWindow: Equatable {
    let windowID: UInt64
    let tabID: UInt64
    let subscriptionID: UInt64
}

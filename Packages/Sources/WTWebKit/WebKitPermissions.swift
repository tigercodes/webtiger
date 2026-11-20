// A page asks to use the camera, the microphone or the device's location (014), on WebKit.
// The adapter reports the request and waits: the core says how the page is answered. It
// decides nothing about the site.
//
// WebKit asks the app about a camera or a microphone since macOS 12 and iOS 15, and about the
// location since macOS 27 and iOS 27; on older systems a page's request for location is
// WebKit's own to answer. When a page is let have a camera or a microphone, the system itself
// asks the person whether the app may use it, the first time, with a dialog of its own.
import WebKit
import WTContract

/// Whether an engine lets a page have what the core grants it.
public enum DeviceAccess: Sendable {
    /// Never: the page is told no, whatever the core said. What every engine is unless it is
    /// made otherwise, so every engine a test makes and fake mode's: letting a page start a
    /// capture makes the system put its own dialog on the screen, and nothing but the app
    /// itself, started for real, may bring that up.
    case never
    /// As the core says.
    case asTheCoreSays
}

/// The requests of one engine's pages that wait for the core's word.
@MainActor
final class Permissions {
    private struct Waiting {
        let webViewID: UInt64
        let answer: (Bool) -> Void
    }

    private let access: DeviceAccess
    private let report: @MainActor (UInt64, WTEngineEvent.OneOf_Kind) -> Void
    /// How many requests this engine has named. Each has the next number, and none is used
    /// twice: the ids are the shell's own (engine.proto, `PermissionRequested`).
    private var named: UInt64 = 0
    private var waiting: [UInt64: Waiting] = [:]

    init(access: DeviceAccess, report: @escaping @MainActor (UInt64, WTEngineEvent.OneOf_Kind) -> Void) {
        self.access = access
        self.report = report
    }

    /// A page of `webViewID` asks, as `origin`, for these. It is reported, and the page waits
    /// until the core has said: true when the page may have what it asked for.
    func ask(webViewID: UInt64, origin: String, kinds: [WTPermissionKind]) async -> Bool {
        named += 1
        let id = named
        var asked = WTPermissionRequested()
        asked.requestID = id
        asked.origin = origin
        asked.kinds = kinds
        return await withCheckedContinuation { continuation in
            waiting[id] = Waiting(webViewID: webViewID) { continuation.resume(returning: $0) }
            report(webViewID, .permissionRequested(asked))
        }
    }

    /// The core's word on a request. One that no longer waits has no page to answer.
    func answer(_ id: UInt64, granted: Bool) {
        waiting.removeValue(forKey: id)?.answer(granted && access == .asTheCoreSays)
    }

    /// A web view is going: what its pages asked is refused. WebKit must be given its
    /// answer: a decision it never gets is the app's fault, and it says so by stopping it.
    func drop(webViewID: UInt64) {
        for (id, request) in waiting where request.webViewID == webViewID {
            waiting[id] = nil
            request.answer(false)
        }
    }

    /// What WebKit's kind of capture asks for, as the contract's kinds.
    static func kinds(of type: WKMediaCaptureType) -> [WTPermissionKind] {
        switch type {
        case .camera: [.camera]
        case .microphone: [.microphone]
        case .cameraAndMicrophone: [.camera, .microphone]
        @unknown default: []
        }
    }

    /// Who asks, as the contract writes an origin: scheme, host, and the port when WebKit
    /// gives one (it gives 0 for the scheme's own).
    static func origin(scheme: String, host: String, port: Int) -> String {
        // An address of the newer kind is written in brackets where a port may follow.
        let named = host.contains(":") && !host.hasPrefix("[") ? "[\(host)]" : host
        return port > 0 ? "\(scheme)://\(named):\(port)" : "\(scheme)://\(named)"
    }
}

/// What WebKit asks the page's delegate (the rest of which is beside the page, in
/// WebKitEngine.swift).
extension Page {
    /// A page asks for the camera, the microphone or both (014). It is reported, with the
    /// frame that asked as who asks, and waits for the core's word. WebKit is never left to
    /// ask the person itself.
    func webView(
        _: WKWebView,
        decideMediaCapturePermissionsFor _: WKSecurityOrigin,
        initiatedBy frame: WKFrameInfo,
        type: WKMediaCaptureType
    ) async -> WKPermissionDecision {
        await asks(frame, for: Permissions.kinds(of: type))
    }

    /// A page asks where the device is (014). WebKit asks the app since system 27; before
    /// that it is WebKit's own to answer.
    @available(macOS 27.0, iOS 27.0, *)
    func webView(
        _: WKWebView,
        requestGeolocationPermissionFor _: WKSecurityOrigin,
        initiatedBy frame: WKFrameInfo
    ) async -> WKPermissionDecision {
        await asks(frame, for: [.location])
    }

    private func asks(_ frame: WKFrameInfo, for kinds: [WTPermissionKind]) async -> WKPermissionDecision {
        guard let engine, !kinds.isEmpty else { return .deny }
        let who = frame.securityOrigin
        let origin = Permissions.origin(scheme: who.protocol, host: who.host, port: who.port)
        return await engine.permissions.ask(webViewID: id, origin: origin, kinds: kinds) ? .grant : .deny
    }
}

public import WTContract

// A core that does what a test tells it to: a screen is tested without the Rust core, by
// scripting the answers and pushing the snapshots it should draw.
import Foundation
import Synchronization

public final class ScriptedCore: CoreClient {
    public typealias Script = @Sendable (WTRequest.OneOf_Kind) throws -> WTResponse.OneOf_Kind

    private struct State {
        var requests: [WTRequest.OneOf_Kind] = []
        var reports: [WTEngineEvent] = []
        var verdicts: [String: NavigationVerdict] = [:]
        var stopped = false
    }

    private let state = Mutex(State())
    private let script: Script
    public let events: AsyncStream<WTEvent.OneOf_Kind>
    private let continuation: AsyncStream<WTEvent.OneOf_Kind>.Continuation

    /// A core that answers every request with what `script` returns. The default answers Ack.
    public init(script: @escaping Script = { _ in .ack(WTAck()) }) {
        self.script = script
        (events, continuation) = AsyncStream.makeStream()
    }

    /// Every request made, in order.
    public var requests: [WTRequest.OneOf_Kind] {
        state.withLock { $0.requests }
    }

    /// Every report made, in order.
    public var reports: [WTEngineEvent] {
        state.withLock { $0.reports }
    }

    public var isStopped: Bool {
        state.withLock { $0.stopped }
    }

    /// Sends the shell an event, as the core would.
    public func emit(_ event: WTEvent.OneOf_Kind) {
        continuation.yield(event)
    }

    /// Sends the shell a window's snapshot.
    public func emit(_ window: WTWindowSnapshot, subscription: UInt64) {
        var snapshot = WTSnapshot()
        snapshot.subscriptionID = subscription
        snapshot.window = window
        emit(.snapshot(snapshot))
    }

    /// Sets the navigation policy's answer for an address. Addresses not set are allowed.
    public func setVerdict(_ verdict: NavigationVerdict, for url: String) {
        state.withLock { $0.verdicts[url] = verdict }
    }

    public func request(_ kind: WTRequest.OneOf_Kind) async throws -> WTResponse.OneOf_Kind {
        let stopped = state.withLock { state in
            state.requests.append(kind)
            return state.stopped
        }
        if stopped {
            throw CoreError.stopped
        }
        return try script(kind)
    }

    public func report(_ event: WTEngineEvent) {
        state.withLock { $0.reports.append(event) }
    }

    public func navigationVerdict(for url: String, webViewID _: UInt64, mainFrame _: Bool, userGesture _: Bool) -> NavigationVerdict {
        state.withLock { $0.verdicts[url] } ?? .allow
    }

    public func stop() {
        state.withLock { $0.stopped = true }
        continuation.finish()
    }
}

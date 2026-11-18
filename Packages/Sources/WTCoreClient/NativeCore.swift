// The Rust core behind its C ABI (Core/crates/wt-ffi/include/webtiger_core.h).
#if WEBTIGER_CORE
    public import WTContract
    import Foundation
    import Synchronization
    import WebTigerCore

    /// The running core. One per process.
    public final class NativeCore: CoreClient {
        /// The contract this shell was built with.
        public static let contractMajor = UInt32(WT_CONTRACT_MAJOR)
        public static let contractMinor = UInt32(WT_CONTRACT_MINOR)

        /// Immutable, so the fast path reads it without a lock.
        private let handle: UInt64
        private let receiver: Receiver
        private let context: Unmanaged<Receiver>
        private let stopped = Mutex(false)

        public var events: AsyncStream<WTEvent.OneOf_Kind> {
            receiver.events
        }

        /// Starts the core. `config` needs no contract version: it is filled in here.
        public static func start(config: WTCoreConfig, host: any HostServices) throws -> NativeCore {
            let library = wt_core_version()
            guard library >> 16 == contractMajor else { throw CoreError.contractMismatch(library: library) }
            var config = config
            config.contractMajor = contractMajor
            config.contractMinor = contractMinor
            let bytes = try config.serializedData()

            let receiver = Receiver()
            // The receiver is the sink's context. It is retained until the core has stopped, after
            // which the sink is never called again.
            let context = Unmanaged.passRetained(receiver)
            let handle = bytes.withUnsafeBytes { buffer in
                wt_core_start(buffer.bindMemory(to: UInt8.self).baseAddress, buffer.count, sink, context.toOpaque())
            }
            guard handle != 0 else {
                context.release()
                throw CoreError.startRefused(code: receiver.fatalCode ?? "start.failed")
            }
            return NativeCore(handle: handle, receiver: receiver, context: context, host: host)
        }

        private init(handle: UInt64, receiver: Receiver, context: Unmanaged<Receiver>, host: any HostServices) {
            self.handle = handle
            self.receiver = receiver
            self.context = context
            // One task answers host requests, one after the other, so they are performed in the
            // order the core made them.
            let requests = receiver.hostRequests
            Task { @MainActor in
                for await (id, kind) in requests {
                    var response = WTHostResponse()
                    response.kind = await host.answer(kind)
                    var envelope = WTEnvelope()
                    envelope.id = id
                    envelope.hostResponse = response
                    _ = Self.send(envelope, to: handle)
                }
            }
        }

        deinit {
            stop()
        }

        public func request(_ kind: WTRequest.OneOf_Kind) async throws -> WTResponse.OneOf_Kind {
            var request = WTRequest()
            request.kind = kind
            let handle = handle
            let receiver = receiver
            return try await withCheckedThrowingContinuation { continuation in
                guard let id = receiver.wait(continuation) else {
                    continuation.resume(throwing: CoreError.stopped)
                    return
                }
                var envelope = WTEnvelope()
                envelope.id = id
                envelope.request = request
                let status = Self.send(envelope, to: handle)
                if status != WT_OK {
                    receiver.fail(id, with: status == WT_UNKNOWN_CORE ? CoreError.stopped : CoreError.notSent(status: status))
                }
            }
        }

        public func report(_ event: WTEngineEvent) {
            var request = WTRequest()
            request.engineEvent = event
            var envelope = WTEnvelope()
            // Id 0: a notification, which the core does not answer.
            envelope.request = request
            _ = Self.send(envelope, to: handle)
        }

        public func navigationVerdict(for url: String, webViewID: UInt64, mainFrame: Bool, userGesture: Bool) -> NavigationVerdict {
            var flags: UInt32 = 0
            if mainFrame {
                flags |= UInt32(WT_NAV_MAIN_FRAME)
            }
            if userGesture {
                flags |= UInt32(WT_NAV_USER_GESTURE)
            }
            var url = url
            let action = url.withUTF8 { text in
                var navigation = wt_navigation(url: text.baseAddress, url_len: text.count, web_view_id: webViewID, flags: flags)
                // The library writes CANCEL on any failure; this is the same answer if it writes nothing.
                var verdict = wt_nav_verdict(action: UInt32(WT_NAV_CANCEL))
                _ = wt_nav_check(handle, &navigation, &verdict)
                return verdict.action
            }
            switch action {
            case UInt32(WT_NAV_ALLOW): return .allow
            case UInt32(WT_NAV_EXTERNAL): return .external
            default: return .cancel
            }
        }

        public func stop() {
            let first = stopped.withLock { stopped in
                defer { stopped = true }
                return !stopped
            }
            guard first else { return }
            // Returns once the core's thread has ended: the sink is silent from here on.
            _ = wt_core_stop(handle)
            receiver.finish()
            context.release()
        }

        private static func send(_ envelope: WTEnvelope, to handle: UInt64) -> Int32 {
            guard let bytes = try? envelope.serializedData() else { return WT_MALFORMED }
            return bytes.withUnsafeBytes { buffer in
                wt_core_send(handle, buffer.bindMemory(to: UInt8.self).baseAddress, buffer.count)
            }
        }
    }

    /// The sink: called on the core's own thread with bytes that are valid only for the call.
    private let sink: wt_sink = { context, bytes, length in
        guard let context else { return }
        let receiver = Unmanaged<Receiver>.fromOpaque(context).takeUnretainedValue()
        let data = bytes.map { Data(bytes: $0, count: length) } ?? Data()
        receiver.receive(data)
    }

    /// What the sink delivers to: the requests waiting for an answer, and the streams events and
    /// host requests leave through.
    private final class Receiver: Sendable {
        private struct State {
            var nextID: UInt64 = 1
            var waiting: [UInt64: CheckedContinuation<WTResponse.OneOf_Kind, any Error>] = [:]
            var finished = false
            var fatalCode: String?
        }

        private let state = Mutex(State())
        let events: AsyncStream<WTEvent.OneOf_Kind>
        private let eventsContinuation: AsyncStream<WTEvent.OneOf_Kind>.Continuation
        let hostRequests: AsyncStream<(UInt64, WTHostRequest.OneOf_Kind)>
        private let hostContinuation: AsyncStream<(UInt64, WTHostRequest.OneOf_Kind)>.Continuation

        init() {
            (events, eventsContinuation) = AsyncStream.makeStream()
            (hostRequests, hostContinuation) = AsyncStream.makeStream()
        }

        /// The code of a STATE_FATAL status, when one arrived: why a start was refused.
        var fatalCode: String? {
            state.withLock { $0.fatalCode }
        }

        /// Registers a request's continuation. Its id, or nil when the core has stopped.
        func wait(_ continuation: CheckedContinuation<WTResponse.OneOf_Kind, any Error>) -> UInt64? {
            state.withLock { state in
                guard !state.finished else { return nil }
                let id = state.nextID
                state.nextID += 1
                state.waiting[id] = continuation
                return id
            }
        }

        func fail(_ id: UInt64, with error: CoreError) {
            state.withLock { $0.waiting.removeValue(forKey: id) }?.resume(throwing: error)
        }

        func receive(_ data: Data) {
            guard let envelope = try? WTEnvelope(serializedBytes: data) else { return }
            switch envelope.body {
            case let .response(response):
                let waiting = state.withLock { $0.waiting.removeValue(forKey: envelope.id) }
                switch response.kind {
                case let .error(error): waiting?.resume(throwing: CoreError.refused(code: error.code, field: error.field))
                case let .some(kind): waiting?.resume(returning: kind)
                case nil: waiting?.resume(throwing: CoreError.refused(code: "response.empty", field: ""))
                }
            case let .event(event):
                guard let kind = event.kind else { return }
                if case let .status(status) = kind, status.state == .fatal {
                    state.withLock { $0.fatalCode = status.code }
                }
                eventsContinuation.yield(kind)
            case let .hostRequest(request):
                guard let kind = request.kind else { return }
                hostContinuation.yield((envelope.id, kind))
            case .request, .hostResponse, nil:
                break
            }
        }

        /// The core has stopped: nothing more will arrive.
        func finish() {
            let waiting = state.withLock { state in
                state.finished = true
                defer { state.waiting = [:] }
                return state.waiting
            }
            for continuation in waiting.values {
                continuation.resume(throwing: CoreError.stopped)
            }
            eventsContinuation.finish()
            hostContinuation.finish()
        }
    }
#endif

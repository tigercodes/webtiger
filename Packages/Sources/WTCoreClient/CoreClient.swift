// The core as Swift sees it (docs/features/001-foundations.md §6): requests in, answers and
// snapshots out, host services answered. The shell holds no browser logic; it asks here.
public import WTContract

/// Why a call to the core failed.
public enum CoreError: Error, Equatable, Sendable {
    /// The core answered with an error. `code` is stable ("tab.unknown"); the shell turns it
    /// into words.
    case refused(code: String, field: String)
    /// The core would not start, with the code its CoreStatus carried ("start.data_dir").
    case startRefused(code: String)
    /// The library speaks another major version of the contract.
    case contractMismatch(library: UInt32)
    /// `wt_core_send` returned this status instead of taking the message.
    case notSent(status: Int32)
    /// The core has stopped.
    case stopped
}

/// What the navigation policy says about a navigation a web view is about to make.
public enum NavigationVerdict: Equatable, Sendable {
    case allow
    case cancel
    /// Not a page: refuse it in the web view and hand the address to the system.
    case external
}

/// What the core asks the platform for (000 §5.5). The engine adapter answers.
public protocol HostServices: AnyObject, Sendable {
    /// Answers one host request. Requests arrive, and are answered, in the order the core
    /// made them: a web view's creation always comes before its first load.
    @MainActor func answer(_ request: WTHostRequest.OneOf_Kind) async -> WTHostResponse.OneOf_Kind

    /// Begins a host request that takes its time, now, in its turn, and gives back the wait
    /// for its answer: the requests after it are answered meanwhile, and its own answer is
    /// sent when it comes. `nil` for a request that is answered in its turn, which is every
    /// one but a web view's picture (059): an engine may take seconds to draw one, and the
    /// next request may be the web view of the tab a person has just opened.
    @MainActor func begin(_ request: WTHostRequest.OneOf_Kind) -> (@MainActor @Sendable () async -> WTHostResponse.OneOf_Kind)?
}

public extension HostServices {
    /// A host that takes its time over nothing.
    @MainActor func begin(_: WTHostRequest.OneOf_Kind) -> (@MainActor @Sendable () async -> WTHostResponse.OneOf_Kind)? {
        nil
    }
}

/// The core. `NativeCore` is the real one; `ScriptedCore` stands in for it in tests.
public protocol CoreClient: AnyObject, Sendable {
    /// Sends a request now, and returns the wait for its answer. The request has left for the
    /// core when this returns, so requests sent one after the other from one thread reach the
    /// core in that order: what a person did first, the core hears first. (An `async` function
    /// does not promise that by itself: two tasks that each call one may run in either order.)
    ///
    /// The answer's wait throws `CoreError.refused` when the answer is an error. It need not
    /// be waited for.
    func send(_ kind: WTRequest.OneOf_Kind) -> CoreAnswer
    /// Reports what a web view did. A notification: nothing comes back.
    func report(_ event: WTEngineEvent)
    /// Everything the core says unasked, in order. One consumer.
    var events: AsyncStream<WTEvent.OneOf_Kind> { get }
    /// The fast path: whether a web view may make a navigation. Synchronous, and callable
    /// from any thread.
    func navigationVerdict(for url: String, webViewID: UInt64, mainFrame: Bool, userGesture: Bool) -> NavigationVerdict
    /// Stops the core. Requests still waiting throw `CoreError.stopped`.
    func stop()
}

/// The wait for the answer to a request that has been sent.
public typealias CoreAnswer = Task<WTResponse.OneOf_Kind, any Error>

public extension CoreClient {
    /// Sends a request and waits for its answer. Throws `CoreError.refused` when the answer
    /// is an error.
    func request(_ kind: WTRequest.OneOf_Kind) async throws -> WTResponse.OneOf_Kind {
        try await send(kind).value
    }

    /// Sends a request whose answer must be an Ack.
    func perform(_ kind: WTRequest.OneOf_Kind) async throws {
        _ = try await request(kind)
    }
}

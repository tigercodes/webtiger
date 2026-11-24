// What is awake (056): every tab that has a page loaded, in every window and Space, and why
// each is awake; and the command that puts the others to sleep. The shell keeps no list of
// its own, and decides nothing: it asks.
public import Observation
public import WTContract
public import WTCoreClient

@MainActor
@Observable
public final class AwakeModel {
    /// The tabs that are awake, in the core's order: this window's first.
    public private(set) var tabs: [WTAwakeTab] = []
    /// How many more are awake in private windows, and are not named here.
    public private(set) var privateTabs = 0
    /// How many tabs are asleep.
    public private(set) var asleep = 0
    /// The core has answered at least once. Until then the list says nothing at all, rather
    /// than that no tab is awake.
    public private(set) var loaded = false

    private let core: any CoreClient
    private let windowID: UInt64
    /// Counts what was asked: an answer to anything but the last question is out of date.
    private var asked = 0

    /// What is awake, as a window is told it.
    public init(core: any CoreClient, windowID: UInt64) {
        self.core = core
        self.windowID = windowID
    }

    /// Asks what is awake now.
    public func reload() {
        asked += 1
        let question = asked
        var query = WTQueryAwake()
        query.windowID = windowID
        // Sent now, after whatever was sent before it: a question that follows the command is
        // answered with what the command left awake.
        let answer = core.send(.queryAwake(query))
        Task { [weak self] in
            guard case let .awakePage(page) = try? await answer.value, let self, question == asked else { return }
            tabs = page.tabs
            privateTabs = Int(page.privateTabs)
            asleep = Int(page.asleep)
            loaded = true
        }
    }

    /// Whether the command would do anything: some tab that is listed is out of sight with
    /// nothing to keep it awake. (The core decides which sleep: this only says whether the
    /// button is worth pressing.)
    public var othersCouldSleep: Bool {
        tabs.contains { !$0.inSight && $0.keepsAwake.isEmpty && !$0.privateWindow }
    }

    /// "Sleep all others": the core puts to sleep every tab that can be, and is asked again
    /// for what is left.
    public func sleepOthers() {
        var command = WTSleepOthers()
        command.windowID = windowID
        _ = core.send(.sleepOthers(command))
        reload()
    }
}

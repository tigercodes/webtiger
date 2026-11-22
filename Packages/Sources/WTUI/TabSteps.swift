// From tab to tab by key (035). Which tab is the next, the one before, the last, or at a place
// is the core's to say, by the row the window shows; the window asks.
public import WTContract

public extension BrowserModel {
    /// Asks the core to bring another tab of the window in front, by where it is in the row:
    /// the next, the one before, or the last.
    func stepTab(_ to: WTStepTab.To) {
        step(to, place: 0)
    }

    /// Asks the core to bring the tab at a place in front, counted from one. Beyond the row,
    /// the core does nothing.
    func showTab(at place: Int) {
        guard place > 0 else { return }
        step(.place, place: UInt32(clamping: place))
    }

    private func step(_ to: WTStepTab.To, place: UInt32) {
        guard let windowID else { return }
        var step = WTStepTab()
        step.windowID = windowID
        step.to = to
        step.place = place
        send(.stepTab(step))
    }
}

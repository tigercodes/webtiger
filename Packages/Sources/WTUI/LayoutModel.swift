import WTContract

// A window's layout (043), the model's part: whether this window draws its tabs down the side,
// and what it asks of the core to switch. Which layout a window has, and that the choice is
// the next window's, are the core's.
import Foundation

public extension BrowserModel {
    /// Whether a window has room for a sidebar before it has said how wide it is: on a Mac,
    /// where every window has; not on a touch screen, where the window says.
    internal static var startsWithRoomForSidebar: Bool {
        #if os(macOS)
            true
        #else
            false
        #endif
    }

    /// The window draws its tabs down the side: the core says the sidebar, and the window
    /// has room for one. Where it has none, the strip is drawn whatever the core says.
    var showsSidebar: Bool {
        hasRoomForSidebar && layout == .sidebar
    }

    /// Switches the window's layout: to the other of what the core last said it is. Not
    /// where there is no room for a sidebar: there is nothing to switch to.
    func switchLayout() {
        guard hasRoomForSidebar, let windowID else { return }
        var ask = WTSetLayout()
        ask.windowID = windowID
        ask.layout = layout == .sidebar ? .strip : .sidebar
        send(.setLayout(ask))
    }
}

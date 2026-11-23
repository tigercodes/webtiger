import WTContract

// A window's layout (043), the model's part: whether this window draws its tabs down the side,
// and what it asks of the core to switch. Which layout a window has, and that the choice is
// the next window's, are the core's.
import Foundation

public extension BrowserModel {
    /// This system can draw a sidebar: the switch is offered. So far the Mac; the iPad is
    /// next (043 §7), and a phone has none.
    static var hasSidebar: Bool {
        #if os(macOS)
            true
        #else
            false
        #endif
    }

    /// The window draws its tabs down the side: the core says the sidebar, and there is one
    /// here to draw. Where there is none, the strip is drawn whatever the core says.
    var showsSidebar: Bool {
        Self.hasSidebar && layout == .sidebar
    }

    /// Switches the window's layout: to the other of what the core last said it is.
    func switchLayout() {
        guard Self.hasSidebar, let windowID else { return }
        var ask = WTSetLayout()
        ask.windowID = windowID
        ask.layout = layout == .sidebar ? .strip : .sidebar
        send(.setLayout(ask))
    }
}

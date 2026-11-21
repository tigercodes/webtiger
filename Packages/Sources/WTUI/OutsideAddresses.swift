// A link pressed in another app (031): the system hands the app its address, and the window's
// model hands it to the core as it was given. Whether it is an address to open, and where it
// opens, is the core's to say: only an address of the web is opened, in a new tab in front.
// Nothing is made of it here.
import SwiftUI
import WTContract

/// A window hears of a link pressed in another app (031). On a Mac the system hands the app
/// the address; the window that is there takes it, and says it takes what comes from outside,
/// so that the system opens no second window for a link. An iPhone hands links only to an app
/// with Apple's browser entitlement: nothing is heard there.
struct OutsideLinks: ViewModifier {
    let model: BrowserModel

    func body(content: Content) -> some View {
        #if os(macOS)
            content
                .onOpenURL { model.openFromOutside($0.absoluteString) }
                .handlesExternalEvents(preferring: ["*"], allowing: ["*"])
        #else
            content
        #endif
    }
}

public extension BrowserModel {
    /// The system handed the app an address from outside it. The core is told, with the
    /// window it is for. Before the window is open the address waits, with any others, in
    /// the order they came: each is a link somebody pressed.
    func openFromOutside(_ address: String) {
        guard let windowID else {
            addressesWaiting.append(address)
            return
        }
        var open = WTOpenAddress()
        open.windowID = windowID
        open.url = address
        send(.openAddress(open))
    }

    /// The window is open: what waited for it is handed over, once.
    internal func openWhatWaited() {
        let waited = addressesWaiting
        addressesWaiting = []
        waited.forEach(openFromOutside)
    }
}

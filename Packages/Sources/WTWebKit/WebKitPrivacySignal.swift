// Global Privacy Control (069) in WebKit: the signal that says "do not sell or share what you
// learn of me". A page's scripts read it as `navigator.globalPrivacyControl`; a server reads
// it as the header `Sec-GPC: 1`.
//
// WebKit has no setting for either. The property is made by a script of the shell's, run
// before any of a page's own, in every frame, in the page's own world: so a page reads it as
// it would read the engine's. The header is added to the loads the shell itself begins;
// WebKit gives a shell no way to add to what a page fetches of its own accord, and those go
// without it.
import Foundation
import WebKit

enum PrivacySignal {
    /// Makes the property, as one the page cannot write, take away or make again.
    static let script = """
    (function () {
        try {
            Object.defineProperty(navigator, 'globalPrivacyControl', {
                value: true, writable: false, configurable: false, enumerable: true
            });
        } catch (e) {}
    })();
    """

    /// Has every page of a web view made from this configuration told the signal.
    @MainActor
    static func tell(_ controller: WKUserContentController) {
        controller.addUserScript(WKUserScript(source: script, injectionTime: .atDocumentStart, forMainFrameOnly: false))
    }

    /// A load the shell begins: with the header, where the web view's pages are told the
    /// signal.
    static func request(for url: URL, told: Bool) -> URLRequest {
        var request = URLRequest(url: url)
        if told {
            request.setValue("1", forHTTPHeaderField: "Sec-GPC")
        }
        return request
    }
}

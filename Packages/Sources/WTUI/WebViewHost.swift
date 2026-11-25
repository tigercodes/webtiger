// Shows the web view the core named. The view is the engine's; this only gives it a place.
//
// While the page waits for an answer to what it said (016), its process answers nothing: not
// the page's own script, and not an assistive technology that asks what is on it. So for that
// long the page is not offered to assistive technologies at all, as it is not while a sheet
// is over a window: what there is to read and to answer is the bar.
import SwiftUI
import WebKit
import WTWebKit

#if os(macOS)
    struct WebViewHost: NSViewRepresentable {
        let engine: WebKitEngine
        let webViewID: UInt64
        /// How many web views the engine has made: when it changes, the web view the core
        /// named is looked for again. It may not have been made when the window was told.
        var made = 0
        /// The page waits for an answer to what it said.
        var pageWaits = false

        func makeNSView(context _: Context) -> WebViewContainer {
            WebViewContainer()
        }

        func updateNSView(_ container: WebViewContainer, context _: Context) {
            container.show(engine.webView(for: webViewID))
            container.pageWaits = pageWaits
        }
    }

    final class WebViewContainer: NSView {
        private weak var shown: WKWebView?
        /// The page waits for an answer to what it said: nothing of it is offered to
        /// assistive technologies meanwhile.
        var pageWaits = false

        override func accessibilityChildren() -> [Any]? {
            pageWaits ? [] : super.accessibilityChildren()
        }

        func show(_ webView: WKWebView?) {
            guard shown !== webView else { return }
            shown?.removeFromSuperview()
            shown = webView
            guard let webView else { return }
            webView.frame = bounds
            webView.autoresizingMask = [.width, .height]
            addSubview(webView)
        }
    }
#else
    struct WebViewHost: UIViewRepresentable {
        let engine: WebKitEngine
        let webViewID: UInt64
        /// How many web views the engine has made: when it changes, the web view the core
        /// named is looked for again. It may not have been made when the window was told.
        var made = 0
        /// The page waits for an answer to what it said.
        var pageWaits = false

        func makeUIView(context _: Context) -> WebViewContainer {
            WebViewContainer()
        }

        func updateUIView(_ container: WebViewContainer, context _: Context) {
            container.show(engine.webView(for: webViewID))
            container.accessibilityElementsHidden = pageWaits
        }
    }

    final class WebViewContainer: UIView {
        private weak var shown: WKWebView?

        func show(_ webView: WKWebView?) {
            guard shown !== webView else { return }
            shown?.removeFromSuperview()
            shown = webView
            guard let webView else { return }
            webView.frame = bounds
            webView.autoresizingMask = [.flexibleWidth, .flexibleHeight]
            addSubview(webView)
        }
    }
#endif

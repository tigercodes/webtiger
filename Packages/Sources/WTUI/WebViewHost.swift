// Shows the web view the core named. The view is the engine's; this only gives it a place.
import SwiftUI
import WebKit
import WTWebKit

#if os(macOS)
    struct WebViewHost: NSViewRepresentable {
        let engine: WebKitEngine
        let webViewID: UInt64

        func makeNSView(context _: Context) -> WebViewContainer {
            WebViewContainer()
        }

        func updateNSView(_ container: WebViewContainer, context _: Context) {
            container.show(engine.webView(for: webViewID))
        }
    }

    final class WebViewContainer: NSView {
        private weak var shown: WKWebView?

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

        func makeUIView(context _: Context) -> WebViewContainer {
            WebViewContainer()
        }

        func updateUIView(_ container: WebViewContainer, context _: Context) {
            container.show(engine.webView(for: webViewID))
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

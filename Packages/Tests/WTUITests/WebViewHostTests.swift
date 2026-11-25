// The place a window gives its tab's web view. The core tells a window the number of the web
// view as soon as it has asked the shell for one, and the shell makes it a moment later: when
// the shell is busy with something else (a picture being drawn, 059), seconds later. A
// window that looked for it in between found none, and must look again when it is made.
import SwiftUI
import WebKit
import WTContract
import WTCoreClient
@testable import WTUI
import WTWebKit
import XCTest

@MainActor
final class WebViewHostTests: XCTestCase {
    /// A window's page area, as the window draws it: with how many web views the engine has
    /// made, which is what makes it look again.
    private struct Hosted: View {
        let engine: WebKitEngine
        let webViewID: UInt64
        var looksAgain = true

        var body: some View {
            WebViewHost(engine: engine, webViewID: webViewID, made: looksAgain ? engine.made.count : 0)
        }
    }

    private func make(_ id: UInt64, in engine: WebKitEngine) async {
        var create = WTWebViewCreate()
        create.webViewID = id
        create.engine = .webkit
        create.loginSet.id = UUID().uuidString.lowercased()
        create.loginSet.ephemeral = true
        guard case .ack = await engine.answer(.webViewCreate(create)) else { return XCTFail("a web view is made") }
    }

    private func destroy(_ id: UInt64, in engine: WebKitEngine) async {
        var destroy = WTWebViewDestroy()
        destroy.webViewID = id
        _ = await engine.answer(.webViewDestroy(destroy))
    }

    /// Lets the window draw itself, a few times over: what it would do by itself on a screen.
    private func settle(_ stage: Snapshot.Stage, until done: () -> Bool) async {
        for _ in 0 ..< 100 where !done() {
            _ = stage.picture()
            await Task.yield()
            try? await Task.sleep(for: .milliseconds(5))
        }
    }

    func testAWebViewMadeAfterTheWindowWasToldItsNumberIsShown() async {
        let engine = WebKitEngine()
        // The window is told of web view 7, which the engine has not made yet: there is
        // nothing to show.
        let stage = Snapshot.Stage(Hosted(engine: engine, webViewID: 7), size: CGSize(width: 300, height: 200), dark: false)
        _ = stage.picture()
        XCTAssertNil(engine.webView(for: 7))

        // It is made, a moment later: the window looks again, and it is in the window.
        await make(7, in: engine)
        await settle(stage) { engine.webView(for: 7)?.superview != nil }
        let webView = engine.webView(for: 7)
        XCTAssertNotNil(webView?.superview, "the web view is given its place when it is made")
        XCTAssertNotNil(webView?.window)
        XCTAssertEqual(webView?.frame.size, CGSize(width: 300, height: 200), "and is as large as its place")

        // Another web view made for another tab changes nothing here.
        await make(8, in: engine)
        await settle(stage) { false }
        XCTAssertNotNil(engine.webView(for: 7)?.superview)
        XCTAssertNil(engine.webView(for: 8)?.superview, "a web view the window was not told of is not shown")
        await destroy(7, in: engine)
        await destroy(8, in: engine)
    }

    /// Lets a scripted core's script reach the core it is the script of.
    private final class Told: @unchecked Sendable {
        var core: ScriptedCore?
    }

    func testTheWindowShowsItsTabsWebViewThoughItWasMadeAfterTheWindowWasTold() async {
        // The window itself, over a core that tells it of a tab with web view 7 at once, as
        // the core does: the engine has not made it.
        var said = WTWindowSnapshot()
        said.windowID = 1
        said.tab.tabID = 2
        said.tab.webViewID = 7
        said.tab.loadState = .loading
        said.tab.address.url = "https://example.com/"
        said.tab.address.display = "example.com"
        var tab = WTTabSummary()
        tab.tabID = 2
        tab.host = "example.com"
        tab.loading = true
        said.tabs = [tab]
        let published = said
        let told = Told()
        let core = ScriptedCore { request in
            switch request {
            case .openWindow:
                var opened = WTWindowOpened()
                opened.windowID = 1
                opened.tabID = 2
                return .windowOpened(opened)
            case .subscribe:
                told.core?.emit(published, subscription: 3)
                var subscribed = WTSubscribed()
                subscribed.subscriptionID = 3
                return .subscribed(subscribed)
            default:
                return .ack(WTAck())
            }
        }
        told.core = core
        let engine = WebKitEngine()
        let session = BrowserSession(core: core, engine: engine)
        let stage = Snapshot.Stage(BrowserWindowView(session: session, finding: nil), size: CGSize(width: 480, height: 400), dark: false)
        _ = await stage.settledPicture()
        XCTAssertNil(engine.webView(for: 7), "the window was told of a web view that is not made")

        // The shell makes it when it comes to that request: the window gives it its place.
        await make(7, in: engine)
        await settle(stage) { engine.webView(for: 7)?.superview != nil }
        XCTAssertNotNil(engine.webView(for: 7)?.superview, "the window shows the web view that was made after it was told")
        await destroy(7, in: engine)
    }

    func testAWindowThatDidNotLookAgainShowedNothing() async {
        // What was so before: the window told of a web view that is made afterwards, with
        // nothing to say that it was. It is the case the test above is about, and is here so
        // that the test above is known to be able to fail.
        let engine = WebKitEngine()
        let stage = Snapshot.Stage(
            Hosted(engine: engine, webViewID: 7, looksAgain: false),
            size: CGSize(width: 300, height: 200),
            dark: false
        )
        _ = stage.picture()
        await make(7, in: engine)
        await settle(stage) { false }
        XCTAssertNil(engine.webView(for: 7)?.superview, "nothing told the window to look again")
        await destroy(7, in: engine)
    }
}

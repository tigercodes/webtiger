// What the adapter suites share: a loopback site, a scripted core, the engine under test, and
// helpers that speak the WebViews host service as the core does.
import WebKit
import WTContract
import WTCoreClient
import WTWebKit
import XCTest

@MainActor
class AdapterTestCase: XCTestCase {
    // XCTest makes a new instance of the case for every test, so these are each test's own.
    let site = TestSite()
    let core = ScriptedCore()
    let engine = WebKitEngine()
    private var created: [UInt64] = []
    private var loginSets: [UUID] = []

    override func setUp() async throws {
        XCTAssertNotEqual(site.port, 0, "the loopback site is listening")
        await WebKitWarmUp.run(loading: site.url("/"), titled: "Home")
        XCTAssertTrue(WebKitWarmUp.started, "WebKit loaded nothing from the loopback site in eight minutes: no test here can pass")
        engine.attach(core)
    }

    override func tearDown() async throws {
        for id in created {
            await destroy(id)
        }
        created = []
        for loginSet in loginSets {
            await remove(loginSet)
        }
        loginSets = []
        // What a test downloaded went to a folder of the engine's own (013), which goes with it.
        try? FileManager.default.removeItem(at: engine.downloadsFolder)
    }

    // MARK: Helpers

    func newLoginSet() -> UUID {
        let id = UUID()
        loginSets.append(id)
        return id
    }

    @discardableResult
    func create(
        _ id: UInt64,
        loginSet: UUID? = nil,
        ephemeral: Bool = false,
        engine kind: WTEngineKind = .webkit
    ) async -> WTHostResponse.OneOf_Kind {
        var create = WTWebViewCreate()
        create.webViewID = id
        create.engine = kind
        // A login set a test did not name is one of its own, deleted with the others.
        let loginSet = loginSet ?? (ephemeral ? UUID() : newLoginSet())
        create.loginSet.id = loginSet.uuidString.lowercased()
        create.loginSet.ephemeral = ephemeral
        let answer = await engine.answer(.webViewCreate(create))
        if case .ack = answer {
            created.append(id)
        }
        return answer
    }

    @discardableResult
    func load(_ id: UInt64, _ url: String) async -> WTHostResponse.OneOf_Kind {
        var load = WTWebViewLoad()
        load.webViewID = id
        load.url = url
        return await engine.answer(.webViewLoad(load))
    }

    func command(_ id: UInt64, _ command: WTNavigationCommand) async {
        var request = WTWebViewCommand()
        request.webViewID = id
        request.command = command
        _ = await engine.answer(.webViewCommand(request))
    }

    func destroy(_ id: UInt64) async {
        var destroy = WTWebViewDestroy()
        destroy.webViewID = id
        _ = await engine.answer(.webViewDestroy(destroy))
        created.removeAll { $0 == id }
    }

    /// Has a web view the engine made unasked destroyed with the others when the test ends.
    func track(_ id: UInt64) {
        created.append(id)
    }

    /// Runs script in a web view's page, as if somebody had clicked: WebKit runs what an app
    /// asks it to evaluate with a user's gesture.
    @discardableResult
    func run(_ script: String, in id: UInt64, file: StaticString = #filePath, line: UInt = #line) async -> Any? {
        guard let webView = engine.webView(for: id) else {
            XCTFail("no web view \(id)", file: file, line: line)
            return nil
        }
        return try? await webView.evaluateJavaScript(script)
    }

    /// The windows a web view's page has asked for, in order.
    func windowsAsked(by id: UInt64) -> [WTNewWindowRequested] {
        reports(id).compactMap {
            if case let .newWindowRequested(asked) = $0 {
                asked
            } else {
                nil
            }
        }
    }

    /// Waits for a web view's page to have asked for `count` windows, and has the last one
    /// destroyed when the test ends. The last one asked for.
    func awaitWindow(
        askedBy id: UInt64,
        count: Int = 1,
        file: StaticString = #filePath,
        line: UInt = #line
    ) async -> WTNewWindowRequested {
        await wait(id, "a window asked for", file: file, line: line) { _ in self.windowsAsked(by: id).count >= count }
        let asked = windowsAsked(by: id).last ?? WTNewWindowRequested()
        if asked.newWebViewID != 0 {
            track(asked.newWebViewID)
        }
        return asked
    }

    /// Deletes a login set, as the engine does it. False while a web view still uses it.
    @discardableResult
    func remove(_ loginSet: UUID) async -> Bool {
        await engine.deleteLoginSet(loginSet)
    }

    /// What the adapter has reported about a web view, in order.
    func reports(_ id: UInt64) -> [WTEngineEvent.OneOf_Kind] {
        core.reports.filter { $0.webViewID == id }.compactMap(\.kind)
    }

    /// Waits until the reports about a web view satisfy `done`, for up to a minute (`Patience`).
    func wait(
        _ id: UInt64,
        _ what: String,
        file: StaticString = #filePath,
        line: UInt = #line,
        until done: ([WTEngineEvent.OneOf_Kind]) -> Bool
    ) async {
        for _ in 0 ..< Patience.steps {
            if done(reports(id)) {
                return
            }
            try? await Task.sleep(for: .milliseconds(10))
        }
        XCTFail("timed out waiting for \(what); reported: \(names(reports(id)))", file: file, line: line)
    }

    func finished(_ reports: [WTEngineEvent.OneOf_Kind], times: Int = 1) -> Bool {
        reports.count(where: {
            if case .navigationFinished = $0 {
                true
            } else {
                false
            }
        }) >= times
    }

    func lastTitle(_ id: UInt64) -> String {
        reports(id).reversed().compactMap {
            if case let .titleChanged(changed) = $0 {
                changed.title
            } else {
                nil
            }
        }.first ?? ""
    }

    func hasTitle(_ title: String) -> ([WTEngineEvent.OneOf_Kind]) -> Bool {
        { reports in reports.contains {
            if case let .titleChanged(changed) = $0 {
                changed.title == title
            } else {
                false
            }
        } }
    }

    /// The reports' kinds, without progress, which arrives at the engine's own pace.
    func names(_ reports: [WTEngineEvent.OneOf_Kind]) -> [String] {
        reports.compactMap { report in
            switch report {
            case .navigationStarted: "started"
            case .navigationCommitted: "committed"
            case .navigationFinished: "finished"
            case let .navigationFailed(failed): "failed(\(failed.code))"
            case .addressChanged: "address"
            case .titleChanged: "title"
            case .historyChanged: "history"
            case .rendererGone: "gone"
            case .newWindowRequested: "window"
            case .closeRequested: "close"
            case .outsideLinkRequested: "outside"
            case .findResult: "find"
            case .downloadRequested: "download"
            case .downloadFinished: "downloaded"
            case let .downloadFailed(failed): "download failed(\(failed.code))"
            case .permissionRequested: "permission"
            case .permissionWithdrawn: "permission withdrawn"
            case let .dialogRequested(dialog): "says(\(dialog.kind))"
            case let .signInRequested(asked): asked.again ? "sign-in(again)" : "sign-in"
            case let .linkOpenRequested(asked): asked.background ? "link(behind)" : "link(in front)"
            case .searchRequested: "search"
            // An engine that hands over a page's icon (046). WebKit hands none.
            case .iconReceived: "icon"
            // What the page is doing that keeps its tab awake (052).
            case let .pageActivityChanged(doing):
                "doing(" + [doing.playsSound ? "sound" : nil, doing.captures ? "capture" : nil, doing.typedInto ? "typed" : nil]
                    .compactMap(\.self).joined(separator: ",") + ")"
            // At the engine's own pace, like a page's progress; and which icons a page
            // names (046 §5a), which the page is asked once it has loaded.
            case .progressChanged, .downloadProgress, .iconNamed: nil
            }
        }
    }

    #if os(macOS)

        // MARK: The mouse

        /// Puts a web view in a window that is not on the screen, so that it can be clicked.
        func window(for id: UInt64) throws -> NSWindow {
            let webView = try XCTUnwrap(engine.webView(for: id))
            let window = NSWindow(
                contentRect: NSRect(x: 0, y: 0, width: 480, height: 360),
                styleMask: [.borderless],
                backing: .buffered,
                defer: false
            )
            window.isReleasedWhenClosed = false
            window.contentView = webView
            return window
        }

        /// Where the middle of an element of the page is, as a window says where the mouse is.
        private func middle(of element: String, in id: UInt64) async throws -> NSPoint {
            let webView = try XCTUnwrap(engine.webView(for: id))
            let middle = await run(
                "(() => { const r = document.getElementById('\(element)').getBoundingClientRect(); "
                    + "return [r.x + r.width / 2, r.y + r.height / 2]; })()",
                in: id
            )
            let place = try XCTUnwrap(middle as? [Double])
            // The page counts down from its top; a window counts up from its bottom.
            let inWebView = NSPoint(x: place[0], y: webView.isFlipped ? place[1] : Double(webView.bounds.height) - place[1])
            return webView.convert(inWebView, to: nil)
        }

        /// Clicks the middle of an element of the page with the mouse, as the system gives a
        /// click to a window: with the keys that are held.
        func click(_ element: String, in id: UInt64, of window: NSWindow, holding keys: NSEvent.ModifierFlags) async throws {
            let webView = try XCTUnwrap(engine.webView(for: id))
            let point = try await middle(of: element, in: id)
            for (kind, number) in [(NSEvent.EventType.leftMouseDown, 1), (.leftMouseUp, 2)] {
                let event = try XCTUnwrap(NSEvent.mouseEvent(
                    with: kind, location: point, modifierFlags: keys, timestamp: ProcessInfo.processInfo.systemUptime,
                    windowNumber: window.windowNumber, context: nil, eventNumber: number, clickCount: 1,
                    pressure: kind == .leftMouseDown ? 1 : 0
                ))
                if kind == .leftMouseDown {
                    webView.mouseDown(with: event)
                } else {
                    webView.mouseUp(with: event)
                }
            }
        }

        /// Clicks the middle of an element of the page with the mouse's other button: the
        /// click that asks for a menu. Only for a page that keeps the engine's menu from
        /// opening: a menu that opened would wait for a choice nobody is there to make.
        func rightClick(_ element: String, in id: UInt64, of window: NSWindow) async throws {
            let webView = try XCTUnwrap(engine.webView(for: id))
            let point = try await middle(of: element, in: id)
            for (kind, number) in [(NSEvent.EventType.rightMouseDown, 1), (.rightMouseUp, 2)] {
                let event = try XCTUnwrap(NSEvent.mouseEvent(
                    with: kind, location: point, modifierFlags: [], timestamp: ProcessInfo.processInfo.systemUptime,
                    windowNumber: window.windowNumber, context: nil, eventNumber: number, clickCount: 1,
                    pressure: kind == .rightMouseDown ? 1 : 0
                ))
                if kind == .rightMouseDown {
                    webView.rightMouseDown(with: event)
                } else {
                    webView.rightMouseUp(with: event)
                }
            }
        }
    #endif
}

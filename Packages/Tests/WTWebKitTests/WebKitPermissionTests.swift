// What a page asks to use of the device's (014), at the seam between the adapter and WebKit:
// what is reported, that the page waits for the core's word, and what each word does. No page
// asks here: on a machine with no camera or microphone WebKit asks nothing, and letting a page
// have one would bring up the system's own dialog (014 §10). What WebKit would call is checked
// to be there for it to call.
import WebKit
import WTContract
import WTCoreClient
@testable import WTWebKit
import XCTest

@MainActor
final class WebKitPermissionTests: AdapterTestCase {
    private let meet = "https://meet.example.com"

    /// The requests the adapter has reported, in order, each with the web view it said asked.
    private var asked: [(webView: UInt64, request: WTPermissionRequested)] {
        core.reports.compactMap { event in
            if case let .permissionRequested(request)? = event.kind {
                (event.webViewID, request)
            } else {
                nil
            }
        }
    }

    /// Has a page of a web view ask, as WebKit would have the adapter do, and waits until the
    /// request is reported. What the page is told, once it is.
    private func ask(_ engine: WebKitEngine, webView: UInt64 = 4, _ kinds: [WTPermissionKind]) async -> Task<Bool, Never> {
        let before = asked.count
        let origin = meet
        let told = Task { await engine.permissions.ask(webViewID: webView, origin: origin, kinds: kinds) }
        while asked.count == before {
            await Task.yield()
        }
        return told
    }

    /// The core's word on a request, as the core sends it.
    private func say(_ engine: WebKitEngine, _ request: UInt64, granted: Bool) async {
        var answer = WTPermissionAnswer()
        answer.requestID = request
        answer.granted = granted
        guard case .ack = await engine.answer(.permissionAnswer(answer)) else {
            return XCTFail("a word about a request is acknowledged")
        }
    }

    /// An engine as the app itself makes one when it is started for real.
    private func theAppsEngine() -> WebKitEngine {
        let engine = WebKitEngine(deviceAccess: .asTheCoreSays)
        engine.attach(core)
        return engine
    }

    func testAPageThatAsksIsReportedWithWhoAsksAndForWhatAndWaits() async {
        let engine = theAppsEngine()

        let told = await ask(engine, [.camera, .microphone])

        XCTAssertEqual(asked.count, 1)
        XCTAssertEqual(asked[0].webView, 4)
        XCTAssertEqual(asked[0].request.origin, meet)
        XCTAssertEqual(asked[0].request.kinds, [.camera, .microphone])
        XCTAssertNotEqual(asked[0].request.requestID, 0, "named by the shell")
        // The page waits for the core's word: nothing has been told it.
        var answered = false
        let watching = Task {
            _ = await told.value
            answered = true
        }
        for _ in 0 ..< 50 {
            await Task.yield()
        }
        XCTAssertFalse(answered)

        await say(engine, asked[0].request.requestID, granted: true)
        await watching.value
        let granted = await told.value
        XCTAssertTrue(granted)
    }

    func testTheCoresNoIsThePagesNo() async {
        let engine = theAppsEngine()
        let told = await ask(engine, [.camera])

        await say(engine, asked[0].request.requestID, granted: false)

        let granted = await told.value
        XCTAssertFalse(granted)
    }

    func testAnEngineNobodyMadeOtherwiseTellsThePageNoWhateverTheCoreSaid() async {
        // What every test's engine is, and fake mode's: it cannot bring the system's dialog up.
        XCTAssertEqual(engine.deviceAccess, .never)
        let told = await ask(engine, [.camera])

        await say(engine, asked[0].request.requestID, granted: true)

        let granted = await told.value
        XCTAssertFalse(granted, "the core said yes, and the page is told no")
    }

    func testEachRequestHasANameOfItsOwnAndIsAnsweredOnce() async {
        let engine = theAppsEngine()
        let first = await ask(engine, [.camera])
        let second = await ask(engine, webView: 5, [.location])
        let names = asked.map(\.request.requestID)
        XCTAssertEqual(Set(names).count, 2)

        // The second is answered, and then "answered" again: the first word stands.
        await say(engine, names[1], granted: false)
        await say(engine, names[1], granted: true)
        let place = await second.value
        XCTAssertFalse(place)
        // A name that was answered is not given again.
        let third = await ask(engine, [.microphone])
        XCTAssertFalse(names.contains(asked[2].request.requestID))
        await say(engine, names[0], granted: true)
        await say(engine, asked[2].request.requestID, granted: true)
        let camera = await first.value
        let microphone = await third.value
        XCTAssertTrue(camera && microphone)
    }

    func testAWebViewThatIsDestroyedRefusesWhatItsPagesAsked() async {
        let engine = theAppsEngine()
        let going = await ask(engine, webView: 4, [.camera])
        let alsoGoing = await ask(engine, webView: 4, [.location])
        let staying = await ask(engine, webView: 5, [.camera])

        var destroy = WTWebViewDestroy()
        destroy.webViewID = 4
        _ = await engine.answer(.webViewDestroy(destroy))

        // WebKit is given its answer for both: one it never got would be the app's fault.
        let camera = await going.value
        let place = await alsoGoing.value
        XCTAssertFalse(camera || place)
        // A word for one of them afterwards is for nothing; another web view's still waits.
        await say(engine, asked[0].request.requestID, granted: true)
        await say(engine, asked[2].request.requestID, granted: true)
        let kept = await staying.value
        XCTAssertTrue(kept)
    }

    func testAWordAboutARequestThereIsNot() async {
        await say(theAppsEngine(), 99, granted: true)
        XCTAssertTrue(asked.isEmpty)
    }

    func testWhoAsksIsWrittenAsAnOrigin() {
        // WebKit gives a scheme, a host, and a port that is 0 for the scheme's own.
        XCTAssertEqual(Permissions.origin(scheme: "https", host: "meet.example.com", port: 0), "https://meet.example.com")
        XCTAssertEqual(Permissions.origin(scheme: "https", host: "meet.example.com", port: 8443), "https://meet.example.com:8443")
        XCTAssertEqual(Permissions.origin(scheme: "http", host: "127.0.0.1", port: 8080), "http://127.0.0.1:8080")
        XCTAssertEqual(Permissions.origin(scheme: "http", host: "::1", port: 8080), "http://[::1]:8080")
        XCTAssertEqual(Permissions.origin(scheme: "http", host: "[::1]", port: 0), "http://[::1]")
    }

    func testWhatWebKitAsksForIsWhatIsReported() {
        XCTAssertEqual(Permissions.kinds(of: .camera), [.camera])
        XCTAssertEqual(Permissions.kinds(of: .microphone), [.microphone])
        XCTAssertEqual(Permissions.kinds(of: .cameraAndMicrophone), [.camera, .microphone])
    }

    func testWhatWebKitWouldCallIsThereForItToCall() async {
        // WebKit asks a web view's delegate only for what the delegate says it answers; what
        // it does not, WebKit answers itself, with a prompt of its own. The page is the
        // delegate, and answers for the camera and the microphone, and on a system that asks,
        // for the location.
        await create(1)
        let delegate = engine.webView(for: 1)?.uiDelegate as? NSObject
        XCTAssertNotNil(delegate)
        let capture = NSSelectorFromString("webView:requestMediaCapturePermissionForOrigin:initiatedByFrame:type:decisionHandler:")
        XCTAssertEqual(delegate?.responds(to: capture), true)
        if #available(macOS 27.0, iOS 27.0, *) {
            let location = NSSelectorFromString("webView:requestGeolocationPermissionForOrigin:initiatedByFrame:decisionHandler:")
            XCTAssertEqual(delegate?.responds(to: location), true)
        }
    }
}

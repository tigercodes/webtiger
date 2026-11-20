// The Permissions list's model (014) against a scripted core: what it asks, what it shows of
// the answers, and what it sends to take back what a site was allowed.
import Foundation
import WTContract
import WTCoreClient
@testable import WTUI
import XCTest

@MainActor
final class PermissionsModelTests: XCTestCase {
    private func site(_ origin: String, _ name: String, _ kinds: [WTPermissionKind]) -> WTSitePermissions {
        var site = WTSitePermissions()
        site.origin = origin
        site.site = name
        site.kinds = kinds
        return site
    }

    /// A core with two sites that were allowed things, which takes back what it is told to.
    private func scripted() -> ScriptedCore {
        let left = Left([
            site("http://127.0.0.1:8080", "http://127.0.0.1:8080", [.location]),
            site("https://meet.example.com", "meet.example.com", [.camera, .microphone]),
        ])
        return ScriptedCore { request in
            switch request {
            case let .forgetPermissions(forget):
                left.change { sites in
                    sites = forget.everything ? [] : sites.filter { $0.origin != forget.origin }
                }
                return .ack(WTAck())
            case .queryPermissions:
                var page = WTPermissionsPage()
                page.sites = left.change { _ in }
                return .permissionsPage(page)
            default:
                return .ack(WTAck())
            }
        }
    }

    /// Lets the model's request tasks run.
    private func settle(until done: () -> Bool = { false }) async {
        for _ in 0 ..< 200 where !done() {
            await Task.yield()
            try? await Task.sleep(for: .milliseconds(2))
        }
    }

    func testTheListIsWhatTheCoreSaysOfTheWindowsSites() async {
        let core = scripted()
        let model = PermissionsModel(core: core, windowID: 7)
        XCTAssertFalse(model.loaded, "nothing is said before the core has answered")

        model.reload()
        await settle { model.loaded }

        guard case let .queryPermissions(query)? = core.requests.last else { return XCTFail("the list was asked for") }
        XCTAssertEqual(query.windowID, 7)
        XCTAssertEqual(model.sites.map(\.site), ["http://127.0.0.1:8080", "meet.example.com"], "in the core's order")
    }

    func testTakingBackOneLeavesTheListAtOnceAndTakingBackAllAsksAgain() async {
        let core = scripted()
        let model = PermissionsModel(core: core, windowID: 7)
        model.reload()
        await settle { model.loaded }

        model.forget(model.sites[1])

        XCTAssertEqual(model.sites.map(\.site), ["http://127.0.0.1:8080"], "it has left, with nothing asked")
        await settle { core.requests.count >= 2 }
        guard case let .forgetPermissions(forget)? = core.requests.last else { return XCTFail("the core was told") }
        XCTAssertEqual(forget.windowID, 7)
        XCTAssertEqual(forget.origin, "https://meet.example.com", "named by its origin, as the core gave it")

        model.forgetEverything()
        await settle { model.sites.isEmpty }

        XCTAssertTrue(model.sites.isEmpty)
        XCTAssertTrue(model.loaded, "the core has answered: no site is allowed anything")
        let sent = core.requests.suffix(2)
        guard case let .forgetPermissions(all)? = sent.first, case .queryPermissions? = sent.last else {
            return XCTFail("everything was taken back, and what is left was asked for")
        }
        XCTAssertTrue(all.everything)
    }

    func testWhatASiteMayUseIsSaidInWords() {
        let british = Locale(identifier: "en_GB")
        func words(_ kinds: [WTPermissionKind]) -> String {
            PermissionsView.allowed(site("https://meet.example.com", "meet.example.com", kinds), locale: british)
        }
        XCTAssertEqual(words([.camera]), "Camera")
        XCTAssertEqual(words([.location]), "Location")
        XCTAssertEqual(words([.camera, .microphone]), "Camera and Microphone")
        XCTAssertEqual(words([.camera, .microphone, .location]), "Camera, Microphone and Location")
        // A kind this build has no words for is left out.
        XCTAssertEqual(words([.microphone, .UNRECOGNIZED(99)]), "Microphone")
    }
}

/// What a scripted core has left of what sites were allowed: its script changes it, from
/// wherever the core runs the script.
private final class Left: @unchecked Sendable {
    private let lock = NSLock()
    private var sites: [WTSitePermissions]

    init(_ sites: [WTSitePermissions]) {
        self.sites = sites
    }

    /// Changes what is left, and gives back what then is.
    @discardableResult
    func change(_ change: (inout [WTSitePermissions]) -> Void) -> [WTSitePermissions] {
        lock.withLock {
            change(&sites)
            return sites
        }
    }
}

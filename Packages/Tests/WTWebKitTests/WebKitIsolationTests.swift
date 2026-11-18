// Isolation (000 §8.4): a security property, tested as one. Sign in under one login set;
// another must see no cookie and no storage, and deleting a login set must leave nothing.
import WebKit
import WTContract
import WTCoreClient
import WTWebKit
import XCTest

@MainActor
final class WebKitIsolationTests: AdapterTestCase {
    func testLoginSetsShareNoCookies() async {
        let personal = newLoginSet()
        let work = newLoginSet()
        await create(1, loginSet: personal)
        await create(2, loginSet: work)
        await create(3, loginSet: personal)

        // Sign in under the first login set.
        await load(1, site.url("/set?v=ada"))
        await wait(1, "the cookie to be set") { self.finished($0) && self.hasTitle("Set")($0) }

        // Another login set sees nothing.
        await load(2, site.url("/whoami"))
        await wait(2, "the other login set's answer") { self.finished($0) && self.lastTitle(2).hasPrefix("who=") }
        XCTAssertEqual(lastTitle(2), "who=none", "a Space with its own logins must not see another's cookie")

        // A second web view in the first login set is signed in.
        await load(3, site.url("/whoami"))
        await wait(3, "the same login set's answer") { self.finished($0) && self.lastTitle(3).hasPrefix("who=") }
        XCTAssertEqual(lastTitle(3), "who=ada", "Spaces that share logins share them")
    }

    func testLoginSetsShareNoLocalStorage() async {
        let personal = newLoginSet()
        let work = newLoginSet()
        await create(1, loginSet: personal)
        await create(2, loginSet: work)
        await load(1, site.url("/storage?v=ada"))
        await wait(1, "the value to be stored") { _ in self.lastTitle(1) == "stored=ada" }
        await load(2, site.url("/storage"))
        await wait(2, "the other login set's answer") { _ in self.lastTitle(2).hasPrefix("stored=") }
        XCTAssertEqual(lastTitle(2), "stored=none")
    }

    func testAPrivateLoginSetIsForgottenWithItsWebView() async {
        await create(1, ephemeral: true)
        await load(1, site.url("/set?v=ada"))
        await wait(1, "the cookie to be set") { self.finished($0) && self.hasTitle("Set")($0) }
        await destroy(1)

        await create(2, ephemeral: true)
        await load(2, site.url("/whoami"))
        await wait(2, "the new private window's answer") { self.finished($0) && self.lastTitle(2).hasPrefix("who=") }
        XCTAssertEqual(lastTitle(2), "who=none")
    }

    func testWebViewsOfOnePrivateLoginSetShareIt() async {
        let window = UUID()
        await create(1, loginSet: window, ephemeral: true)
        await create(2, loginSet: window, ephemeral: true)
        await load(1, site.url("/set?v=ada"))
        await wait(1, "the cookie to be set") { self.finished($0) && self.hasTitle("Set")($0) }
        // A private window's second tab is signed in where its first one signed in.
        await load(2, site.url("/whoami"))
        await wait(2, "the other tab's answer") { self.finished($0) && self.lastTitle(2).hasPrefix("who=") }
        XCTAssertEqual(lastTitle(2), "who=ada", "one private login set is one identity")

        // And the login set is forgotten when the last of its web views is destroyed, not the first.
        await destroy(1)
        await create(3, loginSet: window, ephemeral: true)
        await load(3, site.url("/whoami"))
        await wait(3, "the third tab's answer") { self.finished($0) && self.lastTitle(3).hasPrefix("who=") }
        XCTAssertEqual(lastTitle(3), "who=ada", "a web view of the login set is still there")
        await destroy(2)
        await destroy(3)
        await create(4, loginSet: window, ephemeral: true)
        await load(4, site.url("/whoami"))
        await wait(4, "the answer after all were closed") { self.finished($0) && self.lastTitle(4).hasPrefix("who=") }
        XCTAssertEqual(lastTitle(4), "who=none", "with its last web view gone, a private login set is forgotten")
    }

    func testASignInForTheSessionOutlivesTheLastWebViewOfItsLoginSet() async throws {
        let loginSet = newLoginSet()
        await create(1, loginSet: loginSet)
        // A cookie with no expiry: WebKit keeps it in the memory of the login set's data store.
        await load(1, site.url("/session?v=ada"))
        await wait(1, "the cookie to be set") { self.finished($0) && self.hasTitle("Session")($0) }
        // The login set's only tab sleeps: it has no web view at all for a while.
        await destroy(1)
        try await Task.sleep(for: .seconds(2))
        await create(2, loginSet: loginSet)
        await load(2, site.url("/whoami"))
        await wait(2, "the woken tab's answer") { self.finished($0) && self.lastTitle(2).hasPrefix("who=") }
        XCTAssertEqual(lastTitle(2), "who=ada", "a tab that sleeps does not sign its Space out")
    }

    func testDeletingALoginSetLeavesNothing() async {
        let loginSet = newLoginSet()
        await create(1, loginSet: loginSet)
        await load(1, site.url("/set?v=ada"))
        await wait(1, "the cookie to be set") { self.finished($0) && self.hasTitle("Set")($0) }
        await load(1, site.url("/storage?v=ada"))
        await wait(1, "the value to be stored") { _ in self.lastTitle(1) == "stored=ada" }
        await destroy(1)

        let removed = await remove(loginSet)
        XCTAssertTrue(removed, "a login set no web view uses can be deleted")

        await create(2, loginSet: loginSet)
        await load(2, site.url("/whoami"))
        await wait(2, "the cookie's absence") { self.finished($0) && self.lastTitle(2).hasPrefix("who=") }
        XCTAssertEqual(lastTitle(2), "who=none")
        await load(2, site.url("/storage"))
        await wait(2, "the storage's absence") { _ in self.lastTitle(2).hasPrefix("stored=") }
        XCTAssertEqual(lastTitle(2), "stored=none")
    }

    /// Runs a script in a web view's page and gives back what it returns.
    private func run(_ script: String, in id: UInt64) async -> Any? {
        guard let webView = engine.webView(for: id) else { return nil }
        return try? await webView.callAsyncJavaScript(script, arguments: [:], in: nil, contentWorld: .page)
    }

    func testLoginSetsShareNoDatabasesAndNoCaches() async {
        let personal = newLoginSet()
        let work = newLoginSet()
        await create(1, loginSet: personal)
        await create(2, loginSet: work)
        await create(3, loginSet: personal)
        for id in [UInt64(1), 2, 3] {
            await load(id, site.url("/"))
            await wait(id, "the page") { self.finished($0) }
        }
        // Under the first login set: a database with a record in it, and a cache with an entry.
        let stored = await run("""
        await new Promise((resolve, reject) => {
            const opening = indexedDB.open("who", 1);
            opening.onupgradeneeded = () => opening.result.createObjectStore("names");
            opening.onerror = () => reject(opening.error);
            opening.onsuccess = () => {
                const writing = opening.result.transaction("names", "readwrite");
                writing.objectStore("names").put("ada", "who");
                writing.oncomplete = () => resolve();
                writing.onerror = () => reject(writing.error);
            };
        });
        const cache = await caches.open("pages");
        await cache.put("/kept", new Response("ada"));
        return true;
        """, in: 1) as? Bool
        XCTAssertEqual(stored, true, "the page could use IndexedDB and the Cache API")

        let count = "return [(await indexedDB.databases()).length, (await caches.keys()).length];"
        let other = await run(count, in: 2) as? [Int]
        XCTAssertEqual(other, [0, 0], "another login set sees no database and no cache")
        let same = await run(count, in: 3) as? [Int]
        XCTAssertEqual(same, [1, 1], "the same login set sees both")
    }

    func testLoginSetsShareNoServiceWorkers() async throws {
        let personal = newLoginSet()
        let work = newLoginSet()
        await create(1, loginSet: personal)
        await create(2, loginSet: work)
        await create(3, loginSet: personal)
        for id in [UInt64(1), 2, 3] {
            await load(id, site.url("/"))
            await wait(id, "the page") { self.finished($0) }
        }
        let offered = await run("return 'serviceWorker' in navigator;", in: 1) as? Bool
        // On iPhone and iPad WebKit gives service workers only to an app with the browser
        // entitlement, which a test process does not hold (001 §10, Q8).
        try XCTSkipUnless(offered == true, "this WebKit offers the test process no service workers")

        let registered = await run("""
        const registration = await navigator.serviceWorker.register("/worker.js");
        return registration.scope;
        """, in: 1) as? String
        XCTAssertEqual(registered, site.url("/"))
        let count = "return (await navigator.serviceWorker.getRegistrations()).length;"
        let other = await run(count, in: 2) as? Int
        XCTAssertEqual(other, 0, "another login set has no worker for the same site")
        let same = await run(count, in: 3) as? Int
        XCTAssertEqual(same, 1, "the same login set has it")
    }

    /// How many times the site was asked for a path.
    private func asked(_ path: String) -> Int {
        site.requests.count { $0 == path }
    }

    func testLoginSetsShareNoHTTPCache() async {
        let personal = newLoginSet()
        let work = newLoginSet()
        await create(1, loginSet: personal)
        await create(2, loginSet: work)
        await create(3, loginSet: personal)

        // A page whose script may be kept for an hour.
        await load(1, site.url("/uses-cached"))
        await wait(1, "the script to run") { _ in self.lastTitle(1) == "Cached" }
        XCTAssertEqual(asked("/cached.js"), 1)

        // Another web view in the same login set gets the script from the cache. The cache is
        // written a moment after the answer came, so this is tried until it holds; without it
        // the next step would show nothing.
        var kept = false
        for attempt in 0 ..< 20 where !kept {
            let before = asked("/cached.js")
            let id = UInt64(100 + attempt)
            await create(id, loginSet: personal)
            await load(id, site.url("/uses-cached"))
            await wait(id, "the script to run again") { _ in self.lastTitle(id) == "Cached" }
            kept = asked("/cached.js") == before
            await destroy(id)
            if !kept {
                try? await Task.sleep(for: .milliseconds(200))
            }
        }
        XCTAssertTrue(kept, "web views in one login set share its cache")

        // Another login set has no copy: the site is asked again.
        let before = asked("/cached.js")
        await load(2, site.url("/uses-cached"))
        await wait(2, "the other login set's script") { _ in self.lastTitle(2) == "Cached" }
        XCTAssertEqual(asked("/cached.js"), before + 1, "a login set's cache is its own")
    }

    /// Not an assertion about WebTiger: a record of what WebKit offers a web view in an app
    /// without the browser entitlement (spike Q8), printed for 001.
    func testWhatWebKitOffersWithoutTheBrowserEntitlement() async {
        await create(1, loginSet: newLoginSet())
        await load(1, site.url("/"))
        await wait(1, "the page") { self.finished($0) }
        let offered = await run("""
        return {
            secureContext: window.isSecureContext,
            serviceWorker: "serviceWorker" in navigator,
            push: "PushManager" in window,
            notifications: "Notification" in window,
            credentials: "credentials" in navigator,
            usb: "usb" in navigator,
            storageEstimate: !!(navigator.storage && navigator.storage.estimate),
        };
        """, in: 1) as? [String: Bool] ?? [:]
        XCTAssertFalse(offered.isEmpty, "the page answered")
        let facts = offered.sorted { $0.key < $1.key }.map { "\($0.key)=\($0.value)" }.joined(separator: " ")
        print("WTFACT WebKit without the browser entitlement: \(facts)")
    }
}

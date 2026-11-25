// The Rust core, through Swift: the same contract the Rust tests drive, crossing the real C ABI.
#if WEBTIGER_CORE
    import WTContract
    import WTCoreClient
    import XCTest

    /// A host that records what the core asks and answers as a shell with a working engine would.
    @MainActor
    private final class RecordingHost: HostServices {
        private(set) var requests: [WTHostRequest.OneOf_Kind] = []

        func answer(_ request: WTHostRequest.OneOf_Kind) async -> WTHostResponse.OneOf_Kind {
            requests.append(request)
            if case let .ping(ping) = request {
                var pong = WTHostPong()
                pong.payload = ping.payload
                return .pong(pong)
            }
            return .ack(WTAck())
        }
    }

    /// The tab snapshots a core has published, in order.
    @MainActor
    private final class TabLog {
        var tabs: [WTTabView] = []

        /// The first snapshot that matches, waiting up to ten seconds for it.
        func first(where matches: (WTTabView) -> Bool) async -> WTTabView? {
            for _ in 0 ..< 2000 {
                if let found = tabs.first(where: matches) {
                    return found
                }
                try? await Task.sleep(for: .milliseconds(5))
            }
            return nil
        }
    }

    /// The last snapshot of a window a core has published.
    @MainActor
    private final class WindowLog {
        var window: WTWindowSnapshot?

        /// The window once it has been published, waiting up to ten seconds for it.
        func first() async -> WTWindowSnapshot? {
            for _ in 0 ..< 2000 where window == nil {
                try? await Task.sleep(for: .milliseconds(5))
            }
            return window
        }
    }

    @MainActor
    final class NativeCoreTests: XCTestCase {
        private func fakeConfig() -> WTCoreConfig {
            var config = WTCoreConfig()
            config.platform = .macos
            config.locale = "en-GB"
            config.testMode = .fake
            var engine = WTEngineInfo()
            engine.kind = .webkit
            engine.version = "test"
            config.engines = [engine]
            return config
        }

        /// Keeps every tab snapshot the core publishes.
        private func tabs(of core: NativeCore) -> TabLog {
            let log = TabLog()
            let events = core.events
            Task { @MainActor in
                for await event in events {
                    if case let .snapshot(snapshot) = event, case let .window(window) = snapshot.model {
                        log.tabs.append(window.tab)
                    }
                }
            }
            return log
        }

        func testTheLibrarySpeaksThisShellsContract() throws {
            let core = try NativeCore.start(config: fakeConfig(), host: RecordingHost())
            defer { core.stop() }
            XCTAssertEqual(NativeCore.contractMajor, 1)
        }

        func testPingGoesToTheCoreAndRoundThroughTheHost() async throws {
            let host = RecordingHost()
            let core = try NativeCore.start(config: fakeConfig(), host: host)
            defer { core.stop() }

            var ping = WTPing()
            ping.payload = Data("hello".utf8)
            guard case let .pong(direct) = try await core.request(.ping(ping)) else { return XCTFail("expected Pong") }
            XCTAssertEqual(direct.payload, Data("hello".utf8))
            XCTAssertFalse(direct.viaHost)
            XCTAssertEqual(direct.contractMajor, NativeCore.contractMajor)

            ping.viaHost = true
            guard case let .pong(round) = try await core.request(.ping(ping)) else { return XCTFail("expected Pong") }
            XCTAssertTrue(round.viaHost)
            XCTAssertEqual(host.requests.count, 1)
        }

        /// A host that takes its time over a ping that says "slow": it is begun when it is
        /// asked, and answered when the test says. Every other ping is answered in its turn.
        @MainActor
        private final class SlowHost: HostServices {
            /// What was asked, in order: "begun slow" for the one that takes its time.
            private(set) var asked: [String] = []
            private var waiting: CheckedContinuation<Void, Never>?
            private var released = false

            func answer(_ request: WTHostRequest.OneOf_Kind) async -> WTHostResponse.OneOf_Kind {
                guard case let .ping(ping) = request else { return .ack(WTAck()) }
                asked.append(String(bytes: ping.payload, encoding: .utf8) ?? "")
                var pong = WTHostPong()
                pong.payload = ping.payload
                return .pong(pong)
            }

            func begin(_ request: WTHostRequest.OneOf_Kind) -> (@MainActor @Sendable () async -> WTHostResponse.OneOf_Kind)? {
                guard case let .ping(ping) = request, ping.payload == Data("slow".utf8) else { return nil }
                asked.append("begun slow")
                return { [self] in
                    if !released {
                        await withCheckedContinuation { waiting = $0 }
                    }
                    asked.append("slow answered")
                    var pong = WTHostPong()
                    pong.payload = ping.payload
                    return .pong(pong)
                }
            }

            /// The slow one's answer may come.
            func release() {
                released = true
                waiting?.resume()
                waiting = nil
            }
        }

        func testAHostRequestThatTakesItsTimeIsBegunInItsTurnAndHoldsUpNoneAfterIt() async throws {
            let host = SlowHost()
            let core = try NativeCore.start(config: fakeConfig(), host: host)
            defer { core.stop() }
            func ping(_ word: String) -> WTRequest.OneOf_Kind {
                var ping = WTPing()
                ping.payload = Data(word.utf8)
                ping.viaHost = true
                return .ping(ping)
            }

            // A ping round the host that takes its time, and two that do not, sent one after
            // the other. The two are answered while the first is still on its way.
            let slow = core.send(ping("slow"))
            guard case let .pong(first) = try await core.request(ping("one")) else { return XCTFail("expected Pong") }
            guard case let .pong(second) = try await core.request(ping("two")) else { return XCTFail("expected Pong") }
            XCTAssertEqual([first.payload, second.payload], [Data("one".utf8), Data("two".utf8)])
            // It was begun in its turn, before either of them was asked.
            XCTAssertEqual(host.asked, ["begun slow", "one", "two"])

            // Its own answer comes when it is ready, and is the answer to its own request.
            host.release()
            guard case let .pong(last) = try await slow.value else { return XCTFail("expected Pong") }
            XCTAssertEqual(last.payload, Data("slow".utf8))
            XCTAssertTrue(last.viaHost)
            XCTAssertEqual(host.asked, ["begun slow", "one", "two", "slow answered"])
        }

        func testAPageLoadsThroughTheContract() async throws {
            let host = RecordingHost()
            let core = try NativeCore.start(config: fakeConfig(), host: host)
            defer { core.stop() }
            let log = tabs(of: core)

            guard case let .windowOpened(opened) = try await core.request(.openWindow(WTOpenWindow())) else {
                return XCTFail("expected WindowOpened")
            }
            var subscribe = WTSubscribe()
            subscribe.window.windowID = opened.windowID
            guard case .subscribed = try await core.request(.subscribe(subscribe)) else { return XCTFail("expected Subscribed") }

            var submit = WTSubmitAddress()
            submit.tabID = opened.tabID
            submit.text = "example.com"
            try await core.perform(.submitAddress(submit))

            // The core asked for a web view in the Space's login set, then for the load: in that order.
            guard host.requests.count == 2, case let .webViewCreate(create) = host.requests[0],
                  case let .webViewLoad(load) = host.requests[1]
            else { return XCTFail("expected a create then a load, got \(host.requests)") }
            XCTAssertEqual(create.engine, .webkit)
            XCTAssertNotNil(UUID(uuidString: create.loginSet.id), "a login set is named by a UUID")
            XCTAssertEqual(load.webViewID, create.webViewID)
            XCTAssertEqual(load.url, "https://example.com/")

            // The engine reports the page arriving; the core's snapshot says what to show.
            var committed = WTEngineEvent()
            committed.webViewID = create.webViewID
            committed.navigationCommitted.url = "https://example.com/"
            core.report(committed)
            var titled = WTEngineEvent()
            titled.webViewID = create.webViewID
            titled.titleChanged.title = "Example Domain"
            core.report(titled)
            var finished = WTEngineEvent()
            finished.webViewID = create.webViewID
            finished.navigationFinished = WTNavigationFinished()
            core.report(finished)

            let tab = await log.first { $0.loadState == .idle && !$0.title.isEmpty }
            XCTAssertEqual(tab?.title, "Example Domain")
            XCTAssertEqual(tab?.webViewID, create.webViewID)
            XCTAssertEqual(tab?.address.display, "example.com")
            XCTAssertEqual(tab?.address.registrableDomain, "example.com")
            XCTAssertEqual(tab?.address.security, .secure)
        }

        func testEveryKindOfAnswerComesBackAsWhatItIs() async throws {
            let core = try NativeCore.start(config: fakeConfig(), host: RecordingHost())
            defer { core.stop() }
            guard case let .windowOpened(opened) = try await core.request(.openWindow(WTOpenWindow())) else {
                return XCTFail("expected WindowOpened")
            }

            // Each list a window can ask for is an answer of its own kind, also when it is empty.
            var history = WTQueryHistory()
            history.windowID = opened.windowID
            guard case let .historyPage(visits) = try await core.request(.queryHistory(history))
            else { return XCTFail("expected a page of history") }
            XCTAssertTrue(visits.entries.isEmpty)
            var suggest = WTSuggestAddresses()
            suggest.windowID = opened.windowID
            suggest.text = "example"
            guard case let .addressSuggestions(offered) = try await core.request(.suggestAddresses(suggest)) else {
                return XCTFail("expected what is offered")
            }
            XCTAssertTrue(offered.suggestions.isEmpty)
            var downloads = WTQueryDownloads()
            downloads.windowID = opened.windowID
            guard case let .downloadsPage(downloaded) = try await core.request(.queryDownloads(downloads)) else {
                return XCTFail("expected the downloads")
            }
            XCTAssertTrue(downloaded.downloads.isEmpty)
            var permissions = WTQueryPermissions()
            permissions.windowID = opened.windowID
            guard case let .permissionsPage(allowed) = try await core.request(.queryPermissions(permissions)) else {
                return XCTFail("expected what sites were allowed")
            }
            XCTAssertTrue(allowed.sites.isEmpty)
            guard case let .searchSettings(search) = try await core.request(.querySearch(WTQuerySearch())) else {
                return XCTFail("expected where searches go")
            }
            XCTAssertFalse(search.engines.isEmpty)
            XCTAssertTrue(search.chosen, "fake mode has nobody to ask")
        }

        func testARefusalIsThrownWithItsCode() async throws {
            let core = try NativeCore.start(config: fakeConfig(), host: RecordingHost())
            defer { core.stop() }
            var submit = WTSubmitAddress()
            submit.tabID = 9999
            submit.text = "example.com"
            do {
                try await core.perform(.submitAddress(submit))
                XCTFail("expected a refusal")
            } catch {
                XCTAssertEqual(error as? CoreError, .refused(code: "tab.unknown", field: "tab_id"))
            }
        }

        /// What is sent first, the core hears first: `send` hands the request to the core before
        /// it returns. Sixty tabs are opened without one answer being waited for; the core
        /// numbers tabs as it makes them, so the ids say in what order it heard.
        func testRequestsReachTheCoreInTheOrderTheyWereSent() async throws {
            let core = try NativeCore.start(config: fakeConfig(), host: RecordingHost())
            defer { core.stop() }
            guard case let .windowOpened(window) = try await core.request(.openWindow(WTOpenWindow())) else {
                return XCTFail("expected WindowOpened")
            }
            var open = WTOpenTab()
            open.windowID = window.windowID

            let answers = (0 ..< 60).map { _ in core.send(.openTab(open)) }

            var ids: [UInt64] = []
            for answer in answers {
                guard case let .tabOpened(opened) = try await answer.value else { return XCTFail("expected TabOpened") }
                ids.append(opened.tabID)
            }
            XCTAssertEqual(ids, ids.sorted(), "each tab was made after the one sent before it")
            XCTAssertEqual(Set(ids).count, 60)
        }

        /// An answer that comes before anyone waits for it is kept, and one nobody waits for
        /// is harmless.
        func testAnAnswerNeedNotBeWaitedForAtOnce() async throws {
            let core = try NativeCore.start(config: fakeConfig(), host: RecordingHost())
            defer { core.stop() }
            _ = core.send(.ping(WTPing()))
            let answer = core.send(.openWindow(WTOpenWindow()))
            try await Task.sleep(for: .milliseconds(200))
            guard case .windowOpened = try await answer.value else { return XCTFail("the answer was kept") }
            // A refusal is the wait's to throw.
            var close = WTCloseTab()
            close.tabID = 9999
            let refused = core.send(.closeTab(close))
            do {
                _ = try await refused.value
                XCTFail("expected a refusal")
            } catch {
                XCTAssertEqual(error as? CoreError, .refused(code: "tab.unknown", field: "tab_id"))
            }
        }

        func testARefusedStartSaysWhy() {
            var config = fakeConfig()
            config.testMode = .live // and no data folder
            XCTAssertThrowsError(try NativeCore.start(config: config, host: RecordingHost())) { error in
                XCTAssertEqual(error as? CoreError, .startRefused(code: "start.data_dir"))
            }
            var none = fakeConfig()
            none.engines = []
            XCTAssertThrowsError(try NativeCore.start(config: none, host: RecordingHost())) { error in
                XCTAssertEqual(error as? CoreError, .startRefused(code: "start.no_engine"))
            }
        }

        func testLiveModeKeepsItsStoreInTheDataFolder() throws {
            let folder = FileManager.default.temporaryDirectory.appending(path: "webtiger-test-\(UUID().uuidString)")
            defer { try? FileManager.default.removeItem(at: folder) }
            var config = fakeConfig()
            config.testMode = .live
            config.dataDir = folder.path(percentEncoded: false)
            let core = try NativeCore.start(config: config, host: RecordingHost())
            core.stop()
            XCTAssertTrue(FileManager.default.fileExists(atPath: folder.appending(path: "webtiger.sqlite").path(percentEncoded: false)))
        }

        /// The tabs come back (009), through Swift and the real core: a core is stopped, and
        /// another started on the same folder is given the first one's tabs, in order, with
        /// the one that was in front loaded and the other asleep.
        func testTheTabsComeBackWhenTheCoreStartsAgain() async throws {
            let folder = FileManager.default.temporaryDirectory.appending(path: "webtiger-test-\(UUID().uuidString)")
            defer { try? FileManager.default.removeItem(at: folder) }
            var config = fakeConfig()
            config.testMode = .live
            config.dataDir = folder.path(percentEncoded: false)

            // The first run: a page in the first tab, and a second tab in front of it.
            let first = try NativeCore.start(config: config, host: RecordingHost())
            guard case let .windowOpened(opened) = try await first.request(.openWindow(WTOpenWindow())) else {
                return XCTFail("expected WindowOpened")
            }
            var submit = WTSubmitAddress()
            submit.tabID = opened.tabID
            submit.text = "example.com"
            try await first.perform(.submitAddress(submit))
            var open = WTOpenTab()
            open.windowID = opened.windowID
            open.text = "example.org"
            _ = try await first.request(.openTab(open))
            // The snapshots are published, and with them the session is written, before the
            // next request is answered.
            try await first.perform(.ping(WTPing()))
            first.stop()

            // The second run, on the same folder.
            let host = RecordingHost()
            let second = try NativeCore.start(config: config, host: host)
            defer { second.stop() }
            let log = WindowLog()
            let events = second.events
            Task { @MainActor in
                for await event in events {
                    if case let .snapshot(snapshot) = event, case let .window(shown) = snapshot.model {
                        log.window = shown
                    }
                }
            }
            guard case let .windowOpened(again) = try await second.request(.openWindow(WTOpenWindow())) else {
                return XCTFail("expected WindowOpened")
            }
            var subscribe = WTSubscribe()
            subscribe.window.windowID = again.windowID
            guard case .subscribed = try await second.request(.subscribe(subscribe)) else { return XCTFail("expected Subscribed") }
            let window = await log.first()

            let tabs = try XCTUnwrap(window).tabs
            XCTAssertEqual(tabs.map(\.host), ["example.com", "example.org"], "the tabs, in their order")
            XCTAssertEqual(window?.tab.tabID, tabs.last?.tabID, "the one that was in front is in front")
            XCTAssertEqual(window?.tab.address.url, "https://example.org/")
            // Only the tab in front was loaded: one web view, one load.
            let loads = host.requests.compactMap { request -> String? in
                if case let .webViewLoad(load) = request {
                    load.url
                } else {
                    nil
                }
            }
            XCTAssertEqual(loads, ["https://example.org/"])
            XCTAssertEqual(host.requests.count, 2, "a web view and its load, and nothing for the tab behind")
        }

        func testTheNavigationPolicyAnswersSynchronously() throws {
            let core = try NativeCore.start(config: fakeConfig(), host: RecordingHost())
            defer { core.stop() }
            func verdict(_ url: String, mainFrame: Bool = true) -> NavigationVerdict {
                core.navigationVerdict(for: url, webViewID: 1, mainFrame: mainFrame, userGesture: false)
            }
            XCTAssertEqual(verdict("https://example.com/"), .allow)
            XCTAssertEqual(verdict("about:blank"), .allow)
            XCTAssertEqual(verdict("javascript:alert(1)"), .cancel)
            XCTAssertEqual(verdict("file:///etc/passwd"), .cancel)
            XCTAssertEqual(verdict("data:text/html,x"), .cancel)
            XCTAssertEqual(verdict("data:text/html,x", mainFrame: false), .allow)
            XCTAssertEqual(verdict("mailto:ada@example.com"), .external)
            XCTAssertEqual(verdict(""), .cancel)
        }

        func testAStoppedCoreRefusesEverything() async throws {
            let core = try NativeCore.start(config: fakeConfig(), host: RecordingHost())
            core.stop()
            core.stop() // twice is harmless
            do {
                try await core.perform(.ping(WTPing()))
                XCTFail("expected the core to have stopped")
            } catch {
                XCTAssertEqual(error as? CoreError, .stopped)
            }
            XCTAssertEqual(core.navigationVerdict(for: "https://example.com/", webViewID: 1, mainFrame: true, userGesture: false), .cancel)
        }

        func testStoppingAnswersWhatWasSentAndNotYetAnswered() async throws {
            let core = try NativeCore.start(config: fakeConfig(), host: RecordingHost())
            let sent = (0 ..< 20).map { _ in core.send(.ping(WTPing())) }
            core.stop()
            // Each was answered, or says the core stopped: none waits for ever.
            for answer in sent {
                do {
                    _ = try await answer.value
                } catch {
                    XCTAssertEqual(error as? CoreError, .stopped)
                }
            }
        }
    }
#endif

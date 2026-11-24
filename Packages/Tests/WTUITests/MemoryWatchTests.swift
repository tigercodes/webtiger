// What the system says of its memory reaches the core as the right word (055): short for the
// memory-pressure source's warning and for an iPhone's memory warning, very short for the
// source's critical. No test makes a system short of memory: the source's words are handed to
// the watch as the source would hand them, the app's warning is posted in the test's own
// process to a centre of the test's own, and on the simulator the system's source is heard for
// real when the simulator gives its own warning, which is a warning and no shortage.
import Dispatch
import WTContract
import WTCoreClient
@testable import WTUI
import XCTest
#if canImport(UIKit)
    import UIKit
#endif

@MainActor
final class MemoryWatchTests: XCTestCase {
    /// The levels the core was told, in order.
    private func told(_ core: ScriptedCore) -> [WTMemoryLevel] {
        core.requests.compactMap { request in
            if case let .memoryShort(short) = request {
                return short.level
            }
            return nil
        }
    }

    func testTheSourcesWarningIsShortAndItsCriticalVeryShort() {
        XCTAssertEqual(MemoryWatch.level(of: .warning), .short)
        XCTAssertEqual(MemoryWatch.level(of: .critical), .veryShort)
        XCTAssertEqual(MemoryWatch.level(of: [.warning, .critical]), .veryShort, "the worse of the two")
        XCTAssertEqual(MemoryWatch.level(of: [.normal, .warning]), .short)
        XCTAssertNil(MemoryWatch.level(of: .normal), "that a shortage is over is no word for the core")
        XCTAssertNil(MemoryWatch.level(of: []))
    }

    func testWhatTheSourceSaysReachesTheCoreAsThatWordAndNothingElseDoes() {
        let core = ScriptedCore()
        let watch = MemoryWatch(core: core)
        watch.heard(.warning)
        watch.heard(.normal)
        watch.heard(.critical)
        watch.heard(.warning)
        XCTAssertEqual(told(core), [.short, .veryShort, .short], "each time it is said, and in order")
        XCTAssertEqual(core.requests.count, 3, "nothing else was asked of the core")
    }

    func testAWatchThatWasNotToldToListenSaysNothing() {
        let core = ScriptedCore()
        let centre = NotificationCenter()
        let watch = MemoryWatch(core: core, centre: centre)
        #if canImport(UIKit)
            centre.post(name: UIApplication.didReceiveMemoryWarningNotification, object: nil)
        #endif
        RunLoop.main.run(until: Date().addingTimeInterval(0.2))
        XCTAssertEqual(core.requests.count, 0)
        watch.stop()
    }

    #if canImport(UIKit)
        /// Lets the main queue run until the core has been told so many times, or two seconds
        /// have passed.
        private func wait(for count: Int, in core: ScriptedCore) {
            let until = Date().addingTimeInterval(2)
            while core.requests.count < count, Date() < until {
                RunLoop.main.run(until: Date().addingTimeInterval(0.02))
            }
        }

        func testTheAppsMemoryWarningIsShortEachTimeAndNotOnceStopped() {
            let core = ScriptedCore()
            let centre = NotificationCenter()
            let watch = MemoryWatch(core: core, centre: centre)
            watch.listenForWarnings()
            // Told to listen twice, it listens once.
            watch.listenForWarnings()
            centre.post(name: UIApplication.didReceiveMemoryWarningNotification, object: nil)
            wait(for: 1, in: core)
            XCTAssertEqual(told(core), [.short], "it comes with the source's warning too: the worse word is the source's")
            centre.post(name: UIApplication.didReceiveMemoryWarningNotification, object: nil)
            wait(for: 2, in: core)
            XCTAssertEqual(told(core), [.short, .short])
            // Another notification is none of its business.
            centre.post(name: UIApplication.didEnterBackgroundNotification, object: nil)
            RunLoop.main.run(until: Date().addingTimeInterval(0.2))
            XCTAssertEqual(told(core).count, 2)

            watch.stop()
            centre.post(name: UIApplication.didReceiveMemoryWarningNotification, object: nil)
            RunLoop.main.run(until: Date().addingTimeInterval(0.2))
            XCTAssertEqual(told(core).count, 2, "stopped, it says nothing")
        }
    #endif

    #if targetEnvironment(simulator)
        /// The system's source, for real, on the simulator. The simulator warns its apps of
        /// memory (its Debug menu's "Simulate Memory Warning") by a file it names to every
        /// process it runs, in `SIMULATOR_MEMORY_WARNINGS`: a change to that file is the
        /// warning, and the system's memory-pressure source is what hears it. Nothing is made
        /// short of memory by it. The file's time is changed here, and nothing else of it.
        func testTheSimulatorsMemoryWarningReachesTheCoreThroughTheSystemsSource() throws {
            let file = try XCTUnwrap(
                ProcessInfo.processInfo.environment["SIMULATOR_MEMORY_WARNINGS"], "the simulator names its warning's file"
            )
            let core = ScriptedCore()
            let watch = MemoryWatch(core: core)
            watch.listenForPressure()
            defer { watch.stop() }
            RunLoop.main.run(until: Date().addingTimeInterval(0.2))
            XCTAssertEqual(told(core), [], "nothing is said until the system says something")

            XCTAssertEqual(utimes(file, nil), 0, "the file's time was changed")
            wait(for: 1, in: core)
            XCTAssertEqual(told(core), [.short], "the simulator's warning is the source's warning: short")
        }
    #endif

    #if WEBTIGER_CORE
        func testFakeModeDoesNotListenToTheSystemAndTheAppItselfDoes() throws {
            XCTAssertFalse(BrowserSession.hearsTheSystemsMemory(fake: true))
            XCTAssertTrue(BrowserSession.hearsTheSystemsMemory(fake: false))
            // A session in fake mode, over the real core: it has no watch.
            let session = try BrowserSession.live(environment: ["WEBTIGER_FAKE": "1"])
            defer { session.core.stop() }
            XCTAssertNil(session.memory)
        }
    #endif
}

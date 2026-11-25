// What the app says of the device it runs on (054): the core works out from it how many tabs
// may be alive at once. And the knobs a suite hands fake mode, the limit among them.
#if WEBTIGER_CORE
    import WTContract
    @testable import WTUI
    import XCTest
    #if canImport(UIKit)
        import UIKit
    #endif

    @MainActor
    final class DeviceTests: XCTestCase {
        func testTheAppSaysWhatItRunsOnAndItsMemory() {
            let device = BrowserSession.device()
            #if os(macOS)
                XCTAssertEqual(device.kind, .desktop)
            #else
                // The simulator this runs on: an iPhone's, unless someone ran it on an iPad's.
                XCTAssertEqual(device.kind, UIDevice.current.userInterfaceIdiom == .pad ? .tablet : .phone)
                XCTAssertEqual(BrowserSession.kind(of: .pad), .tablet)
                XCTAssertEqual(BrowserSession.kind(of: .phone), .phone)
                XCTAssertEqual(BrowserSession.kind(of: .unspecified), .phone, "anything else is taken for a phone")
            #endif
            // The device's own memory, in megabytes, as the kernel says it another way.
            var bytes: UInt64 = 0
            var size = MemoryLayout<UInt64>.size
            XCTAssertEqual(sysctlbyname("hw.memsize", &bytes, &size, nil, 0), 0, "the kernel says the memory")
            XCTAssertGreaterThan(bytes, 1024 * 1024 * 1024, "more than a gigabyte, on anything this runs on")
            XCTAssertEqual(UInt64(device.megabytes), bytes / (1024 * 1024))
        }

        func testASuitesKnobsAreHandedToFakeMode() {
            XCTAssertEqual(BrowserSession.fakeOptions([:]), WTFakeOptions(), "nothing said is nothing set")
            let all = BrowserSession.fakeOptions([
                "WEBTIGER_FAKE_START": "https://example.com/",
                "WEBTIGER_FAKE_FIRST_RUN": "1",
                "WEBTIGER_FAKE_CLOCK_STEP": "86400",
                "WEBTIGER_FAKE_SLEEP_AFTER": "4",
                "WEBTIGER_FAKE_LIVE_TABS": "2",
                "WEBTIGER_FAKE_MEMORY_SHORT_EVERY": "3",
                "WEBTIGER_FAKE_STANDARD": "1",
            ])
            XCTAssertEqual(all.startAddress, "https://example.com/")
            XCTAssertTrue(all.firstRun)
            XCTAssertEqual(all.secondsARead, 86400)
            XCTAssertEqual(all.sleepAfterSeconds, 4)
            XCTAssertEqual(all.liveTabs, 2)
            XCTAssertEqual(all.memoryShortEverySeconds, 3)
            XCTAssertTrue(all.standardSession)
            // Only "1" asks for the standard session.
            for said in ["0", "yes", "", "true"] {
                XCTAssertFalse(BrowserSession.fakeOptions(["WEBTIGER_FAKE_STANDARD": said]).standardSession, said)
            }
            // What is no count is no limit, and no seconds are never.
            for said in ["many", "-1", "", "2.5"] {
                XCTAssertEqual(BrowserSession.fakeOptions(["WEBTIGER_FAKE_LIVE_TABS": said]).liveTabs, 0, said)
                XCTAssertEqual(BrowserSession.fakeOptions(["WEBTIGER_FAKE_MEMORY_SHORT_EVERY": said]).memoryShortEverySeconds, 0, said)
            }
        }
    }
#endif

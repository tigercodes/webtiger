// When the system is short of memory (055), in the app.
//
// In fake mode, with the real core and the real engine: the core is told every three seconds,
// by the suite's knob (`WEBTIGER_FAKE_MEMORY_SHORT_EVERY`), as a shell tells it when its
// system says so. No test makes a system short of memory: it is everybody's memory. The tabs
// left behind are asleep seconds later, with no time and no limit of live tabs to put them
// there; the one in front is not, however often the core is told; one that is pressed wakes
// at the page it was at with its way back.
//
// And on the iPhone simulator, the app itself on a profile of the suite's own, which listens
// to its system: the simulator's own warning of memory is given, as its Debug menu's
// "Simulate Memory Warning" gives it, and the tabs left behind are asleep. Nothing is made
// short of memory by that: it is the warning alone, to the simulator's apps alone.
import XCTest

@MainActor
final class MemoryTests: AppTestCase {
    /// Sends the tab in front to a page of the test's site.
    private func go(to path: String, titled title: String, in app: XCUIApplication) {
        let field = app.textFields["address-field"]
        openLocation(field, in: app)
        field.typeText("127.0.0.1:\(site.port)\(path)\n")
        XCTAssertTrue(pageShows(title, in: app), "the page at \(path) loaded")
    }

    /// Opens a tab in front of the others, at a page of the test's site.
    private func openTab(at path: String, titled title: String, in app: XCUIApplication) {
        press(app.buttons["new-tab"])
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "an empty tab")
        go(to: path, titled: title, in: app)
    }

    /// Waits for a button to be one that can be pressed.
    private func expectEnabled(_ button: XCUIElement, _ why: String, file: StaticString = #filePath, line: UInt = #line) {
        let enabled = XCTNSPredicateExpectation(predicate: NSPredicate(format: "isEnabled == true"), object: button)
        XCTAssertEqual(XCTWaiter.wait(for: [enabled], timeout: 30), .completed, why, file: file, line: line)
    }

    func testTheTabsLeftBehindSleepWhenTheCoreIsToldTheSystemIsShortOfMemory() {
        let app = launch(start: site.url("/"), memoryShortEvery: 3)
        XCTAssertTrue(pageShows("Home", in: app), "the first tab's page loaded")
        // The first tab goes on to Landed: it has somewhere to go back to. Two more come in
        // front of it, one after the other.
        go(to: "/landed", titled: "Landed", in: app)
        openTab(at: "/whoami", titled: "who=none", in: app)
        openTab(at: "/tall", titled: "Tall", in: app)
        let tabs = app.windows.element(boundBy: 0).buttons.matching(identifier: "tab")
        let first = tabs.element(boundBy: 0)
        let second = tabs.element(boundBy: 1)
        let third = tabs.element(boundBy: 2)

        // Seconds on, both that were left behind are asleep, each where it was in its row and
        // by the name it had. Nothing else would have put them there: a tab sleeps by itself
        // after half an hour, and fake mode has no limit of live tabs.
        expect(first, toSay: "Asleep", "a tab left behind sleeps when the system is short of memory")
        expect(second, toSay: "Asleep", "and the other with it")
        XCTAssertEqual(first.label, "Landed")
        XCTAssertEqual(second.label, "who=none")
        // The tab in front is awake, however often the core is told: twice more in this time.
        sleep(7)
        expect(third, toSay: "", "the tab in sight does not sleep")
        XCTAssertTrue(pageShows("Tall", in: app))

        // Pressed, the first wakes at the page it was at, and its way back came with it.
        press(first)
        XCTAssertTrue(pageShows("Landed", in: app), "the tab woke where it was")
        expect(first, toSay: "", "and is awake")
        let back = app.buttons["back"]
        expectEnabled(back, "a tab that wakes from its state has the list it had")
        press(back)
        XCTAssertTrue(pageShows("Home", in: app), "back goes back, to the page before it slept")
        // And the tab it was woken over is out of sight in its turn: it sleeps when the core
        // is next told.
        expect(third, toSay: "Asleep", "the tab that is now behind sleeps in its turn")
        expect(first, toSay: "", "the tab in sight stays awake")
    }

    #if os(iOS)
        /// Gives the simulator's warning of memory to the apps it runs. The simulator names a
        /// file to every process it runs, and a change to that file is the warning: the
        /// system's memory-pressure source hears it. Only the file's time is changed. No
        /// device has this: the test is the simulator's.
        private func warnOfMemory(file: StaticString = #filePath, line: UInt = #line) {
            guard let named = ProcessInfo.processInfo.environment["SIMULATOR_MEMORY_WARNINGS"] else {
                return XCTFail("the simulator names its warning's file", file: file, line: line)
            }
            XCTAssertEqual(utimes(named, nil), 0, "the file's time was changed", file: file, line: line)
        }

        /// The app itself, with nothing said of memory by a suite: the system's warning reaches
        /// the core through the app's own listener, and the tabs left behind sleep.
        func testTheAppHearsItsSystemsMemoryWarningAndTheTabsLeftBehindSleep() {
            let app = launch(onProfile: "ui-test-memory", fresh: true)
            // A fresh profile is a first run (020): it asks where searches go before anything else.
            let question = app.buttons["search-engine-ecosia"]
            XCTAssertTrue(question.waitForExistence(timeout: 30), "a fresh profile is asked where searches go")
            press(question)
            XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "a fresh profile opens one empty tab")

            // Three tabs, as many as an iPhone holds alive: nothing but the warning will put
            // one to sleep in the time this takes.
            go(to: "/landed", titled: "Landed", in: app)
            openTab(at: "/whoami", titled: "who=none", in: app)
            openTab(at: "/tall", titled: "Tall", in: app)
            let tabs = app.windows.element(boundBy: 0).buttons.matching(identifier: "tab")
            let first = tabs.element(boundBy: 0)
            let second = tabs.element(boundBy: 1)
            let third = tabs.element(boundBy: 2)
            sleep(3)
            expect(first, toSay: "", "nothing has put the first to sleep")
            expect(second, toSay: "", "nor the second")

            // The warning. The two left behind sleep, where they were and by the names they
            // had; the one in sight stays.
            warnOfMemory()
            expect(first, toSay: "Asleep", "a tab left behind sleeps when the system warns of memory")
            expect(second, toSay: "Asleep", "and the other with it")
            XCTAssertEqual(first.label, "Landed")
            expect(third, toSay: "", "the tab in sight does not sleep")
            XCTAssertTrue(pageShows("Tall", in: app))

            // Pressed, the first wakes where it was.
            press(first)
            XCTAssertTrue(pageShows("Landed", in: app), "the tab woke where it was")
            expect(first, toSay: "", "and is awake")
        }
    #endif
}

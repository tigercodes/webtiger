// What the apps' UI suites share: the loopback site the pages come from, how the app is started
// (in fake mode, or on a profile of the suite's own), and Apple's accessibility audit with the
// exceptions it is allowed, each with its reason.
import XCTest
#if os(macOS)
    import AppKit
#endif

@MainActor
class AppTestCase: XCTestCase {
    let site = TestSite()

    override func setUp() async throws {
        continueAfterFailure = false
        XCTAssertNotEqual(site.port, 0, "the loopback site is listening")
    }

    /// Starts the app in fake mode, at an address or at an empty tab.
    ///
    /// Nothing a page keeps outlives the app in fake mode (a cookie with an expiry, storage):
    /// the engine keeps it in memory there, as the core keeps its store. Each test starts
    /// with nobody signed in anywhere, whatever the test before it did.
    ///
    /// With `clockStep`, the core's clock moves that many seconds each time it is read, and
    /// not one: for a test that must see a day pass (049).
    func launch(start: String?, firstRun: Bool = false, icons: Bool = false, clockStep: Int = 0) -> XCUIApplication {
        let app = inFakeMode(start: start, firstRun: firstRun)
        if clockStep > 0 {
            app.launchEnvironment["WEBTIGER_FAKE_CLOCK_STEP"] = String(clockStep)
        }
        if icons {
            // In fake mode the app asks for no page's icon (046 §5a) unless a suite says so;
            // and then of this device only, where the suite's site is.
            app.launchEnvironment["WEBTIGER_FAKE_ICONS"] = "1"
        }
        app.launch()
        return app
    }

    /// Starts the app in fake mode by handing it an address, as the system starts an app for
    /// a link pressed in another one (031). The address goes to the app under test and to no
    /// other. (XCTest starts a copy of the app for every address it is asked to open: a copy
    /// that is already running is not handed one this way.)
    func launch(start: String?, byLink address: URL) -> XCUIApplication {
        // Starting the app the usual way ends the copy the test before left running; being
        // started for an address does not. So that copy is ended here first.
        endEveryCopy()
        let app = inFakeMode(start: start, firstRun: false)
        app.open(address)
        return app
    }

    /// Ends the app under test, and waits until no copy of it is running: a copy left behind
    /// is in the way of the next test, whose clicks land on its window.
    func endEveryCopy(file: StaticString = #filePath, line: UInt = #line) {
        #if os(macOS)
            func copies() -> Int {
                NSRunningApplication.runningApplications(withBundleIdentifier: "com.tigercodes.webtiger").count
            }
            let waitedSince = Date()
            while copies() > 0, Date().timeIntervalSince(waitedSince) < 30 {
                XCUIApplication().terminate()
                usleep(300_000)
            }
            XCTAssertEqual(copies(), 0, "copies of the app are still running", file: file, line: line)
        #else
            XCUIApplication().terminate()
        #endif
    }

    /// The app, to be started in fake mode: nothing of the person's is read or written.
    private func inFakeMode(start: String?, firstRun: Bool) -> XCUIApplication {
        let app = XCUIApplication()
        app.launchEnvironment["WEBTIGER_FAKE"] = "1"
        if let start {
            app.launchEnvironment["WEBTIGER_FAKE_START"] = start
        }
        if firstRun {
            // Nobody has chosen where searches go yet (020): the app asks before anything else.
            app.launchEnvironment["WEBTIGER_FAKE_FIRST_RUN"] = "1"
        }
        startWithOneWindow(app)
        return app
    }

    /// A Mac puts back the windows an app had when it last ended, however many. A test starts
    /// from the one window the app opens by itself, whatever the run before it left open.
    private func startWithOneWindow(_ app: XCUIApplication) {
        #if os(macOS)
            app.launchArguments += ["-ApplePersistenceIgnoreState", "YES"]
        #endif
    }

    /// Launches the app itself, not in fake mode, on a profile of the suite's own: a store on
    /// disk in a folder apart from anyone's data, which the next launch finds as it was left.
    func launch(onProfile profile: String, fresh: Bool) -> XCUIApplication {
        let app = XCUIApplication()
        app.launchEnvironment["WEBTIGER_PROFILE"] = profile
        if fresh {
            app.launchEnvironment["WEBTIGER_PROFILE_FRESH"] = "1"
        }
        startWithOneWindow(app)
        app.launch()
        return app
    }

    /// Waits for the page titled `title` to be the one on screen.
    func pageShows(_ title: String, in app: XCUIApplication) -> Bool {
        #if os(macOS)
            // The window takes the page's title, which the core publishes.
            app.windows[title].waitForExistence(timeout: 30)
        #else
            app.webViews.staticTexts[title].waitForExistence(timeout: 30)
        #endif
    }

    /// Puts the keyboard in the address field with the whole address selected, so what is typed
    /// next replaces it: ⌘L on a Mac, a tap on a phone.
    func openLocation(_ field: XCUIElement, in app: XCUIApplication) {
        #if os(macOS)
            app.typeKey("l", modifierFlags: .command)
        #else
            field.tap()
        #endif
    }

    func press(_ button: XCUIElement) {
        #if os(macOS)
            button.click()
        #else
            button.tap()
        #endif
    }

    func value(of field: XCUIElement) -> String {
        field.value as? String ?? ""
    }

    /// Waits for the address field to show `text`.
    func expect(_ field: XCUIElement, toShow text: String, file: StaticString = #filePath, line: UInt = #line) {
        let shown = NSPredicate(format: "value == %@", text)
        let waited = XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: shown, object: field)], timeout: 30)
        XCTAssertEqual(waited, .completed, "the address field shows \(value(of: field)), not \(text)", file: file, line: line)
    }

    /// Runs Apple's accessibility audit and names every element it faults: its kind, its
    /// identifier and its label, never the whole tree.
    func audit(_ app: XCUIApplication, _ state: String, file: StaticString = #filePath, line: UInt = #line) {
        var faults: [String] = []
        do {
            try app.performAccessibilityAudit { issue in
                let element = issue.element
                // A container that only groups other elements needs no description of its own.
                if issue.auditType == .sufficientElementDescription, let kind = element?.elementType,
                   kind == .group || kind == .other, element?.identifier.isEmpty ?? true {
                    return true
                }
                // A tab with no title is named by its site, and a site's name is what it is: the
                // audit takes a host for a label nobody could read.
                if issue.auditType == .sufficientElementDescription, element?.identifier == "tab" {
                    return true
                }
                // An element cut by the window's edge is not drawn whole, and what is not drawn
                // has no contrast to measure. (A long screen that scrolls has one at its foot.)
                if issue.auditType == .contrast, let frame = element?.frame, !app.windows.firstMatch.frame.contains(frame) {
                    return true
                }
                // A control that cannot be pressed is drawn dimmed on purpose, to say so: it has
                // no contrast to meet (WCAG 1.4.3 excepts components that are not active).
                if issue.auditType == .contrast, element?.isEnabled == false {
                    return true
                }
                // Contrast is measured from the screen's pixels. Where a point is one pixel,
                // small text is drawn in greys that never reach its colour, and plain black on
                // white is faulted: GitHub's Mac does it, and the recording in its results
                // shows why. It is measured where a point has more pixels than one.
                if issue.auditType == .contrast, Self.screenIsCoarse {
                    return true
                }
                #if os(macOS)
                    // A fault with no element cannot be named, and so cannot be mended. macOS
                    // 27.0 reports one such mismatch in the hierarchy for every state of the
                    // window; 27.2 reports none.
                    if issue.auditType == .parentChild, element == nil {
                        return true
                    }
                #endif
                #if !os(macOS)
                    if Self.isNoFaultOnAPhone(issue, in: app) {
                        return true
                    }
                #endif
                // What the element is, where it is, and Apple's own account of the fault: enough
                // to find it from a runner's log, where nobody can look at the window.
                let frame = element.map { "\(Int($0.frame.minX)),\(Int($0.frame.minY)) \(Int($0.frame.width))x\(Int($0.frame.height))" }
                let what = [element.map { "\($0.elementType.rawValue)" }, element?.identifier, element?.label, frame]
                    .compactMap(\.self).filter { !$0.isEmpty }.joined(separator: " / ")
                faults.append("\(issue.compactDescription) [\(what)]: \(issue.detailedDescription)")
                return true
            }
        } catch {
            faults.append("the audit could not run: \(error.localizedDescription)")
        }
        XCTAssertTrue(faults.isEmpty, "\(state): \(faults.joined(separator: "; "))", file: file, line: line)
    }

    #if !os(macOS)
        /// What the audit faults on an iPhone that is no fault: each with its reason.
        private static func isNoFaultOnAPhone(_ issue: XCUIAccessibilityAuditIssue, in app: XCUIApplication) -> Bool {
            let element = issue.element
            // What is under the keyboard is not drawn, and has no contrast to measure: a
            // page that scrolls has its foot there while a field of it has the keyboard. The
            // frame the keyboard says it has is its rows of keys. The row of suggested words
            // above them covers the page as they do (45 points of it, measured on the iPhone
            // 17's simulator: the keys began at 590 and a heading at 552 was under the row),
            // and is counted with them.
            if issue.auditType == .contrast, let frame = element?.frame, app.keyboards.firstMatch.exists {
                let keys = app.keyboards.firstMatch.frame
                let covered = CGRect(x: keys.minX, y: keys.minY - 48, width: keys.width, height: keys.height + 48)
                if covered.intersects(frame) {
                    return true
                }
            }
            // A one-line text field scrolls what does not fit: that is not clipped text.
            if issue.auditType == .textClipped, element?.elementType == .textField {
                return true
            }
            // A tab in the strip shows one line of its title, and is read out whole.
            if issue.auditType == .textClipped, element?.identifier == "tab" {
                return true
            }
            // What the strip has scrolled part of the way out of view is not drawn whole: a tab,
            // or the name inside it. What is cut off has no contrast to measure, and what is
            // measured there is whatever is beside the strip. (With two tabs an iPhone's strip
            // scrolls.)
            if issue.auditType == .contrast, let frame = element?.frame {
                let strip = app.descendants(matching: .any).matching(identifier: "tab-strip").firstMatch
                if strip.exists, !strip.frame.contains(frame), frame.minY >= strip.frame.minY - 1, frame.maxY <= strip.frame.maxY + 1 {
                    return true
                }
            }
            // The title of what is offered under the address field is one line, cut at
            // its end, and is read out whole: beside the words that say a row is an
            // open tab (038) it is cut sooner at the largest text sizes.
            if issue.auditType == .textClipped, element?.identifier == "suggestion-title" {
                return true
            }
            // What a page draws is the page's own, and the audit is of the app's
            // controls: the test site's pages are plain small text, and a real page is
            // whatever its author made it. WebKit's elements say what they are.
            if issue.detailedDescription.contains("WebAccessibilityObjectWrapper") {
                return true
            }
            // The system's keyboard is the system's: its row of suggested words is on
            // the screen whenever a field of the app's has the keyboard.
            if issue.detailedDescription.contains("TUIPrediction") || element?.elementType == .key
                || element?.elementType == .keyboard {
                return true
            }
            // A visit's address in the History list may be any length: it is cut to
            // two lines, in the middle, and is read out whole.
            if issue.auditType == .textClipped, element?.identifier == "history-address" {
                return true
            }
            // And so a tidied tab's address in the Archive (049): the same line, cut
            // the same way.
            if issue.auditType == .textClipped, element?.identifier == "archive-address" {
                return true
            }
            // The same of a place offered under the address field (011): its address
            // is cut to one line, in the middle.
            if issue.auditType == .textClipped, element?.identifier == "suggestion-address" {
                return true
            }
            return false
        }
    #endif

    /// Whether a point of the screen is a single pixel: too coarse for contrast to be measured.
    static var screenIsCoarse: Bool {
        #if os(macOS)
            (NSScreen.main?.backingScaleFactor ?? 2) < 2
        #else
            false
        #endif
    }
}

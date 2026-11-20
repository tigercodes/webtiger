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

    func launch(start: String?, firstRun: Bool = false) -> XCUIApplication {
        let app = XCUIApplication()
        app.launchEnvironment["WEBTIGER_FAKE"] = "1"
        if let start {
            app.launchEnvironment["WEBTIGER_FAKE_START"] = start
        }
        if firstRun {
            // Nobody has chosen where searches go yet (020): the app asks before anything else.
            app.launchEnvironment["WEBTIGER_FAKE_FIRST_RUN"] = "1"
        }
        app.launch()
        return app
    }

    /// Launches the app itself, not in fake mode, on a profile of the suite's own: a store on
    /// disk in a folder apart from anyone's data, which the next launch finds as it was left.
    func launch(onProfile profile: String, fresh: Bool) -> XCUIApplication {
        let app = XCUIApplication()
        app.launchEnvironment["WEBTIGER_PROFILE"] = profile
        if fresh {
            app.launchEnvironment["WEBTIGER_PROFILE_FRESH"] = "1"
        }
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
                    // A one-line text field scrolls what does not fit: that is not clipped text.
                    if issue.auditType == .textClipped, element?.elementType == .textField {
                        return true
                    }
                    // A tab in the strip shows one line of its title, and is read out whole.
                    if issue.auditType == .textClipped, element?.identifier == "tab" {
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
                    // The same of a place offered under the address field (011): its address
                    // is cut to one line, in the middle.
                    if issue.auditType == .textClipped, element?.identifier == "suggestion-address" {
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

    /// Whether a point of the screen is a single pixel: too coarse for contrast to be measured.
    static var screenIsCoarse: Bool {
        #if os(macOS)
            (NSScreen.main?.backingScaleFactor ?? 2) < 2
        #else
            false
        #endif
    }
}

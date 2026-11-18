// The W0 exit, as a test: the app starts the Rust core, the core asks the shell for a web view
// in the Space's login set, WebKit loads a page, and what the address field shows is what the
// core published. Fake mode keeps the store in memory, so the test leaves nothing behind.
import XCTest

@MainActor
final class LaunchTests: XCTestCase {
    private let site = TestSite()

    override func setUp() async throws {
        continueAfterFailure = false
        XCTAssertNotEqual(site.port, 0, "the loopback site is listening")
    }

    private func launch(start: String?) -> XCUIApplication {
        let app = XCUIApplication()
        app.launchEnvironment["WEBTIGER_FAKE"] = "1"
        if let start {
            app.launchEnvironment["WEBTIGER_FAKE_START"] = start
        }
        app.launch()
        return app
    }

    /// Waits for the page titled `title` to be the one on screen.
    private func pageShows(_ title: String, in app: XCUIApplication) -> Bool {
        #if os(macOS)
            // The window takes the page's title, which the core publishes.
            app.windows[title].waitForExistence(timeout: 30)
        #else
            app.webViews.staticTexts[title].waitForExistence(timeout: 30)
        #endif
    }

    /// Puts the keyboard in the address field with the whole address selected, so what is typed
    /// next replaces it: ⌘L on a Mac, a tap on a phone.
    private func openLocation(_ field: XCUIElement, in app: XCUIApplication) {
        #if os(macOS)
            app.typeKey("l", modifierFlags: .command)
        #else
            field.tap()
        #endif
    }

    private func press(_ button: XCUIElement) {
        #if os(macOS)
            button.click()
        #else
            button.tap()
        #endif
    }

    private func value(of field: XCUIElement) -> String {
        field.value as? String ?? ""
    }

    /// Waits for the address field to show `text`.
    private func expect(_ field: XCUIElement, toShow text: String, file: StaticString = #filePath, line: UInt = #line) {
        let shown = NSPredicate(format: "value == %@", text)
        let waited = XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: shown, object: field)], timeout: 30)
        XCTAssertEqual(waited, .completed, "the address field shows \(value(of: field)), not \(text)", file: file, line: line)
    }

    func testAPageLoadsThroughTheCore() {
        let app = launch(start: site.url("/"))
        let field = app.textFields["address-field"]
        XCTAssertTrue(field.waitForExistence(timeout: 30))
        XCTAssertTrue(pageShows("Home", in: app), "the page loaded")
        // The field shows what the core says the address is: plain http keeps its scheme.
        expect(field, toShow: "http://127.0.0.1:\(site.port)")
        XCTAssertTrue(app.images["security-insecure"].exists, "an http page is marked not secure")
    }

    func testTypingAnAddressLoadsIt() {
        let app = launch(start: nil)
        XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "a new tab is empty")
        let field = app.textFields["address-field"]
        XCTAssertTrue(field.waitForExistence(timeout: 30), "the address field exists")
        openLocation(field, in: app)
        // No scheme typed: the core decides this is an address on the loopback interface.
        field.typeText("127.0.0.1:\(site.port)/landed\n")
        XCTAssertTrue(pageShows("Landed", in: app), "the page loaded")
        expect(field, toShow: "http://127.0.0.1:\(site.port)/landed")

        // Back is offered once there is somewhere to go back to.
        openLocation(field, in: app)
        field.typeText("127.0.0.1:\(site.port)/\n")
        XCTAssertTrue(pageShows("Home", in: app), "typing replaced the address; the field shows \(value(of: field))")
        let back = app.buttons["back"]
        XCTAssertTrue(back.isEnabled, "back is offered")
        press(back)
        XCTAssertTrue(pageShows("Landed", in: app), "back goes back")
    }

    func testAPageThatCannotBeReachedSaysSoInWords() {
        // RFC 6761 reserves .invalid: it never resolves.
        let app = launch(start: "http://webtiger-test.invalid/")
        XCTAssertTrue(app.staticTexts["This page didn’t load"].waitForExistence(timeout: 30))
        XCTAssertTrue(app.staticTexts["The site’s address couldn’t be found."].exists, "the code became words")
        XCTAssertTrue(app.buttons["retry"].exists)
    }

    /// Runs Apple's accessibility audit and names every element it faults: its kind, its
    /// identifier and its label, never the whole tree.
    private func audit(_ app: XCUIApplication, _ state: String, file: StaticString = #filePath, line: UInt = #line) {
        var faults: [String] = []
        do {
            try app.performAccessibilityAudit { issue in
                let element = issue.element
                // A container that only groups other elements needs no description of its own.
                if issue.auditType == .sufficientElementDescription, let kind = element?.elementType,
                   kind == .group || kind == .other, element?.identifier.isEmpty ?? true {
                    return true
                }
                #if !os(macOS)
                    // A one-line text field scrolls what does not fit: that is not clipped text.
                    if issue.auditType == .textClipped, element?.elementType == .textField {
                        return true
                    }
                #endif
                let what = [element.map { "\($0.elementType.rawValue)" }, element?.identifier, element?.label]
                    .compactMap(\.self).filter { !$0.isEmpty }.joined(separator: " / ")
                faults.append("\(issue.compactDescription) [\(what)]")
                return true
            }
        } catch {
            faults.append("the audit could not run: \(error.localizedDescription)")
        }
        XCTAssertTrue(faults.isEmpty, "\(state): \(faults.joined(separator: "; "))", file: file, line: line)
    }

    /// Apple's accessibility audit over each state of the window (000 §8.6): labels, contrast,
    /// hit regions, clipped text.
    func testEveryStateOfTheWindowPassesTheAccessibilityAudit() {
        continueAfterFailure = true
        let empty = launch(start: nil)
        XCTAssertTrue(empty.staticTexts["empty-tab"].waitForExistence(timeout: 30))
        audit(empty, "an empty tab")
        empty.terminate()

        let failed = launch(start: "http://webtiger-test.invalid/")
        XCTAssertTrue(failed.buttons["retry"].waitForExistence(timeout: 30))
        audit(failed, "a failed page")
    }
}

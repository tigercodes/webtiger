// Dragging in the row (045 §5b) in the app, in fake mode, with the real core and the real
// engine. A tab is picked up and dropped: on another tab, and is then just before it; on "New
// Tab", and is the last of the tabs; on a Group's chip, and is a Site of that Group. On a Mac,
// with the tabs down the side: on the line under the Groups, and is a Site in no Group; and a
// Group dropped on another is before it. What a drop did is undone as any move is. Nothing is
// loaded by any of it.
//
// On a Mac, where a thing is dragged with the pointer. On a touch screen nothing is dragged
// yet (045 §9).
import XCTest

#if os(macOS)
    @MainActor
    final class DraggingTests: AppTestCase {
        /// Picks a thing up and lets it go on another: slowly, and held a moment where it is let
        /// go, as a hand does it, so that the place it is over has been told.
        private func drag(_ thing: XCUIElement, onto place: XCUIElement) {
            thing.click(forDuration: 0.5, thenDragTo: place, withVelocity: .slow, thenHoldForDuration: 0.5)
        }

        /// Undoes what was last done to the row, by its command's words in the Edit menu.
        private func undo(_ words: String, by tab: XCUIElement, in app: XCUIApplication, line: UInt = #line) {
            choose(words, for: tab, in: app, menu: "Edit", line: line)
        }

        /// A window with two tabs, "Landed" and "who=none", the second in front.
        private func twoTabs() -> XCUIApplication {
            let app = launch(start: site.url("/landed"))
            XCTAssertTrue(pageShows("Landed", in: app), "the first tab's page loaded")
            let field = app.textFields["address-field"]
            press(app.buttons["new-tab"])
            XCTAssertTrue(app.staticTexts["empty-tab"].waitForExistence(timeout: 30), "a second, empty tab")
            openLocation(field, in: app)
            field.typeText("127.0.0.1:\(site.port)/whoami\n")
            XCTAssertTrue(pageShows("who=none", in: app), "the second tab's page loaded")
            return app
        }

        func testATabIsDroppedOnAnotherOnNewTabAndOnAGroup() {
            let app = twoTabs()
            let tabs = app.windows.element(boundBy: 0).buttons.matching(identifier: "tab")
            let chip = app.descendants(matching: .any).matching(identifier: "group").firstMatch
            expect(tabs, toBe: ["Landed", "who=none"], "two tabs, in the order they were opened")
            let asked = site.requests.count

            // Dropped on the first, the second is put before it; it is in front still.
            drag(tabs.element(boundBy: 1), onto: tabs.element(boundBy: 0))
            expect(tabs, toBe: ["who=none", "Landed"], "the tab dropped on the first is before it")
            XCTAssertTrue(pageShows("who=none", in: app), "and is in front still, with its page")
            // Dropped on "New Tab", which ends the row, it is the last of the tabs. No tab was opened.
            drag(tabs.element(boundBy: 0), onto: app.buttons["new-tab"])
            expect(tabs, toBe: ["Landed", "who=none"], "the tab dropped on New Tab is the last")
            // And undone: it is before the other again.
            undo("Undo Move Tab", by: tabs.element(boundBy: 1), in: app)
            expect(tabs, toBe: ["who=none", "Landed"], "the move was undone")

            // A Group is made for "Landed". The other tab, dropped on the Group's chip, is a Site
            // of that Group: the last of them.
            press(tabs.element(boundBy: 1))
            XCTAssertTrue(pageShows("Landed", in: app), "the tab to be grouped is in front")
            makeGroup("Work", for: tabs.element(boundBy: 1), in: app)
            XCTAssertTrue(chip.waitForExistence(timeout: 30), "the Group's chip is in the row")
            expect(tabs, toBe: ["Landed", "who=none"], "the Group's Site is before the tab that is none")
            expect(tabs.element(boundBy: 1), toSay: "", "the other is a tab like any other")
            drag(tabs.element(boundBy: 1), onto: chip)
            expect(tabs.element(boundBy: 1), toSay: "Site", "dropped on the Group, the tab is kept")
            expect(chip, toSay: "Group, open", "the Group is open still")
            // Shut, the Group holds them both: the tab went into it.
            press(chip)
            expect(chip, toSay: "Group, shut, Sites: 2", "the Group has two Sites")
            press(chip)
            expect(chip, toSay: "Group, open", "open again")
            // Undone, it is a tab again.
            undo("Undo Move Tab", by: tabs.element(boundBy: 1), in: app)
            expect(tabs.element(boundBy: 1), toSay: "", "undone, it is a tab like any other")
            XCTAssertEqual(site.requests.count, asked, "none of it loaded anything")
        }

        func testDownTheSideATabDroppedOnTheLineIsASiteAndAGroupDroppedOnAnotherIsBeforeIt() {
            let app = twoTabs()
            let window = app.windows.element(boundBy: 0)
            let tabs = window.buttons.matching(identifier: "tab")
            let chips = app.descendants(matching: .any).matching(identifier: "group")
            let asked = site.requests.count
            switchLayout("Tabs at the Side", in: app)
            expect(app.descendants(matching: .any).matching(identifier: "sidebar"), toCount: 1, "the tabs are down the side")

            // A row dropped on another is put before it.
            drag(tabs.element(boundBy: 1), onto: tabs.element(boundBy: 0))
            expect(tabs, toBe: ["who=none", "Landed"], "the row dropped on the first is above it")

            // A Group is made for "Landed": there is a line under it now, and the other tab
            // is under the line. Dropped on the line, the tab is a Site in no Group.
            press(tabs.element(boundBy: 1))
            XCTAssertTrue(pageShows("Landed", in: app), "the tab to be grouped is in front")
            makeGroup("Work", for: tabs.element(boundBy: 1), in: app)
            expect(chips, toCount: 1, "the Group's chip is in the sidebar")
            expect(tabs, toBe: ["Landed", "who=none"], "the Group's Site, the line, and the tab")
            let site = tabs.element(boundBy: 0)
            let other = tabs.element(boundBy: 1)
            // The line is between the last row above it and the first under it.
            let middle = (site.frame.maxY + other.frame.minY) / 2
            let line = window.coordinate(withNormalizedOffset: .zero)
                .withOffset(CGVector(dx: other.frame.midX - window.frame.minX, dy: middle - window.frame.minY))
            other.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.5))
                .click(forDuration: 0.5, thenDragTo: line, withVelocity: .slow, thenHoldForDuration: 0.5)
            let kept = tabs.matching(NSPredicate(format: "label == %@", "who=none")).firstMatch
            expect(kept, toSay: "Site", "dropped on the line, the tab is a Site")
            expect(chips.firstMatch, toSay: "Group, open", "and is in no Group: the Group has its one Site")
            press(chips.firstMatch)
            expect(chips.firstMatch, toSay: "Group, shut, Sites: 1", "one Site, as before")
            press(chips.firstMatch)
            expect(chips.firstMatch, toSay: "Group, open", "open again")

            // A second Group, for that Site. Dropped on the first Group, it is before it.
            press(kept)
            XCTAssertTrue(pageShows("who=none", in: app), "the Site to be grouped is in front")
            makeGroup("Play", for: kept, in: app)
            expect(chips, toBe: ["Work", "Play"], "the Groups in the order they were made")
            drag(chips.element(boundBy: 1), onto: chips.element(boundBy: 0))
            expect(chips, toBe: ["Play", "Work"], "the Group dropped on the first is before it")
            XCTAssertEqual(self.site.requests.count, asked, "none of it loaded anything")
        }

        /// A thing that can be dragged still says what it is now. Found by these tests: with
        /// dragging first put on a row's own button, a row of the sidebar went on telling a
        /// screen reader what it was when it was made.
        func testDownTheSideARowThatCanBeDraggedSaysWhatItIsNow() {
            let app = twoTabs()
            let field = app.textFields["address-field"]
            let tabs = app.windows.element(boundBy: 0).buttons.matching(identifier: "tab")
            let chip = app.descendants(matching: .any).matching(identifier: "group").firstMatch
            switchLayout("Tabs at the Side", in: app)
            expect(app.descendants(matching: .any).matching(identifier: "sidebar"), toCount: 1, "the tabs are down the side")
            expect(tabs, toBe: ["Landed", "who=none"], "two rows")
            XCTAssertEqual([tabs.element(boundBy: 0).isSelected, tabs.element(boundBy: 1).isSelected], [false, true])

            // Kept while the sidebar shows it: it says it is a Site, and is first.
            choose("Keep as Site", for: tabs.element(boundBy: 1), in: app)
            expect(tabs, toBe: ["who=none", "Landed"], "the Site is first")
            expect(tabs.element(boundBy: 0), toSay: "Site", "kept, down the side")
            // Its page changes: it has the new page's name, and says it has wandered.
            openLocation(field, in: app)
            field.typeText("127.0.0.1:\(site.port)/with-icon\n")
            XCTAssertTrue(pageShows("With icon", in: app), "the Site went to another page")
            expect(tabs, toBe: ["With icon", "Landed"], "the row has its page's name")
            expect(tabs.element(boundBy: 0), toSay: "Site, Away", "and says it is away from its home")
            // The other row comes in front: it is the one that is chosen.
            press(tabs.element(boundBy: 1))
            XCTAssertTrue(pageShows("Landed", in: app), "the other tab is in front")
            XCTAssertEqual([tabs.element(boundBy: 0).isSelected, tabs.element(boundBy: 1).isSelected], [false, true])
            // In a Group, and the Group shut and opened: the chip says so each time.
            makeGroup("Work", for: tabs.element(boundBy: 1), in: app)
            XCTAssertTrue(chip.waitForExistence(timeout: 30), "the Group's chip is in the sidebar")
            expect(chip, toSay: "Group, open", "a new Group")
            // The sidebar has places to drop on now, the line under the Group among them:
            // none of them is a control, and Apple's audit faults nothing in it.
            audit(app, "the sidebar, whose rows can be dragged")
            press(chip)
            expect(chip, toSay: "Group, shut, Sites: 1", "shut")
            press(chip)
            expect(chip, toSay: "Group, open", "open again")
            // A Favourite: its tile says so.
            press(tabs.matching(NSPredicate(format: "label == %@", "With icon")).firstMatch)
            XCTAssertTrue(pageShows("With icon", in: app), "the Site is in front")
            choose("Add to Favourites", for: tabs.element(boundBy: 0), in: app)
            expect(tabs.element(boundBy: 0), toSay: "Favourite, Away", "made a Favourite, its tile says so")
            XCTAssertTrue(tabs.element(boundBy: 0).isSelected, "and that it is the one in front")
        }
    }
#endif

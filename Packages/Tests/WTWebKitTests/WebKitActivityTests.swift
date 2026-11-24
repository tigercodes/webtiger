// What a page is doing that would keep its tab awake (052), through the adapter, against
// real WebKit and the test site: that it was typed into, and that it plays a sound, each
// reported when it begins and ends, with all the adapter knows as it now is. What is typed
// here is typed as an editing command, which is what a key press is to the page; what is
// played is silence.
import WebKit
import WTContract
import WTCoreClient
@testable import WTWebKit
import XCTest

@MainActor
final class WebKitActivityTests: AdapterTestCase {
    /// What the adapter has said the page of a web view is doing, in order: each as "sound",
    /// "capture", "typed" joined by commas, and empty for none of them.
    private func doings(_ id: UInt64) -> [String] {
        reports(id).compactMap { report in
            guard case let .pageActivityChanged(doing) = report else { return nil }
            return [doing.playsSound ? "sound" : nil, doing.captures ? "capture" : nil, doing.typedInto ? "typed" : nil]
                .compactMap(\.self).joined(separator: ",")
        }
    }

    /// Waits for the adapter to have said so many things of what a web view's page is doing.
    private func awaitDoings(_ id: UInt64, _ count: Int, file: StaticString = #filePath, line: UInt = #line) async {
        await wait(id, "\(count) reports of what the page is doing; said: \(doings(id))", file: file, line: line) { _ in
            self.doings(id).count >= count
        }
    }

    /// A web view on the page with the fields and the sound.
    private func page(_ id: UInt64 = 1, at path: String = "/doing", titled title: String = "Doing") async {
        await create(id)
        await load(id, site.url(path))
        await wait(id, "the page") { self.finished($0) && self.hasTitle(title)($0) }
    }

    /// Types into a field of the page, as an editing command.
    private func type(_ text: String, into field: String, of id: UInt64 = 1, inFrame: Bool = false) async {
        let document = inFrame ? "frames[0].document" : "document"
        await run("\(document).getElementById('\(field)').focus(); \(document).execCommand('insertText', false, '\(text)')", in: id)
    }

    /// Nothing more is said in a while: what was said is all there is.
    private func nothingMore(_ id: UInt64, than count: Int, file: StaticString = #filePath, line: UInt = #line) async {
        try? await Task.sleep(for: .milliseconds(400))
        XCTAssertEqual(doings(id).count, count, "said: \(doings(id))", file: file, line: line)
    }

    // MARK: Typing

    func testTypingIntoAFieldIsSaidOnceAndNothingOfWhatWasTyped() async {
        await page()
        await nothingMore(1, than: 0)

        await type("a draft", into: "line")
        await awaitDoings(1, 1)
        XCTAssertEqual(doings(1), ["typed"])
        // More of it changes nothing: it is said when it begins.
        await type(" of a letter", into: "line")
        await type("and more", into: "lines")
        await nothingMore(1, than: 1)
        // Nothing of what was typed is in anything the adapter reported.
        let said = core.reports.map { "\($0)" }.joined()
        XCTAssertFalse(said.contains("draft") || said.contains("letter"))
    }

    func testALargerFieldAndWordsThatCanBeEditedAreTypedIntoAndABoxThatIsTickedIsNot() async {
        await page(1)
        await type("lines", into: "lines")
        await awaitDoings(1, 1)
        XCTAssertEqual(doings(1), ["typed"])

        await page(2)
        await type("more ", into: "words", of: 2)
        await awaitDoings(2, 1)
        XCTAssertEqual(doings(2), ["typed"])

        // A box is ticked, by a click: that is no typing.
        await page(3)
        await run("document.getElementById('tick').click()", in: 3)
        let ticked = await run("document.getElementById('tick').checked", in: 3) as? Bool
        XCTAssertEqual(ticked, true, "the box took the click")
        await nothingMore(3, than: 0)
    }

    func testWhatAPagesOwnScriptPutsInAFieldIsNotTyping() async {
        await page()
        // The page fills its field itself, and says an input happened: no key went in.
        await run(
            "const line = document.getElementById('line'); line.value = 'filled in'; "
                + "line.dispatchEvent(new InputEvent('input', { bubbles: true }))",
            in: 1
        )
        await nothingMore(1, than: 0)
        // And nothing of the adapter's is the page's to call.
        let reachable = await run(
            "typeof webkit !== 'undefined' && !!webkit.messageHandlers && 'doing' in webkit.messageHandlers",
            in: 1
        ) as? Bool
        XCTAssertEqual(reachable, false, "the page cannot say what it is doing in the adapter's name")
    }

    func testAFrameThatIsTypedIntoIsThePageTypedInto() async {
        await page(1, at: "/doing-in-a-frame", titled: "Doing in a frame")
        await wait(1, "the frame's page") { _ in true }
        for _ in 0 ..< Patience.steps {
            if await run("!!frames[0].document.getElementById('line')", in: 1) as? Bool == true {
                break
            }
            try? await Task.sleep(for: .milliseconds(10))
        }
        await type("in the frame", into: "line", inFrame: true)
        await awaitDoings(1, 1)
        XCTAssertEqual(doings(1), ["typed"])
    }

    func testWhatWasTypedIsOfThePageAndAPageThatFollowsStartsWithNone() async {
        await page()
        await type("a draft", into: "line")
        await awaitDoings(1, 1)

        // Another page commits. The adapter says nothing of it by itself: the core takes a
        // commit as the end of what the page before was doing.
        await load(1, site.url("/tall"))
        await wait(1, "the next page") { self.finished($0, times: 2) && self.hasTitle("Tall")($0) }
        await nothingMore(1, than: 1)
        // Typed into in its turn, the new page is said to be: the adapter began again.
        await type("another", into: "note")
        await awaitDoings(1, 2)
        XCTAssertEqual(doings(1), ["typed", "typed"])
        // In the order of things: the second was said after the second page's commit.
        let order = names(reports(1)).filter { $0 == "committed" || $0.hasPrefix("doing") }
        XCTAssertEqual(order, ["committed", "doing(typed)", "committed", "doing(typed)"])
    }

    // MARK: Sound

    func testASoundThatPlaysIsSaidAndSaidToHaveStopped() async {
        await page()
        await run("document.getElementById('tune').play()", in: 1)
        await awaitDoings(1, 1)
        XCTAssertEqual(doings(1), ["sound"])
        // It plays on, again and again: nothing more is said while it does.
        await nothingMore(1, than: 1)

        await run("document.getElementById('tune').pause()", in: 1)
        await awaitDoings(1, 2)
        XCTAssertEqual(doings(1), ["sound", ""])

        // Typed into, and playing again: all the adapter knows, as it now is.
        await type("a draft", into: "line")
        await awaitDoings(1, 3)
        await run("document.getElementById('tune').play()", in: 1)
        await awaitDoings(1, 4)
        XCTAssertEqual(doings(1), ["sound", "", "typed", "sound,typed"])
    }

    func testASoundThatIsTurnedOffOrAllTheWayDownIsNotASound() async {
        await page()
        // Turned off before it plays: it plays, and nothing is heard.
        await run("const tune = document.getElementById('tune'); tune.muted = true; tune.play()", in: 1)
        await nothingMore(1, than: 0)
        // Turned on again while it plays: now it is a sound.
        await run("document.getElementById('tune').muted = false", in: 1)
        await awaitDoings(1, 1)
        XCTAssertEqual(doings(1), ["sound"])
        // All the way down: not a sound any more; and up again. On a Mac. On an iPhone the
        // loudness is the device's and never the page's: a page that sets it changes nothing
        // that is heard, WebKit says no change happened, and the sound is a sound still.
        await run("document.getElementById('tune').volume = 0", in: 1)
        #if os(iOS)
            await nothingMore(1, than: 1)
        #else
            await awaitDoings(1, 2)
            await run("document.getElementById('tune').volume = 0.5", in: 1)
            await awaitDoings(1, 3)
            XCTAssertEqual(doings(1), ["sound", "", "sound"])
        #endif
    }

    func testASoundGoesWithItsPage() async {
        await page()
        await run("document.getElementById('tune').play()", in: 1)
        await awaitDoings(1, 1)
        await load(1, site.url("/landed"))
        await wait(1, "the next page") { self.finished($0, times: 2) && self.hasTitle("Landed")($0) }
        // Whatever the page that went said as it went, the last word is not of a sound; and
        // nothing was said after the commit of the page that plays none.
        XCTAssertNotEqual(doings(1).last, "sound,typed")
        let afterCommit = names(reports(1)).drop { $0 != "committed" }.dropFirst().drop { $0 != "committed" }
        XCTAssertFalse(afterCommit.contains { $0.hasPrefix("doing(sound") }, "after the second commit: \(Array(afterCommit))")
    }

    // MARK: The camera and the microphone

    func testACameraOrAMicrophoneThatIsHeldIsHeldAlsoWhenItIsMuted() {
        // WebKit says of itself whether a page holds the camera or the microphone. No page can
        // be given one here (014 §10), so what is held to is the adapter's reading of what
        // WebKit says.
        XCTAssertFalse(PageDoing.captures(camera: .none, microphone: .none))
        XCTAssertTrue(PageDoing.captures(camera: .active, microphone: .none))
        XCTAssertTrue(PageDoing.captures(camera: .none, microphone: .active))
        XCTAssertTrue(PageDoing.captures(camera: .muted, microphone: .none))
        XCTAssertTrue(PageDoing.captures(camera: .none, microphone: .muted))
    }

    func testWhatIsToldIsAllThatIsKnownAndOnlyWhenItChanges() {
        var doing = PageDoing()
        XCTAssertNil(doing.news(camera: .none, microphone: .none), "a page that does nothing has nothing to tell")
        doing.heard(frame: "a", sound: true, typed: false)
        var told = doing.news(camera: .none, microphone: .none)
        XCTAssertEqual(told?.playsSound, true)
        XCTAssertEqual(told?.typedInto, false)
        XCTAssertNil(doing.news(camera: .none, microphone: .none), "said once")
        // Another frame plays too, and the first stops: the page plays still.
        doing.heard(frame: "b", sound: true, typed: true)
        told = doing.news(camera: .none, microphone: .none)
        XCTAssertEqual(told?.typedInto, true)
        doing.heard(frame: "a", sound: false, typed: false)
        XCTAssertNil(doing.news(camera: .none, microphone: .none), "one frame's sound is the page's")
        // The last stops: no sound; what was typed is typed still, whichever frame it was in.
        doing.heard(frame: "b", sound: false, typed: false)
        told = doing.news(camera: .none, microphone: .none)
        XCTAssertEqual(told?.playsSound, false)
        XCTAssertEqual(told?.typedInto, true)
        // The microphone is taken: said with the rest.
        told = doing.news(camera: .none, microphone: .active)
        XCTAssertEqual(told?.captures, true)
        XCTAssertEqual(told?.typedInto, true)
        // Another page: none of it, and nothing to tell until something is so.
        doing.committed()
        XCTAssertNil(doing.news(camera: .none, microphone: .none))
        told = doing.news(camera: .active, microphone: .none)
        XCTAssertEqual(told?.captures, true, "a camera the new page holds already is said")
        XCTAssertEqual(told?.typedInto, false)
    }
}

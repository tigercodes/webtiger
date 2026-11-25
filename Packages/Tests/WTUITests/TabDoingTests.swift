// Marks for what a tab is doing (058): which of the core's reasons has a mark in the row, which
// mark a tab with several has, and the words said of it beside its name. The shell decides
// nothing about a tab: the core says its reasons (052), and these are tests of what is drawn
// and said for them.
import SwiftUI
import WTContract
@testable import WTUI
import XCTest
#if canImport(AppKit)
    import AppKit
#elseif canImport(UIKit)
    import UIKit
#endif

@MainActor
final class TabDoingTests: XCTestCase {
    private func tab(_ reasons: [WTKeepAwake], kept: Bool = false, away: Bool = false, asleep: Bool = false) -> WTTabSummary {
        var tab = WTTabSummary()
        tab.title = "A page"
        tab.keepsAwake = reasons
        tab.kept = kept
        tab.away = away
        tab.asleep = asleep
        return tab
    }

    func testSoundAndTheCameraOrMicrophoneHaveAMarkAndTheOtherReasonsHaveNone() {
        XCTAssertEqual(TabDoing(of: tab([.sound])), .sound)
        XCTAssertEqual(TabDoing(of: tab([.capture])), .capture)
        XCTAssertNil(TabDoing(of: tab([])))
        for other in [WTKeepAwake.typed, .download, .question, .loading, .unspecified, .UNRECOGNIZED(99)] {
            XCTAssertNil(TabDoing(of: tab([other])), "\(other)")
        }
        // Every reason the contract has is one of the two, or has no mark: a new one gets a
        // mark only when somebody gives it one.
        for reason in WTKeepAwake.allCases {
            XCTAssertEqual(TabDoing(of: tab([reason])) != nil, reason == .sound || reason == .capture, "\(reason)")
        }
    }

    func testATabHasOneMarkAndTheCameraOrMicrophoneComesBeforeSound() {
        // In the core's order (sound, then the camera), and with others among them.
        XCTAssertEqual(TabDoing(of: tab([.sound, .capture])), .capture)
        XCTAssertEqual(TabDoing(of: tab([.capture, .sound])), .capture)
        XCTAssertEqual(TabDoing(of: tab([.sound, .typed, .loading])), .sound)
        XCTAssertEqual(TabDoing(of: tab([.typed, .capture, .question])), .capture)
    }

    func testEachMarkIsAPictureTheSystemHasAndHasItsOwnWords() {
        for doing in [TabDoing.capture, .sound] {
            #if canImport(AppKit)
                XCTAssertNotNil(NSImage(systemSymbolName: doing.symbol, accessibilityDescription: nil), "\(doing.symbol)")
            #else
                XCTAssertNotNil(UIImage(systemName: doing.symbol), "\(doing.symbol)")
            #endif
        }
        // A speaker for sound; a ring round a dot, as a thing that records has, for the camera
        // or the microphone.
        XCTAssertEqual(TabDoing.sound.symbol, "speaker.wave.2.fill")
        XCTAssertEqual(TabDoing.capture.symbol, "record.circle")
        let english = Locale(identifier: "en")
        XCTAssertEqual(TabDoing.sound.words.string(english), "Playing sound")
        XCTAssertEqual(TabDoing.capture.words.string(english), "Using the camera or microphone")
    }

    /// A tab's own label, drawn alone and small: the mark is a part of it that no comparison
    /// can overlook. (In a picture of a whole window it is a few dots among many: the window's
    /// picture was the same, as pictures are compared, with the marks and without. Found by
    /// leaving the mark out, 058 §7.)
    private func label(_ reasons: [WTKeepAwake]) throws -> Picture {
        let view = TabLabel(tab: tab(reasons), isFront: false)
            .padding(.horizontal, 8)
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .leading)
        return try XCTUnwrap(Snapshot.Stage(view, size: CGSize(width: 110, height: 28), dark: false).picture())
    }

    func testATabsMarkIsDrawnAndTheTwoMarksAreTwoPictures() throws {
        let none = try label([])
        XCTAssertEqual(try label([]).difference(from: none).share, 0, "the same label is the same picture")
        let sound = try label([.sound])
        let capture = try label([.capture])
        // Each mark is drawn: more of the label differs than a picture's comparison lets pass.
        XCTAssertGreaterThan(sound.difference(from: none).share, Picture.shareTolerance, "a speaker is drawn")
        XCTAssertGreaterThan(capture.difference(from: none).share, Picture.shareTolerance, "a recording mark is drawn")
        // And they are not the same mark.
        XCTAssertGreaterThan(sound.difference(from: capture).share, 0, "the two are two pictures")
        // With both, the camera's; and a reason with no mark draws nothing.
        XCTAssertEqual(try label([.sound, .capture]).difference(from: capture).share, 0, "with both, the camera's mark")
        XCTAssertEqual(try label([.typed, .loading]).difference(from: label([.loading])).share, 0, "typed text draws nothing")
    }

    func testWhatATabIsDoingIsSaidBesideItsNameAfterWhatItIs() {
        let english = Locale(identifier: "en")
        let said = { (tab: WTTabSummary) in TabStrip.said(of: tab, locale: english) }
        XCTAssertEqual(said(tab([])), "")
        XCTAssertEqual(said(tab([.sound])), "Playing sound")
        XCTAssertEqual(said(tab([.capture])), "Using the camera or microphone")
        XCTAssertEqual(said(tab([.sound, .capture])), "Using the camera or microphone", "one thing is said, as one mark is shown")
        // A reason with no mark is not said in the row: the list of what is awake says it.
        XCTAssertEqual(said(tab([.typed, .loading])), "")
        // After what the tab is, and where.
        XCTAssertEqual(said(tab([.sound], kept: true)), "Site, Playing sound")
        XCTAssertEqual(said(tab([.capture], kept: true, away: true)), "Site, Away, Using the camera or microphone")
        // A tab that is asleep has no page to be doing anything, and the core gives it no
        // reason: it says that it is asleep, as it did.
        XCTAssertEqual(said(tab([], kept: true, asleep: true)), "Site, Asleep")
    }
}

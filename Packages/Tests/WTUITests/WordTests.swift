// The catalogue's strings by name (040): every name the tool wrote is a string the String
// Catalog has, a key made while the app runs is answered for only when the catalogue has it,
// and arguments are put in.
import SwiftUI
@testable import WTUI
import XCTest

final class WordTests: XCTestCase {
    /// The String Catalog and the names are written from one file by one tool: every key the
    /// names know is a string the bundle has, in words that are not the key.
    func testEveryNameIsAStringTheBundleHas() {
        XCTAssertGreaterThan(Word.keys.count, 200)
        let missing = Word.keys.sorted().filter { Word(key: $0).string() == $0 }
        XCTAssertEqual(missing, [], "names with no string behind them")
        XCTAssertEqual(Words.tabsNew.key, "tabs.new")
        XCTAssertEqual(Words.tabsNew.string(), "New Tab")
        XCTAssertEqual(Words.spaceNewSharedDetail.key, "space.new.shared.detail")
    }

    /// A key made while the app runs: answered for when the catalogue has it, and not
    /// otherwise, so that no key's own text is ever shown for words.
    func testAKeyMadeAtRunTimeIsAStringOnlyWhenTheCatalogueHasIt() {
        XCTAssertEqual(Word(named: "error.net.dns")?.string(), Words.errorNetDns.string())
        XCTAssertNil(Word(named: "error.net.no_such_code"))
        XCTAssertNil(Word(named: ""))
        XCTAssertNil(Word(named: "tabs.new "))
        // What the app does with that: an error's words by its code, the nearest it has.
        XCTAssertEqual(FailedPageView.words(for: "net.dns"), Words.errorNetDns.string())
        XCTAssertEqual(FailedPageView.words(for: "net.dns.something_new"), Words.errorNetDns.string())
        XCTAssertEqual(FailedPageView.words(for: "no.such.code"), Words.errorNetFailed.string())
        XCTAssertEqual(SpaceColour.word("blue"), Words.spaceColourBlue.string())
        XCTAssertEqual(SpaceColour.word("no-such-colour"), "")
        XCTAssertEqual(SpaceColour.word(""), "")
    }

    func testArgumentsArePutIn() {
        XCTAssertEqual(Words.findCount.filled(2, 5), "2 of 5")
        XCTAssertEqual(Words.errorStartDetail.filled("store.open"), "Error code: store.open")
        // A string with no arguments is its words.
        XCTAssertEqual(Words.tabsNew.filled(), "New Tab")
    }
}

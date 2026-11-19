// The design catalogue as pictures (004 §4): the token sections and each state of the window,
// in the light and the dark appearance, compared with the pictures checked in for this system.
import SwiftUI
import WTDesign
@testable import WTUI
import XCTest

@MainActor
final class CatalogueSnapshotTests: XCTestCase {
    private static let tokensSize = CGSize(width: 480, height: 820)
    private static let windowSize = CGSize(width: 480, height: CatalogueWindow.height)

    private func verify(_ view: some View, size: CGSize, named name: String, file: StaticString = #filePath, line: UInt = #line) async {
        for dark in [false, true] {
            let stage = Snapshot.Stage(view, size: size, dark: dark)
            let picture = await stage.settledPicture()
            Snapshot.verify(picture, named: "\(name)-\(dark ? "dark" : "light")", file: file, line: line)
        }
    }

    private func skipWithoutPictures() throws {
        try XCTSkipUnless(
            Snapshot.hasPictures(),
            "no pictures were recorded for \(Snapshot.system): record them with WEBTIGER_RECORD_SNAPSHOTS=1 and look at each one"
        )
    }

    func testTheTokenSections() async throws {
        try skipWithoutPictures()
        await verify(CatalogueTokens().padding(Tokens.Space.l), size: Self.tokensSize, named: "tokens")
    }

    func testEveryStateOfTheWindow() async throws {
        try skipWithoutPictures()
        for state in CatalogueState.allCases {
            await verify(CatalogueWindow(state: state), size: Self.windowSize, named: "window-\(state.rawValue)")
        }
    }

    func testEveryStateOfTheHistoryList() async throws {
        try skipWithoutPictures()
        for state in CatalogueHistory.allCases {
            await verify(CatalogueHistoryList(state: state), size: Self.windowSize, named: "history-\(state.rawValue)")
        }
    }

    func testStartFailed() async throws {
        try skipWithoutPictures()
        await verify(StartFailedView(code: "store.open"), size: Self.windowSize, named: "start-failed")
    }

    #if os(iOS)
        /// The words of a failure at the largest text size, on a screen as narrow as any: they
        /// wrap and scroll, and none is cut short. (Found by Apple's audit on the iPhone Duo's
        /// simulator, where the message read "The site's address c…".)
        func testAFailedPageAtTheLargestTextSize() async throws {
            try skipWithoutPictures()
            let page = FailedPageView(code: "net.dns") {}.environment(\.dynamicTypeSize, .accessibility5)
            await verify(page, size: CGSize(width: 320, height: 568), named: "failed-largest-text")
        }
    #endif

    // MARK: The catalogue's lists, and the comparison itself

    /// The repository's Design/tokens.json, found from this file.
    private func tokenFile() throws -> [String: Any] {
        var root = URL(fileURLWithPath: #filePath)
        for _ in 0 ..< 4 {
            root.deleteLastPathComponent()
        }
        let data = try Data(contentsOf: root.appending(path: "Design/tokens.json"))
        return try XCTUnwrap(JSONSerialization.jsonObject(with: data) as? [String: Any])
    }

    func testEveryTokenIsInTheCatalogue() throws {
        let file = try tokenFile()
        let colors = try XCTUnwrap(file["color"] as? [String: Any])
        XCTAssertEqual(Set(Tokens.Catalogue.colors.map(\.name)), Set(colors.keys), "every colour, by its name")
        let groups: [(String, [(name: String, value: CGFloat)])] = [
            ("space", Tokens.Catalogue.space), ("radius", Tokens.Catalogue.radius), ("size", Tokens.Catalogue.size),
        ]
        for (group, listed) in groups {
            let written = try XCTUnwrap(file[group] as? [String: Double], group)
            XCTAssertEqual(Dictionary(uniqueKeysWithValues: listed.map { ($0.name, Double($0.value)) }), written, group)
        }
    }

    func testTheComparisonSeesAChange() throws {
        let size = CGSize(width: 200, height: 100)
        let swatch = { (color: Color) in
            Snapshot.Stage(color.frame(width: 60, height: 60), size: size, dark: false).picture()
        }
        let accent = try XCTUnwrap(swatch(Tokens.Palette.accent))
        let again = try XCTUnwrap(swatch(Tokens.Palette.accent))
        XCTAssertEqual(accent.difference(from: again).share, 0, "the same view is the same picture")

        // Another colour in the same place: 60 by 60 of 200 by 100 points, 18 % of the picture.
        let secure = try XCTUnwrap(swatch(Tokens.Palette.secure))
        let changed = accent.difference(from: secure).share
        XCTAssertEqual(changed, 0.18, accuracy: 0.01, "every pixel of the swatch differs, and no other")
        XCTAssertGreaterThan(changed, Picture.shareTolerance, "a recoloured swatch is a failure")

        // A different size is a different picture.
        let other = try XCTUnwrap(Snapshot.Stage(Tokens.Palette.accent, size: CGSize(width: 100, height: 100), dark: false).picture())
        XCTAssertEqual(accent.difference(from: other).share, 1)
    }
}

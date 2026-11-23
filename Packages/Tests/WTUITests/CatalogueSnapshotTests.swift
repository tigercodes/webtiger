// The design catalogue as pictures (004 §4): the token sections and each state of the window,
// in the light and the dark appearance, compared with the pictures checked in for this system.
import SwiftUI
import WTDesign
@testable import WTUI
import XCTest

@MainActor
final class CatalogueSnapshotTests: XCTestCase {
    /// Tall enough for every token: the colours a Space can have are nine of them (025).
    private static let tokensSize = CGSize(width: 480, height: 1180)
    private static let windowSize = CGSize(width: 480, height: CatalogueWindow.height)
    /// For the states of the window whose bar is taller than the window's usual picture, so
    /// that all of it is seen.
    private static let tallWindowSize = CGSize(width: 480, height: CatalogueWindow.height + 160)
    private static let tallStates: Set<CatalogueState> = [.promptSignInAgain]
    /// A failed page with the way to go on past a certificate under it, and with the warning
    /// (026): as tall as a page.
    private static let pageStates: Set<CatalogueState> = [.failedCertificate, .failedCertificateWarning]
    /// For the window with its tabs down the side (043): wide enough for a sidebar and a
    /// page beside it, and tall enough for every row.
    private static let wideWindowSize = CGSize(width: 720, height: 480)
    private static let wideStates: Set<CatalogueState> = [.sidebar]
    /// For a page over the window that is longer than any window's picture.
    private static let pageSize = CGSize(width: 480, height: 700)

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
            let tall = Self.tallStates.contains(state) ? Self.tallWindowSize : Self.windowSize
            let usual = Self.pageStates.contains(state) ? Self.pageSize : tall
            let size = Self.wideStates.contains(state) ? Self.wideWindowSize : usual
            await verify(CatalogueWindow(state: state), size: size, named: "window-\(state.rawValue)")
        }
    }

    func testEveryStateOfTheHistoryList() async throws {
        try skipWithoutPictures()
        for state in CatalogueHistory.allCases {
            await verify(CatalogueHistoryList(state: state), size: Self.windowSize, named: "history-\(state.rawValue)")
        }
    }

    func testEveryStateOfTheDownloadsList() async throws {
        try skipWithoutPictures()
        for state in CatalogueDownloads.allCases {
            await verify(CatalogueDownloadsList(state: state), size: Self.windowSize, named: "downloads-\(state.rawValue)")
        }
    }

    func testEveryStateOfThePermissionsList() async throws {
        try skipWithoutPictures()
        for state in CataloguePermissions.allCases {
            await verify(CataloguePermissionsList(state: state), size: Self.windowSize, named: "permissions-\(state.rawValue)")
        }
    }

    func testEveryStateOfTheSearchEngines() async throws {
        try skipWithoutPictures()
        for state in CatalogueSearch.allCases {
            // Taller than a window's picture, so that the whole list is seen.
            await verify(CatalogueSearchEngines(state: state), size: Self.tallWindowSize, named: "search-\(state.rawValue)")
        }
    }

    func testEveryStateOfTheSpaces() async throws {
        try skipWithoutPictures()
        for state in CatalogueSpaces.allCases {
            // Taller than a window's picture, so that the whole list, or page, is seen: the
            // page that makes a Space has its name, its logins, its colour and what was refused.
            await verify(CatalogueSpacesList(state: state), size: Self.pageSize, named: "spaces-\(state.rawValue)")
        }
    }

    func testEveryStateOfTheGroupPage() async throws {
        try skipWithoutPictures()
        for state in CatalogueGroups.allCases {
            // As tall as a page: on a phone the colours take two rows, and the buttons are
            // under what was refused.
            await verify(CatalogueGroupPage(state: state), size: Self.pageSize, named: "group-\(state.rawValue)")
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

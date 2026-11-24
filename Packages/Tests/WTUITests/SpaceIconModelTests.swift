// A Space's icon in the windows (050), against a scripted core: what the window says of the
// Space it shows, the icons the list says there are, what making a Space and editing one ask
// of the core; and, drawn, that the chip and the list have the icon in them. The shell has no
// list of icons of its own: every one it offers is one the core listed.
import SwiftUI
import WTContract
import WTCoreClient
import WTDesign
@testable import WTUI
import WTWebKit
import XCTest

@MainActor
final class SpaceIconModelTests: XCTestCase {
    private nonisolated static let house = "\u{1F3E0}"
    private nonisolated static let briefcase = "\u{1F4BC}"
    private nonisolated static let heart = "\u{2764}\u{FE0F}"

    /// What the scripted core lists, and what it refuses.
    private final class Told: @unchecked Sendable {
        private let lock = NSLock()
        private var icon = ""
        private var refusal: String?

        func refuse(with code: String?) {
            lock.withLock { refusal = code }
        }

        func answer(_ request: WTRequest.OneOf_Kind) -> WTResponse.OneOf_Kind? {
            lock.withLock {
                switch request {
                case .querySpaces:
                    var page = WTSpacesPage()
                    var home = WTSpaceSummary()
                    home.spaceID = 1
                    var work = WTSpaceSummary()
                    work.spaceID = 2
                    work.name = "Work"
                    work.icon = icon
                    page.spaces = [home, work]
                    page.icons = [SpaceIconModelTests.house, SpaceIconModelTests.briefcase, SpaceIconModelTests.heart]
                    return .spacesPage(page)
                case let .iconSpace(given):
                    if let refusal {
                        var refused = WTError()
                        refused.code = refusal
                        return .error(refused)
                    }
                    icon = given.icon
                    return .ack(WTAck())
                case .createSpace:
                    if let refusal {
                        var refused = WTError()
                        refused.code = refusal
                        return .error(refused)
                    }
                    var made = WTSpaceCreated()
                    made.spaceID = 3
                    return .spaceCreated(made)
                default:
                    return nil
                }
            }
        }
    }

    private let told = Told()

    private func core() -> ScriptedCore {
        let told = told
        return ScriptedCore { request in
            if let answer = told.answer(request) {
                return answer
            }
            switch request {
            case .openWindow:
                var opened = WTWindowOpened()
                opened.windowID = 1
                opened.tabID = 2
                return .windowOpened(opened)
            case .subscribe:
                var subscribed = WTSubscribed()
                subscribed.subscriptionID = 3
                return .subscribed(subscribed)
            default:
                return .ack(WTAck())
            }
        }
    }

    private func settle(until done: () -> Bool = { false }) async {
        for _ in 0 ..< 200 where !done() {
            await Task.yield()
            try? await Task.sleep(for: .milliseconds(2))
        }
    }

    /// A model whose window the core says shows the Space "Work", with this icon.
    private func showingWork(icon: String = "") async -> (BrowserModel, ScriptedCore) {
        let core = core()
        let model = BrowserModel(session: BrowserSession(core: core, engine: WebKitEngine()))
        await model.open()
        var window = WTWindowSnapshot()
        window.windowID = 1
        window.tab.tabID = 2
        window.spaceID = 2
        window.spaceName = "Work"
        window.spaceIcon = icon
        core.emit(window, subscription: 3)
        await settle { model.spaceID == 2 }
        return (model, core)
    }

    /// What was asked of the core about a Space, in order, in a few words each. (An icon is
    /// said by its code points: a picture in a message of a failed test helps nobody.)
    private func asked(_ core: ScriptedCore) -> [String] {
        func points(_ icon: String) -> String {
            icon.unicodeScalars.map { String($0.value, radix: 16) }.joined(separator: "+")
        }
        return core.requests.compactMap {
            switch $0 {
            case let .createSpace(ask): "make '\(ask.name)' with \(points(ask.icon))"
            case let .renameSpace(ask): "name \(ask.spaceID) '\(ask.name)'"
            case let .colourSpace(ask): "colour \(ask.spaceID) '\(ask.colour)'"
            case let .iconSpace(ask): "icon \(ask.spaceID) \(points(ask.icon))"
            case let .setSpaceTidying(ask): "tidy \(ask.spaceID) after \(ask.afterSeconds)"
            default: nil
            }
        }
    }

    // MARK: What is asked

    func testTheWindowSaysItsSpacesIconAndTheListSaysTheIconsThereAre() async {
        let (model, core) = await showingWork(icon: Self.briefcase)
        XCTAssertEqual(model.spaceIcon, Self.briefcase)
        XCTAssertEqual(model.spaceIcons, [], "nothing until the core has listed the Spaces")

        model.openSpaces()
        await settle { model.spaces != nil }
        XCTAssertEqual(model.spaceIcons, [Self.house, Self.briefcase, Self.heart], "the core's, in its order")

        // Another Space, with none: the window says none.
        var window = WTWindowSnapshot()
        window.windowID = 1
        window.tab.tabID = 2
        window.spaceID = 1
        core.emit(window, subscription: 3)
        await settle { model.spaceID == 1 }
        XCTAssertEqual(model.spaceIcon, "")
    }

    func testASpaceIsMadeWithTheIconChosenOrWithNone() async {
        let (model, core) = await showingWork()
        _ = await model.makeSpace(named: "Travel", sharingLoginsWith: nil, colour: "", icon: Self.heart)
        _ = await model.makeSpace(named: "Plain", sharingLoginsWith: nil)
        XCTAssertEqual(asked(core), ["make 'Travel' with 2764+fe0f", "make 'Plain' with "])
    }

    func testEditingASpaceAsksForItsIconAfterItsColourAndBeforeItsTime() async {
        let (model, core) = await showingWork()

        let given = await model.editSpace(SpaceEdit(icon: Self.house))
        let none = await model.editSpace(SpaceEdit(icon: ""))
        let all = await model.editSpace(SpaceEdit(name: "Desk", colour: "red", icon: Self.briefcase, tidyAfter: 86400))
        let nothing = await model.editSpace(SpaceEdit())

        XCTAssertTrue(given && none && all && nothing)
        XCTAssertEqual(
            asked(core),
            ["icon 2 1f3e0", "icon 2 ", "name 2 'Desk'", "colour 2 'red'", "icon 2 1f4bc", "tidy 2 after 86400"]
        )
        // The model gives nothing by itself: the icon it shows is the core's word.
        XCTAssertEqual(model.spaceIcon, "")
    }

    func testAnIconTheCoreWillNotTakeSaysWhyInWordsAndLeavesTheTimeUnasked() async {
        let (model, core) = await showingWork()
        told.refuse(with: "space.icon_unknown")

        let refused = await model.editSpace(SpaceEdit(icon: "\u{1F600}", tidyAfter: 43200))

        XCTAssertFalse(refused)
        XCTAssertEqual(model.spaceRefusal, "space.icon_unknown")
        XCTAssertEqual(asked(core), ["icon 2 1f600"], "the time was not asked for")
        let words = SpacesView.words(for: "space.icon_unknown", unknown: Words.errorSpaceRenameFailed)
        XCTAssertEqual(words, "That icon isn’t one a Space can have.")
        // Taken, it says nothing of before.
        told.refuse(with: nil)
        let taken = await model.editSpace(SpaceEdit(icon: Self.house))
        XCTAssertTrue(taken)
        XCTAssertNil(model.spaceRefusal)
    }

    func testBeforeTheCoreHasSaidWhichSpaceNoIconIsAsked() async {
        let core = core()
        let model = BrowserModel(session: BrowserSession(core: core, engine: WebKitEngine()))
        await model.open()
        let asked = await model.iconSpace(Self.house)
        XCTAssertFalse(asked)
        XCTAssertEqual(self.asked(core), [], "no Space is shown yet: there is none to give an icon")
    }

    #if WEBTIGER_CORE
        /// The real core lists icons, each of which it takes for a Space and says back as it
        /// was given; and one it did not list is refused, with nothing changed.
        func testTheRealCoreTakesEveryIconItListsAndNoOther() async throws {
            let session = try BrowserSession.live(environment: ["WEBTIGER_FAKE": "1"])
            defer { session.core.stop() }
            func page() async throws -> WTSpacesPage {
                guard case let .spacesPage(page) = try await session.core.send(.querySpaces(WTQuerySpaces())).value else {
                    throw XCTSkip("the core lists the Spaces")
                }
                return page
            }
            let listed = try await page()
            XCTAssertEqual(listed.icons.count, 24)
            XCTAssertEqual(Set(listed.icons).count, listed.icons.count, "each once")
            let space = try XCTUnwrap(listed.spaces.first)
            XCTAssertEqual(space.icon, "", "a Space is made with none")
            for icon in listed.icons {
                XCTAssertFalse(icon.isEmpty)
                XCTAssertEqual(icon.count, 1, "one picture: \(icon.unicodeScalars.map(\.value))")
                var give = WTIconSpace()
                give.spaceID = space.spaceID
                give.icon = icon
                guard case .ack = try await session.core.send(.iconSpace(give)).value else {
                    return XCTFail("the core takes \(icon.unicodeScalars.map(\.value))")
                }
                let now = try await page()
                XCTAssertEqual(now.spaces.first?.icon, icon)
            }
            // One it did not list: refused by its code, and the Space has the last it was given.
            var other = WTIconSpace()
            other.spaceID = space.spaceID
            other.icon = "\u{1F600}"
            do {
                let answer = try await session.core.send(.iconSpace(other)).value
                guard case let .error(refused) = answer else { return XCTFail("refused") }
                XCTAssertEqual(refused.code, "space.icon_unknown")
            } catch let CoreError.refused(code, _) {
                XCTAssertEqual(code, "space.icon_unknown")
            }
            let after = try await page()
            XCTAssertEqual(after.spaces.first?.icon, listed.icons.last)
        }
    #endif

    func testThePagesOfferNoneAndThenTheCoresIconsAndNothingWhereItListsNone() {
        XCTAssertEqual(
            SpaceIcons.choices(among: [Self.house, Self.heart]),
            ["", Self.house, Self.heart],
            "none first, then the core's in its order"
        )
        XCTAssertEqual(SpaceIcons.choices(among: []), [], "a core that lists no icons has none to choose among: not even none")
    }

    // MARK: Drawn

    /// How many pixels of a picture have a colour in them: are far from any grey. Words and
    /// the chip's own fill are greys; an emoji is not.
    private func coloured(_ picture: Picture?) -> Int {
        guard let picture else { return -1 }
        var found = 0
        for index in stride(from: 0, to: picture.pixels.count, by: 4) {
            let (red, green, blue) = (Int(picture.pixels[index]), Int(picture.pixels[index + 1]), Int(picture.pixels[index + 2]))
            if max(red, green, blue) - min(red, green, blue) > 90 {
                found += 1
            }
        }
        return found
    }

    private func drawn(_ view: some View, size: CGSize = CGSize(width: 200, height: 60)) async -> Picture? {
        await Snapshot.Stage(view, size: size, dark: false).settledPicture()
    }

    /// The pictures' own check lets a change as small as an icon by, so that the icon is
    /// drawn is counted here: by its colour, a red heart's.
    func testTheChipDrawsTheIconInThePlaceOfTheInitialAndBeforeTheName() async {
        for width in [UserInterfaceSizeClass.compact, .regular] {
            let plain = await drawn(SpaceChip(name: "Work") {}.environment(\.horizontalSizeClass, width))
            let withIcon = await drawn(SpaceChip(name: "Work", icon: Self.heart) {}.environment(\.horizontalSizeClass, width))
            XCTAssertGreaterThanOrEqual(coloured(plain), 0, "the chip was drawn")
            XCTAssertLessThan(coloured(plain), 10, "a chip with no icon is words on its fill: \(width)")
            XCTAssertGreaterThan(coloured(withIcon), 100, "the icon is drawn: \(width)")
        }
    }

    func testTheListDrawsASpacesIconBesideItsNameAndThePagesOfferTheIcons() async {
        func space(_ id: UInt64, _ name: String, icon: String) -> WTSpaceSummary {
            var space = WTSpaceSummary()
            space.spaceID = id
            space.name = name
            space.logins = UInt32(id)
            space.icon = icon
            return space
        }
        func list(icon: String, icons: [String] = [], renaming: Bool = false) -> some View {
            SpacesView(
                spaces: [space(1, "", icon: ""), space(2, "Work", icon: icon)], shownID: 2, icons: icons, renaming: renaming,
                show: { _ in }, make: { _, _, _, _ in }, edit: { _ in }, done: {}
            )
        }
        let size = CGSize(width: 480, height: 700)
        let plain = await drawn(list(icon: ""), size: size)
        let withIcon = await drawn(list(icon: Self.heart), size: size)
        XCTAssertLessThan(coloured(plain), 10, "a list of Spaces with no icons and no colours")
        XCTAssertGreaterThan(coloured(withIcon), 100, "the Space's icon is in its row")

        // The page that edits a Space: no icons to choose among where the core lists none;
        // and the core's, each drawn, where it lists them.
        let none = await drawn(list(icon: "", renaming: true), size: size)
        let offered = await drawn(list(icon: "", icons: [Self.heart, Self.house, Self.briefcase], renaming: true), size: size)
        XCTAssertLessThan(coloured(none), 10)
        XCTAssertGreaterThan(coloured(offered), 300, "three icons to choose among")
    }
}

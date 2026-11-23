public import WTContract

// Moving a tab in its row (045), the model's part: what the window asks of the core to move a
// tab one place, when the command has something to do, and the two commands with their words,
// which go by how the row is drawn. What a row's parts are, and how far a tab can go among
// those of its own, are the core's.
import SwiftUI

public extension BrowserModel {
    /// Whether a step that way would move the tab: the core's word.
    func canMove(_ tab: WTTabSummary, _ step: WTRowStep) -> Bool {
        switch step {
        case .back: tab.stepsBack
        case .forward: tab.stepsForward
        default: false
        }
    }

    /// The tab in front as the row has it: what the menu bar's commands are about.
    var frontTabInTheRow: WTTabSummary? {
        guard let frontTabID else { return nil }
        return tabs.first { $0.tabID == frontTabID }
    }

    /// Moves a tab one place among the tabs of its part of the row.
    func moveTab(_ id: UInt64, _ step: WTRowStep) {
        var ask = WTMoveTab()
        ask.tabID = id
        ask.by = step
        send(.moveTab(ask))
    }

    /// The same for the tab in front, when the step would move it.
    func moveTheTabInFront(_ step: WTRowStep) {
        guard let tab = frontTabInTheRow, canMove(tab, step) else { return }
        moveTab(tab.tabID, step)
    }
}

extension BrowserModel {
    /// The two commands for the tab in front, as it is drawn: what the menu bar lists. In a
    /// column when the window draws its tabs down the side; but a Favourite's tile is in a
    /// row wherever the tabs are.
    func movesOfTheTabInFront(rightToLeft: Bool) -> [TabMove] {
        TabMove.both(atTheSide: showsSidebar && frontTabInTheRow?.favourite != true, rightToLeft: rightToLeft)
    }
}

/// One of the two commands that move a tab a place (045): the step it asks of the core, and
/// its words.
struct TabMove: Identifiable {
    let step: WTRowStep
    let words: Word

    var id: Int {
        step.rawValue
    }

    /// The two, in the order a menu lists them, for a tab as it is drawn. Down the side, up
    /// is towards the row's start. Along the top (and for a Favourite's tile, which is in a
    /// row wherever the tabs are) the row runs the way words do: where they run from right
    /// to left, left is towards its end.
    static func both(atTheSide: Bool, rightToLeft: Bool) -> [TabMove] {
        if atTheSide {
            return [TabMove(step: .back, words: Words.tabsMoveUp), TabMove(step: .forward, words: Words.tabsMoveDown)]
        }
        return [
            TabMove(step: rightToLeft ? .forward : .back, words: Words.tabsMoveLeft),
            TabMove(step: rightToLeft ? .back : .forward, words: Words.tabsMoveRight),
        ]
    }
}

/// The commands that move a tab a place (045), for a menu: each grey when the core says the
/// step would move nothing; neither for a tab that can go no way.
struct MoveTabItems: View {
    let tab: WTTabSummary
    let model: BrowserModel
    /// The tab is drawn in a column: a row of the sidebar, and not a Favourite's tile.
    var atTheSide = false
    @Environment(\.layoutDirection) private var direction

    var body: some View {
        if tab.stepsBack || tab.stepsForward {
            ForEach(TabMove.both(atTheSide: atTheSide, rightToLeft: direction == .rightToLeft)) { move in
                Button {
                    model.moveTab(tab.tabID, move.step)
                } label: {
                    Text(move.words)
                }
                .disabled(!model.canMove(tab, move.step))
            }
        }
    }
}

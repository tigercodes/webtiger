// Dragging in the row (045 §5b), the views' part: a tab or a Group is picked up, and a place
// takes it. What a drag carries out of the window is nothing: the thing's name stays in the
// app, and the system is handed a kind only this app knows, for this app alone.
//
// On a Mac, with a pointer. On a touch screen nothing is dragged yet: there these do nothing
// (045 §9).
import SwiftUI
import UniformTypeIdentifiers
import WTDesign

/// What is being dragged in the app now.
@MainActor
enum RowDragging {
    /// The thing, and the window whose row it is of: it is dropped in that row or nowhere.
    struct Held {
        let window: UInt64
        let item: RowItem
    }

    static var held: Held?

    /// The kind of thing a drag is, to the system: one only this app knows.
    static let kind = UTType(exportedAs: "com.tigercodes.webtiger.row-item")

    /// What the system is handed for a drag: the kind, and no bytes, offered to this app alone.
    static func carried() -> NSItemProvider {
        let carried = NSItemProvider()
        carried.registerDataRepresentation(forTypeIdentifier: kind.identifier, visibility: .ownProcess) { done in
            done(Data(), nil)
            return nil
        }
        return carried
    }
}

// Where these go, and why they hide what they are (found by test, 045 §8; not understood):
// they are put on the whole of a row (a tab's button and the button at its end together),
// never on the tab's button itself, and what they return is a view of no known kind. Put on
// the button, or with both left the kind of view they are, a row of the sidebar, whose rows
// are made as they come into view, went on telling a screen reader what it was when it was
// made: its old name after its page changed, that it was no Site when it had become one, that
// it was in front when it no longer was. (One of the two hidden is enough, as far as was
// tried; both are.) `DraggingTests` holds this in place.
extension View {
    /// The view is a thing of the row that can be dragged.
    @ViewBuilder
    func dragged(as item: RowItem, in model: BrowserModel) -> some View {
        #if os(macOS)
            AnyView(onDrag {
                RowDragging.held = model.windowID.map { RowDragging.Held(window: $0, item: item) }
                return RowDragging.carried()
            })
        #else
            self
        #endif
    }

    /// The view is a place of the row that takes what is dropped on it. `mark` says how the
    /// place is marked while something it would take is held over it.
    @ViewBuilder
    func takesDrops(at place: RowPlace, in model: BrowserModel, mark: RowDropMark = .round) -> some View {
        #if os(macOS)
            AnyView(modifier(TakesDrops(place: place, model: model, mark: mark)))
        #else
            self
        #endif
    }
}

/// How a place is marked while a thing is held over it.
enum RowDropMark {
    /// A line before it, where the thing would go: along the top, at its leading edge.
    case before
    /// A line above it: down the side.
    case above
    /// Drawn round, as chosen: a Group's chip, and the end of a part.
    case round
}

#if os(macOS)
    private struct TakesDrops: ViewModifier {
        let place: RowPlace
        let model: BrowserModel
        let mark: RowDropMark
        @State private var marked = false

        func body(content: Content) -> some View {
            content
                .overlay(alignment: mark == .above ? .top : .leading) {
                    if marked {
                        switch mark {
                        case .before: Tokens.Palette.accent.frame(width: 2).accessibilityHidden(true)
                        case .above: Tokens.Palette.accent.frame(height: 2).accessibilityHidden(true)
                        case .round:
                            RoundedRectangle(cornerRadius: Tokens.Radius.field)
                                .strokeBorder(Tokens.Palette.accent, lineWidth: 2)
                                .accessibilityHidden(true)
                        }
                    }
                }
                .onDrop(of: [RowDragging.kind], delegate: Taker(place: place, model: model, marked: $marked))
        }
    }

    /// What a place does with a drag that comes over it.
    private struct Taker: DropDelegate {
        let place: RowPlace
        let model: BrowserModel
        @Binding var marked: Bool

        /// The thing held, when it is of this window's row and the place takes it.
        private var taken: RowItem? {
            guard let held = RowDragging.held, held.window == model.windowID, model.takes(held.item, at: place) else { return nil }
            return held.item
        }

        func validateDrop(info _: DropInfo) -> Bool {
            taken != nil
        }

        func dropEntered(info _: DropInfo) {
            marked = taken != nil
        }

        func dropExited(info _: DropInfo) {
            marked = false
        }

        func dropUpdated(info _: DropInfo) -> DropProposal? {
            DropProposal(operation: taken == nil ? .forbidden : .move)
        }

        func performDrop(info _: DropInfo) -> Bool {
            marked = false
            guard let taken else { return false }
            RowDragging.held = nil
            model.drop(taken, at: place)
            return true
        }
    }
#endif

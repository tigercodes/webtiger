// The phone's switcher (047): in a window with no room for a sidebar, a button in the toolbar
// that says how many tabs the row has, and a sheet as tall as the window with the row as the
// sidebar has it. The switcher is the sidebar shown another way: it has no model of its own,
// and every row asks what it asks in the sidebar. A tab brought in front puts it away.
import SwiftUI
import WTContract
import WTDesign

/// The button that opens the switcher: a square with the number of tabs the row has. The
/// number is the core's: those of shut Groups are among them.
struct SwitcherButton: View {
    let count: Int
    let action: () -> Void
    @Environment(\.locale) private var locale

    /// What the square shows: the number, while it has room for it.
    static func shown(_ count: Int) -> String {
        count > 99 ? "99+" : "\(count)"
    }

    var body: some View {
        Button(action: action) {
            Text(verbatim: Self.shown(count))
                .font(.caption.weight(.semibold))
                .monospacedDigit()
                .lineLimit(1)
                .minimumScaleFactor(0.7)
                .frame(width: 22, height: 22)
                .overlay {
                    RoundedRectangle(cornerRadius: 5).strokeBorder(.primary, lineWidth: 1.5)
                }
                .frame(minWidth: BrowserWindowView.hitSize, minHeight: BrowserWindowView.hitSize)
                .contentShape(Rectangle())
        }
        .buttonStyle(.borderless)
        .accessibilityLabel(Text(verbatim: Words.switcherOpen.filled(count, locale: locale)))
        .accessibilityIdentifier("switcher")
    }
}

/// The switcher itself: the Space at its head, its title and the way out; then the row as
/// the sidebar has it, with no toolbar and no buttons of the window's.
struct Switcher: View {
    let model: BrowserModel
    /// The sheet is put away: a tab was brought in front, or the person is done.
    let onDone: () -> Void
    /// The Space's chip was pressed: the switcher makes way for the list of Spaces.
    let onSpaces: () -> Void

    var body: some View {
        // (No name is given to the whole of it: a name on a view is given to everything in
        // it that has none of its own to keep, and its rows are found by theirs.)
        Sidebar(model: model, onSpaces: onSpaces, showsFoot: false, onChosen: onDone) {
            head
        }
    }

    private var head: some View {
        HStack(spacing: Tokens.Space.s) {
            if model.isPrivate {
                PrivateChip(action: onSpaces)
            } else {
                SpaceChip(name: model.spaceName, colour: model.spaceColour, action: onSpaces)
            }
            Spacer(minLength: Tokens.Space.s)
            Text(Words.switcherTitle)
                .font(.headline)
                .accessibilityAddTraits(.isHeader)
            Spacer(minLength: Tokens.Space.s)
            Button(action: onDone) {
                Text(Words.switcherDone)
                    .frame(minWidth: BrowserWindowView.hitSize, minHeight: BrowserWindowView.hitSize)
                    .contentShape(Rectangle())
            }
            .accessibilityIdentifier("switcher-done")
        }
        .padding(.horizontal, Tokens.Space.m)
        .padding(.vertical, Tokens.Space.xs)
    }
}

/// The switcher over a window, while it is shown. It is there where the sidebar is not: a
/// window that comes to have room for a sidebar puts it away. And it makes way for the pages
/// that are the window's own, which cannot be shown over it: the page of a Group.
struct SwitcherSheet: ViewModifier {
    let model: BrowserModel
    @Binding var isShown: Bool
    let onSpaces: () -> Void

    func body(content: Content) -> some View {
        content
            .sheet(isPresented: $isShown) {
                Switcher(model: model, onDone: { isShown = false }, onSpaces: {
                    isShown = false
                    onSpaces()
                })
                .presentationDetents([.large])
            }
            .onChange(of: model.hasRoomForSidebar) { _, hasRoom in
                if hasRoom {
                    isShown = false
                }
            }
            .onChange(of: model.groupAsked != nil) { _, asked in
                if asked {
                    isShown = false
                }
            }
    }
}

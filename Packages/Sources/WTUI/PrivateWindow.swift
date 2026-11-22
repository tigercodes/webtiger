// A private window (037): what it shows of itself. Where an ordinary window names its Space,
// a private one says "Private"; the chip opens a page that says what private means and what it
// does not, with the ways out of it. Nothing is decided here: the core keeps nothing of a
// private window, and this says so.
public import SwiftUI
import WTDesign

public extension BrowserWindowView {
    /// The name of the scene whose windows are private ones, on a Mac: what New Private
    /// Window opens.
    static let privateWindows = "private"
}

/// The chip of a private window, where an ordinary window's names its Space: the word, on a
/// narrow strip too. One letter would be any Space's, and this is where the window says what
/// it is.
struct PrivateChip: View {
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: Tokens.Space.xs) {
                Text("private.chip", bundle: .module)
                    .fontWeight(.semibold)
                    .lineLimit(1)
                Image(systemName: "chevron.down")
                    .font(.caption.weight(.semibold))
                    .accessibilityHidden(true)
            }
            .padding(.horizontal, Tokens.Space.m)
            .frame(minHeight: BrowserWindowView.hitSize)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .background(Tokens.Palette.fieldFill, in: RoundedRectangle(cornerRadius: Tokens.Radius.field))
        .accessibilityLabel(Text("private.window", bundle: .module))
        .accessibilityIdentifier("private-chip")
    }
}

/// The two sentences that say what a private window is, and what it is not.
struct PrivateWords: View {
    var body: some View {
        VStack(spacing: Tokens.Space.s) {
            Text("private.keeps", bundle: .module)
                .accessibilityIdentifier("private-keeps")
            Text("private.hides", bundle: .module)
                .foregroundStyle(.secondary)
                .accessibilityIdentifier("private-hides")
        }
    }
}

/// What a private window can be asked from the page behind its chip.
struct PrivateWays {
    /// Closes the private window, with everything of it.
    let close: () -> Void
    /// Shows the other window again and leaves the private one open, where the app shows one
    /// window at a time; with the name of the Space the other window shows. None on a Mac,
    /// where both windows are on the screen.
    var leave: (other: String, go: () -> Void)?
}

/// The page behind a private window's chip, over the window.
struct PrivateSheet: ViewModifier {
    @Binding var isShown: Bool
    let ways: PrivateWays
    /// What the page was asked for, done once the page has gone: a Mac's window does not
    /// close while a sheet is over it, and a phone's is not taken away from under one.
    @State private var asked: (() -> Void)?

    func body(content: Content) -> some View {
        #if os(macOS)
            // A Mac's window lets go of its sheet a moment after SwiftUI says the sheet was
            // dismissed, and until it has, the window does not close (the app's test found
            // it: Close Private Window closed nothing). So what was asked waits for the
            // window's own word that its sheet has ended.
            content
                .sheet(isPresented: $isShown) { page }
                .onReceive(NotificationCenter.default.publisher(for: NSWindow.didEndSheetNotification)) { _ in
                    doWhatWasAsked()
                }
        #else
            content.sheet(isPresented: $isShown, onDismiss: doWhatWasAsked) { page }
        #endif
    }

    private func doWhatWasAsked() {
        let now = asked
        asked = nil
        now?()
    }

    private var page: some View {
        PrivatePage(ways: ways) { then in
            asked = then
            isShown = false
        }
    }
}

/// What the page behind the chip says, and the ways out of it.
private struct PrivatePage: View {
    let ways: PrivateWays
    /// Takes the page away, and then does what was asked, if anything was.
    let leave: ((() -> Void)?) -> Void
    @Environment(\.locale) private var locale

    var body: some View {
        VStack(alignment: .leading, spacing: Tokens.Space.l) {
            Text("private.window", bundle: .module)
                .font(.title2.weight(.semibold))
                .accessibilityAddTraits(.isHeader)
            PrivateWords()
            VStack(alignment: .leading, spacing: Tokens.Space.s) {
                Button { leave(ways.close) } label: {
                    Text("private.close", bundle: .module).frame(minHeight: BrowserWindowView.hitSize)
                }
                .accessibilityIdentifier("private-close")
                if let other = ways.leave {
                    Button { leave(other.go) } label: {
                        Text(verbatim: String(
                            format: String(localized: "private.back", bundle: .module, locale: locale),
                            SpaceName.shown(other.other, locale: locale)
                        ))
                        .frame(minHeight: BrowserWindowView.hitSize)
                    }
                    .accessibilityIdentifier("private-leave")
                }
                Button { leave(nil) } label: {
                    Text("spaces.done", bundle: .module).frame(minHeight: BrowserWindowView.hitSize)
                }
                .keyboardShortcut(.cancelAction)
                .accessibilityIdentifier("private-done")
            }
        }
        .padding(Tokens.Space.xl)
        .frame(maxWidth: 480, alignment: .leading)
        // A container of its own for assistive services: named without its name being
        // given to everything in it.
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier("private-page")
    }
}

/// The way to a private window where the app shows one window at a time (037 §5.3): what an
/// ordinary window is given.
struct PrivateDoor {
    /// A private window is open, shown or behind.
    var isOpen = false
    /// Opens a private window, or shows the one there is.
    var show: () -> Void = {}
}

/// The last row of the list of Spaces: the private window, which is no Space. It opens one, or
/// shows the one that is open, and says so in words.
struct PrivateRow: View {
    let door: PrivateDoor

    var body: some View {
        Button(action: door.show) {
            HStack(spacing: Tokens.Space.m) {
                // No mark: it is no Space. The room of one, so that its name is under theirs.
                Color.clear.frame(width: 10, height: 10)
                VStack(alignment: .leading, spacing: 0) {
                    Text("private.window", bundle: .module)
                        .fontWeight(.semibold)
                    Text("private.row.detail", bundle: .module)
                        .font(.callout)
                        .fixedSize(horizontal: false, vertical: true)
                }
                Spacer(minLength: Tokens.Space.m)
                if door.isOpen {
                    Text("private.row.open", bundle: .module)
                        .font(.callout)
                        .accessibilityIdentifier("space-private-open")
                }
            }
            .frame(maxWidth: .infinity, minHeight: BrowserWindowView.hitSize, alignment: .leading)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .padding(.horizontal, Tokens.Space.l)
        .padding(.vertical, Tokens.Space.xs)
        .accessibilityIdentifier("space-private")
    }
}

/// The app's window where it shows one at a time (a phone, 037 §5.3): the window the app
/// opened with, or a private window in its place while one is shown. Only the window that is
/// shown is drawn; the other is kept by its model, with its pages as they were. A private
/// window that is closed is closed in the core, and is gone with everything of it.
public struct BrowserWindowsView: View {
    private let session: BrowserSession
    @State private var first: BrowserModel
    /// The private window, while there is one, shown or behind.
    @State private var hidden: BrowserModel?
    @State private var showsPrivate = false

    public init(session: BrowserSession) {
        self.session = session
        _first = State(initialValue: BrowserModel(session: session))
    }

    public var body: some View {
        if showsPrivate, let hidden {
            BrowserWindowView(
                model: hidden, session: session, door: nil,
                ways: PrivateWays(
                    close: {
                        showsPrivate = false
                        hidden.close()
                        self.hidden = nil
                    },
                    leave: (other: first.spaceName, go: { showsPrivate = false })
                )
            )
            // A window of its own: nothing of the other's view is kept for it.
            .id(ObjectIdentifier(hidden))
        } else {
            BrowserWindowView(
                model: first, session: session,
                door: PrivateDoor(isOpen: hidden != nil, show: {
                    if hidden == nil {
                        let made = BrowserModel(session: session)
                        made.opensPrivate = true
                        hidden = made
                    }
                    showsPrivate = true
                }),
                ways: nil
            )
            .id(ObjectIdentifier(first))
        }
    }
}

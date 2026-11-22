// The window's tabs in a row (005 §5), with the buttons that stand beside them.
import SwiftUI
import WTContract
import WTDesign

/// The window's tabs in a row: each by its name, the one in front marked, a button to close each
/// and one to open another (005 §5).
struct TabStrip: View {
    let model: BrowserModel
    /// Opens the find bar (010), the zoom bar (012) and the History list (008): their buttons stand at the end of
    /// the row, beside the tabs and not among them, where they stay when the tabs scroll.
    var onFind: () -> Void = {}
    var onZoom: () -> Void = {}
    var onHistory: () -> Void = {}
    /// Opens the list of Spaces (024): the chip that names the one shown stands before the
    /// tabs, and stays there when they scroll.
    var onSpaces: () -> Void = {}
    /// How wide a tab's name may be. It grows with the text's size, so that at the largest
    /// sizes a short name is still whole; a long title is cut to one line at any size.
    @ScaledMetric private var widestName: CGFloat = 180

    var body: some View {
        HStack(spacing: 0) {
            // A private window has no Space (037): its chip says what it is.
            Group {
                if model.isPrivate {
                    PrivateChip(action: onSpaces)
                } else {
                    SpaceChip(name: model.spaceName, colour: model.spaceColour, action: onSpaces)
                }
            }
            .padding(.leading, Tokens.Space.m)
            tabs
            Button(action: onFind) {
                Label { Text(Words.findOpen) } icon: { Image(systemName: "magnifyingglass") }
                    .labelStyle(.iconOnly)
                    .frame(minWidth: BrowserWindowView.hitSize, minHeight: BrowserWindowView.hitSize)
                    .contentShape(Rectangle())
            }
            .buttonStyle(.borderless)
            // There is nothing to find in a tab with no page.
            .disabled(!model.hasPage)
            .accessibilityIdentifier("find")
            Button(action: onZoom) {
                Label { Text(Words.zoomOpen) } icon: { Image(systemName: "textformat.size") }
                    .labelStyle(.iconOnly)
                    .frame(minWidth: BrowserWindowView.hitSize, minHeight: BrowserWindowView.hitSize)
                    .contentShape(Rectangle())
            }
            .buttonStyle(.borderless)
            // There is nothing to zoom without a page that can be.
            .disabled(model.zoom == nil)
            .accessibilityIdentifier("zoom")
            Button(action: onHistory) {
                Label { Text(Words.historyOpen) } icon: { Image(systemName: "clock.arrow.circlepath") }
                    .labelStyle(.iconOnly)
                    .frame(minWidth: BrowserWindowView.hitSize, minHeight: BrowserWindowView.hitSize)
                    .contentShape(Rectangle())
                    // A download is coming (013): a dot says so, where the list of them opens.
                    .overlay(alignment: .topTrailing) {
                        if model.downloadsActive > 0 {
                            Circle()
                                .fill(Tokens.Palette.accent)
                                .frame(width: Tokens.Space.s, height: Tokens.Space.s)
                                .padding(Tokens.Space.xs)
                                .accessibilityHidden(true)
                        }
                    }
            }
            .buttonStyle(.borderless)
            .accessibilityValue(model.downloadsActive > 0 ? Text(Words.downloadsActive) : Text(verbatim: ""))
            .accessibilityIdentifier("history")
            .padding(.trailing, Tokens.Space.m)
        }
    }

    private var tabs: some View {
        ScrollViewReader { row in
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: Tokens.Space.xs) {
                    ForEach(model.tabs, id: \.tabID) { tab in
                        chip(tab).id(tab.tabID)
                    }
                    Button {
                        model.openTab()
                    } label: {
                        Label { Text(Words.tabsNew) } icon: { Image(systemName: "plus") }
                            .labelStyle(.iconOnly)
                            .frame(minWidth: BrowserWindowView.hitSize, minHeight: BrowserWindowView.hitSize)
                            .contentShape(Rectangle())
                    }
                    .buttonStyle(.borderless)
                    #if !os(macOS)
                        // Held, it offers the tab that was last closed back (033).
                        .modifier(OffersClosedTab(model: model))
                    #endif
                        // A screen reader is told of it by its name, while it is there.
                        .accessibilityActions {
                            if model.canReopenClosedTab {
                                ReopenClosedTab(model: model)
                            }
                        }
                        .accessibilityIdentifier("new-tab")
                }
                .padding(.horizontal, Tokens.Space.m)
            }
            // The tab that comes in front is brought into view.
            .onChange(of: model.tab.tabID) { _, front in
                row.scrollTo(front)
            }
        }
        .frame(minHeight: BrowserWindowView.hitSize + Tokens.Space.s)
        #if os(macOS)
            // A right-click in the strip, beside the tabs, offers the tab that was last closed
            // back (033). Not on a button: on the Mac a button takes a right-click for itself,
            // and no menu comes of it, its own or the strip's.
            .contentShape(Rectangle())
            .modifier(OffersClosedTab(model: model))
        #endif
            .accessibilityElement(children: .contain)
            .accessibilityLabel(Text(Words.tabsList))
            .accessibilityIdentifier("tab-strip")
    }

    private func chip(_ tab: WTTabSummary) -> some View {
        let isFront = tab.tabID == model.tab.tabID
        return HStack(spacing: 0) {
            Button {
                model.activate(tab.tabID)
            } label: {
                HStack(spacing: Tokens.Space.xs) {
                    if tab.loading {
                        // A dot, not a spinner: a tab that is loading is marked, and stays still.
                        Circle()
                            .fill(Tokens.Palette.accent)
                            .frame(width: 6, height: 6)
                            .accessibilityHidden(true)
                    }
                    name(tab)
                        .lineLimit(1)
                        .fontWeight(isFront ? .semibold : .regular)
                }
                .padding(.leading, Tokens.Space.s)
                .frame(maxWidth: widestName, minHeight: BrowserWindowView.hitSize, alignment: .leading)
                .fixedSize(horizontal: true, vertical: false)
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)
            .accessibilityIdentifier("tab")
            .accessibilityAddTraits(isFront ? .isSelected : [])
            Button {
                model.closeTab(tab.tabID)
            } label: {
                Label { Text(Words.tabsClose) } icon: { Image(systemName: "xmark") }
                    .labelStyle(.iconOnly)
                    .font(.caption)
                    .frame(minWidth: BrowserWindowView.hitSize, minHeight: BrowserWindowView.hitSize)
                    .contentShape(Rectangle())
            }
            .buttonStyle(.borderless)
            .accessibilityIdentifier("close-tab")
        }
        .background(isFront ? Tokens.Palette.fieldFill : .clear, in: RoundedRectangle(cornerRadius: Tokens.Radius.field))
    }

    /// A tab's name: its page's title, or its site, or the words for a tab with no page.
    private func name(_ tab: WTTabSummary) -> Text {
        if !tab.title.isEmpty {
            Text(verbatim: tab.title)
        } else if !tab.host.isEmpty {
            Text(verbatim: tab.host)
        } else {
            Text(Words.tabsUntitled)
        }
    }
}

/// A menu with "Reopen Closed Tab" (033): on a phone, of the button for a new tab, held; on the
/// Mac, of the strip of tabs, right-clicked.
///
/// With no closed tab to bring back there is no menu at all: an item that does nothing is not
/// shown. The menu is put on and taken off as a whole, and not left there with nothing in it.
private struct OffersClosedTab: ViewModifier {
    let model: BrowserModel

    func body(content: Content) -> some View {
        if model.canReopenClosedTab {
            content.contextMenu { ReopenClosedTab(model: model) }
        } else {
            content
        }
    }
}

/// "Reopen Closed Tab": the core is asked to undo.
private struct ReopenClosedTab: View {
    let model: BrowserModel

    var body: some View {
        Button { model.reopenClosedTab() } label: { Text(Words.tabsReopen) }
            .accessibilityIdentifier("reopen-closed-tab")
    }
}

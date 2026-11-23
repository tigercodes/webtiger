// The parts of a tab as a window draws it, whichever way its tabs are laid out (043): what
// stands before its name and its name, a Favourite's tile (042), the button at its end, its
// own menu, and the window's three buttons. The strip puts them in a row; the sidebar in rows.
import SwiftUI
import WTContract
import WTDesign

/// What a tab shows of itself: a dot while it loads, a Site's mark, its site's icon when the
/// window has it (046), and its name.
struct TabLabel: View {
    let tab: WTTabSummary
    let isFront: Bool
    var icon: CGImage?

    var body: some View {
        HStack(spacing: Tokens.Space.xs) {
            if tab.loading {
                // A dot, not a spinner: a tab that is loading is marked, and stays still.
                Circle()
                    .fill(Tokens.Palette.accent)
                    .frame(width: 6, height: 6)
                    .accessibilityHidden(true)
            }
            if tab.kept {
                // A Site (039): a mark before its name; said in words beside it.
                // Filled at its home; an outline when it has wandered from it.
                Image(systemName: tab.away ? "bookmark" : "bookmark.fill")
                    .font(.caption2)
                    .accessibilityHidden(true)
            }
            if let icon {
                // Its site's icon (046), before its name.
                SiteIcon(picture: icon, side: SiteIcon.inARow, asleep: tab.asleep)
            }
            name
                .lineLimit(1)
                .fontWeight(isFront ? .semibold : .regular)
                // Asleep, it has no page: its name is fainter, and it is said so. Fainter by
                // less than the system's secondary colour, which is too faint to read at this
                // size by the measure the iPhone's accessibility audit uses.
                .opacity(tab.asleep ? TabStrip.asleepOpacity : 1)
        }
    }

    /// A tab's name: its page's title, or its site, or the words for a tab with no page.
    private var name: Text {
        if !tab.title.isEmpty {
            Text(verbatim: tab.title)
        } else if !tab.host.isEmpty {
            Text(verbatim: tab.host)
        } else {
            Text(Words.tabsUntitled)
        }
    }
}

/// A Favourite's tile (042): its site's icon, when the window has it (046); until then a
/// letter, the first of the name it was kept with. Its whole name is said to a screen reader,
/// and shown where a pointer rests on it. It has no button at its end: a tile is as small as
/// it can be pressed.
struct FavouriteTile: View {
    let tab: WTTabSummary
    let model: BrowserModel
    /// It was brought in front from the switcher (047), which puts itself away.
    var onChosen: () -> Void = {}
    @Environment(\.locale) private var locale

    var body: some View {
        let isFront = tab.tabID == model.tab.tabID
        let name = TabStrip.shownName(of: tab, locale: locale)
        Button {
            model.activate(tab.tabID)
            onChosen()
        } label: {
            face(isFront: isFront)
                .frame(minWidth: BrowserWindowView.hitSize, minHeight: BrowserWindowView.hitSize)
                .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .background(isFront ? Tokens.Palette.fieldFill : .clear, in: RoundedRectangle(cornerRadius: Tokens.Radius.field))
        // A line round it: a tile, not a name.
        .overlay {
            RoundedRectangle(cornerRadius: Tokens.Radius.field).strokeBorder(Tokens.Palette.hairline, lineWidth: 1)
        }
        .help(Text(verbatim: name))
        .accessibilityLabel(Text(verbatim: name))
        .accessibilityIdentifier("tab")
        .accessibilityAddTraits(isFront ? .isSelected : [])
        .accessibilityValue(Text(verbatim: TabStrip.said(of: tab, locale: locale)))
        #if !os(macOS)
            .contextMenu { TabMenu(tab: tab, model: model) }
        #endif
    }
}

private extension FavouriteTile {
    /// What a tile shows: the icon alone, or the letter.
    @ViewBuilder
    func face(isFront: Bool) -> some View {
        if let icon = model.icon(of: tab) {
            SiteIcon(picture: icon, side: SiteIcon.onATile, asleep: tab.asleep)
        } else {
            Text(verbatim: TabStrip.letter(of: tab, locale: locale))
                .font(.callout)
                .fontWeight(isFront ? .bold : .medium)
                // Asleep, it has no page: fainter, as a sleeping tab's name is.
                .opacity(tab.asleep ? TabStrip.asleepOpacity : 1)
        }
    }
}

/// The button at a tab's end: it closes the tab; for a Site it puts it to sleep (039), and
/// says so; and a sleeping Site has none, there being no page to close.
struct TabCloser: View {
    let tab: WTTabSummary
    let model: BrowserModel

    var body: some View {
        if tab.kept, tab.asleep {
            // A sleeping Site has no page to close: its place stays as wide as it was.
            Color.clear
                .frame(width: BrowserWindowView.hitSize, height: BrowserWindowView.hitSize)
                .accessibilityHidden(true)
        } else {
            Button {
                model.closeTab(tab.tabID)
            } label: {
                // For a Site the same button puts it to sleep (039), and says so.
                Label { Text(tab.kept ? Words.sitesSleep : Words.tabsClose) } icon: { Image(systemName: "xmark") }
                    .labelStyle(.iconOnly)
                    .font(.caption)
                    .frame(minWidth: BrowserWindowView.hitSize, minHeight: BrowserWindowView.hitSize)
                    .contentShape(Rectangle())
            }
            .buttonStyle(.borderless)
            .accessibilityIdentifier("close-tab")
        }
    }
}

#if !os(macOS)
    /// A tab's own menu, by a long press. For a tab with something to keep, or a Site: home,
    /// for a Site that is away (039); kept or let go; among the Favourites or out of them
    /// (042); and into a Group or out of one (041). For any tab that has somewhere to go: a
    /// place along its row (045). (On a Mac a button takes a right-click for itself and no
    /// menu comes of it, 033: the menu bar and its keys are the ways there.)
    struct TabMenu: View {
        let tab: WTTabSummary
        let model: BrowserModel
        /// The tab is drawn in a column: a row of the sidebar.
        var atTheSide = false
        @Environment(\.locale) private var locale

        var body: some View {
            // What was last done to the row, undone (044): at the top, while there is something.
            TreeUndoItem(undo: model.treeUndo)
            arranging
            MoveTabItems(tab: tab, model: model, atTheSide: atTheSide)
        }

        @ViewBuilder private var arranging: some View {
            if tab.kept || !tab.host.isEmpty {
                if tab.away {
                    Button {
                        model.sendHome(tab.tabID)
                    } label: {
                        Text(verbatim: BrowserModel.returnWords(to: tab.siteName, locale: locale))
                    }
                }
                // A Favourite's tile has no button at its end: its page is put to sleep here.
                if tab.favourite, !tab.asleep {
                    Button {
                        model.closeTab(tab.tabID)
                    } label: {
                        Text(Words.sitesSleep)
                    }
                }
                Button {
                    model.keepOrLetGo(tab.tabID)
                } label: {
                    Text(tab.kept ? Words.sitesLetGo : Words.sitesKeep)
                }
                if model.canBeGrouped(tab) {
                    // Among the Favourites, or out of them (042): grey when the row is full.
                    Button {
                        model.setFavourite(tab.tabID, !tab.favourite)
                    } label: {
                        Text(tab.favourite ? Words.favouritesRemove : Words.favouritesAdd)
                    }
                    .disabled(!model.canFavourite(tab))
                    Menu {
                        MoveToGroupItems(
                            groups: model.groups, current: tab.groupID,
                            move: { model.move(tab.tabID, toGroup: $0) },
                            new: { model.askForANewGroup(for: tab.tabID) }
                        )
                    } label: {
                        Text(Words.groupsMove)
                    }
                }
            }
        }
    }
#endif

/// The window's buttons: the find bar's (010), the zoom bar's (012) and the History list's
/// (008); and on a touch screen, in a window wide enough, the one that puts the tabs down the
/// side or back along the top (043): a Mac has that in its menu bar. Beside the tabs and not
/// among them.
struct WindowButtons: View {
    let model: BrowserModel
    let onFind: () -> Void
    let onZoom: () -> Void
    let onHistory: () -> Void

    var body: some View {
        // One beside the next, as close as they stood in the strip.
        HStack(spacing: 0) {
            #if !os(macOS)
                if model.hasRoomForSidebar {
                    Button {
                        model.switchLayout()
                    } label: {
                        Label {
                            Text(model.layout == .sidebar ? Words.layoutTop : Words.layoutSide)
                        } icon: {
                            Image(systemName: model.layout == .sidebar ? "rectangle.topthird.inset.filled" : "sidebar.leading")
                        }
                        .labelStyle(.iconOnly)
                        .frame(minWidth: BrowserWindowView.hitSize, minHeight: BrowserWindowView.hitSize)
                        .contentShape(Rectangle())
                    }
                    .buttonStyle(.borderless)
                    .accessibilityIdentifier("layout")
                }
            #endif
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
        }
    }
}

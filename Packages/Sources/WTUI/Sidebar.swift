// The sidebar (043): the window's tabs down its side. The same row the strip draws along the
// top, in the core's order: the Favourites as tiles, then the tree of Sites and Groups, then
// under a line the tabs that are no Sites. Nothing here is decided: every row asks what its
// chip in the strip asks.
import SwiftUI
import WTContract
import WTDesign

/// What the sidebar draws, in its three parts.
struct SidebarParts: Equatable {
    /// The Favourites (042): a row of tiles.
    var favourites: [WTTabSummary] = []
    /// The Sites that are in no Group, and each Group with its Sites after it (041).
    var tree: [StripItem] = []
    /// The tabs that are no Sites.
    var others: [WTTabSummary] = []

    /// The row in the core's order (`StripItem.row`), taken apart by what each thing is:
    /// nothing is put in another order within a part.
    init(tabs: [WTTabSummary], groups: [WTGroupSummary]) {
        for item in StripItem.row(tabs: tabs, groups: groups) {
            switch item {
            case let .tab(tab) where tab.favourite:
                favourites.append(tab)
            case let .tab(tab) where !tab.kept:
                others.append(tab)
            default:
                tree.append(item)
            }
        }
    }
}

/// The sidebar: the toolbar and the address field the window gives it, the Favourites, the
/// tree, the other tabs, and at its foot the Space and the window's buttons.
struct Sidebar<Toolbar: View>: View {
    let model: BrowserModel
    var onFind: () -> Void = {}
    var onZoom: () -> Void = {}
    var onHistory: () -> Void = {}
    var onSpaces: () -> Void = {}
    /// Whether the Space and the window's buttons are at its foot: not in the switcher
    /// (047), which has the Space at its head and is over the window, not beside it.
    var showsFoot = true
    /// A tab was brought in front from here, or a new one opened: the switcher puts itself
    /// away. The sidebar does nothing.
    var onChosen: () -> Void = {}
    /// Back, forward and reload, and the address field: the window's own, at this width.
    /// (In the switcher: its head.)
    @ViewBuilder let toolbar: Toolbar

    /// As wide as a name needs.
    static var width: CGFloat {
        260
    }

    var body: some View {
        let parts = SidebarParts(tabs: model.tabs, groups: model.groups)
        VStack(alignment: .leading, spacing: 0) {
            toolbar
            if !parts.favourites.isEmpty {
                tiles(parts.favourites)
            }
            rows(parts)
            if showsFoot {
                foot
            }
        }
        .frame(maxHeight: .infinity, alignment: .top)
    }

    /// The Favourites: tiles in a row, on a second row when there are many.
    private func tiles(_ favourites: [WTTabSummary]) -> some View {
        LazyVGrid(
            columns: [GridItem(.adaptive(minimum: BrowserWindowView.hitSize), spacing: Tokens.Space.xs)],
            alignment: .leading, spacing: Tokens.Space.xs
        ) {
            ForEach(favourites, id: \.tabID) { tab in
                // A tile can be dragged, and a tab dropped on it is put just before it,
                // a Favourite (045 §5b).
                FavouriteTile(tab: tab, model: model, onChosen: onChosen)
                    .dragged(as: .tab(tab.tabID), in: model)
                    .takesDrops(at: .tab(tab.tabID), in: model, mark: .before)
            }
        }
        .padding(.horizontal, Tokens.Space.m)
        .padding(.vertical, Tokens.Space.xs)
        // Past the last tile: the end of the Favourites.
        .contentShape(Rectangle())
        .takesDrops(at: .endOfFavourites, in: model)
    }

    /// The tree, a line, the other tabs, and the row that opens a tab.
    private func rows(_ parts: SidebarParts) -> some View {
        ScrollViewReader { list in
            ScrollView {
                // The rows are made as they come into view: a row of thousands is not drawn
                // to show the thirty that fit (043 §8a).
                LazyVStack(alignment: .leading, spacing: 2) {
                    ForEach(parts.tree) { item in
                        switch item {
                        case let .tab(tab):
                            // A Group's Site is set in under its Group.
                            dragged(tab).padding(.leading, tab.groupID == 0 ? 0 : Tokens.Space.l).id(tab.tabID)
                        case let .group(group):
                            // A Group is dragged among the Groups; a tab dropped on it is
                            // the last of its Sites (045 §5b).
                            GroupChip(
                                group: group,
                                toggle: { model.shutOrOpen(group) },
                                edit: { model.askToEdit(group) },
                                remove: { model.removeGroup(group.groupID) },
                                undo: model.treeUndo
                            )
                            .dragged(as: .group(group.groupID), in: model)
                            .takesDrops(at: .group(group.groupID), in: model)
                        }
                    }
                    // Under a line, the tabs that are only passing through. The line ends the
                    // Sites that are in no Group, and the Groups: what is dropped on it is the
                    // last of those.
                    if !parts.tree.isEmpty || !parts.favourites.isEmpty {
                        // (The line is a place to drop on, and no control: a screen reader is
                        // told nothing of it.)
                        Divider().padding(.vertical, Tokens.Space.xs)
                            .contentShape(Rectangle())
                            .takesDrops(at: .endOfSites, in: model)
                            .accessibilityHidden(true)
                    }
                    ForEach(parts.others, id: \.tabID) { tab in
                        dragged(tab).id(tab.tabID)
                    }
                    // "New Tab" ends the row: a tab dropped on it is the last of the tabs.
                    newTab.takesDrops(at: .endOfTabs, in: model)
                }
                .padding(.horizontal, Tokens.Space.m)
                .padding(.vertical, Tokens.Space.xs)
                .frame(maxWidth: .infinity, alignment: .leading)
            }
            // The tab that comes in front is brought into view.
            .onChange(of: model.tab.tabID) { _, front in
                list.scrollTo(front)
            }
        }
        // A right-click beside the rows offers the tab that was last closed back (033).
        .contentShape(Rectangle())
        .modifier(OffersClosedTab(model: model))
        .accessibilityElement(children: .contain)
        .accessibilityLabel(Text(Words.tabsList))
        .accessibilityIdentifier("sidebar")
    }

    /// A tab's row as a thing that can be dragged, and that takes a tab dropped on it, which
    /// is then put just before it (045 §5b).
    private func dragged(_ tab: WTTabSummary) -> some View {
        row(tab)
            .dragged(as: .tab(tab.tabID), in: model)
            .takesDrops(at: .tab(tab.tabID), in: model, mark: .above)
    }

    /// A tab as a row: what the strip's chip shows, with room for its name, and the button at
    /// its end where the row ends.
    private func row(_ tab: WTTabSummary) -> some View {
        let isFront = tab.tabID == model.tab.tabID
        return HStack(spacing: 0) {
            Button {
                model.activate(tab.tabID)
                onChosen()
            } label: {
                TabLabel(tab: tab, isFront: isFront, icon: model.icon(of: tab))
                    .padding(.leading, Tokens.Space.s)
                    .frame(maxWidth: .infinity, minHeight: BrowserWindowView.hitSize, alignment: .leading)
                    .contentShape(Rectangle())
            }
            .buttonStyle(.plain)
            .accessibilityIdentifier("tab")
            .accessibilityAddTraits(isFront ? .isSelected : [])
            .accessibilityValue(Text(verbatim: TabStrip.said(of: tab)))
            #if !os(macOS)
                .contextMenu { TabMenu(tab: tab, model: model, atTheSide: true) }
            #endif
            TabCloser(tab: tab, model: model)
        }
        .background(isFront ? Tokens.Palette.fieldFill : .clear, in: RoundedRectangle(cornerRadius: Tokens.Radius.field))
    }

    private var newTab: some View {
        Button {
            model.openTab()
            onChosen()
        } label: {
            Label { Text(Words.tabsNew) } icon: { Image(systemName: "plus") }
                .padding(.leading, Tokens.Space.s)
                .frame(maxWidth: .infinity, minHeight: BrowserWindowView.hitSize, alignment: .leading)
                .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        // A screen reader is told of the closed tab to bring back, while there is one.
        .accessibilityActions {
            if model.canReopenClosedTab {
                ReopenClosedTab(model: model)
            }
        }
        .accessibilityIdentifier("new-tab")
    }

    /// At the foot: the Space the window shows (a private window says what it is), and the
    /// window's buttons. Side by side where they fit; where they do not (a touch screen's
    /// buttons are a finger wide), the buttons under the Space. They are never wider than the
    /// sidebar: what is wider than its place is drawn over the place beside it.
    private var foot: some View {
        ViewThatFits(in: .horizontal) {
            HStack(spacing: 0) {
                space
                Spacer(minLength: Tokens.Space.s)
                WindowButtons(model: model, onFind: onFind, onZoom: onZoom, onHistory: onHistory)
            }
            VStack(alignment: .leading, spacing: 0) {
                space
                WindowButtons(model: model, onFind: onFind, onZoom: onZoom, onHistory: onHistory)
            }
        }
        .padding(.horizontal, Tokens.Space.m)
        .padding(.vertical, Tokens.Space.xs)
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    @ViewBuilder private var space: some View {
        if model.isPrivate {
            PrivateChip(action: onSpaces)
        } else {
            SpaceChip(name: model.spaceName, colour: model.spaceColour, icon: model.spaceIcon, action: onSpaces)
        }
    }
}

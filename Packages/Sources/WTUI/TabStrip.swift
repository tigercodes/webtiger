// The window's tabs in a row (005 §5), with the buttons that stand beside them.
import SwiftUI
import WTContract
import WTDesign

/// The window's tabs in a row: each by its name, the one in front marked, a button to close each
/// and one to open another (005 §5).
struct TabStrip: View {
    @Environment(\.locale) private var locale
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
                    // The Sites that are in no Group, each Group with its Sites after it, and
                    // the other tabs (041): the core's two lists, in its order.
                    ForEach(StripItem.row(tabs: model.tabs, groups: model.groups)) { item in
                        switch item {
                        case let .tab(tab):
                            // A Favourite is a tile (042); any other tab has its name.
                            if tab.favourite {
                                tile(tab).id(tab.tabID)
                            } else {
                                chip(tab).id(tab.tabID)
                            }
                        case let .group(group):
                            GroupChip(
                                group: group,
                                toggle: { model.shutOrOpen(group) },
                                edit: { model.askToEdit(group) },
                                remove: { model.removeGroup(group.groupID) }
                            )
                        }
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

    /// A Favourite's tile (042): a letter, until sites have icons; the first of the name it
    /// was kept with. Its whole name is said to a screen reader, and shown where a pointer
    /// rests on it. It has no button at its end: a tile is as small as it can be pressed.
    private func tile(_ tab: WTTabSummary) -> some View {
        let isFront = tab.tabID == model.tab.tabID
        let name = Self.shownName(of: tab, locale: locale)
        return Button {
            model.activate(tab.tabID)
        } label: {
            Text(verbatim: Self.letter(of: tab, locale: locale))
                .font(.callout)
                .fontWeight(isFront ? .bold : .medium)
                // Asleep, it has no page: fainter, as a sleeping tab's name is.
                .opacity(tab.asleep ? TabStrip.asleepOpacity : 1)
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
        .accessibilityValue(Text(verbatim: Self.said(of: tab, locale: locale)))
        #if !os(macOS)
            .contextMenu { menu(of: tab) }
        #endif
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
                    if tab.kept {
                        // A Site (039): a mark before its name; said in words below.
                        // Filled at its home; an outline when it has wandered from it.
                        Image(systemName: tab.away ? "bookmark" : "bookmark.fill")
                            .font(.caption2)
                            .accessibilityHidden(true)
                    }
                    name(tab)
                        .lineLimit(1)
                        .fontWeight(isFront ? .semibold : .regular)
                        // Asleep, it has no page: its name is fainter, and it is said so. Fainter
                        // by less than the system's secondary colour, which is too faint to read
                        // at this size by the measure the iPhone's accessibility audit uses.
                        .opacity(tab.asleep ? TabStrip.asleepOpacity : 1)
                }
                .padding(.leading, Tokens.Space.s)
                .frame(maxWidth: widestName, minHeight: BrowserWindowView.hitSize, alignment: .leading)
                .fixedSize(horizontal: true, vertical: false)
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)
            .accessibilityIdentifier("tab")
            .accessibilityAddTraits(isFront ? .isSelected : [])
            .accessibilityValue(Text(verbatim: Self.said(of: tab, locale: locale)))
            #if !os(macOS)
                // A tab's own menu, by a long press. (On a Mac a button takes a right-click for
                // itself and no menu comes of it, 033: the menu bar and ⌘D are the ways there.)
                .contextMenu { menu(of: tab) }
            #endif
            closer(of: tab)
        }
        .background(isFront ? Tokens.Palette.fieldFill : .clear, in: RoundedRectangle(cornerRadius: Tokens.Radius.field))
    }

    #if !os(macOS)
        /// A tab's own menu: for a tab with something to keep, or a Site. Home, for a Site that
        /// is away (039); kept or let go; and into a Group or out of one (041).
        @ViewBuilder
        private func menu(of tab: WTTabSummary) -> some View {
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
    #endif

    /// The button at a tab's end: it closes the tab; for a Site it puts it to sleep (039), and
    /// says so; and a sleeping Site has none, there being no page to close.
    @ViewBuilder
    private func closer(of tab: WTTabSummary) -> some View {
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

    /// How strongly a sleeping tab's name is drawn: against white this is a contrast of 8 to 1.
    static let asleepOpacity = 0.7

    /// What is said of a tab beside its name (039): that it is a Site, that it is asleep, both,
    /// or nothing.
    static func said(of tab: WTTabSummary, locale: Locale = .current) -> String {
        var said: [String] = []
        // A Favourite is a Site: it is said to be the one, not both.
        if tab.favourite {
            said.append(Words.favouritesFavourite.string(locale))
        } else if tab.kept {
            said.append(Words.sitesSite.string(locale))
        }
        if tab.away {
            said.append(Words.sitesAway.string(locale))
        }
        if tab.asleep {
            said.append(Words.sitesAsleep.string(locale))
        }
        return said.formatted(.list(type: .and, width: .narrow).locale(locale))
    }

    /// A tab's name as words: its page's title, or its site, or the words for a tab with no
    /// page.
    static func shownName(of tab: WTTabSummary, locale: Locale = .current) -> String {
        if !tab.title.isEmpty {
            tab.title
        } else if !tab.host.isEmpty {
            tab.host
        } else {
            Words.tabsUntitled.string(locale)
        }
    }

    /// The letter on a Favourite's tile: the first of the name it was kept with, which does
    /// not change as it wanders; of the name it shows, when it was kept with none.
    static func letter(of tab: WTTabSummary, locale: Locale = .current) -> String {
        SpaceName.initial(of: tab.siteName.isEmpty ? shownName(of: tab, locale: locale) : tab.siteName)
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

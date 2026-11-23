// Dragging in the row (045 §5b), the model's part: what is dragged, where it may be dropped,
// and what the core is then asked. A shell names the thing and the place; what the thing
// becomes there, and whether it may, are the core's.
import WTContract

/// What is dragged: a tab, by its row, its chip or its tile; or a Group, by its chip. By the
/// app's own name for it, and nothing of its page.
enum RowItem: Hashable, Codable, Sendable {
    case tab(UInt64)
    case group(UInt64)
}

/// Where a thing is dropped.
enum RowPlace: Hashable, Sendable {
    /// On a tab, or a Favourite's tile: just before it.
    case tab(UInt64)
    /// On a Group's chip: for a tab, into the Group; for a Group, before it.
    case group(UInt64)
    /// Past the last of the Favourites' tiles.
    case endOfFavourites
    /// Past the last Site that is in no Group: the line under the Groups, which is also where
    /// a Group is put after the last.
    case endOfSites
    /// On "New Tab", which ends the row.
    case endOfTabs
}

extension BrowserModel {
    /// Whether a place takes a thing: so that a place that would do nothing is not marked.
    /// A thing is not dropped on itself; a Group goes only among Groups. Whether a tab may
    /// become what the place would make it is the core's to say, when it is dropped.
    func takes(_ item: RowItem, at place: RowPlace) -> Bool {
        guard windowID != nil else { return false }
        switch (item, place) {
        case let (.tab(dragged), .tab(target)): return dragged != target
        case (.tab, _): return true
        case let (.group(dragged), .group(target)): return dragged != target
        case (.group, .endOfSites): return true
        case (.group, _): return false
        }
    }

    /// A thing was dropped at a place: the core is asked to move it there. Nothing is asked
    /// for a place that does not take it.
    func drop(_ item: RowItem, at place: RowPlace) {
        guard takes(item, at: place) else { return }
        switch item {
        case let .tab(id):
            var ask = WTMoveTab()
            ask.tabID = id
            switch place {
            case let .tab(target): ask.beforeTabID = target
            case let .group(group): ask.endOfGroupID = group
            case .endOfFavourites: ask.endOf = .favourites
            case .endOfSites: ask.endOf = .sites
            case .endOfTabs: ask.endOf = .tabs
            }
            send(.moveTab(ask))
        case let .group(id):
            var ask = WTMoveGroup()
            ask.groupID = id
            // Before that Group; or, at the line under the Groups, after the last (0).
            if case let .group(target) = place {
                ask.beforeGroupID = target
            }
            send(.moveGroup(ask))
        }
    }
}

public import WTContract

// Favourites (042), the model's part: what the window asks of the core to make a tab one of
// its row's Favourites, or to stop it being one. Which tabs are Favourites, where they sit and
// how many a row may have are the core's.
import Foundation

public extension BrowserModel {
    /// A tab can be made a Favourite, or is one that can stop being one: the command has
    /// something to do. A tab with nothing to keep, a private window's tab, and any tab while
    /// the core says the row is full, cannot be made one.
    func canFavourite(_ tab: WTTabSummary) -> Bool {
        tab.favourite || (canBeGrouped(tab) && !favouritesFull)
    }

    /// The same for the tab in front: what the menu's item asks.
    var canFavouriteTheTabInFront: Bool {
        tab.favourite || (canGroupTheTabInFront && !favouritesFull)
    }

    /// Marks a tab as a Favourite, or unmarks it. A tab that is no Site is kept by it: the
    /// core's rule.
    func setFavourite(_ id: UInt64, _ favourite: Bool) {
        var ask = WTSetFavourite()
        ask.tabID = id
        ask.favourite = favourite
        send(.setFavourite(ask))
    }

    /// Makes the tab in front a Favourite, or stops it being one: the other of what the core
    /// last said of it.
    func addOrRemoveTheTabInFrontAsAFavourite() {
        guard let frontTabID, canFavouriteTheTabInFront else { return }
        setFavourite(frontTabID, !tab.favourite)
    }
}

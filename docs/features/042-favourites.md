# 042 — Favourites: the few Sites that are always first (W2)

| | |
|---|---|
| **Status** | **Designed; nothing built.** The choices of §6 are put to the owner |
| **Phase** | W2 of [000](000-master-plan.md) §9: "Favourites". 000 §5.6: "A row of icon tiles at the top of a Space"; §6.1: in the tab strip, "icon-only tabs" |
| **Contract** | To be 1.32 (§5) |
| **Builds on** | [039](039-sites.md): Sites. [041](041-groups.md): the row as a tree, and where a Site sits in it |
| **Last updated** | 2026-11-22 |

## 1. User story

Four or five of my Sites I open all day: mail, the calendar, the chat. I make them Favourites. They are the first things in the window, small, each a tile, and always in the same place, whatever else the row holds.

## 2. What is in it, and what is not

**In:** a Site can be made a Favourite and stop being one; Favourites are first in the row, before every other Site and every Group; a shell draws one as a tile with no name beside it; they are kept across a quit.

**Not in, each with where it goes:**

- **A site's icon.** A tile wants the site's own icon, and no shell has one yet: nothing asks an engine for a page's icon. Until that is built (a feature of its own: what is fetched, with whose logins, and where it is kept), a tile shows the first letter of the Site's name (D4).
- **The sidebar**, where Favourites are a row of tiles at the top (000 §6.1). The strip is still the only layout.
- **Dragging** a tab onto the row, and dragging the tiles into another order: with the sidebar, as for Groups (041).
- **The same Favourites in every Space.** They are a Space's (D3).

## 3. The rule

**A Favourite is a Site that is marked as one.** Everything a Site is, it is: closed, it sleeps and stays; pressed, it wakes at its home; it can wander and be sent back (039).

**Making and unmaking.** `SetFavourite` marks a tab, or unmarks it. A tab that is no Site is kept first, and one that cannot be kept is refused as keeping refuses it. A private window has none.

**Where they sit.** A row is: its Favourites, in the order they were made; its Sites that are in no Group; its Groups, each with its Sites; and the tabs that are no Sites. A Favourite is in no Group: marked, a Site leaves the Group it was in; put in a Group, it is a Favourite no longer (D2). Unmarked, it is a Site in no Group, the last of those.

**How many.** A row has at most twelve (D5): `favourite.too_many`.

**Let go.** A Favourite that is let go (039) is a tab like any other, and no Favourite.

**Across a quit.** They come back as they were, in their order.

**Why the core.** Which Sites are Favourites and where they sit is the model a strip and a sidebar must both draw.

## 4. Where it is kept

A saved tab gains whether it is a Favourite (schema 12).

## 5. The contract (to be 1.32)

Added; nothing is changed or removed.

```proto
// Marks a tab as a Favourite, or unmarks it. Answered with Ack, also when it is so already;
// "tab.unknown"; "tab.no_page"; "window.private"; or "favourite.too_many".
message SetFavourite { uint64 tab_id = 1; bool favourite = 2; }

message TabSummary { … bool favourite = 10; }   // it is one of the row's Favourites
message TabView    { … bool favourite = 20; }   // the tab in front is one
```

The Favourites are the first of `tabs`, in their order.

## 6. Decisions

**All of these are put to the owner; none is built.** The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | A Favourite is a Site with a mark, not a thing of its own | One tab, one page, one place it sleeps: a tile that opened a new tab at each press would be a bookmark, and the plan has none | A tile that opens its address in a new tab each time |
| D2 | A Favourite is in no Group | The row of tiles is its own place, before the tree; a tile inside a folder would be hidden with it | A Site may be a Favourite and in a Group, shown in both |
| D3 | Favourites are a Space's | It is the plan's word (000 §5.6), and a tab has one Space's logins: the same tile in a Space with other logins would be another page | The same Favourites in every Space that shares the logins |
| D4 | Until sites have icons, a tile shows the first letter of the Site's name, and says the whole name to a screen reader and when the pointer rests on it | A tile with nothing on it is no tile; an icon fetched in a hurry would be a privacy question answered in passing | Wait for icons before building Favourites |
| D5 | At most twelve in a row | A row of tiles is for a few; twelve is what a phone's strip can still scroll through at a glance | No limit; or fewer |
| D6 | The command: "Add to Favourites", and "Remove from Favourites" for a tab that is one; in a tab's own menu, and on the Mac in the File menu for the tab in front | Plain names, where "Keep as Site" and "Move to Group" are | A star in the address field |

## 7. In what order

1. **The core, the store and the contract.**
2. **The shells**: the tile, the command, the words.
3. **Icons**, as their own feature.

## 8. Tests

To be filled in as built.

## 9. What is not verified

To be filled in as built.

## 10. Done checklist

- [ ] The owner's word on §6.
- [ ] The core, the store and the contract.
- [ ] The Apple shell.
- [ ] The Android shell.
- [ ] `Tools/test.sh all` green, and `android-device`.

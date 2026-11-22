# 042 — Favourites: the few Sites that are always first (W2)

| | |
|---|---|
| **Status** | **The core, the store and the contract are built and tested, on the recommendations of §6, which are provisional:** they are put to the owner and not yet answered. **The shells draw no Favourite yet** (§7) |
| **Phase** | W2 of [000](000-master-plan.md) §9: "Favourites". 000 §5.6: "A row of icon tiles at the top of a Space"; §6.1: in the tab strip, "icon-only tabs" |
| **Contract** | 1.32 (§5). Store schema 12 (§4) |
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

## 5. The contract (1.32)

Added; nothing is changed or removed.

```proto
// Marks a tab as a Favourite, or unmarks it. Answered with Ack, also when it is so already;
// "tab.unknown"; "tab.no_page"; "window.private"; or "favourite.too_many".
message SetFavourite { uint64 tab_id = 1; bool favourite = 2; }

message TabSummary { … bool favourite = 10; }   // it is one of the row's Favourites
message TabView    { … bool favourite = 20; }   // the tab in front is one
message WindowSnapshot { … bool favourites_full = 11; }   // the row can have no more
```

`favourites_full` is so that no shell needs the number twelve: it greys "Add to Favourites" by what the core says.

The Favourites are the first of `tabs`, in their order.

**As built, where the design did not say:**

- **Unmarking a tab that is no Favourite** is not refused and changes nothing: it is not kept by it.
- **A row's twelve are its own:** another window's row, and another Space's, have theirs. One that is a Favourite already is not counted again; the thirteenth is refused before it is kept.
- **A Favourite is never out of the way:** it is in no Group, so no shut Group hides it.
- **What a store says that the core never writes** (a Site of a Group marked a Favourite; a tab that is no Site marked one): the Site comes back a Favourite in no Group, and the tab a tab.

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

| Where | What |
|---|---|
| The store (`session.rs`, 1 more) | A Site says whether it is a Favourite, and it is there after the store is opened again; a store from before Favourites (version 11) has its tabs and Sites as they were, none a Favourite, and one can be kept in it |
| The core, through the contract (`favourites.rs`, 7) | **Marking:** a tab that is no Site is kept, a Favourite, the first of the row and still in front, and says so; one marked is the last of the Favourites, wherever it was; a Site is marked as a tab is; marked again it is where it was; unmarked it is a Site, the last of those in no Group; none of it loads a page or destroys a web view. **What is refused:** a tab there is not, a tab with no page, a private window's; unmarking a tab that is none keeps nothing. **Twelve:** the thirteenth is refused and not kept; one that is a Favourite is not counted again; room again when one is unmarked; another window has its own twelve; the window is told when its row is full, at twelve and not at eleven. **Groups:** a Site of a Group that is marked leaves it and the Group holds one fewer; a Favourite put in a Group is one no longer; taken out of no Group it is as it was; a shut Group hides no Favourite; removing a Group leaves them. **A Site:** closed, a Favourite sleeps at its home and is one still; let go it is a tab and none; kept again it does not go back among them. **Across a quit** (a store on disk): in the order they were made, each asleep at its home; unmarked and quit again. **A store that says what the core never writes:** only a Site is a Favourite, and none is in a Group |
| The rule broken, a part at a time | 25 parts, in the model and in the store: 23 caught. The two that were not were no faults: a condition that could never differ and a move the row's order already makes, both taken out of the code |

## 9. What is not verified

- **Nothing draws a Favourite as one.** The shells are told (1.32) and draw it as the Site it is, first in the row; no shell can ask for one yet.
- **The choices of §6** are the recommendations, not the owner's word.

## 10. Done checklist

- [ ] The owner's word on §6.
- [x] The core, the store and the contract.
- [ ] The Apple shell.
- [ ] The Android shell.
- [ ] `Tools/test.sh all` green, and `android-device`.

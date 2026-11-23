# 044 — Undo for what is done to the tree (W2)

| | |
|---|---|
| **Status** | **Designed; nothing built.** The choices of §6 are put to the owner |
| **Phase** | W2 of [000](000-master-plan.md) §9: "undo". 000 §6.2: "Undo for every organising action, including closing a tab and deleting a Group" |
| **Contract** | To be 1.34 (§5) |
| **Builds on** | [033](033-a-closed-tab-comes-back.md): the journal's first form, and undo for a closed tab ("each joins the journal with its own feature"). [039](039-sites.md), [041](041-groups.md), [042](042-favourites.md): what is done to the tree, none of which can be undone yet |
| **Last updated** | 2026-11-22 |

## 1. User story

I remove a Group I did not mean to remove, or let go of a Site by the wrong key, or move a tab into the wrong Group. I ask for it to be undone, and the row is as it was.

## 2. What is in it, and what is not

**In:** what is done to arrange a row can be undone: a tab kept or let go; a Group made, named, coloured or removed; a tab put in a Group or taken out; a Favourite made or unmade. The window is told what an undo would undo, so that a shell can name it.

**Not in, each with where it goes:**

- **What is no arranging:** a Group shut or opened, the layout switched, a tab brought in front. They are how the row is looked at.
- **A closed tab**, which has its own way back (033), and keeps it (D1).
- **Spaces:** making, naming and colouring one. A Space is not a row's arrangement, and deleting one is not built.
- **Redo.**
- **Across a quit** (D3).

## 3. The rule

**An arrangement is undone as a whole.** Before the core does any of the things of §2 to a row, it keeps the row's arrangement as it is: which tabs are Sites (with the home and the name of each), which Group each is in, which are Favourites, the order of the tabs, and the Groups with their names and colours. Undoing puts the row back to that:

- every tab that is still in the row is what it was, and where it was among those that are still there;
- a tab closed since then stays closed (033 is how it comes back); a tab opened since then stays, where the row's order puts a tab of its kind;
- the Groups are the Groups there were, by name and colour; whether each is shut is left as it is now;
- no page is loaded, closed, put to sleep or woken by it.

**One line for each row**, the newest first, fifty deep. Undo undoes the newest and takes it off the line. A thing that changed nothing (a Site kept again; a tab moved to the Group it is in) is not kept.

**The window is told** what its newest is, by its kind: `site.kept`, `site.let_go`, `group.made`, `group.removed`, `group.renamed`, `group.coloured`, `group.moved`, `favourite.added`, `favourite.removed`; or nothing.

**A private window** has no Sites, Groups or Favourites, and nothing to undo.

**Why the core.** What an arrangement was, and what putting it back means when tabs have come and gone since, is the model's.

## 4. Where it is kept

In the core's memory, for the run (D3).

## 5. The contract (to be 1.34)

Added; nothing is changed or removed.

```proto
// Undoes the newest thing done to arrange the row a window shows (044). Answered with Ack;
// "window.unknown"; or "undo.nothing".
message UndoArrangement { uint64 window_id = 1; }

message WindowSnapshot { … string undo_arrangement = 13; }   // the kind of the newest; empty for none
```

`Undo` (033) stays what it is: the closed tab's.

## 6. Decisions

**All of these are put to the owner; none is built.** The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | Two ways back, apart: a closed tab's (033: ⇧⌘T, the held button), and the arrangement's | "Reopen Closed Tab" is what every browser has, and it should bring back the tab that was closed whatever was arranged since; the owner chose its place (WT-D25). One line for both would make it do nothing until an arrangement had been undone first | One line for everything a person did, newest first, as 033's `Undo` was first described |
| D2 | The whole arrangement is kept and put back, not the one thing that was done | One rule for nine kinds of thing, and for those to come (dragging); and it is right when tabs have gone and come since | An entry of its own for each kind, which undoes only that |
| D3 | For the run only: after a quit there is nothing of the arrangement to undo | An arrangement names tabs, and a tab is not the same thing after a quit: it comes back by its place, asleep. A closed tab is kept across a quit because it is one address and one place | The arrangements written to the store with the session, each row's fifty |
| D4 | The command says what it undoes: "Undo Remove Group", "Undo Move to Group", and so on; grey with nothing to undo | A person should see what will happen before it does | "Undo", always |
| D5 | On the Mac: in the Edit menu, ⌘Z, when the keyboard is not in a field of words (where ⌘Z undoes typing, as it does everywhere) | It is where Undo is on a Mac. In a field, typing's undo is the one a person means | In the File menu, with a key of its own |
| D6 | On a touch screen: in the menu of the thing nearest to what was done (a tab's own menu, a Group's own menu), as "Undo …" at its top while there is something to undo | No new control on a strip that is full (WT-D25's reason) | A line across the window with "Undo" after each thing done |

## 7. In what order

1. **The core and the contract.**
2. **The shells.**

## 8. Tests

To be filled in as built.

## 9. What is not verified

To be filled in as built.

## 10. Done checklist

- [ ] The owner's word on §6.
- [ ] The core and the contract.
- [ ] The shells.
- [ ] `Tools/test.sh all` green, and `android-device`.

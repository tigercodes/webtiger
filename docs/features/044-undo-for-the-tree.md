# 044 — Undo for what is done to the tree (W2)

| | |
|---|---|
| **Status** | **Built and tested in the core and in the Apple and Android shells.** D5 and D6, where it is asked from, are the owner's (WT-D32, 2026-11-23: "use suggested choices for the blocked items"); D1 to D4 are still the recommendations, and are built |
| **Phase** | W2 of [000](000-master-plan.md) §9: "undo". 000 §6.2: "Undo for every organising action, including closing a tab and deleting a Group" |
| **Contract** | 1.34 (§5). Nothing is kept in the store |
| **Builds on** | [033](033-a-closed-tab-comes-back.md): the journal's first form, and undo for a closed tab ("each joins the journal with its own feature"). [039](039-sites.md), [041](041-groups.md), [042](042-favourites.md): what is done to the tree, none of which can be undone yet |
| **Last updated** | 2026-11-23 |

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

**The window is told** what its newest is, by its kind: `site.kept`, `site.let_go`, `group.made`, `group.removed`, `group.renamed`, `group.coloured`, `group.moved`, `favourite.added`, `favourite.removed`; or nothing. (Since [045](045-moving-in-the-row.md): `tab.moved` and `group.reordered` too.)

**A private window** has no Sites, Groups or Favourites, and nothing to undo.

**Why the core.** What an arrangement was, and what putting it back means when tabs have come and gone since, is the model's.

## 4. Where it is kept

In the core's memory, for the run (D3).

## 5. The contract (1.34)

Added; nothing is changed or removed.

```proto
// Undoes the newest thing done to arrange the row a window shows (044). Answered with Ack;
// "window.unknown"; or "undo.nothing".
message UndoArrangement { uint64 window_id = 1; }

message WindowSnapshot { … string undo_arrangement = 13; }   // the kind of the newest; empty for none
```

`Undo` (033) stays what it is: the closed tab's.

**As built, where the design did not say:**

- **A Group that comes back** (its removal undone) is shut or open as it was when it was removed; one that is still there is as it is now.
- **A tab that has come since** is after the others when a row is put back: it is no Site, whatever was done to it since having been undone before.
- **Each window's row has its own line**, as it has its own Groups (041): what is done in one window is not another's to undo.
- **What is put back is then what is kept** across a quit, as any arrangement is: only the line of what could be undone is the run's.
- **A kind a shell does not know** is undone all the same: the kind is a name for the command, and nothing goes by it.

**The shells, as built** (D4 to D6):

- **The command says what it undoes** (D4): "Undo Keep as Site", "Undo Stop Keeping", "Undo New Group", "Undo Remove Group", "Undo Rename Group", "Undo Group Colour", "Undo Move to Group", "Undo Add to Favourites", "Undo Remove from Favourites", and since 045 "Undo Move Tab" and "Undo Move Group". A kind a shell has no words for is undone all the same: the command says "Undo".
- **On the Mac** (D5): the Edit menu's first item, ⌘Z, in the place of the system's own. **Typing comes first, as everywhere:** where the keyboard is in something that has typing to undo (the address field, the find bar, a page's own field), the command is that thing's, says "Undo", and undoes the typing by the system's own word for it. With no typing to undo there, it is the tree's, says what it undoes, and is grey when there is nothing. Redo (⇧⌘Z) is the field's alone: an arrangement undone is not done again (§2). The item is read anew each time a menu opens: where the keyboard is, and whether there is typing to undo there, are the system's to say, and it changes them without a word.
- **On a touch screen** (D6): at the top of a tab's own menu and of a Group's own menu, with a line under it, while there is something to undo. A tab with nothing else in its menu (an empty one) has a menu for it. On the Mac a Group's menu has it too.
- **An iPad with a keyboard has no key for it:** its ⌘Z is the field's, as the system has it. The menus are its way.

## 6. Decisions

**D5 and D6 are the owner's (WT-D32). D1 to D4 are put to the owner, and built as recommended.** The first column is the recommendation.

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

| Where | What |
|---|---|
| The core, through the contract (`tree_undo.rs`, 6) | **Each thing, the newest first:** nine things done to a row (a tab kept; a Group made; a tab put in it; the Group named, and coloured; a Favourite made, and unmade; a Site let go; the Group removed), the window told after each what an undo would undo; undone one by one, the row is each time as it was before that thing, and at the end as it began; nothing to undo is refused, and a window there is not; none of it loads a page or destroys a web view. **Not kept:** what changed nothing (a Site kept again, a tab that is none let go or taken out or unmarked, a Group given the name and colour it has), what was refused, and what is no arranging (a Group shut and opened, the layout switched, a tab brought in front); a private window has nothing to undo. **Tabs that have come and gone:** a Group with two Sites removed while shut; then a tab closed, a tab opened, a Site of the Group put to sleep; undone, the Group is back, shut as it was, with its Sites, the sleeping one asleep; the closed tab is not in the row in any way (the row's second place is the tab that came since), and the tab that came since stays; the closed tab comes back by its own undo, which is no arranging. **A Group still there** is shut as it is now. **Each row its own, fifty deep:** another window's row has nothing to undo of this one's; of sixty-one things done the newest fifty are undone, and the row is as it was after the eleventh. **Across a quit** (a store on disk): what was undone is what is kept, and there is nothing of the arrangement to undo afterwards |
| The Apple shell's model (`TreeUndoModelTests`, 2, against a scripted core) | The command is there when the core says there is something to undo, with the words for it, and asks the core to undo the arrangement of its window's row; it is not there, and nothing is asked, before the window is open and when the core says there is nothing. The words for each of the eleven kinds; and "Undo" for a kind the shell has no words for. Broken in 6 parts, 6 caught |
| The Mac app (`TreeUndoTests`, 1, with the real core and the real engine) | With nothing done, the Edit menu's item says "Undo" and is grey. A tab kept: it says "Undo Keep as Site"; chosen, the tab is a tab again, and the item is grey. Kept again and ⌘Z: the same. Kept again, and "abc" typed in the address field: the item says "Undo", ⌘Z undoes the typing (the field has the page's address again) and the Site is a Site still; then the item says "Undo Keep as Site" again, and ⌘Z undoes that |
| The iPhone app (`TreeUndoTests`, 1, on the simulator) | A tab kept, and undone from its own menu, where the command is at the top. A Group made for the tab (two things: the Group made, the tab put in it): from the Group's own menu "Undo Move to Group" takes the tab out, and it is no Site; then "Undo New Group", and the Group is gone. Nothing was loaded |
| The Android shell on the JVM (`TreeUndoTest`, 5) | The model: as Apple's. The words, as Apple's. The strip drawn: the command is first in a tab's menu, above the rest, says what it undoes, asks the core and puts the menu away; a Group's menu has it above "Edit Group…"; an empty tab's menu is the command alone; with nothing to undo no menu has it. Down the side, a row's menu and a Group's have it too. Broken in 15 parts, 15 caught |
| The Android app on the emulator (`AppTreeUndoTest`, 1, the real core and the real WebView) | A tab kept from its menu, then "Undo Keep as Site" from the top of the same menu: a tab again. Made a Favourite, which keeps it too: one "Undo Add to Favourites" takes it all the way back. With nothing left to undo, the menu has no such command |
| The rule broken, a part at a time | 19 parts, in the model and where the requests are answered: 18 caught, one once the test looked for a closed tab at the row's second place. The one that was not was no fault: the row was put in a row's order again after an undo, which what is put back already is; taken out of the code |

## 9. What is not verified

- **D1 to D4** are the recommendations, not the owner's word.
- **Typing in a page, on the Mac.** That ⌘Z is the page's when a page's own field has typing to undo is the same lines as for the address field, which is tested; no test typed in a page and pressed ⌘Z.
- **Redo** is handed to whatever has the keyboard, by the system's own word; no test pressed it.
- **The Mac's item while a menu is open and the keyboard moves:** it is read when a menu opens, and not again until the next.
- **An iPad** (its menus are the iPhone's; nothing ran there). **A screen reader.** **A real phone.**
- **Apple's views were not broken on purpose:** that the command is at the top of the menus, and what the Edit menu does, are seen by the apps' tests only.
- **Windows:** its strings are in resources that were never built on a PC.
- **A long row.** Each thing that arranges a row copies the row's arrangement, twice: nothing for a row of hundreds, and not measured for thousands ([043](043-the-sidebar.md) §8a).
- **Dragging**, when it comes, is to be one more thing that arranges a row, and is kept the same way: nothing here needs to change for it, and nothing here was tested with it.

## 10. Done checklist

- [ ] The owner's word on §6.
- [x] The core and the contract.
- [x] The shells.
- [x] The core: `Tools/test.sh all` green, and `android-device`, on 2026-11-22: `lint`, `core` (602, and the strings tool's 24), `unit` (293 on the Mac), `build`, the Mac's UI suite (44), `ios` (275 on the iPhone simulator), the iPhone's UI suite (34), `android` (411 and 32 on the JVM) and `android-device` (159 on the emulator). No shell changed but for the contract's number.
- [ ] The shells: the gate.

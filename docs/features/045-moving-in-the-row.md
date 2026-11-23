# 045 — Moving a tab, and a Group, in the row (W2)

| | |
|---|---|
| **Status** | **The core, the contract and the commands are built and tested, on the recommendations of §6, which are provisional:** they are put to the owner and not yet answered. A tab is moved a place by a command in every shell. **Dragging is not built:** it waits for undo for the tree to be in the apps (D2) |
| **Phase** | W2 of [000](000-master-plan.md) §9: "the tree with drag and drop". 000 §5.6: "Dragging it into a Group makes it a Site" |
| **Contract** | 1.35 (§5), and 1.36 for the commands (§5a). Nothing new is kept in the store: a row's order, and its Groups', are written already |
| **Builds on** | [005](005-tabs.md) ("Moving a tab within a window: with the tree and drag and drop"); [039](039-sites.md), [041](041-groups.md), [042](042-favourites.md): the row's order, which nothing a person does can change yet but by making and unmaking; [044](044-undo-for-the-tree.md): what is moved can be put back |
| **Last updated** | 2026-11-23 |

## 1. User story

My tabs are in the order I opened them, my Sites in the order I kept them, my Groups in the order I made them. I want them in an order of my own: this tab before that one; this Site in that Group, third; this Group before the other.

## 2. What is in it, and what is not

**In:** a tab is moved to another place in its row; a Group is moved among its row's Groups. The core decides what a move means; a shell says only what was moved and where it was dropped. By dragging, and by commands that do not need a pointer.

**Not in, each with where it goes:**

- **To another window, or another Space:** with several windows side by side (000 §9, W5).
- **A link, or a file, dragged onto the row** to open it: with the tree's other drops, later.
- **Groups in Groups** ([041](041-groups.md) D1).
- **The order of Spaces.**

## 3. The rule

**A row has parts, in an order that does not change** (041, 042): the Favourites; the Sites that are in no Group; each Group's Sites, Group by Group; the tabs that are no Sites. Within a part, the order is the person's.

**A tab that is moved becomes what its new neighbours are.** It is put just before another tab of its row, or at the end of a part, and is then of that part:

| Moved to | It becomes | As if by |
|---|---|---|
| among the Favourites | a Favourite (a Site first, if it was none) | "Add to Favourites" |
| among the Sites in no Group | a Site, in no Group and no Favourite | "Keep as Site"; "No Group"; "Remove from Favourites" |
| among a Group's Sites, or onto the Group | a Site of that Group | "Move to Group" |
| among the tabs that are no Sites | a tab that is no Site | "Stop Keeping" |

and it is refused as those commands are: a tab with no page of the web's cannot become a Site; a row has at most twelve Favourites; nothing of a private window is kept (its tabs move among themselves, which keeps nothing). **A move that is refused moves nothing.**

**Within its part** a tab is only put in its place: it is what it was.

**By a step,** for the keyboard: one place towards the row's start, or its end, among the tabs of its own part. At the end of its part it stays: a step never makes a tab another thing.

**A Group is moved whole,** before another Group or after the last, and its Sites go with it. Groups stay where Groups are in a row: after the Sites that are in no Group.

**Nothing else changes.** No page is loaded, closed, put to sleep or woken; the tab in front stays in front; a tab moved into a shut Group is out of the way, as that Group's Sites are, unless it is the one in front.

**It can be undone** (044): a move is a thing that arranges a row, and is kept as one (`tab.moved`, `group.reordered`). A move that changed nothing is not.

**Why the core.** What a row's parts are, and what a tab becomes among them, is the model's: a shell that worked it out would be a second place for the rule of 041 and 042.

## 4. Where it is kept

The order of a row's tabs and of its Groups is the session's, written as it is now.

## 5. The contract (1.35)

Added; nothing is changed or removed.

```proto
// Moves a tab to another place in its row (045). Answered with Ack; "tab.unknown";
// "move.nowhere" (no place named); "move.other_row" (the tab named as the place is not in
// the row); "group.unknown"; and what keeping a tab or making a Favourite is refused with
// ("tab.no_page", "window.private", "favourite.too_many").
message MoveTab {
  uint64 tab_id = 1;
  oneof to {
    // Just before this tab of the same row: the moved tab becomes of its part.
    uint64 before_tab_id = 2;
    // The end of a part of the row.
    RowPart end_of = 3;
    // The end of a Group.
    uint64 end_of_group_id = 4;
    // One place, among the tabs of its own part.
    RowStep by = 5;
  }
}
enum RowPart { ROW_PART_UNSPECIFIED = 0; ROW_PART_FAVOURITES = 1; ROW_PART_SITES = 2; ROW_PART_TABS = 3; }
enum RowStep { ROW_STEP_UNSPECIFIED = 0; ROW_STEP_BACK = 1; ROW_STEP_FORWARD = 2; }

// Moves a Group among its row's Groups (045): before another, or (0) after the last.
// Answered with Ack; "group.unknown".
message MoveGroup { uint64 group_id = 1; uint64 before_group_id = 2; }
```

**As built, where the design did not say:**

- **A tab there is not is refused first,** whatever place was named; and every refusal names the field of the request that was at fault.
- **Before itself, or where it is already,** a tab or a Group stays, and nothing is kept to undo.
- **A step goes by the row's order,** shown or not: a Site that is in front in a shut Group steps among that Group's Sites, which are out of the way.
- **The place may be a tab that is out of the way** (a shut Group's Site): the moved tab is put before it, and is out of the way with it unless it is the one in front.
- **A private window's tabs move among themselves,** and its moves are undone as any: only becoming a Site is refused there.

What a drop in a drawn row is, for a shell: on a tab, `before_tab_id`; on a Group's chip, `end_of_group_id`; after the last Favourite, the last Site in no Group, or the last tab, `end_of`. A shell does not work out what the tab becomes.

## 5a. The commands (1.36)

A command that moves a tab by a step is offered when the step would move it, and the core says when: a shell does not work out the row's parts.

```proto
message TabSummary { … bool steps_back = 11; bool steps_forward = 12; }
```

**Their words go by how the row is drawn,** which is the shell's: along the top, "Move Tab Left" and "Move Tab Right"; down the side, "Move Tab Up" and "Move Tab Down" (a Favourite's tile is in a row there too: left and right). Where the row runs from right to left, left is towards its end. **Their places:** the tab's own menu on a touch screen; on the Mac the Window menu, for the tab in front, under "Show Previous Tab". No key (D6).

**As built:**

- **A tab with nowhere to go has no such commands** in its menu; one that can go one way has both, the other grey. In the Mac's Window menu the two are always listed, grey when the tab in front cannot go that way, or no window is in front.
- **An empty tab has a menu on a touch screen now,** when it has somewhere to go: the commands that move it, and no other. (It had none: it has nothing to keep.)
- **The shells work nothing out about the row:** which way a tab can go is read from what the core says of it; which words a command has is from how the shell itself drew the tab.

## 6. Decisions

**All of these are put to the owner.** D1 to D4 are built in the core as recommended, and D6 in the shells; D5, dragging, is not built. The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | A tab that is moved becomes what its new neighbours are: among Sites a Site, in a Group that Group's, among Favourites a Favourite | The plan's own words for a Group (000 §5.6), and 042's for the Favourites. One gesture, and the row says what will happen: the tab is where it was dropped | A tab moves within its own part only; becoming another thing is by command |
| D2 | **That goes both ways: a Site moved among the tabs that are no Sites is a Site no longer** | A row that takes a drop in one direction and refuses it in the other has to be learnt. What is lost is the Site's home and its name, and undo (044) brings them back: **which is why undo for the tree should be in the apps before dragging is** | A Site stops at the end of the Sites, and is let go by its command only |
| D3 | By the keyboard, a step moves a tab within its part and never out of it | A key pressed once too often should not unmake a Site | A step goes on into the next part, and the tab becomes of it |
| D4 | A Group is moved whole among the Groups, and stays after the Sites that are in no Group | The row's parts are in one order everywhere (041): it is what lets a shut Group, a phone's strip and the sidebar show the same row | Groups and loose Sites in any order among one another: a tree of two kinds of thing at one level |
| D5 | Dragging in the sidebar and in the strip alike, on every system; on a touch screen a tab is lifted by a long press and then dragged, and let go where it was it shows its menu, as now | It is how each system moves things, and the menu keeps its place | Dragging in the sidebar only, where there is room |
| D6 | The commands, for the keyboard and for a screen reader: "Move Tab Left" and "Move Tab Right" (at the side, "Up" and "Down"), in the tab's menu and on the Mac in the Window menu; no key of their own yet | Every system's browser has the commands; their keys differ and several are taken (034 to 036) | Keys now: ⌃⇧PageUp and ⌃⇧PageDown, as two other browsers have |

## 7. In what order

1. **The core and the contract.**
2. **The commands** in the shells (D6): they need no dragging, and are what a keyboard and a screen reader use.
3. **Dragging** on the Mac, then on a touch screen (D5): after undo for the tree is in the apps (D2).

## 8. Tests

| Where | What |
|---|---|
| The core, through the contract (`moving.rs`, 9) | **Within its part:** among the tabs that are no Sites, the Sites in no Group, a Group's Sites and the Favourites, a tab is put before another, before the first, and at the part's end, and is what it was; the tab in front stays in front; nothing is loaded and no web view destroyed. **To another part:** a tab before a Site, or at the Sites' end, is a Site; a Site before a tab that is none, or at the tabs' end, is a Site no longer; onto a Group, or before one of its Sites, a tab is that Group's; a Group's Site among the Sites in none is in none; among the Favourites a tab, and a Group's Site, is a Favourite; a Favourite in a Group, or among the Sites, is one no longer. **Refused, and nothing moved:** a tab there is not (first, whatever else was said); no place, and a part or a step this core does not know; a place in another window's row, and one there is not; a Group there is not; an empty tab among the Sites, in a Group, among the Favourites (and it moves among the tabs as any); a thirteenth Favourite, before one and at their end; in a private window, becoming a Site or a Favourite (its tabs move among themselves). Each refusal names the field at fault. **A step:** one place among the tabs of its part; at the part's end, and at the row's, it stays and is what it was; the steps that moved nothing are not kept to undo. **A Group moved whole:** before another, after the last, between two; its Sites go with it; before itself and where it is, nothing; one there is not, either way, and one of another row, refused; a shut Group is moved shut; the Sites in no Group stay before every Group. **Undone:** a move, and a Group's, by undo for the tree, named `tab.moved` and `group.reordered`; a Site that a move unmade is a Site again, where it was. **Into a shut Group:** a tab is out of the way with its Sites, the tab in front is shown still; opened, the Group has them in the order they were put in. **Across a quit** (a store on disk): the tabs' order and the Groups' are as they were left |
| The core, what a window is told (`moving.rs`, 1 more: 10) | Of each tab, which way a step would move it: of four tabs of one part the first has nowhere back to go and the last nowhere forward; two Sites and two tabs each go as far as their part, and what is said is what a step then does; a tab alone in its part goes nowhere; and the Site in front in a shut Group still has that Group's Sites, out of the way, to step among. Broken in 6 parts, 6 caught |
| The Apple shell's model (`MovingModelTests`, 3, against a scripted core) | The core is asked for a step, for the tab in front and for a tab by its menu, and only the way the core says the tab can go; after the core says where the tabs are now, the other way. The commands' words: along the top left is back, and where words run from right to left it is forward, listed in the same order; down the side up is back, whichever way words run. What the menu bar lists for the tab in front: left and right along the top; up and down at the side; left and right for a Favourite in front at the side, and in a window with no room for a sidebar |
| The apps (`MovingTests`, 1, on the Mac and the iPhone simulator, with the real core and the real engine) | Of two tabs the last cannot go right (the command is grey) and goes left: it has changed places with the other and is in front still, with its page; then the first cannot go left and goes right; nothing was loaded. By the Window menu on the Mac, by the tab's own menu on the iPhone. On the Mac, with the tabs at the side: "Move Tab Up" and "Move Tab Down" do the same, and the last cannot go down |
| The Android shell on the JVM (`MovingTest`, 5) | The model asks the core for a step. The words by how the row is drawn, as on Apple's. The strip drawn: a tab's menu has the two commands, grey the way it cannot go, and nothing is asked that way; the tab in front between two goes either way; a Favourite's tile moves as any tab; a tab alone in its part has the rest of its menu and no such command; an empty tab with somewhere to go has the two and no other. Drawn from right to left: "Move Tab Left" is forward. In a wide window with the tabs at the side: a row says up and down, and a Favourite's tile left and right |
| The Android app on the emulator (`AppMovingTest`, 1) | As `MovingTests`, from the tab's own menu, with the real core and the real WebView; and a tab that is not in front moves, and stays behind |
| The shells broken, a part at a time | Apple's model in 12 parts, 12 caught. Android's in 17: 16 caught; the one that was not was a condition that could never be otherwise (a tile is no row of the sidebar), taken out. Apple's views (that a menu's item is grey; that the sidebar's rows say up and down) are seen by the apps' tests only, which were not run against a broken shell |
| The rule broken, a part at a time | 34 parts, in the model and where the requests are answered. 32 caught: 26 at once; one that did not build, once written so that it did; one once the tests read which field a refusal names; four written after those. Two were not, and neither is a fault: each was the model's own telling of the window that its row had changed, and the window is told all the same where the row's arrangement is kept (044). The one for a Group moved was taken out of the code, where the row is now arranged and the window told by one call; the one for a step stays |

## 9. What is not verified

- **Only a step is asked for by a shell.** The rest of the contract's moves (before a tab, to a part's end, onto a Group; a Group among the Groups) is what dragging will send: nothing in an app sends it yet.
- **An iPad.** That the rows of an iPad's sidebar say up and down is one argument where the row's menu is made; the words' choice is tested, and nothing pressed it on an iPad or its simulator.
- **From right to left.** The words and the steps are tested in the models, and Android's menu drawn from right to left on the JVM; no app was run in a language that runs so. The Mac's menu bar goes by the app's direction.
- **A screen reader**, and **the keyboard alone**: the commands are in menus, with no key (D6). On the Mac the Window menu is reached by the keyboard; nobody listened to what is said.
- **Windows:** the four strings are in its resources, which were never built on a PC.
- **The choices of §6** are the recommendations, not the owner's word.
- **A place in another Space's row of the same window** is refused as one in another window's is, by the same lines; the test has another window.
- **A step among Sites that are out of the way** (the tab in front, in a shut Group): by the row's order, as built; not tested.
- **A long row.** A move puts the row in order again and, to be undone, copies its arrangement twice (044 §9): nothing for hundreds of tabs, not measured for thousands ([043](043-the-sidebar.md) §8a).
- **Several windows showing one Space:** each has its own row (039 D6), and a move is of the row it is made in.

## 10. Done checklist

- [ ] The owner's word on §6.
- [x] The core and the contract.
- [x] The commands in the shells.
- [ ] Dragging.
- [x] The core: `Tools/test.sh all` green, and `android-device`, in the night of 2026-11-22 to 23: `lint`, `core` (611, and the strings tool's 24), `unit` (293 on the Mac), `build`, the Mac's UI suite (45), `ios` (275 on the iPhone simulator), the iPhone's UI suite (35), `android` (411 and 32 on the JVM) and `android-device` (159 on the emulator). No shell changed but for the contract's number.
- [x] The commands: `Tools/test.sh all` green, and `android-device`, on 2026-11-23: `lint`, `core` (612, and the strings tool's 24), `unit` (296 on the Mac), `build`, the Mac's UI suite (46), `ios` (278 on the iPhone simulator), the iPhone's UI suite (36), `android` (416 and 32 on the JVM) and `android-device` (160 on the emulator).

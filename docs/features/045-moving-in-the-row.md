# 045 — Moving a tab, and a Group, in the row (W2)

| | |
|---|---|
| **Status** | **The core, the contract and the commands are built and tested in every shell; dragging is built and tested on the Mac.** A tab is moved a place by a command everywhere, and on the Mac a tab or a Group is dragged, along the top and down the side (§5b). **On a touch screen nothing is dragged yet:** on the iPhone's simulator a thing was picked up and never dropped when a test tried it, and Android's is not started. The recommendations of §6 are provisional: they are put to the owner and not yet answered |
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

## 5b. Dragging (built on the Mac)

Nothing is added to the contract: a drop sends what §5 has had since 1.35, and what it means is the core's.

**What is dragged:** a tab, by its row, its chip or its tile; a Group, by its chip. **What a drag carries is the app's own name for the thing, and nothing else:** not the page's address, not its title. It is of a kind only this app knows, and is offered to this app alone: let go over another app, or over a page, nothing is dropped. (A tab dragged out as a link, and a link dragged in to be opened, are the tree's other drops: not this.)

**Where it may be dropped, and what the shell then asks:**

| Dropped on | A tab asks | A Group asks |
|---|---|---|
| a tab, or a Favourite's tile | to be put just before it (`before_tab_id`) | nothing |
| a Group's chip | to be the last of that Group (`end_of_group_id`) | to be put before that Group (`before_group_id`) |
| the end of the Favourites (the row of tiles, past the last) | the end of that part (`end_of`) | nothing |
| the end of the Sites in no Group, and the line under the Groups | the end of the Sites in no Group; the line, being under the Groups, is also where a Group is put after the last | after the last Group (`before_group_id` 0) |
| "New Tab", which ends the row | the end of the tabs that are no Sites | nothing |

A shell works out none of what the tab becomes: it names the thing dropped and the place, and draws what the core then says. **A drop that the core refuses moves nothing,** and the row is as it was; the reason is not said (a row that does not take a drop says so by not taking it), as for a command that is refused.

**While it is dragged** the place it would go is marked: a line before the tab it would be put before, and the chip or the end it would go to drawn as chosen. A thing held over itself marks nothing.

**On the Mac** a thing is dragged with the pointer. **On a touch screen** it is lifted by a long press and then dragged; let go where it was, its menu is shown, as now (D5). The commands (§5a) stay: they are what a keyboard and a screen reader use, and dragging takes nothing from them.

**It is undone as any move is** (044), from where undo is asked.

**In what order:** Apple's windows, which share their views (the Mac first, since a pointer's drag can be driven by a test; then a touch screen); then Android.

**As built on the Mac, where the design did not say:**

- **What the system is handed is a kind and no bytes.** The thing that is held, and the window whose row it is of, stay in the app; a place takes it only in that window's row. The kind is the app's own (`com.tigercodes.webtiger.row-item`), and is offered to this app alone. **It has to be declared in the app's own list of what it is** (its `Info.plist`): until it was, a drag began and no place was offered it.
- **Along the top** a thing is dropped on a tab or a tile (and is put before it), on a Group's chip, and on the button for a new tab, which ends the row. **The strip has no place for the end of the Favourites, nor for the end of the Sites that are in no Group:** its parts follow one another with nothing between. Down the side there is one for each (past the last tile; the line under the Groups); along the top those are reached by the commands.
- **Down the side:** a row (the line that marks it is above it), a tile, a Group's chip, the room past the last tile, the line under the Groups, and the row for a new tab. A window with no Favourite has no tiles to drop among: the first is made by its command.
- **The line under the Groups is nine points high:** a pointer's target. It is a place to drop on and no control: a screen reader is told nothing of it.
- **A row that can be dragged must still say what it is.** With dragging first put on a tab's own button, a row of the sidebar (whose rows are made as they come into view) went on telling a screen reader what it was when it was made: its old name after its page changed, that it was no Site when it had become one, that it was in front when it no longer was. The strip's tabs did not. What holds: dragging is put on the whole row, not on the button, and at least one of its two parts (what picks up, what takes a drop) is handed to the system as a view of no known kind. Why is not understood; a test of the app holds it in place.
- **Nothing is dragged on a touch screen.** On the iPhone's simulator, with a test's hand, a thing is picked up after a long press (which is its menu's too), the tab it is held over is offered it and says it would take it, and when it is let go nothing is dropped: the drag leaves the place instead. So it was with dragging on the row and with dragging on the same thing as the tab's menu, and whether the place said it would move the thing or copy it. Whether a person's hand drops what a test's hand does not was not tried: nobody was at the simulator. Until it is known, nothing there is changed: a long press is the menu, as before, and the code that drags is the Mac's alone.

## 6. Decisions

**All of these are put to the owner.** D1 to D4 are built in the core as recommended, and D6 in the shells; D5, dragging, is built on the Mac. The first column is the recommendation.

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
3. **Dragging** on the Mac (D5): after undo for the tree is in the apps (D2). **Built.**
4. **Dragging on a touch screen:** Apple's, then Android's.

## 8. Tests

| Where | What |
|---|---|
| The core, through the contract (`moving.rs`, 9) | **Within its part:** among the tabs that are no Sites, the Sites in no Group, a Group's Sites and the Favourites, a tab is put before another, before the first, and at the part's end, and is what it was; the tab in front stays in front; nothing is loaded and no web view destroyed. **To another part:** a tab before a Site, or at the Sites' end, is a Site; a Site before a tab that is none, or at the tabs' end, is a Site no longer; onto a Group, or before one of its Sites, a tab is that Group's; a Group's Site among the Sites in none is in none; among the Favourites a tab, and a Group's Site, is a Favourite; a Favourite in a Group, or among the Sites, is one no longer. **Refused, and nothing moved:** a tab there is not (first, whatever else was said); no place, and a part or a step this core does not know; a place in another window's row, and one there is not; a Group there is not; an empty tab among the Sites, in a Group, among the Favourites (and it moves among the tabs as any); a thirteenth Favourite, before one and at their end; in a private window, becoming a Site or a Favourite (its tabs move among themselves). Each refusal names the field at fault. **A step:** one place among the tabs of its part; at the part's end, and at the row's, it stays and is what it was; the steps that moved nothing are not kept to undo. **A Group moved whole:** before another, after the last, between two; its Sites go with it; before itself and where it is, nothing; one there is not, either way, and one of another row, refused; a shut Group is moved shut; the Sites in no Group stay before every Group. **Undone:** a move, and a Group's, by undo for the tree, named `tab.moved` and `group.reordered`; a Site that a move unmade is a Site again, where it was. **Into a shut Group:** a tab is out of the way with its Sites, the tab in front is shown still; opened, the Group has them in the order they were put in. **Across a quit** (a store on disk): the tabs' order and the Groups' are as they were left |
| The core, what a window is told (`moving.rs`, 1 more: 10) | Of each tab, which way a step would move it: of four tabs of one part the first has nowhere back to go and the last nowhere forward; two Sites and two tabs each go as far as their part, and what is said is what a step then does; a tab alone in its part goes nowhere; and the Site in front in a shut Group still has that Group's Sites, out of the way, to step among. Broken in 6 parts, 6 caught |
| The Apple shell's model (`MovingModelTests`, 3, against a scripted core) | The core is asked for a step, for the tab in front and for a tab by its menu, and only the way the core says the tab can go; after the core says where the tabs are now, the other way. The commands' words: along the top left is back, and where words run from right to left it is forward, listed in the same order; down the side up is back, whichever way words run. What the menu bar lists for the tab in front: left and right along the top; up and down at the side; left and right for a Favourite in front at the side, and in a window with no room for a sidebar |
| The apps (`MovingTests`, 1, on the Mac and the iPhone simulator, with the real core and the real engine) | Of two tabs the last cannot go right (the command is grey) and goes left: it has changed places with the other and is in front still, with its page; then the first cannot go left and goes right; nothing was loaded. By the Window menu on the Mac, by the tab's own menu on the iPhone. On the Mac, with the tabs at the side: "Move Tab Up" and "Move Tab Down" do the same, and the last cannot go down |
| The Android shell on the JVM (`MovingTest`, 5) | The model asks the core for a step. The words by how the row is drawn, as on Apple's. The strip drawn: a tab's menu has the two commands, grey the way it cannot go, and nothing is asked that way; the tab in front between two goes either way; a Favourite's tile moves as any tab; a tab alone in its part has the rest of its menu and no such command; an empty tab with somewhere to go has the two and no other. Drawn from right to left: "Move Tab Left" is forward. In a wide window with the tabs at the side: a row says up and down, and a Favourite's tile left and right |
| The Android app on the emulator (`AppMovingTest`, 1) | As `MovingTests`, from the tab's own menu, with the real core and the real WebView; and a tab that is not in front moves, and stays behind |
| Dragging, the Apple shell's model (`DraggingModelTests`, 2, against a scripted core) | A tab dropped on a tab is asked to be put before it; on a Group's chip, to be the last of the Group; past the tiles, on the line and on the row for a new tab, to be the last of that part. A Group dropped on a Group is asked to be put before it; on the line, after the last. Before the window is open nothing is asked. A thing on itself, and a Group on a tab, past the tiles or on the row for a new tab, are taken nowhere and ask nothing; a tab and a Group of the same number are not the same thing. Broken in 17 parts, 17 caught |
| Dragging, the Mac app (`DraggingTests`, 3, with the real core and the real engine) | **Along the top:** of two tabs the second, dropped on the first, is before it and in front still; dropped on the button for a new tab it is the last, and no tab was opened; undone from the Edit menu, it is where it was. Dropped on a Group's chip it is a Site, and the Group, shut, has two; undone, it is a tab like any other. **Down the side:** a row dropped on another is above it; a tab dropped on the line under the Group is a Site, and the Group has its one Site still; a second Group, dropped on the first, is before it. Nothing was loaded by any of it. **A row that can be dragged says what it is now:** kept while the sidebar shows it, it says it is a Site; its page changed, it has the new name and says it is away; the other row brought in front is the one chosen; a Group's chip says shut and open; a Favourite's tile says so; and Apple's audit faults nothing in a sidebar with places to drop on |
| Dragging, the Mac's wiring broken, a part at a time, under the app's tests | 14 parts. 11 caught: each place that takes no drop, each thing that cannot be picked up, what is picked up not noted, and both parts of dragging left the kind of view they are (the rows then say what they were). Three were not: either part alone left the kind of view it is (one hidden is enough, and both are); and the line under the Groups left a control to a screen reader, which the Mac's audit does not fault (the iPhone's did, when there was dropping there) |
| Found by these tests | That the kind of thing a drag is must be declared by the app. That a row which can be dragged stopped saying what it is (§5b). That on a touch screen the line under the Groups was a control eight points high, by Apple's audit of the switcher: there is no dropping on a touch screen now, and the line is nothing to a screen reader on any |
| The shells broken, a part at a time | Apple's model in 12 parts, 12 caught. Android's in 17: 16 caught; the one that was not was a condition that could never be otherwise (a tile is no row of the sidebar), taken out. Apple's views (that a menu's item is grey; that the sidebar's rows say up and down) are seen by the apps' tests only, which were not run against a broken shell |
| The rule broken, a part at a time | 34 parts, in the model and where the requests are answered. 32 caught: 26 at once; one that did not build, once written so that it did; one once the tests read which field a refusal names; four written after those. Two were not, and neither is a fault: each was the model's own telling of the window that its row had changed, and the window is told all the same where the row's arrangement is kept (044). The one for a Group moved was taken out of the code, where the row is now arranged and the window told by one call; the one for a step stays |

## 9. What is not verified

- **Dragging on a touch screen:** not built (§5b). On the iPhone, the iPad and Android a tab is moved by its commands.
- **What dragging did not try:** a drop past the last tile, and a tile picked up, in the app (the model's test has the first; the wiring is the same line as a row's); a thing dragged from one window to another, which by its lines is taken nowhere; a thing let go over another app, or over the page; a row that scrolls while something is held over it; a row drawn from right to left; a move of a Group undone.
- **What is marked while a thing is held** (the line before a tab, the place drawn round): no picture has it, and no test looks; the tests see where a thing is after it is let go.
- **Why a row that can be dragged stopped saying what it is,** and so whether what holds now holds on another version of the system. The app's test would say.
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
- [x] Dragging on the Mac.
- [ ] Dragging on a touch screen.
- [x] The core: `Tools/test.sh all` green, and `android-device`, in the night of 2026-11-22 to 23: `lint`, `core` (611, and the strings tool's 24), `unit` (293 on the Mac), `build`, the Mac's UI suite (45), `ios` (275 on the iPhone simulator), the iPhone's UI suite (35), `android` (411 and 32 on the JVM) and `android-device` (159 on the emulator). No shell changed but for the contract's number.
- [x] The commands: `Tools/test.sh all` green, and `android-device`, on 2026-11-23: `lint`, `core` (612, and the strings tool's 24), `unit` (296 on the Mac), `build`, the Mac's UI suite (46), `ios` (278 on the iPhone simulator), the iPhone's UI suite (36), `android` (416 and 32 on the JVM) and `android-device` (160 on the emulator).
- [x] Dragging on the Mac: `Tools/test.sh all` green, and `android-device`, on 2026-11-23: `lint`, `core` (634, and the strings tool's 24), `unit` (325 on the Mac), `build`, the Mac's UI suite (54), `ios` (307 on the iPhone simulator), the iPhone's UI suite (41), `android` (437 and 32 on the JVM) and `android-device` (167 on the emulator).

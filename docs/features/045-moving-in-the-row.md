# 045 — Moving a tab, and a Group, in the row (W2)

| | |
|---|---|
| **Status** | **Designed; nothing built.** The choices of §6 are put to the owner |
| **Phase** | W2 of [000](000-master-plan.md) §9: "the tree with drag and drop". 000 §5.6: "Dragging it into a Group makes it a Site" |
| **Contract** | To be 1.35 (§5). Nothing new is kept in the store: a row's order, and its Groups', are written already |
| **Builds on** | [005](005-tabs.md) ("Moving a tab within a window: with the tree and drag and drop"); [039](039-sites.md), [041](041-groups.md), [042](042-favourites.md): the row's order, which nothing a person does can change yet but by making and unmaking; [044](044-undo-for-the-tree.md): what is moved can be put back |
| **Last updated** | 2026-11-22 |

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

## 5. The contract (to be 1.35)

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

What a drop in a drawn row is, for a shell: on a tab, `before_tab_id`; on a Group's chip, `end_of_group_id`; after the last Favourite, the last Site in no Group, or the last tab, `end_of`. A shell does not work out what the tab becomes.

## 6. Decisions

**All of these are put to the owner; none is built.** The first column is the recommendation.

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

To be filled in as built.

## 9. What is not verified

To be filled in as built.

## 10. Done checklist

- [ ] The owner's word on §6.
- [ ] The core and the contract.
- [ ] The commands in the shells.
- [ ] Dragging.
- [ ] `Tools/test.sh all` green, and `android-device`.

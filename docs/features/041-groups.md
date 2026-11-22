# 041 — Groups: Sites in folders (W2)

| | |
|---|---|
| **Status** | **The core, the store and the contract are built and tested, on the recommendations of §6, which are provisional:** they were put to the owner on 2026-11-22 and not yet answered. **The shells draw no Group yet** (§7) |
| **Phase** | W2 of [000](000-master-plan.md) §9: "Spaces, Groups, Sites; … the tree with drag and drop". This is the tree, without drag and drop |
| **Contract** | 1.31 (§5). Store schema 11 (§4) |
| **Builds on** | [039](039-sites.md): Sites, which until now have no parent. [024](024-several-spaces.md), [025](025-space-colours.md): Spaces, and the colours a Space can have |
| **Last updated** | 2026-11-22 |

## 1. User story

I have a dozen Sites and they are of three kinds: work, the house, what I read. I make a Group for each and put its Sites in it. A Group can be shut: its Sites are out of the way, and one press has them back. The Groups, and what is in them, are there tomorrow.

## 2. What is in it, and what is not

**In:** a Space's Sites can be put in Groups; a Group has a name and may have a colour; it can be shut and opened; Groups are kept across a quit; the shells draw a Group in the strip and offer to make one, name it, put a Site in it and take one out.

**Not in, each with where it goes:**

- **Drag and drop.** Moving is by command first; dragging comes with the sidebar, where there is room to drag.
- **The sidebar**, where a Group is a folder of a tree (000 §6.1). The strip is still the only layout.
- **Groups in Groups.** The plan has Groups that nest (000 §5.6). One level is built first (D1).
- **Tabs that are no Sites in a Group.** A Group holds Sites: putting a tab in one keeps it (000 §5.6: "dragging it into a Group makes it a Site").
- **A split saved with its Group** (W5); **sync** (W9); **the Groups of an import** (the Arc importer's file has them, [002](002-arc-import.md); nothing reads them into a window yet).

## 3. The rule

**A row is a tree, one level deep.** A Space's row is: its Sites that are in no Group; then its Groups, in the order they were made, each with its Sites in theirs; and then the tabs that are no Sites. A Site is in one Group or in none. (Sites and Groups in one order of the person's own choosing come with dragging: until something can be moved, there is no other order to keep.)

**Making and naming.** A Group is made in the row a window shows (`MakeGroup`), with a name, after the Groups there are. It is renamed (`RenameGroup`) and given a colour or none (`ColourGroup`; the colours are a Space's, 025). A name is what a Space's name may be (024): trimmed, not empty, not too long.

**Putting in and taking out.** `MoveToGroup` puts a tab in a Group, at its end: a tab that is no Site is kept first (039), and one with no page to keep is refused. With no Group named, a Site is taken out, to the end of the Sites that are in no Group. A private window has no Groups, as it has no Sites.

**Shut and open.** A Group is shut or opened (`ShutGroup`). Shut, its Sites are not in the row a shell is told to draw, but for the one in front: a tab that is in front is always shown. Shutting a Group changes no tab: none is closed, none put to sleep, none woken (D3). A step along the row (035) does not go into a shut Group; a tab of one that is brought in front some other way (chosen while typing, 038) is shown, and the Group stays shut.

**Removing.** `RemoveGroup` takes a Group away and leaves its Sites, as Sites with no Group, after those there were (D4).

**Across a quit.** Groups come back with their names, colours, order, what is in them, and whether they were shut.

**Why the core.** Where a Site is, which Groups there are and what a shut Group hides are the model that a sidebar and a strip must both draw, and that sync will carry.

## 4. Where it is kept

The table the plan names, `node` (000 §5.6): a row for each Group of a Space, with its place, name, colour and whether it is shut (schema 11). A saved tab gains the place of the Group it is in. A Site keeps its home and name where 039 put them: moving them into `node` is for the day a Site can be without a tab.

## 5. The contract (1.31)

Added; nothing is changed or removed.

```proto
message MakeGroup   { uint64 window_id = 1; string name = 2; }   // → GroupMade { group_id }
message RenameGroup { uint64 group_id = 1; string name = 2; }
message ColourGroup { uint64 group_id = 1; string colour = 2; }  // "" for none
message ShutGroup   { uint64 group_id = 1; bool shut = 2; }
message RemoveGroup { uint64 group_id = 1; }
// Puts a tab in a Group, at its end; group_id 0 takes a Site out of its Group.
message MoveToGroup { uint64 tab_id = 1; uint64 group_id = 2; }

message GroupSummary {
  uint64 group_id = 1;
  string name = 2;      // redacted
  string colour = 3;
  bool shut = 4;
  uint32 sites = 5;     // how many Sites it holds, shown or not
}

message WindowSnapshot { … repeated GroupSummary groups = 10; }  // in the Groups' order
message TabSummary     { … uint64 group_id = 9; }                // 0: in no Group
```

A shell draws the row from the two lists: the tabs of `tabs` that are Sites in no Group; then each Group of `groups`, in order, with the tabs of `tabs` that say they are in it; then the other tabs. A Group with no tab to show (shut, or empty) is drawn from `groups` alone. `tabs` has the Sites of a shut Group left out, but for the tab in front.

Codes: `group.unknown` (also a Group of another row); `group.name_empty` (as `space.name_empty`); `group.colour_unknown`; `group.too_many` (a row has at most 64); `tab.no_page`, `tab.unknown`, `window.private`, `window.unknown` as elsewhere.

**As built, where the design did not say:**

- **Which rows.** A Group belongs to a row: a Space's row in one window. A second window's row of the same Space has its own Groups, as it has its own tabs; only the window with the session (009) has its Groups kept, as only its tabs are.
- **A Group that is not shown** (its Space is not the one the window shows) can be named, coloured, shut and removed by its id.
- **Two Groups may have one name.**
- **A Site of a Group that is closed** sleeps in its Group (039). **Let go** (039), it leaves its Group, and kept again it is in none.
- **Closing a tab brings no Site out of a shut Group**: the tab that comes forward is one that is shown, and when none is, an empty tab is opened. A tab closed by a page that it had opened still goes back to that page's tab, shut Group or not: that tab is then in front, and shown.
- **What is written.** A Space's Groups are written before its tabs, each only when it differs from what the store has. A Site says which Group it is in by the Group's place; a place with no Group at it (the Groups were written and the tabs were not) is no Group, and the row is put in a row's order when it comes back.

## 6. Decisions

**All of these are put to the owner; none is built.** The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | One level of Groups first | A strip has no room to show depth, and one level is what most people use. The table and the contract can hold a parent later without changing what is there | Groups in Groups from the start, as the plan's final shape |
| D2 | A Group holds Sites only; putting a tab in one keeps it | A Group is a place for what is kept: a tab that comes and goes would vanish from its Group when closed. It is the plan's rule (000 §5.6) | Any tab can be in a Group, as Chrome's tab groups |
| D3 | Shutting a Group hides its Sites and changes none of them | Out of sight is the point; pages that are playing or loading go on. Putting them to sleep is a second thing a person may or may not want | Shutting a Group puts its Sites to sleep, to free memory |
| D4 | Removing a Group leaves its Sites, with no Group | The Group was a way to arrange them, and removing the arrangement should lose nothing | Removing a Group removes its Sites too, with an undo |
| D5 | A Group's colours are a Space's (025), or none | One palette, already drawn and named in every shell | Colours of its own |
| D6 | In the strip a Group is a chip with its name before its Sites; a press shuts or opens it; shut, it shows how many Sites it holds | 000 §6.1: "coloured chips that open into tabs" | A Group's Sites behind a menu of the chip |
| D7 | The commands: "New Group…", "Rename Group…", "Remove Group", and for a tab "Move to Group" with the Groups by name and "No Group" | Plain names for plain things; "Move to Group" keeps a tab that is no Site | "Add to Group" / "Ungroup" |

## 7. In what order

1. **The core, the store and the contract**: the rule of §3 through `FakeShell`, over a store on disk for what comes back.
2. **The shells**: the chip, shutting and opening, the commands, the words.
3. **Dragging**, with the sidebar.

## 7a. The shells (designed; not built)

**The strip.** The row is drawn from the two lists of §5: the Sites in no Group; each Group's chip and after it the Group's Sites; the other tabs. The chip has a folder for its mark (an outline when open, filled when shut), in the Group's colour when it has one, and the Group's name; shut, the number of Sites it holds stands after the name. A press shuts or opens it. A screen reader is told its name, that it is a Group, whether it is open or shut, and, shut, how many Sites it holds.

**A tab's menu** gains "Move to Group": the Groups by name, "No Group" for a tab that is in one, and "New Group…", which asks for a name and a colour, makes the Group and puts the tab in it. A Group is made by putting something in it: no command makes an empty one. On a phone and on Android it is in the menu a tab has by a long press; on the Mac it is in the File menu, for the tab in front, where "Keep as Site" is.

**A Group's menu** has "Edit Group…" (its name and colour, on the page that made it) and "Remove Group". By a long press on the chip on a phone and on Android; by a right-click on it on the Mac.

**The page** is a Space's page made smaller: a name, the colours of a Space with "None" first, and two buttons. What the core refuses is said on it in the words for its code.

**No key is new.** Shutting the Group of the tab in front by a key, and stepping onto a chip with the keyboard, are not in this part.

## 8. Tests

| Where | What |
|---|---|
| The store (`session.rs`, 2 more) | A Space's Groups are kept in their order with name, colour and whether shut, a Site says which it is in, saving again replaces them as a whole, and they are there after the store is opened again. A store from before Groups (version 10) has none, its tabs and its Site are as they were, and a Group can be kept in it |
| The core, through the contract (`groups.rs`, 14) | **Making and naming:** a Group is after the Groups there are, its name tidied as a Space's, with no colour, open, empty; two may have one name; named again; a colour, another, none; ten things refused, each by its code, and a private window has none. No more than 64 in a row, and room again when one is removed. **Putting in and taking out:** a tab put in a Group is kept, a Site of that Group, and still in front; the row is the Sites in no Group, then the Groups' Sites Group by Group in the Groups' order, then the other tabs; a tab put in is at its Group's end, also in the Group it is in; from one Group into another; taken out, it is after the Sites in no Group; taken out of none changes nothing and keeps nothing. **What is refused:** a tab or Group there is not, a tab with no page, a Group of another Space's row, of another window's row, a private window's tab; none of it keeps a tab. **Shut:** its Sites are left out of what the window is told to draw and the Group says how many it holds; shut twice and opened twice; the tab in front is shown though its Group is shut, and is out of the way when another is in front; one chosen while typing is shown and the Group stays shut; a tab put in a shut Group is out of the way at once; none of it loads a page, destroys a web view or puts a tab to sleep. **A step** goes among the tabs that are shown, round the ends and by place, and through the Group again when it is open. **Closing** the tab in front brings forward one that is shown, and with none shown an empty tab; a Site of a shut Group that is closed sleeps out of the way, and its awake neighbour in the Group does not come forward. **Removing** leaves its Sites, after those in no Group, shown; the Group can no longer be used. **A Site asleep** is in its Group; **let go**, in none. **Rows:** another Space's row has its own Groups, and one that is not shown can be named and shut. **Across a quit** (a store on disk): names, colours, order, the empty one, the shut one still shut, each Site in its Group, the one in front shown though its Group is shut; opening a Group wakes nothing; changed and quit again; quit with nothing changed. Each Space's Groups come back with it. Only the window with the session has its Groups. A Site whose Group the store has lost is a Site in no Group, in a row's order |
| The rule broken, a part at a time | 40 parts, in the model, in what is written and read, and in the store: all caught. Two tests were added for what the first twelve did not reach (a second window; a store that lost a Group) before the run |

## 9. What is not verified

- **Nothing draws a Group.** The shells are told of them (1.31) and ignore them: a tab of a shut Group would vanish from a strip with nothing to say where it went. No shell can ask for one yet, so none can be made outside a test.
- **The choices of §6** are the recommendations, not the owner's word.
- **Undo after the last shown tab was closed.** With Sites out of the way in a shut Group and no other tab, closing the tab in front opens an empty tab; bringing the closed tab back (033) then leaves that empty tab beside it. A row with one tab only has its empty stand-in make way.
- **Many Groups.** 64 are made in a test; nothing measures a row of many Groups and many Sites.

## 10. Done checklist

- [ ] The owner's word on §6.
- [x] The core, the store and the contract.
- [ ] The Apple shell.
- [ ] The Android shell.
- [x] Part 1 (the core, the store and the contract): `Tools/test.sh all` green, and `android-device`, on 2026-11-22: `lint`, `core` (583, and the strings tool's 24), `unit` (279 on the Mac), `build`, the Mac's UI suite (40), `ios` (261 on the iPhone simulator, one skipped as before), the iPhone's UI suite (30), `android` (386 and 32 on the JVM) and `android-device` (157 on the emulator). No shell asks for a Group or draws one yet: their suites say that nothing else changed.
- [ ] Part 2 (the shells): the gate.

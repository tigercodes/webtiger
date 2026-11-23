# 041 — Groups: Sites in folders (W2)

| | |
|---|---|
| **Status** | **Built and tested in the core and in the Apple and Android shells, on the recommendations of §6, which are provisional:** they were put to the owner on 2026-11-22 and not yet answered. Dragging, and the sidebar, are not built (§7) |
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

**A row is a tree, one level deep.** A Space's row is (after its Favourites, since [042](042-favourites.md)): its Sites that are in no Group; then its Groups, in the order they were made, each with its Sites in theirs; and then the tabs that are no Sites. A Site is in one Group or in none. (Sites and Groups in one order of the person's own choosing come with dragging: until something can be moved, there is no other order to keep.)

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

A shell draws `tabs` in the order it is given them, and never in another, and puts each Group's chip before the first tab that says it is in that Group. A Group with no tab to show (shut, or empty) is drawn from `groups` alone, after the Group before it; the first of the Groups stands after the Sites that are in no Group, which is before the first tab that is no Site. `tabs` has the Sites of a shut Group left out, but for the tab in front.

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

## 7a. The shells

**The strip.** The row is drawn from the two lists as §5 says: the tabs in the core's order, with each Group's chip put before its Sites. The chip has a folder for its mark (an outline when open, filled when shut), in the Group's colour when it has one, a line round it, and the Group's name; shut, the number of Sites it holds stands after the name ("House (3)"). A press shuts or opens it. A screen reader is told its name, that it is a Group, whether it is open or shut, and, shut, how many Sites it holds. Putting the chips among the tabs is all a shell does: the order of the tabs, which of them are shown, and the count are the core's. (As first written each shell put the tabs in a row's order itself, Sites first. An older test of Android's strip, which lists its tabs in an order no core sends, failed when the whole suite was run, and showed it: a shell that sorts would hide a core that did not.)

**A tab's menu** gains "Move to Group": the Groups by name and "No Group", of which the one that is the tab's is marked, and "New Group…", which asks for a name and a colour, makes the Group and puts the tab in it. A Group is made by putting something in it: no command makes an empty one. On a phone and on Android it is in the menu a tab has by a long press (on Android under a heading, a menu there having none within it); on the Mac it is in the File menu, for the tab in front, where "Keep as Site" is. A tab with nothing to keep, and a private window's tab, do not have it.

**A Group's menu** has "Edit Group…" (its name and colour, on the page that made it) and "Remove Group". By a long press on the chip on a phone and on Android; by a right-click on it on the Mac. (On the Mac a button takes a right-click for itself and no menu comes of it, 033: the chip is pressed as a gesture there, and says to a screen reader that it is a button.)

**The page** is a Space's page made smaller: a name, the colours of a Space with "None" first, "Cancel" and "Make Group" or "Save". It scrolls. What the core refuses is said on it in the words for its code, and it then stays open; the name goes to the core as it was typed. An edit asks for the name when it is another, and then for the colour when it is another.

**No key is new.** Shutting the Group of the tab in front by a key, and stepping onto a chip with the keyboard, are not in this part (§9).

**Words** (`Localization/strings.json`): `groups.move`, `groups.none`, `groups.new`; `group.new.title`, `group.edit.title`, `group.name`, `group.make`, `group.save`, `group.cancel`, `group.edit`, `group.remove`, `group.shut_label`, `group.said.open`, `group.said.shut`; `error.group.name_empty`, `.unknown`, `.too_many`, `.colour_unknown`, `.failed`, `.change_failed`.

## 8. Tests

| Where | What |
|---|---|
| The store (`session.rs`, 2 more) | A Space's Groups are kept in their order with name, colour and whether shut, a Site says which it is in, saving again replaces them as a whole, and they are there after the store is opened again. A store from before Groups (version 10) has none, its tabs and its Site are as they were, and a Group can be kept in it |
| The core, through the contract (`groups.rs`, 14) | **Making and naming:** a Group is after the Groups there are, its name tidied as a Space's, with no colour, open, empty; two may have one name; named again; a colour, another, none; ten things refused, each by its code, and a private window has none. No more than 64 in a row, and room again when one is removed. **Putting in and taking out:** a tab put in a Group is kept, a Site of that Group, and still in front; the row is the Sites in no Group, then the Groups' Sites Group by Group in the Groups' order, then the other tabs; a tab put in is at its Group's end, also in the Group it is in; from one Group into another; taken out, it is after the Sites in no Group; taken out of none changes nothing and keeps nothing. **What is refused:** a tab or Group there is not, a tab with no page, a Group of another Space's row, of another window's row, a private window's tab; none of it keeps a tab. **Shut:** its Sites are left out of what the window is told to draw and the Group says how many it holds; shut twice and opened twice; the tab in front is shown though its Group is shut, and is out of the way when another is in front; one chosen while typing is shown and the Group stays shut; a tab put in a shut Group is out of the way at once; none of it loads a page, destroys a web view or puts a tab to sleep. **A step** goes among the tabs that are shown, round the ends and by place, and through the Group again when it is open. **Closing** the tab in front brings forward one that is shown, and with none shown an empty tab; a Site of a shut Group that is closed sleeps out of the way, and its awake neighbour in the Group does not come forward. **Removing** leaves its Sites, after those in no Group, shown; the Group can no longer be used. **A Site asleep** is in its Group; **let go**, in none. **Rows:** another Space's row has its own Groups, and one that is not shown can be named and shut. **Across a quit** (a store on disk): names, colours, order, the empty one, the shut one still shut, each Site in its Group, the one in front shown though its Group is shut; opening a Group wakes nothing; changed and quit again; quit with nothing changed. Each Space's Groups come back with it. Only the window with the session has its Groups. A Site whose Group the store has lost is a Site in no Group, in a row's order |
| The rule broken, a part at a time | 40 parts, in the model, in what is written and read, and in the store: all caught. Two tests were added for what the first twelve did not reach (a second window; a store that lost a Group) before the run |
| The Apple shell's model (`GroupModelTests`, 6, on the Mac and the iPhone simulator) | The row of the two lists: the Sites in no Group, each Group and its tabs, the other tabs; no Groups; a shut Group with nothing shown; a tab that names a Group the list has not is drawn as one in none; every tab once; and the tabs never in another order than the core's, whatever that is. What a chip shows and says, open and shut. A press asks for the other of what the core said; a tab is moved and taken out; a Group removed; an empty tab and a private window's are not offered a Group. A new Group: the colours asked for, the Group made by the name as typed, coloured when a colour was chosen, the tab put in it, the page closed. A Group the core will not make: the page stays with the core's code, nothing coloured or moved; its words, and the words for a code with none. An edit: nothing asked when nothing changed, the name alone, the colour alone and to none, both with the name first; a refused name leaves the colour unasked and the page open |
| The Apple shell's pictures | The window with Groups (an open one with a colour and its Site in front, a shut one that says how many it holds, a tab) and the page (making one; editing one the core would not change), light and dark, on the Mac and the iPhone simulator: 12 pictures, each looked at. As first drawn the page was cut at the top and bottom on the iPhone: it scrolls now, and is pictured as tall as a page |
| The apps (`GroupTests`, 2, on the Mac and the iPhone simulator, with the real core and the real engine) | A tab is put in a new Group by "New Group…" (nothing made of no name), has its chip, and is a Site; with a second tab somewhere else, a press on the chip puts the Site out of the way and the chip says how many it holds, and another press has it back, neither loading anything. The Group's own menu edits it: the page holds its name, and the chip has the new one. "No Group" takes the Site out: a Site still, and the Group, shut, holds none. Put back by the Group's name; with the other tab in front the shut Group hides it. Removed while shut, the chip is gone and the Site stays. Apple's accessibility audit on the page and on the strip with a shut Group |
| The apps across a quit (`QuitTests`, 1) | A shut Group and its Site come back when the app itself starts again, on a store on disk: [043](043-the-sidebar.md) §8 says how |
| The Android shell on the JVM (`GroupsTest`, 9) | The same row, with the same cases, and the same model against a scripted core. Drawn by Compose: a chip for each Group by its name, shut with its count, what each says, the two marks, and where each stands among the tabs; a press asks the core. A Group's own menu: the page with its name, saved by what changed; removed. A tab's own menu: the Groups by name with the tab's own marked, another chosen, "No Group", and for a tab that is no Site "No Group" marked; "New Group…" opens the page, which makes nothing of no name, with the core's colours. What the core refuses is on the page in words, and "Cancel" leaves it. An empty tab and a private window's tab have no "Move to Group" |
| Android's pictures | The window with Groups at a phone's width and a tablet's, and the page in its two states, light and dark: 8 pictures, each looked at. At a phone's width the strip shows the beginning of the row (the open Group, its Site): the tablet's shows all of it |
| The Android app on the emulator (`AppGroupsTest`, 1) | As `GroupTests`, in one test, with the real core and the real WebView |
| The shells broken, a part at a time | Android, 27 parts (the row, the model's requests, the chip's words and marks, the menus, the page): all caught by `GroupsTest`. Apple, 23 parts of the row, the chip's words and the model: all caught by `GroupModelTests` (two of them once they were written so that they compiled). The row's parts were broken again after it was rewritten to keep the core's order |

## 9. What is not verified

- **By the keyboard alone.** No key shuts or opens a Group, and on the Mac the chip is not a control the Tab key stops at (it is pressed as a gesture, so that its right-click menu works). "Move to Group" is in the Mac's menu bar; a Group's own menu is not. A screen reader is given the chip as a button with its press: **no screen reader was run**.
- **A phone's strip is narrow.** With Groups the row is longer, and a phone shows the part round the tab in front. Nothing was changed for it: the sidebar is where a tree has room.
- **The Group being edited is removed meanwhile.** The page stays open until its button is pressed, and then says "That Group isn’t there any more.". Seen in a picture, not in an app.
- **A real phone, an iPad, Windows.** The apps were run on the Mac, the iPhone simulator and the Android emulator.
- **The choices of §6** are the recommendations, not the owner's word.
- **Undo after the last shown tab was closed.** With Sites out of the way in a shut Group and no other tab, closing the tab in front opens an empty tab; bringing the closed tab back (033) then leaves that empty tab beside it. A row with one tab only has its empty stand-in make way.
- **Many Groups.** 64 are made in a test; nothing measures a row of many Groups and many Sites.

## 10. Done checklist

- [ ] The owner's word on §6.
- [x] The core, the store and the contract.
- [x] The Apple shell.
- [x] The Android shell.
- [x] Part 1 (the core, the store and the contract): `Tools/test.sh all` green, and `android-device`, on 2026-11-22: `lint`, `core` (583, and the strings tool's 24), `unit` (279 on the Mac), `build`, the Mac's UI suite (40), `ios` (261 on the iPhone simulator, one skipped as before), the iPhone's UI suite (30), `android` (386 and 32 on the JVM) and `android-device` (157 on the emulator). No shell asks for a Group or draws one yet: their suites say that nothing else changed.
- [x] Part 2 (the shells): the same again, on 2026-11-22: `lint`, `core` (583, and the strings tool's 24), `unit` (286 on the Mac), `build`, the Mac's UI suite (42), `ios` (268 on the iPhone simulator, one skipped as before), the iPhone's UI suite (32), `android` (397 and 32 on the JVM) and `android-device` (158 on the emulator).

# 041 — Groups: Sites in folders (W2)

| | |
|---|---|
| **Status** | **Designed; nothing built.** The choices of §6 are put to the owner |
| **Phase** | W2 of [000](000-master-plan.md) §9: "Spaces, Groups, Sites; … the tree with drag and drop". This is the tree, without drag and drop |
| **Contract** | To be 1.31 (§5) |
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

**A row is a tree, one level deep.** A Space's row is: its Sites and Groups, in an order of their own; in each Group, its Sites, in theirs; and then the tabs that are no Sites. A Site is in one Group or in none.

**Making and naming.** A Group is made in the row a window shows (`MakeGroup`), with a name, at the end of the Sites and Groups. It is renamed (`RenameGroup`) and given a colour or none (`ColourGroup`; the colours are a Space's, 025). A name is what a Space's name may be (024): trimmed, not empty, not too long.

**Putting in and taking out.** `MoveToGroup` puts a tab in a Group, at its end: a tab that is no Site is kept first (039), and one with no page to keep is refused. With no Group named, a Site is taken out, to the end of the Sites and Groups. A private window has no Groups, as it has no Sites.

**Shut and open.** A Group is shut or opened (`ShutGroup`). Shut, its Sites are not in the row a shell is told to draw, but for the one in front: a tab that is in front is always shown. Shutting a Group changes no tab: none is closed, none put to sleep, none woken (D3). A step along the row (035) does not go into a shut Group; a tab of one that is brought in front some other way (chosen while typing, 038) is shown, and the Group stays shut.

**Removing.** `RemoveGroup` takes a Group away and leaves its Sites where it was, as Sites with no Group (D4).

**Across a quit.** Groups come back with their names, colours, order, what is in them, and whether they were shut.

**Why the core.** Where a Site is, which Groups there are and what a shut Group hides are the model that a sidebar and a strip must both draw, and that sync will carry.

## 4. Where it is kept

The table the plan names, `node` (000 §5.6): a row for each Group of a Space, with its name, colour, place and whether it is shut (schema 11). A saved tab gains the Group it is in. A Site keeps its home and name where 039 put them: moving them into `node` is for the day a Site can be without a tab.

## 5. The contract (to be 1.31)

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

message WindowSnapshot { … repeated GroupSummary groups = N; }   // in the row's order
message TabSummary     { … uint64 group_id = 9; }                // 0: in no Group
```

A shell draws the row from `tabs`, in order, with each Group's own mark where its first tab would be: `groups` says where the Groups are among the Sites by what `tabs` holds, and a Group with no tab to show (shut, or empty) is drawn from `groups` alone, after the Group or Site before it.

Codes: `group.unknown`; `group.name_empty` (as `space.name_empty`); `group.colour_unknown`; `tab.no_page`, `tab.unknown`, `window.private`, `window.unknown` as elsewhere.

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

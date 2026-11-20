# 024 — Several Spaces (W2)

| | |
|---|---|
| **Status** | Designed. Nothing is built yet: there is one Space, which the app makes itself |
| **Phase** | W2 of [000](000-master-plan.md) §9: "Spaces with their login sets … shared logins". The first of W2: the model has several Spaces, and a window shows one of them. Colours and icons, the tree of Groups and Sites, Favourites, both layouts, several windows and private windows are W2's later features |
| **Contract** | 1.18: `QuerySpaces`, `SpacesPage`, `SpaceSummary`, `CreateSpace`, `SpaceCreated`, `RenameSpace`, `SwitchSpace`; `WindowSnapshot.space_id` |
| **Builds on** | [001](001-foundations.md): the store's `space` and `login_set`, many Spaces to one login set from the first schema (000 WT-D9). [005](005-tabs.md): a window's tabs. [009](009-session-restore.md): the tabs come back |
| **Last updated** | 2026-11-20 |

## 1. User story

I keep work and home apart. I make a Space and call it "Work". It has its own logins: signed in to the same site, I am somebody else there than at home. Or, for a second project at the same job, I make a Space that shares Work's logins.

My window shows one Space at a time: its tabs. I switch the window to the other Space and see that one's tabs; when I come back, the tabs I left are as I left them, and the one that was in front is in front.

## 2. What is in it, and what is not

**In:** the core knows several Spaces, in an order; a Space is made with a name, and with logins of its own or another Space's; it is renamed; a window has the Space it shows, and is switched to another; a tab belongs to the Space it was opened in, and its pages run in that Space's logins; each Space's tabs come back when the app starts again, and the window shows the Space it showed. The shells get a way to switch, and to make a Space.

**Not in** (each its own feature, later in W2):

- **A Space's colour and icon**, and its defaults (engine, Guard, search, tidying).
- **Deleting a Space**, with what becomes of its tabs and its logins; **clearing a Space's data**.
- **Moving a tab to another Space.** With other logins it is another page.
- **Groups, Sites, Favourites, temporary tabs**: the tree. Until then a Space's tabs are a row, as now.
- **Several windows**, each with its own Space; **private windows**.
- **Putting the Spaces in another order.**

## 3. The rule

**A Space** has a name, a place in the order, and a login set. The Space the app made itself has no name of its own: a shell shows its own word for it ("Home"), until the person names it.

**Making one** (`CreateSpace`): with a name, and either logins of its own (a new login set, which is the default) or the logins of a Space there is. It is put after the others. A name is what is left of what was typed when the white space at its ends is gone; it may not be empty, and is cut at 60 characters. Two Spaces may have the same name. There may be 32 Spaces: one more is refused (`space.too_many`). A Space to share logins with that is not there is refused (`space.unknown`), and nothing is made.

**Sharing is chosen when a Space is made**, and not changed after: changing a Space's logins is moving everything in it to other logins, which is not this feature.

**Renaming** (`RenameSpace`) gives a Space a name, by the same rule for names. The app's own Space can be named like any other.

**A window shows one Space.** It starts with the Space it showed when the app last ran, or the first. Its snapshot says which (`space_id`), and its tabs are that Space's tabs.

**Switching** (`SwitchSpace`) changes the Space a window shows:

- the tabs of the Space that is left stay as they are: their pages go on, and nothing is asked of their web views. (Putting them to sleep is W3.) Which of them was in front is remembered;
- the window then shows the tabs of the other Space, with the one in front that was in front when it was last shown. A Space with no tabs is given one empty tab, as a new window is;
- switching to the Space the window shows changes nothing. A Space that is not there is refused (`space.unknown`).

**A tab belongs to the Space it was opened in**: the window's Space when the person opened it, and the asking tab's when a page or a menu opened it (a link in a new tab, a window a page asked for, a search for selected words). Its web view is made in that Space's login set, as every web view is made in a login set already.

**What is kept per login set stays so**: history, downloads, what sites were allowed. The Library shows those of the login set of the Space the window shows; two Spaces that share logins share them.

**The tabs come back per Space** (009): each Space's tabs are saved as a window's are now, and given back when the Space is first shown again. Only the Space a window shows has its tabs made when the app starts: the others' are made when they are switched to.

**What waits for a tab's answer waits with it.** A prompt of a tab in a Space that is not shown is still that tab's, and is shown when the tab is (014).

## 4. The contract (1.18)

Added; nothing is changed or removed.

```proto
// browser.proto
message QuerySpaces {}                      // answered with SpacesPage

message SpacesPage {
  repeated SpaceSummary spaces = 1;         // in their order
}

message SpaceSummary {
  uint64 space_id = 1;
  string name = 2;                          // redacted. Empty: the app's own Space, unnamed
  // Spaces with the same number here share their logins. It is a number of
  // the core's own for this run, and says nothing else of the login set.
  uint32 logins = 3;
}

message CreateSpace {                       // answered with SpaceCreated, or an error
  string name = 1;                          // redacted
  uint64 share_logins_with = 2;             // a Space's id; 0 for logins of its own
}

message SpaceCreated {
  uint64 space_id = 1;
}

message RenameSpace {                       // answered with Ack, or an error
  uint64 space_id = 1;
  string name = 2;                          // redacted
}

message SwitchSpace {                       // answered with Ack, "window.unknown" or "space.unknown"
  uint64 window_id = 1;
  uint64 space_id = 2;
}

message WindowSnapshot {
  ...
  uint64 space_id = N;                      // the Space the window shows
}
```

Errors: `space.unknown`, `space.too_many`, `space.name_empty`.

A window's snapshot is published again when its Space changes; a shell that shows the list of Spaces asks for it when it needs it, and again after it has made or renamed one.

## 5. The store

The first schema has what this needs: `space (id, name, position, login_set_id)` and `login_set`. New here: listing the Spaces in order; making one, with a new login set or another's; renaming one; and, with the session (009), which Space the window showed.

## 6. The shells

- **A way to switch**: where the layout has it (000 §6.1). With the tab strip, which is the layout there is: a chip at the leading end of the strip that names the Space the window shows, and opens the list of Spaces. On a phone, the same chip.
- **The list**: the Spaces in their order, the one shown marked; a press switches. At its end, "New Space…".
- **Making one**: its name, and a choice between logins of its own and those of a Space there is, which says what that means in a sentence.

Their designs are written with each shell's part.

## 7. Tests

| Where | What |
|---|---|
| Store | Spaces are listed in their order; one is made after the others, with logins of its own or another's; it is renamed; all of it is still there when the store is opened again |
| Core (through `FakeShell`) | A Space is made, and listed, with the name as tidied; one too many, an empty name and logins of a Space that is not there are refused, and nothing is made. A window is switched: the other Space's tabs are shown, and back again the tabs that were left are as they were, with the same one in front; nothing was asked of their web views meanwhile. A tab opened in a Space is in it and in no other, and its web view is made in that Space's login set: of its own, or the one it shares. A link's tab, a page's window and a search open in the asking tab's Space. The Library is the shown Space's login set's. The tabs of each Space come back, and the window shows the Space it showed |
| The shells | Written with each |

## 8. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | A window shows one Space; a tab is in one Space | 000 §5.6. What is open in "Work" is not in the way at home |
| D2 | Logins of its own by default; sharing is chosen at making, and stays | 000 WT-D9. Keeping apart is what a Space is for; sharing is the exception, said once |
| D3 | The tabs of a Space that is not shown stay awake, for now | Sleeping is W3, with its budgets. Until then nothing a person left is lost or reloaded |
| D4 | Only the shown Space's tabs are made at start | A start that makes every Space's web views is slower for each Space a person has |
| D5 | 32 Spaces, and 60 characters to a name | Limits a person does not meet, that a list and a chip can still show |

## 9. What is not verified

Written when built.

## 10. Done checklist

- [ ] The store: listing, making, renaming; the window's Space with the session.
- [ ] Contract 1.18 and its version in every place it is written.
- [ ] The core: the rule of §3.
- [ ] The Mac.
- [ ] Android.
- [ ] iPhone.
- [ ] `Tools/test.sh all` green, and `android-device`.
- [ ] Windows shell.

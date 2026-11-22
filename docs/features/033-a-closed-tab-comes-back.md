# 033 — A closed tab comes back: undo, and the journal's first form (W1, W2)

| | |
|---|---|
| **Status** | Built and tested in the store, the core and every shell there is: the journal, undo for a closed tab, and a way to ask on each system. The Mac: ⇧⌘T, and a right-click in the strip of tabs. The iPhone, the iPad and Android: the button for a new tab, held (the owner's decision, §6). Not run on a phone, nor with a screen reader (§9) |
| **Phase** | W1 of [000](000-master-plan.md) §9: "the store and journal", the journal's first form. W2: "undo", for the one organising action there is to undo so far, closing a tab (000 §6.2: "Undo for every organising action, including closing a tab") |
| **Contract** | 1.24: `Undo`; `WindowSnapshot.undo` |
| **Builds on** | [005](005-tabs.md): a tab closed. [009](009-session-restore.md): a tab kept by its address and its title, and brought back asleep. [024](024-several-spaces.md): a window shows one Space's tabs. [008](008-history.md): forgetting |
| **Last updated** | 2026-11-21 |

## 1. User story

I close a tab I did not mean to close. I ask for it back, and it is where it was, in front, with its page loading. I ask again, and the one I closed before it is back too. It still works after I have quit and started again.

## 2. What is in it, and what is not

**In:** the store has a journal; closing a tab is written in it before the tab is closed; the last thing written for the Space a window shows can be undone, which for a closed tab brings it back; the window is told whether there is something to undo, and what kind of thing.

**Not in:**

- **Any other action.** Making, naming and colouring a Space are not undone yet: each joins the journal with its own feature. There is no redo.
- **What the page was besides its address**: its way back and forward, what was typed in it, where it was scrolled to. The page is loaded again. (What an engine can keep of a page is W3's, with sleeping tabs.)
- **A list of closed tabs to choose from.** History has every place; this is the one step back.
- **A tab a page closed itself.** The person did not close it.
- **A private Space's tabs.** Nothing of a private Space is written (000 §5.6).
- **Sync.** W9 reads the journal; what of it travels, and in what form, is W9's to say.

## 3. The rule

**The journal** is a table of the store: what the person did to organise their browsing, each entry with when, what kind of thing it was, the Space it was done in, and what is needed to undo it. It is written **before** the thing is done.

**Closing a tab is journalled** when the person closes it and it has an address of the web: the Space, the place it had in the row, its address and its title (as the session keeps them, 009). An empty tab is not: there is nothing to bring back. If the journal cannot be written, the tab is closed all the same: a tab that will not close is worse than one that cannot be brought back.

**Undo** (`Undo`, for a window) undoes the newest entry of the Space the window shows:

- for a closed tab: a tab opens at the place the closed one had (at the end, when the row is shorter now), in front, and loads the address, as the person's own: the field says where it goes from the start, and the mark waits for the page ([029](029-the-mark-waits-for-the-page.md)). Until its page says a title, the tab has the one it had;
- **when the row's only tab is the empty one that was put there** when the row's last tab was closed (005), the tab that comes back takes its place: bringing it back takes the empty one away again. An empty tab the person opened themselves stays, and so does that one once they have gone somewhere in it;
- the entry is then gone: what was undone is not undone twice;
- nothing to undo is refused (`undo.nothing`), and so is a window the core does not have (`window.unknown`).

**Each Space has its own.** Undo in the Space a window shows undoes what was last done in that Space, not what was last done anywhere.

**The window is told** (`WindowSnapshot.undo`) what an undo would undo now, by its kind: `tab.closed`, or nothing. A shell offers "Reopen Closed Tab" by it, and offers nothing when there is nothing.

**It outlives a quit.** The journal is in the store: a tab closed before the app was closed can be brought back after.

**How much is kept.** The fifty newest closed tabs of each Space. An older one is in history, where every place is.

**Forgetting.** A closed tab is a place the person visited. When history is forgotten for a Space's logins (008), whatever is named to be forgotten, every closed tab of the Spaces with those logins is forgotten with it: what the person asked to have forgotten does not come back by undo. (More is forgotten than was named; that is the safe side.)

## 4. The contract (1.24)

Added; nothing is changed or removed.

```proto
// browser.proto
// Undoes the last organising action of the Space a window shows (033): so
// far, brings back the tab that was last closed there. Answered with
// TabOpened for a tab brought back; "window.unknown"; or "undo.nothing".
message Undo {
  uint64 window_id = 1;
}

message WindowSnapshot {
  // ...
  // What Undo would undo now in the Space the window shows, by its kind:
  // "tab.closed", or empty when there is nothing to undo.
  string undo = 8;
}
```

## 5. The store

A table, `journal` (the store's ninth version), with the columns that closing a tab needs; each later kind of entry adds what it needs.

| Column | What |
|---|---|
| `id` | Never used twice: the newest is the largest |
| `at` | When, in milliseconds |
| `kind` | `tab.closed` |
| `space_id` | The Space it was done in |
| `position`, `url`, `title` | For a closed tab: its place in the row, its address, its title |

An entry that is undone is deleted, and so are a Space's entries beyond its fifty newest.

## 6. The shells

**The Apple shell's model** (`UndoModel.swift`) keeps what the core says an undo would undo, offers a closed tab back only while that is `tab.closed` (a kind this shell does not know is not offered as a closed tab), and asks the core to undo, with the window.

**The Mac**: "Reopen Closed Tab", ⇧⌘T, as every browser has it; there while the window is told there is a closed tab to bring back, and doing nothing otherwise. It is in the File menu, with the window's other commands ([034](034-the-macs-menus.md)). An iPad with a keyboard has the same key.

**What the Mac's app test found.** The rule as first built took away any empty tab that was the row's only tab when a closed tab came back. In the app: a new tab opened with ⌘T, the page's tab closed, ⇧⌘T; the page's tab came back and the new tab the person had just opened was gone. Only the empty tab that the core itself puts in the place of a row's last tab makes way now (§3).

**The iPhone, the iPad and Android: the button for a new tab, held.** A finger held on the button that opens a new tab brings up a menu with "Reopen Closed Tab". Chosen, the core is asked to undo.

> **The owner's decision (2026-11-21, WT-D25): the held button is kept.** It was built as provisional and put to them. The other course was a line across the window when a tab is closed ("Tab closed", with "Undo"), as one browser has it. The held button was taken because it adds nothing to a strip that is already full on a phone ([024](024-several-spaces.md) found how little room there is), and because a line that goes away by itself after some seconds is one a person who reads slowly, or by ear, may miss.

- **Nothing is offered when there is nothing to bring back**: the button held is then only a button, and a finger held on it and lifted is a press of it. An item that does nothing is not shown ([022](022-picture-menu.md) D5).
- **A screen reader is told**: the button says it has the action, by name, while there is a closed tab to bring back.
- **Android** (`NewTabButton` in `BrowserWindow.kt`): the button is the app's own, with a press and a hold, and the menu is Compose's own over the window.
- **The iPhone and the iPad** (`TabStrip.swift`): the system's own menu for a held control.

**The Mac, with the mouse: a right-click in the strip of tabs**, beside the tabs, brings up the same menu. Not on the button for a new tab, as was first built, nor on a tab: on the Mac a button takes a right-click for itself, and no menu comes of it, the button's own or the strip's. (Found by the app's test: with the menu on the button nothing came, however it was put there; a menu on the strip came when the strip was right-clicked, and not when a button in it was.) A window whose strip is full of tabs has nowhere to right-click; ⇧⌘T is always there.

**Where the menu is put on and taken off.** With nothing to bring back there is no menu at all, on any system: it is not left there with nothing in it.

## 7. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | Undo is one request, and the core says what it undoes | The plan's undo is of every organising action; a shell that asked for "the closed tab back" would need a new request for each kind. The window is told the kind, for the words |
| D2 | Per Space | A window shows one Space. Undoing something in a Space that is not on the screen would change what the person cannot see |
| D3 | The page is loaded again | What the address is, the core knows. What else a page was is the engine's to keep, and that is W3 |
| D4 | The journal has the columns its one kind needs | As the store's other tables began (009, 012). A general shape for entries is better made when there is a second kind to make it for |
| D5 | Forgetting history forgets the closed tabs, all of them for those logins | The person asked for something not to be kept. Working out which closed tabs a time or one visit covers gains little, and getting it wrong keeps what was to be forgotten |
| D6 | On a phone, the button for a new tab, held (the owner's, WT-D25) | §6: no new chrome on a narrow strip, and nothing that is there only for a moment |
| D7 | On the Mac the menu is the strip's, not the button's | §6: a button on the Mac gives no menu for a right-click. A right-click in the strip of tabs is where other browsers on the Mac have it |

## 8. Tests

| Where | What |
|---|---|
| Store (`wt-store`, 6 more: `journal.rs`) | A closed tab is written, and taken newest first, once; each Space has its own, and a Space there is not has none and is written nothing; only the fifty newest of a Space are kept, and tidying one Space leaves another's; forgetting takes every Space of a login set and no other's, and leaves nothing of them in the table; closed tabs outlive a restart; a store from before the journal gains it and keeps its tabs |
| Core (`wt-app/tests/undo.rs`, 8, through `FakeShell`) | A closed tab comes back at its place, in front, with the title it had and its address in the field from the start, no mark until the page arrives, loaded once; there is then nothing more to undo. Closed twice, they come back newest first, each where it was, and at the end when the row is shorter. The row's only tab closed and brought back leaves no empty tab; an empty tab the person opened stays, and so does the stand-in once they have gone somewhere in it. An empty tab closed is nothing to undo; nothing to undo is refused, and a window there is not is refused first. Each Space has its own closed tabs. Forgetting history forgets them, whatever was named: everything, since a time, or one visit that is not even there. Closed before a quit, a tab comes back after it (over a store on disk, started three times), and the window is told so from its first snapshot. A tab a page closed itself is not brought back. Eighteen parts of the rule checked by breaking them: all caught |
| The Apple shell's model (`UndoModelTests`, 2, on the Mac and the iPhone simulator) | A closed tab is offered back only while the core says there is one, and asked for with the window, with one request; a kind of undo this shell does not know is not offered as a closed tab. Four parts checked by breaking them: all caught |
| The Mac's and the iPhone's apps (`UndoTests`, 1 on each, fake mode: the real core and real WebKit) | A page's tab, a new tab opened beside it, the page's tab closed with its button. The menu is asked for (a right-click in the strip beside the tabs on the Mac; a finger held on the button for a new tab on the iPhone): "Reopen Closed Tab" is there, and asking opened no tab. Chosen: the tab is back where it was, the first of the two, in front, its page loaded again, the field saying where it is. Asked for again, there is no menu; on the Mac there was none before anything was closed either. With the menu always there, both fail |
| Android's window (`UndoTest`, 3, on the JVM, over a scripted core) | The model: a closed tab is offered back only while the core says there is one; asked for with the window, in one request; a kind of undo this shell does not know is not offered. The button for a new tab: a screen reader is told what holding it does, by name; held, the menu comes, nothing is asked yet and no tab is opened; chosen, the core is asked to undo and the menu goes; a press is still a new tab. With nothing to bring back, the button cannot be held for anything and a screen reader is told of nothing; and the menu goes when the core says there is nothing more. Eleven parts checked by breaking them: all caught |
| Android's app, on the emulator (`AppUndoTest`, 1, fake mode: the real core and the real WebView) | A page's tab closed with its button, an empty one left; the button for a new tab held: the menu, and no new tab; chosen: two tabs, the first in front, its page loaded again and its address in the field. Passed three runs one after another, each in a process of its own |
| By hand (2026-11-21, the emulator, the app in fake mode, an address on the device's own loopback that refuses) | The window's only tab closed; the button held: the menu at the button, one line, well inside a phone's width. Chosen: the tab back with its address, and the empty tab that had taken its place gone. Held again with nothing more to bring back: no menu, and a new tab (the hold is then a press). The app's data was cleared afterwards |
| The Mac's app (`LaunchTests`, 1, fake mode: the real core and real WebKit) | ⇧⌘T with nothing closed does nothing. A page's tab, a new tab opened beside it, the page's tab closed: ⇧⌘T brings it back where it was, the first of the two, in front, its page loaded again (the site was asked for it a second time), the field saying where it is; the new tab is still there. Asked again, nothing comes. Passed three runs one after another. Its first run found the fault of §6 |

## 9. What is not verified

- **A phone.** Android ran on an emulator; the iPhone on the simulator.
- **A screen reader.** The button says it has the action, in code, on every system (Android's is tested as what the system is told; Apple's is not tested at all: its tests cannot ask what actions a control has). Nobody listened to one.
- **The iPad.** It has the iPhone's code; no test ran at its size.
- **Why a button on the Mac gives no menu for a right-click.** Seen, in the app's test, three ways; not explained. The Mac's menu is on the strip because that is where one came.
- **A strip full of tabs on the Mac**: nowhere beside the tabs to right-click. ⇧⌘T is there, and the File menu has the command ([034](034-the-macs-menus.md)).
- **A journal that cannot be written** (a disk that is full). The rule is that the tab closes all the same and cannot be brought back; no test has a store that refuses.
- **Very many closed tabs.** Fifty are kept for a Space; fifty-three were written in a test.
- **What sync will want of the journal** (W9). Its columns are what one kind of entry needs.
- **A private Space.** There is none to make yet (private windows are W2's); when there is, nothing of it may be written here.

## 10. Done checklist

- [x] Contract 1.24 and its version in every place it is written.
- [x] The store's journal (its ninth version).
- [x] The core: the rule of §3.
- [x] The Mac: the command, in the app.
- [x] The iPhone, the iPad and Android: a way to ask (the owner's, §6); the Mac: a right-click in the strip.
- [x] `Tools/test.sh all` green, and `android-device`, on 2026-11-21: `lint`, `core` (519), `unit` (267 on the Mac), `build`, the Mac's UI suite (30), `ios` (249 on the iPhone simulator, one skipped as before), the iPhone's UI suite (26), `android` (351 and 32 on the JVM) and `android-device` (147 on the emulator).
- [x] `Tools/test.sh all` green, and `android-device`, with the shells' ways to ask, on 2026-11-21: `lint`, `core` (519), `unit` (267 on the Mac), `build`, the Mac's UI suite (31), `ios` (249 on the iPhone simulator, one skipped as before), the iPhone's UI suite (27), `android` (354 and 32 on the JVM) and `android-device` (148 on the emulator).

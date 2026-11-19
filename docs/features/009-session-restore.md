# 009 — The tabs come back (W1)

| | |
|---|---|
| **Status** | Built and tested in the store and the core, and through each shell's client with the real core. The shells needed no change. The apps themselves were not quit and started again by a test (§7) |
| **Phase** | W1 of [000](000-master-plan.md) §9: "session restore after a crash". The store's `tab` table (000 §5.6), in its first form |
| **Contract** | No change: a restored tab is a tab |
| **Builds on** | [005](005-tabs.md): tabs; [008](008-history.md): the store written while browsing |
| **Last updated** | 2026-11-19 |

## 1. User story

I quit WebTiger, or it crashes, or the battery dies. When I open it again my tabs are there, in the order I had them, with the one I was looking at in front. Only that one loads; the others load when I go to them.

Until now a new window was always one empty tab: everything open was lost with the app.

## 2. What is in it, and what is not

**In:** the tabs of the window are written to the store as they change, and the first window opened after a start is given them back.

**Not in, each with where it goes:**

- **Several windows.** The session is one window's: the first opened in a run. Another window opened beside it starts empty and is not saved. Windows in the plural are W2.
- **Each tab's back and forward, its scroll position and what was typed in a form.** The engines can save those (001 §11.3, Q5), through a host request that does not exist yet. A restored tab loads its address afresh. That is W3's sleeping tabs, which this is the first step towards.
- **A way to say "do not bring them back".** With settings.
- **Private windows** (W2): nothing of one is ever written.

## 3. What is written, and when

For each tab of the session's window, in order: its address, its title, and whether it is the one in front. Written to the store whenever that changes, in one transaction, at the moment the window's snapshots are published: so what the store has is what the person last saw, and a crash loses nothing that was on screen for longer than that.

| | What is written |
|---|---|
| A tab with a page | The page's address and title |
| A tab on its way somewhere (a load that has not committed) | Where it was sent: it goes there again |
| A tab whose page failed to load | The address it failed at: with the network back, it loads |
| An empty tab, or a page that is not the web's (`about:blank`) | An empty tab |
| A window closed, or the app quit | Nothing changes: closing the window is not closing its tabs. They are there the next time a window is opened, in the same run or the next |
| A tab closed | It is gone from the session, and its address from the store |

- **A title is written once a page:** the first the page says. A page that changes its title ten times a second does not write to the disk ten times a second (as 008 §3). The title is only what the strip shows until the page is loaded again.
- **Only `http` and `https` addresses** are written, as in history.

## 4. What comes back

The first window opened after a start has the tabs the store has, in their order, with the same one in front. A store with none gives one empty tab, as before.

- **The session is one window's**: the first opened, and, once that is closed, the next one opened. A window opened beside it starts with one empty tab and is not kept.
- **A restored tab is asleep until it comes in front.** It has its address and its title, and no web view: it costs the shell nothing. The strip shows it like any other tab.
- **The tab in front is never asleep.** It is loaded as the window opens; and whenever another restored tab comes in front (it is chosen, or the tab in front of it is closed), that one is loaded then, once.
- **A restored tab is a tab.** Typing an address into one that has not loaded yet loads what was typed. Closing one asks the shell for nothing.
- **Loading a restored page is a visit** (008 §3): the page was seen again.
- In fake mode the store is in memory, so nothing is there to come back, and a window opens on its start address as before.

## 5. The store

Migration 3 adds `tab`, the table 000 §5.6 names, with the columns this needs; the rest of that table's (the Site a tab belongs to, its saved state, its snapshot) arrive with what uses them.

```sql
CREATE TABLE tab (
    space_id INTEGER NOT NULL REFERENCES space (id),
    position INTEGER NOT NULL,
    url TEXT NOT NULL,                 -- '' for an empty tab
    title TEXT NOT NULL,
    front INTEGER NOT NULL CHECK (front IN (0, 1)),
    PRIMARY KEY (space_id, position)
) STRICT, WITHOUT ROWID;
```

A Space's tabs are replaced as a whole, in one transaction: they are few, and a half-written session is worse than an old one.

## 6. Tests

| Where | What |
|---|---|
| Store (`wt-store`, 5) | The migration from version 2, which keeps its history. Tabs are saved and read back in order, with which is in front; saving again replaces them; no tabs is no rows; another Space's are not touched, and a Space the store does not have can have none; they survive the store being dropped without a word |
| Core (`wt-app/tests/session.rs`, 16 through `FakeShell` over a store on disk, started twice) | Each row of §3's table, the store itself read for what it holds. The tabs come back in order with the same one in front; only it is loaded; another is loaded when it comes in front, by being chosen and by the one in front being closed, and once. Typing into a tab that has not loaded; closing one. A crash (no window closed, nothing said) is how every one of these tests ends its first run. A window closed and opened again has its tabs. A second window starts empty and is not the session. The title kept is the first a page said. Loading a page that came back is a visit. Fake mode is as it was. Six of the rules checked by breaking them |
| Each shell's client, with the real core | Swift (`NativeCoreTests`) and Kotlin (`NativeCoreTest`, through JNI): a core is stopped and another started on the same folder; the tabs are there in order, the one that was in front is in front, and the host was asked for one web view and one load |

## 7. What is not verified

- **The apps, quit and started again.** The UI suites run in fake mode, which keeps no session, and no test starts an app twice on a store of its own: that needs a way to give an app a data folder for a test, which it does not have. What was tested is everything beneath the window: the core, and each shell's client over it. The windows draw whatever tabs the core says, as they have since 005.
- **A real quit on a phone**, where the system ends the app without a word: it is what the tests' "dropped without a word" stands for, and was not tried on a device.
- **Many tabs.** The session is rewritten whole when it changes; with hundreds of tabs that was not measured.

## 8. Done checklist

- [x] The store: migration 3; saving and reading a Space's tabs. 5 tests.
- [x] The core: writing the session as it changes; giving it back; waking a tab that comes in front.
- [x] Core tests through `FakeShell`, over a store on disk: 16.
- [x] Through each shell's client with the real core: Swift and Kotlin.
- [ ] The apps: quit and start again, on the Mac, the iPhone simulator and the Android emulator (§7).
- [x] `Tools/test.sh all` green, and `android-device`.

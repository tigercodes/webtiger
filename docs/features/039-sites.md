# 039 — Sites: a tab that is kept (W2)

| | |
|---|---|
| **Status** | **Designed; nothing built.** The choices of §6 are put to the owner before any code |
| **Phase** | W2 of [000](000-master-plan.md) §9: "Spaces, Groups, Sites". This is Sites, without Groups: the first step of the tree |
| **Contract** | To be 1.29 (§5) |
| **Builds on** | [005](005-tabs.md): tabs and their row. [009](009-session-restore.md): tabs that come back asleep. [024](024-several-spaces.md): a row for each Space. [033](033-a-closed-tab-comes-back.md): the journal |
| **Last updated** | 2026-11-22 |

## 1. User story

There are pages I am always in: my mail, my calendar, the board my team works from. I keep such a tab. It becomes a **Site**: it sits at the start of the row, before the tabs that come and go, and it is there tomorrow. Closing it does not lose it: it goes to sleep and its place stays; a press wakes it. It knows where its home is: however far I wandered in it, it comes back as the place I kept. When I no longer want it, I stop keeping it and it is a tab like any other.

## 2. What is in it, and what is not

**In:** a tab can be kept, and let go again; a kept tab is a Site: first in the row, asleep and not gone when it is closed, home again when it wakes; kept across a quit; told to the shells, which mark it and offer the two commands.

**Not in, each with where it goes:**

- **Groups** (folders of Sites, the tree, drag and drop): the next document. A Site has no parent yet.
- **The sidebar.** The strip is the only layout so far (000 §6.1); a Site is a tab of the strip with a mark.
- **Favourites** (a row of tiles): after the tree.
- **Temporary tabs tidied into an archive** after a time (000 §5.6): W3. Here a tab that is not a Site is simply a tab, as it has been.
- **A Site's own engine and Guard settings** (W4, W6), **"keep awake"** (W3), **its saved cookies across a quit** (000 §4.7).
- **Sending an awake Site back home** without closing it, and saying that it has wandered: a second part (§7).
- **Sync** of Sites (W9); **bringing in the Sites of an import**: the Arc importer's file already has each Space's pinned tree as Sites and Groups ([002](002-arc-import.md)), and nothing reads it into a window yet.

## 3. The rule

**Keeping.** A tab that shows a page of the web's can be kept (`KeepTab`). It becomes a Site: its **home** is the address the page has now, its **name** the title it has now. It moves to the end of the Sites of its row: a row is its Sites, in their order, and then its other tabs, in theirs. A tab with no page, and a tab in a private window ([037](037-private-windows.md): nothing of one is kept), cannot be kept.

**Letting go.** A Site can be let go (`LetGoSite`): it is a tab like any other again, the first of the row's other tabs, where it is and as it is.

**Closing a Site puts it to sleep.** Its web view is destroyed and its page is gone; its place in the row stays, with its name; it is at its home again (D1). Nothing is written to the journal of closed tabs: nothing was closed that could be brought back, it is still there. When it was in front, the tab that comes forward is the nearest one that is awake or is no Site (the one after, else the one before): closing a Site should not wake another (D2). When there is none, an empty tab is opened at the end and comes in front, as when a row's last tab is closed (005).

**Waking.** A Site that is asleep is woken by being brought in front (a press, a key, a step along the row, being chosen from what is offered while typing): its home is loaded. It is what already happens to a tab that came back asleep after a quit (009).

**Across a quit.** Sites come back as Sites, in their order, asleep at their homes, except one that was in front, which is loaded where it was as any tab in front is. The other tabs come back as they do now.

**What is offered while typing** ([038](038-an-open-tab-is-offered.md)): a Site is an open tab, awake or asleep, and is offered as one, by its name and its home when it is asleep.

**Why the core.** Which tabs are Sites, where they sit, what closing one means and which tab comes forward are rules about browsing, the same on every system; and they are what sync will carry.

## 4. Where it is kept

The session's table of tabs (009) gains two columns: whether a tab is kept, and its home (schema 10). The plan's tree (`node`: a Group or a Site, its parent and position; 000 §5.6) arrives with Groups, when there is a parent to record; until then a Site's place is its place in its row.

## 5. The contract (to be 1.29)

Added; nothing is changed or removed.

```proto
// Keeps a tab: it becomes a Site (039), whose home is the page it shows.
// Answered with Ack, also when it is one already; "tab.unknown";
// "tab.no_page" for a tab with no page of the web's; "window.private".
message KeepTab { uint64 tab_id = 1; }

// Lets a Site go: it is a tab like any other again. Answered with Ack, also
// when it is no Site; or "tab.unknown".
message LetGoSite { uint64 tab_id = 1; }

message TabSummary {
  …
  // The tab is a Site: kept, first in the row, asleep and not gone when closed.
  bool kept = 5;
  // It has no page now: a Site that was closed, or a tab that came back and
  // has not been in front since.
  bool asleep = 6;
}

message TabView {
  …
  bool kept = N;   // the tab in front is a Site
}
```

`CloseTab` and `ActivateTab` mean for a Site what §3 says; a shell asks as before.

## 6. Decisions

**All of these are put to the owner; none is built.** The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | A closed Site sleeps **at its home** | A Site is the place that was kept: "my mail", not "the message I happened to be reading". It is the simplest thing to say, and it is what 000 §5.6 says of a Site ("it can wander from its home address and be sent back") | It sleeps **where it was**, and going home is a command of its own. Or Arc's way: the first close sends it home, the second puts it to sleep |
| D2 | Closing a Site brings forward the nearest tab that is awake or is no Site; with none, an empty tab | Closing one Site should not load another | The next tab in the row, whatever it is, as for any tab |
| D3 | The key is ⌘D on a Mac and Ctrl+D with a keyboard on Android: "Keep as Site", and for a Site "Stop Keeping" | It is the key other browsers give to keeping a page (a bookmark), and Arc gives to pinning | No key yet: the tab's menu only |
| D4 | The words: **Site**; "Keep as Site"; "Stop Keeping"; a sleeping Site is said to be "asleep" to a screen reader | The plan's word (000 §5.6). "Pin" says where a thing is, not what it is for | "Pinned tab", "Pin Tab", "Unpin Tab", as Arc, Chrome and Safari have it |
| D5 | In the strip a Site is a tab with a small mark before its name, and its close button puts it to sleep; asleep, its name is drawn fainter (and said) | The strip is the only layout yet; icons alone need favicons, which are not fetched yet | Icon-only Sites, once there are icons |
| D6 | A Site is of the row of the window it was kept in; another window showing the same Space has its own row, as now | Rows are a window's today (024). One Space's Sites shown in every window that shows the Space is the tree's question, and is asked with the tree | Sites belong to the Space now: every window showing it shows them, and a Site is awake in one |
| D7 | No Sites in a private window | Nothing of one is kept (037) | — |

## 7. In what order

1. **The core and the contract**: the rule of §3, the two columns, through `FakeShell`, over a store on disk for what comes back after a quit.
2. **The shells**: the mark, the two commands (the tab's menu; the Mac's File menu and key; a held tab on a phone), a sleeping Site's look, the words.
3. **Home again without closing**: an awake Site that is not at its home says so, and one command sends it back.

## 8. Tests

To be filled in as built: the core's rule and its mutants; the store's migration; the shells' models; pictures of a row with Sites, awake and asleep; the apps, in fake mode.

## 9. What is not verified

To be filled in as built.

## 10. Done checklist

- [ ] The owner's word on §6.
- [ ] The core, the store and the contract.
- [ ] The Apple shell.
- [ ] The Android shell.
- [ ] `Tools/test.sh all` green, and `android-device`.

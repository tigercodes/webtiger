# 039 — Sites: a tab that is kept (W2)

| | |
|---|---|
| **Status** | **The core, the store and the contract are built and tested** (part 1 of §7), on the choices of §6, which are **the owner's since 2026-11-22** (WT-D29): a tab can be kept and let go, a Site is first in its row, sleeps at its home when closed, wakes when brought in front, and comes back after a quit. **The shells show a Site and offer the command** (part 2, §7a): the Mac, the iPhone simulator and an Android emulator |
| **Phase** | W2 of [000](000-master-plan.md) §9: "Spaces, Groups, Sites". This is Sites, without Groups: the first step of the tree |
| **Contract** | 1.29: `KeepTab`, `LetGoSite`; `TabSummary.kept`, `TabSummary.asleep`, `TabView.kept` |
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

The session's table of tabs (009) gains two columns (schema 10): a Site's home, and its name. A tab with no home is no Site. The name is kept apart from the tab's title because a Site that wandered has another page's title, and is called by its own again when it sleeps. The plan's tree (`node`: a Group or a Site, its parent and position; 000 §5.6) arrives with Groups, when there is a parent to record; until then a Site's place is its place in its row.

## 5. The contract (1.29)

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
  bool kept = 17;   // the tab in front is a Site
}
```

`CloseTab` and `ActivateTab` mean for a Site what §3 says; a shell asks as before.

## 6. Decisions

**The owner's, 2026-11-22 (WT-D29): all seven as recommended.** They were put to the owner with the core built on D1, D2, D6 and D7; D3, D4 and D5 are the shells', and are built next.

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

## 7a. The shells (part 2)

Nothing is decided in a shell: it asks `KeepTab` and `LetGoSite`, and draws what the snapshot says (`kept`, `asleep`).

**The command** is one, with two names: "Keep as Site" for a tab that is none, "Stop Keeping" for a Site. It is for the tab in front, and is grey for a tab with no page.

| Where | How |
|---|---|
| The Mac | File menu, after Reopen Closed Tab; ⌘D |
| The iPhone and the iPad | A tab's own menu (a long press on it); ⌘D with a keyboard |
| Android | A tab's own menu (a long press on it); Ctrl+D with a keyboard ([036](036-androids-keys.md)) |

On the Mac a tab has no menu of its own: a button there takes a right-click for itself (033 found it), so the menu bar and the key are the ways.

**In the strip** a Site has a small mark before its name, and is said to be a Site to a screen reader. Asleep, its name is drawn fainter, and it is said to be asleep. The button at a Site's end puts it to sleep, and says so; a sleeping Site has no such button: there is no page to close.

**The words** (WT-D29): "Keep as Site", "Stop Keeping", "Site", "Asleep", "Put to Sleep".

**As built**, and what building it found:

- **A sleeping name is drawn at seven tenths of its strength**, not in the system's fainter grey: the iPhone's accessibility audit measures that grey as too faint to read at a tab's size. (Android's is the colour Material gives such text, which its own checks pass.)
- **On the Mac, ⌘W closes the window when nothing else would be left to show.** ⌘W on a window's last tab has always closed the window (005). A sleeping Site is no tab to be left with: when the tab in front is the only one that is not a Site asleep, ⌘W closes the window. Without this ⌘W went round for ever: the core opens an empty tab when a row's last Site is closed, and closing that tab woke the Site again. The app's first key test found it, with a Site as the window's only tab. The Sites are kept, and are there when a window next opens. The button at a tab's end is not the key: it asks the core, which puts the Site to sleep and opens the empty tab.
- **A tab with no page has no menu on a phone**, and the menu bar's item is grey for it. A tab whose page never arrived does have the menu; the core refuses to keep it, and nothing is said.

## 8. Tests

| Where | What |
|---|---|
| The store (`wt-store`, 2 more: `session.rs`) | A Site's home and name are written with its tab and read back; a tab that is no Site has neither. A store from before Sites (version 9) opens at this version with its tabs as they were, none a Site, and a Site can then be kept in it. The older-version tests of every table now take the two columns away as well |
| The core (`wt-app/tests/sites.rs`, 9, through `FakeShell`; two over a store on disk) | **Keeping:** the tab in front, kept, is the first of the row and still in front; the next one kept goes after the Sites there are; kept again, a Site is as it was; nothing is loaded or destroyed; the tab in front says whether it is a Site. **What cannot be kept:** an empty tab, a tab whose page never arrived, a page that is not the web's (`tab.no_page`); a tab there is not (`tab.unknown`); a private window's tab (`window.private`). **Closing:** a Site that wandered and is closed has its web view destroyed and stays in the row by the name it was kept with, asleep; the tab after it comes in front; nothing is loaded; no closed tab is offered back; closed again, it is as it was. **Waking:** a press loads its home, not where it had wandered. **What comes forward:** a Site before it that is awake when there is nothing after; a sleeping Site is passed over; with no tab that is awake or is no Site, an empty tab at the end, and nothing loaded; a tab that is no Site further along before a sleeping Site nearer; the one after before the one before. **Letting go:** it is the first of the other tabs, as it is; closed then, it is closed and can be brought back, after the Sites; a sleeping Site let go is a sleeping tab. **Nothing among the Sites:** a tab asked for after the first Site, a link of a Site's page opened in a new tab, a window a Site's page asks for, a closed tab brought back to a place that is now a Site's. **By key and by typing:** a sleeping Site is offered while typing by its name and home, which wakes nothing, and a step along the row wakes it. **After a quit:** Sites are Sites, in their order; one that was not in front is asleep at its home by its name; the one in front is loaded where it was and still knows its home; a Site let go comes back as a tab |
| By breaking it | Twenty-six parts of the rule, each then run against these tests, the store's and the tabs', the undo's and the session's: all caught, two of them only after the tests were given the case (a tab to come forward on either side; a closed tab whose old place is now a Site's) |

| The Apple shell's model and strip (`SiteModelTests`, 3, on the Mac and the iPhone simulator, over a scripted core) | The command does nothing before the window is open and for a tab with no page; with a page it asks the core to keep the tab; for a Site, awake or asleep, it asks to let it go. A tab of the strip is kept or let go by what the core says of it. What the strip says of a tab: nothing, "Site", "Asleep", "Site, Asleep" |
| Android's model, strip and keys (`SitesTest`, 3; `KeysTest`, the same 6 with Ctrl+D in them; Compose on Robolectric over a scripted core) | The same of the model. In the strip: two Sites are marked; each tab says what it is; the button at an awake Site's end says "Put to Sleep", a sleeping Site has none, the others say "Close Tab"; a Site's button asks the core to close it. A tab's own menu, held: a Site's lets it go, a page's keeps it, an empty tab has none, and a press is still a press. Ctrl+D asks for the command, and D alone, with Shift, with Meta, and with Control and Alt asks for nothing |
| Pictures (`window-sites`, light and dark, on the Mac, the iPhone simulator and Robolectric) | A Site awake in front, with its mark; a Site asleep, marked, its name fainter and no button at its end; a tab that is none. Recorded and looked at |
| The Mac's app (`SiteTests`, 2, fake mode: the real core and real WebKit) | From the File menu: kept, the tab says it is a Site and its button says "Put to Sleep"; it wanders; a second tab; closed by its button it stays, by the name it was kept with, asleep, the other tab in front, nothing loaded, and only the other tab has a button; Apple's accessibility audit over that; pressed, it wakes at its home, which is loaded once more; Stop Keeping, and its button closes again. By key: ⌘D keeps; the item is grey for an empty tab; ⌘W puts the Site to sleep and the other tab comes in front; the menu names the command for a Site; ⌘D lets it go; and ⌘W on the last tab there is to show beside a sleeping Site closes the window |
| The iPhone's app (`SiteTests`, 1) | The same as the Mac's first, from the tab's own menu (a long press) |
| Android's app on the emulator (`AppSitesTest`, 1, fake mode: the real core and the real WebView) | The same, from the tab's own menu; and Ctrl+D, sent as a keyboard sends it, lets the tab in front go and keeps it again |
| The shells, by breaking them | Eleven parts (Android: a Site asked to be kept and a tab let go, Ctrl+D asking nothing, a tab saying nothing of itself, a sleeping Site with a button, an empty tab with a menu, a Site's button saying Close Tab, no mark; Apple: a tab of the strip always kept, a sleeping Site in front that cannot be let go, the strip not saying asleep, not saying Site), each run against the shells' model and strip tests: all caught. The apps' own tests were not run against them |

**Found on the way, in 038:** a tab that came back asleep after a quit was not offered while typing, though 038 said it was. The core asked a tab for the page that owns its web view, and a sleeping tab has no web view. It is asked for the page it is at, awake or asleep; a test over a store on disk has it (`a_tab_that_came_back_asleep_is_offered_while_typing`).

## 9. What is not verified

- **The Mac's close rule and the tab's menu on an iPhone were not broken on purpose**: the apps' tests have them, and no part of them was taken out to see a test fail.
- **A real phone, an iPad, a keyboard on an iPad.** The iPad has ⌘D by the same button nobody sees as its other keys; no test presses it.
- **A screen reader.** What is said of a tab is read by the tests; nobody listened.
- **A tab whose page never arrived, kept from its menu**: the core refuses, and the window says nothing of it.
- **Many Sites.** The plan's standard session has 200 (000 §7); the row's order is recounted for each tab put into it. Not measured.
- **Several windows showing one Space** (D6): each has its own row, as before; a Site is of the row it was kept in.
- **A Site whose home no longer loads**: it wakes to a failed page, as any tab does; nothing says its home is gone.
- **What a Site was doing when it was closed**: a page that was asking for something, a find, a zoom of its own. They go with the page, as when a tab is closed; only the closing of prompts with a tab was ever tested (014), not with a Site.

## 10. Done checklist

- [x] The owner's word on §6 (2026-11-22).
- [x] The core, the store and the contract.
- [x] The Apple shell.
- [x] The Android shell.
- [x] Part 1 (the core): `Tools/test.sh all` green, and `android-device`, on 2026-11-22: `lint`, `core` (564), `unit` (272 on the Mac), `build`, the Mac's UI suite (38), `ios` (254 on the iPhone simulator, one skipped as before), the iPhone's UI suite (29), `android` (381 and 32 on the JVM) and `android-device` (156 on the emulator). No shell changed but for the contract's version.
- [x] Part 2 (the shells): the same again, on 2026-11-22: `lint`, `core` (564, and the strings tool's 24), `unit` (278 on the Mac), `build`, the Mac's UI suite (40), `ios` (260 on the iPhone simulator, one skipped as before), the iPhone's UI suite (30), `android` (384 and 32 on the JVM) and `android-device` (157 on the emulator).
- [ ] Part 3 (home again without closing).

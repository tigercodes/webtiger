# 049 — Tidying temporary tabs, and the Archive they go to (W2, W3)

| | |
|---|---|
| **Status** | **Designed; nothing built.** The choices of §6 are put to the owner |
| **Phase** | W2 of [000](000-master-plan.md) §9: "temporary tabs"; and W3's Archive, which tidying cannot be without. 000 §5.6: "Temporary tab. Opened by a link or by typing. It lives below the Sites, and is tidied into the Archive after the chosen time, or never." 000 §3: "Tab tidying is a setting with a **Never**" (WT-D15) |
| **Contract** | To be 1.41 (§5). The store's schema to be 14: an archive, and each Space's time (§4) |
| **Builds on** | [039](039-sites.md): a tab that is kept is a Site, and is never a temporary one. [041](041-groups.md), [042](042-favourites.md): the row's parts. [024](024-several-spaces.md): a row for each Space. [009](009-session-restore.md): the tabs that come back at a start. [008](008-history.md): a list in the Library, asked for by pages |
| **Last updated** | 2026-11-23 |

## 1. User story

I open links all day. By the evening my row has forty tabs I shall not look at again, and the six I want are among them. I would like the ones I have not looked at since yesterday put away by themselves: not thrown away, put somewhere I can get them back from. Never the ones I kept. And if I would rather it did not, it does not.

## 2. What is in it, and what is not

**In:** each Space has a time after which its temporary tabs are tidied, or never. A tab that is tidied leaves the row and is listed in the Space's Archive, by its page's name and address and when it was tidied. From the Archive a tab is brought back, or forgotten.

**Not in, each with where it goes:**

- **The Archive's list in the windows:** in the Library, beside History and Downloads, with the shells' part of this.
- **The first run's two styles** (000 §6.3: Classic, with no tidying; Tidy, with twelve hours): with the first run's other choices. Until then a Space is made with no tidying.
- **A tab's page as it was** (its scroll, what was typed in it, its way back): a tab that is brought back is loaded again at its address. Keeping more is with the saved state of a sleeping tab (000 §5.7, W3).
- **Putting tabs to sleep to save memory** (000 §5.7): another rule, with other times. A tab that is tidied was not necessarily asleep, and one that sleeps is not tidied by sleeping.
- **Sync** of the Archive (W7).

## 3. The rule

**A temporary tab is a tab that is no Site.** A Site, a Favourite and a Group's Site are kept: they are never tidied, however long they sleep.

**A temporary tab is tidied when it has not been in front for its Space's time.** The time is counted from when the tab was last the one in front in its window: when it was shown, not when it was opened or when its page last loaded (D2). A tab that was never in front (opened behind, by a link) counts from when it was opened.

**Never tidied:** the tab in front of a window, while it is; a tab of a private window, of which nothing is written (037); a tab with no page, which has nothing to bring back; a tab in a Space whose time is Never.

**What tidying does:** the tab leaves its row, its page is let go, and an entry is written in its Space's Archive: the page's address, its name, and the time. A window is never left with no tab: where the last tab of a row is tidied, an empty one is opened, as when the last tab is closed (005).

**When it is looked at:** when the app starts, before a window is first told of its row, so that a tab that is past its time never shows and goes; and from then on by the core's own clock, often enough that a tab is tidied within a few minutes of its time. The clock is the core's injected one: a test moves it.

**The Archive** is a Space's own, newest first. An entry **brought back** is a tab again: at the end of the row's temporary tabs, in front, loading its address; and the entry is gone. An entry **forgotten** is gone. The Archive keeps the thousand newest of a Space (D6): the oldest go as new ones come.

**It is undone by bringing back:** tidying is not one of the arrangements that ⌘Z undoes (044). It was not done by the person's hand, and may be many tabs at once, hours ago.

**Nothing leaves the device**, and nothing of it is in a log.

**Why the core.** Which tab is a temporary one, how long it has been out of sight, and what is kept of it are the model's and the store's; a shell that timed tabs would be a second clock.

## 4. Where it is kept

In the store (schema 14): `space.tidy_after`, the Space's time in seconds, 0 for never; and a table `archive(id, space_id, url, title, tidied_at)`, a thousand to a Space. A Space that is deleted takes its Archive with it. When each tab was last in front is kept with the session's tabs (009), so that a quit does not start every tab's time again.

## 5. The contract (to be 1.41)

Added; nothing is changed or removed.

```proto
// A Space's time: after how long out of sight its temporary tabs are tidied. 0: never.
message SpaceSummary { … uint32 tidy_after_seconds = N; }
message SetSpaceTidying { uint64 space_id = 1; uint32 after_seconds = 2; }   // one of the times the core lists; "space.unknown", "tidy.time_unknown"
message SpacesPage { … repeated uint32 tidy_times = N; }                     // the times a Space can have, in seconds, 0 first

// The Archive of the Space a window shows, newest first, by pages.
message QueryArchive { uint64 window_id = 1; int64 before = 2; uint32 limit = 3; }
message ArchivePage { repeated ArchiveEntry entries = 1; bool more = 2; }
message ArchiveEntry { int64 id = 1; string url = 2; string title = 3; int64 tidied_at = 4; }   // the address and the name are redacted in logs
message RestoreArchived { uint64 window_id = 1; int64 entry_id = 2; }       // answered with TabOpened; "archive.unknown"
message ForgetArchived { uint64 window_id = 1; oneof which { int64 entry_id = 2; bool all = 3; } }
```

A window is told that its row has changed when a tab is tidied, as when one is closed: by its snapshot. Nothing new is told for it.

## 6. Decisions

**All of these are put to the owner; none is built.** The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | **A Space is made with no tidying** until the first run offers its two styles | The plan's rule above its four pillars: a familiar browser first, and tidying is there when wanted and never required (WT-D15). A tab that goes by itself, in a browser nobody told to do so, is the thing the plan says not to be | Twelve hours from the start, as the "Tidy" style has it |
| D2 | The time is counted from when the tab was last in front | "Tabs I have not looked at since yesterday" is what a person means. A tab that plays music behind, or reloads itself, is still one nobody looked at | From when its page last loaded: a page that reloads itself is never tidied. From when it was opened: a tab in use all day goes at its hour |
| D3 | The times: never, 12 hours, a day, a week, 30 days | The plan's twelve hours, and three that people say. Few enough to choose among without a field to type in | Any number of hours, typed |
| D4 | The time is the Space's | The plan has it among a Space's own defaults (000 §5.3): a Space for reading is tidied, one for work is not | One time for the app |
| D5 | Never the tab in front of a window; nor a tab with no page | The one in front is being looked at, however long it has been there. An empty tab has nothing to bring back: it is closed by its window's own rules | The tab in front goes with the rest when its time comes |
| D6 | The Archive keeps a Space's thousand newest | It is a place to get a tab back from, not a second history: history has every visit already (008). A thousand is weeks of heavy use | Thirty days of them, however many. Or all, for ever |
| D7 | A tab brought back is at the end of the temporary tabs, in front, and loads its address | It is where a new tab goes, and it is what the person asked to see | Where it was in the row: the row has changed since, and the place means nothing |
| D8 | Tidying is not undone by ⌘Z | It was not the person's act, and bringing back is its undoing, one tab at a time and at any later day | One undo brings back all that the last look tidied |

## 7. In what order

1. **The core and the store:** the rule against the scripted engine and a clock a test moves; the Archive; the contract.
2. **The shells:** a Space's time, where a Space is made and edited; the Archive in the Library.

## 8. Tests

To be filled in as built.

## 9. What is not verified

To be filled in as built.

## 10. Done checklist

- [ ] The owner's word on §6.
- [ ] The core, the store and the contract.
- [ ] The shells.
- [ ] `Tools/test.sh all` green, and `android-device`.

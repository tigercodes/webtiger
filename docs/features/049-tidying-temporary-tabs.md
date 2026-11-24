# 049 — Tidying temporary tabs, and the Archive they go to (W2, W3)

| | |
|---|---|
| **Status** | **Built and tested in the core, the store, and the shells of Apple's systems and Android, on the recommendations of §6, which are provisional:** they are put to the owner and not yet answered. A Space is given its time where it is edited; the core tidies; the Archive is the Library's fourth list. A Space has no time until a person gives it one: nothing is tidied in a browser nobody told to. The Windows shell has neither (it is not built) |
| **Phase** | W2 of [000](000-master-plan.md) §9: "temporary tabs"; and W3's Archive, which tidying cannot be without. 000 §5.6: "Temporary tab. Opened by a link or by typing. It lives below the Sites, and is tidied into the Archive after the chosen time, or never." 000 §3: "Tab tidying is a setting with a **Never**" (WT-D15) |
| **Contract** | 1.41 (§5), and 1.42 for the shells (§5a). The store's schema is 14: an archive, each Space's time, and when a saved tab was last in front (§4) |
| **Builds on** | [039](039-sites.md): a tab that is kept is a Site, and is never a temporary one. [041](041-groups.md), [042](042-favourites.md): the row's parts. [024](024-several-spaces.md): a row for each Space. [009](009-session-restore.md): the tabs that come back at a start. [008](008-history.md): a list in the Library, asked for by pages |
| **Last updated** | 2026-11-23 |

## 1. User story

I open links all day. By the evening my row has forty tabs I shall not look at again, and the six I want are among them. I would like the ones I have not looked at since yesterday put away by themselves: not thrown away, put somewhere I can get them back from. Never the ones I kept. And if I would rather it did not, it does not.

## 2. What is in it, and what is not

**In:** each Space has a time after which its temporary tabs are tidied, or never. A tab that is tidied leaves the row and is listed in the Space's Archive, by its page's name and address and when it was tidied. From the Archive a tab is brought back, or forgotten.

**Not in, each with where it goes:**

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

## 5. The contract (1.41)

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

**As built, where the design did not say:**

- **A tab's time is counted by looks.** The core looks once a minute while some Space is tidied; when a Space is given its time; and when a window opens. A tab that is in front of its row at a look is noted as in front. One that has left the front since the look before, and one that was never looked at, start their time at that look. So a tab's time is counted from the look after it was last in front: a minute late at the most, and the clock is never read because a tab was brought in front.
- **The tab in front of its row is never tidied, whether or not its Space is the one its window shows.** The design had only the tab in front of a window. A row is then never left with no tab, and nothing has to be opened in a Space nobody is looking at.
- **Tabs that were open before a Space was given its time count from when it was given.**
- **While no Space is tidied the clock is not read for it, and the look does not go round.** It is set going by the first request after some Space has a time, and stops when none has.
- **A tab leaves its row once the Archive has it,** and not before: a store that will not write leaves the tab where it is, to be looked at again.
- **A tidied tab is not a closed one:** it is not what "Reopen Closed Tab" brings back (033). Its page is let go, and its web view destroyed.
- **Bringing back takes the entry out first,** and then opens a tab as any is opened. A page that then fails to load is a failed page in a tab; the entry is gone.
- **A page of the Archive is a hundred at the most,** and a hundred when the shell does not say.
- **Giving a Space no time again** leaves its Archive as it is.
- **The numbers:** `SpacesPage.tidy_times = 3`; `SpaceSummary.tidy_after_seconds = 5`; the four requests are 51 to 54; `ArchivePage` is the sixteenth kind of answer.

## 5a. In the windows (contract 1.42)

**A Space's time is chosen on the page that edits the Space** (024 §6.3), under its name and its colour: the times the core lists, each a row to press, with the one the Space has marked. "Never" is the first. The words for a time are the strings', found by its number of seconds, as a colour's are by its name: a time this build has no words for is left out, and never shown as a number. Saved, the page asks for what changed: the name, the colour, and then the time. A time the core will not take is said in words on the page, as a name it will not take is.

**The page that makes a Space does not ask** (D9). A Space is made with no time (D1), and is given one where it is edited.

**The Archive is the Library's fourth list** (D10), after History, Downloads and Permissions: of the Space the window shows. Each tidied tab is a row: the page's name, its address as the address field shows one, and when it was tidied, in the person's own words for dates. A page with no name is named by its address. **Pressed, a row is a tab again** and the Library closes, as it does when a visit is chosen (D11). A button beside it forgets it. "Forget All…" asks first, and is not to be pressed where there is nothing to forget. "Show Older Tabs" asks for the next page. An Archive that lists nothing says so, and says how a Space comes to be tidied: nothing is, until a person says so, and an empty Archive would otherwise be a list with no way into it.

**A private window's Library has no Archive** (D12): nothing of a private window is tidied, or written.

**Four names are more than a phone's row is wide.** On an iPhone the switch between the lists is moved sideways to reach its end, as Android's is, so that no name is cut short or broken; on a Mac the sheet is wider.

**The shell times nothing and keeps nothing.** It asks for the list, says which row was pressed, and says which time was chosen.

**The contract, 1.42.** Added; nothing is changed or removed.

```proto
message ArchiveEntry { … string display = 5; }       // the address as the address field shows it; redacted in logs
message FakeOptions { … uint32 seconds_a_read = 3; }  // how far fake mode's clock moves each time it is read; 0: one second
```

**As built, where this design did not say:**

- **On a phone the page that edits a Space no longer raises the keyboard when it opens** (D13). It did, for the name; with the times on the page, the keyboard covered them. The name is typed when its field is pressed. On a Mac the keyboard is in the name's field as before; the page that makes a Space is as it was.
- **The switch opens with the name of the list that is shown in sight,** on an iPhone and on Android, where the four names are more than the row is wide.
- **An Archive that lists nothing can be moved up and down,** so that at the largest text sizes its words are not cut short (Apple's audit, on an iPhone).
- **A Space's name field says what it is to a screen reader while it is empty** (Apple's audit, of the page that edits the Space nobody named: a gap that was there before this).
- **"Forget All…" is not to be pressed** where the Archive lists nothing.
- **The times are not drawn** where the core lists fewer than two that this build has words for.

An Archive entry had its address only as it is loaded; a list shows an address as the address field does, and how that is written is the core's (the address field never lies). **Fake mode's clock at a suite's pace** is for the apps' own tests: a UI suite cannot move the core's clock, and a tab is tidied after twelve hours at the soonest. With `WEBTIGER_FAKE_CLOCK_STEP=86400` (on Android `--ei webtiger.fake.clock_step 86400`) the clock moves a day each time it is read. The rule that runs is the core's own, at its own look once a minute; only the clock is quicker. Outside fake mode there is no such thing.

## 6. Decisions

**All of these are put to the owner.** They are built in the core as recommended; D5 is built wider than it was put (the tab in front of its row, shown or not). The first column is the recommendation.

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
| D9 | A Space's time is chosen where the Space is edited, and not where it is made | The page that makes a Space asks three things already, and a Space is made with no time (D1). Until the first run offers its two styles, tidying is something a person turns on for a Space they have | Asked where a Space is made as well |
| D10 | The Archive is the Library's fourth list | It is where what was visited and what was downloaded are looked for; the plan has it there (000 §5.6, W3). The row of tabs gains no button | A row of its own in the list of Spaces; a page of its own |
| D11 | A row pressed is a tab again, and the Library closes | It is what choosing a visit does, and the tab that came back is what the person asked to see | The Library stays open, to bring back several |
| D12 | A private window's Library has no Archive | It would list nothing, always: nothing of a private window is tidied or written (037) | The list, empty |
| D13 | On a phone the page that edits a Space does not raise the keyboard when it opens | The times are where the keyboard would be, and a person who came to change the time would have to put the keyboard away first. The name is a press away | The keyboard at once, as before, and the page moved up by hand |

## 7. In what order

1. **The core and the store:** the rule against the scripted engine and a clock a test moves; the Archive; the contract. **Built.**
2. **The shells:** a Space's time, where a Space is edited; the Archive in the Library. **Built**, on Apple's systems and Android.

## 8. Tests

| Where | What |
|---|---|
| The store (`archive.rs`, 6) | A Space has no time until it is given one, and one given none again is not listed; a Space there is not has none to be given. A tidied tab is listed newest first in its own Space's Archive, with its name and when; a tab with no address, and a Space there is not, are not listed. The Archive is read a page at a time, and says whether there are older ones. A tab taken out is what was listed, once, and is listed no longer; another Space's is not this Space's to take or to forget; one is forgotten, and then all of a Space's, and the other Space's stay. A Space keeps its thousand newest, counted apart from another's. A store from before tidying gains it and keeps what it had: its Space has no time, its tab is there and is not known to have been in front |
| The core, through the contract (`tidying.rs`, 8, against the scripted engine, on a clock the test moves) | **The rule:** a Space given twelve hours; a minute short of it nothing has gone; at twelve hours the two tabs that were not in front are in the Archive, newest first, by name and address and the time, the window is told, and their web views are gone; the tab in front stays, thirty days on. **A tab's time starts again** when it has been in front: the tab that was looked at goes thirteen hours after it left the front, not after it was opened. **Never:** a Site, a Favourite, a Group's Site, an empty tab; nor anything of a private window, which has no Archive and is no Space to be given a time. **A Space's time:** the times there are; none until one is given; a time that is none of them and a Space there is not are refused; a Space with no time keeps its tabs while another's go; a Space out of sight is tidied, but for the tab that was in front of its row; given no time again, what is there stays. **The Archive:** a page at a time; a tab brought back is the last of the row, in front, at its address, and listed no longer, and cannot be brought back twice; one forgotten, then all; a window there is not is refused; a Space's Archive is its own. **A quit** does not start a tab's time again: six hours, a quit, seven more, a quit, and at the third start the window is first told of a row without the two tabs, which are in the Archive. **The clock** is read once a minute while a Space is tidied, once when it is given its time, and not at all for this while none is |
| The tests there were | The 637 the core had and the 85 of the store pass unchanged, but that every store "from before" is made by undoing one thing more |
| The rule broken, a part at a time | 46 parts, in the model, the app, the dispatcher, the store and the session. 45 caught: 42 at once; two once the tests said more (a Space with no time was looked at only once, so no tab's time had begun; and the test's minute was the core's own word for how often it looks); one added. One was not: that a private window has an Archive, which nothing can show, since nothing of a private window is ever tidied |
| The core, 1.42 (`tidying.rs`, +1; 9 in all) | An Archive entry has its address as the address field shows it. In fake mode a suite can have a day pass each time the clock is read: with fake mode as it is, two minutes on, nothing has gone; with a day a read, the look after the Space is given its time tidies the tabs out of sight, at whole days after fake mode's "now" |
| The Apple shell's models, against a scripted core (`TidyingModelTests`, 10) | **The Archive:** asked of the window's Space, newest first, and older ones after the last shown; a tab brought back is asked of the core and leaves the list at once, and the model opens nothing itself; one the Archive has no longer is not brought back, and the list is asked for again; one is forgotten, and then all, and what is left is asked for. **A Space's time:** the times are the core's, and a Space says its own; editing asks for the time after the name and the colour, and only when it is another; a time the core will not take says why, in words; before the core has said which Space, nothing is asked; every time has words, and another number has none. **With the real core:** it lists the times this shell has words for, never first, and takes each of them |
| Pictures (Apple: `archive-list`, `archive-empty`, and `spaces-rename` anew, in both appearances, on the Mac and the iPhone simulator; Android: the same, and the Downloads and Permissions lists anew) | The Library at the width its sheet is let be on a Mac (580 points), and at a phone's (390): the four names. Each looked at. Android's eight Downloads and Permissions pictures were recorded again, because the switch over them has a fourth name: they differ from the ones before in the switch's row and nowhere else (0.1 to 0.2 % of their pixels, which the tolerance had let pass) |
| The apps, in fake mode, with the real core and the real engine, the clock a day a read (`TidyingTests`, 1, on the Mac and the iPhone simulator) | Three tabs. The Archive says nothing has been tidied, and has nothing to forget. The page that edits the Space has the five times, never chosen; twelve hours is chosen and saved, and nothing goes at once. **The test waits for the core's own look, a minute and more: the two tabs that are not in front are gone, and the one in front stays with its page.** The Space says its time; it is given none again. The Archive lists the two, newest first; one is forgotten; the other, pressed, is a tab again, the last of the row, in front, loading its address, and the Library has closed; the Archive lists nothing. Apple's audit at four of its states. And in `PrivateTests`: a private window's Library has its other lists and no Archive |
| The Android shell, against a scripted core (`ArchiveModelTest` 4, `ArchiveListTest` 7, `SpaceTidyingTest` 6) | The Archive's model as Apple's. The list drawn: a tidied tab by its name, its address and when; a row pressed is brought back and the list's owner told; a button forgets; "Forget All…" asks first, and cancel forgets nothing; an Archive with nothing says so and how a Space comes to be tidied; where the names are more than the row is wide the one shown is brought in sight; from the window, the Library has the Archive, and a tab brought back closes it without the model opening anything. A Space's time: the page has the times that have words, with the Space's own chosen; a time is asked after the name and the colour, and only when it is another; a refusal in words; a core that lists none has no choice; every time has words; a private window's Library has no Archive and never asks for one |
| The Android shell over the real core, on the JVM (`ThroughTheCoreTest`, +2) | With the clock a day a read: the window's model gives the Space twelve hours; at the core's next look (a window that opens) the tab out of sight is tidied and its web view destroyed; the Archive's model lists it by its name, its address and the address as the field shows it; brought back, it is a tab again, in front, and a new web view is asked for its address. And a launch hands the clock's pace on, in fake mode only |
| Android, on the emulator (`AppTidyingTest`, 1) | The real app: the page that edits the Space has the five times, never chosen; twelve hours chosen and saved is the Space's when the page opens again; it is given none again; the Library's Archive says nothing has been tidied. With the accessibility checks. (No tab is tidied in the device suite: its tests share one core, whose clock moves a second a read) |
| Android, on the emulator, by hand (2026-11-23) | The debug app in fake mode with the clock a day a read, and the test site over the loopback: two tabs; the Space given twelve hours on its page, which opened with no keyboard; about a minute later the tab out of sight had gone, and the one in front stayed; the Archive listed it by its name, its address and "Tidied" with a date; pressed, it was a tab again, the last of the row, in front, and the Archive said nothing was tidied |
| The shells broken, a part at a time | Apple's models: 20 parts, 20 caught. Android: 28 parts, 28 caught (one of them the row not being moved to the list shown, which a test found was so in the first place: it was asked before the row was laid out) |

## 9. What is not verified

- **The Windows shell** has neither a Space's time nor the Archive: it is not built.
- **A real phone, and an iPad.** The apps ran on the Mac, the iPhone simulator and the Android emulator. A wide window on a touch screen was not run for this.
- **A screen reader** was not listened to: Apple's audit and Android's checks ran, and each row says what it is, but nobody heard a time chosen or a tab brought back.
- **The largest text sizes** were not looked at: the audit says nothing is clipped at them, and the phone's switch is moved sideways there.
- **Apple's views, broken a part at a time:** only the models were. The views are held by the apps' UI tests and the pictures.
- **A tab tidied in Android's device suite:** by hand on the emulator, and on the JVM over the real core, not by the suite (above).
- **The choices of §6** are the recommendations, not the owner's word.
- **A real day.** In the core's tests every time is a clock a test moved, and every minute the runtime's own, which a test runs through at once. In the apps (the Mac, the iPhone simulator, the Android emulator) the look went round once on the core's thread by the system's clock, a minute after a Space was given its time, with the core's clock a day a read. Hours of it, by a clock nobody hurried, were not run.
- **A store that will not write the Archive:** by its lines the tab stays in its row; no test made a store refuse.
- **What a look costs** where there are thousands of tabs: every tab of every tidied Space is looked at once a minute; not measured ([043](043-the-sidebar.md) §8a).
- **Several windows showing one Space:** each has its own row, and each row its own tab in front; the Archive is the Space's. Tested with one window.
- **A tab that is doing something out of sight** (playing, downloading, waiting on a question) is tidied like any other: that is the rule (D2), and no test has such a tab.
- **A Space that is deleted:** Spaces cannot be deleted yet; its Archive is to go with it.

## 10. Done checklist

- [ ] The owner's word on §6.
- [x] The core, the store and the contract.
- [x] The shells: Apple's systems and Android.
- [x] The core: `Tools/test.sh all` green, and `android-device`, on 2026-11-23: `lint`, `core` (651, and the strings tool's 24), `unit` (331 on the Mac), `build`, the Mac's UI suite (56), `ios` (313 on the iPhone simulator), the iPhone's UI suite (43), `android` (444 and 32 on the JVM) and `android-device` (168 on the emulator).
- [x] The shells: `Tools/test.sh all` green, and `android-device`, on 2026-11-23: `lint`, `core` (652, and the strings tool's 24), `unit` (342 on the Mac), `build`, the Mac's UI suite (57), `ios` (324 on the iPhone simulator), the iPhone's UI suite (44), `android` (465 and 32 on the JVM) and `android-device` (169 on the emulator).

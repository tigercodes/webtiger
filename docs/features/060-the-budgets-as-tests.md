# 060 — The budgets as tests (W3)

| | |
|---|---|
| **Status** | **Built, in the core and for the Mac's app: what the core holds was measured, the standard session is a thing a test can ask for and a thing fake mode can start an app on, the core counts its own timers' wake-ups, seven counted budgets are tests that fail at the budget every run, three timed ones are measurements that fail at the budget when they are run** (`Tools/test.sh budgets`)**, and the Mac app's start and its own memory on the standard session are measured by a script that fails at the budget** (`Tools/perf/mac.sh`). The one budget that was found not met is met since [061](061-a-space-put-away.md), and no test is set aside (§8). On the recommendations of §6, used on the owner's word of 2026-11-24 (to use what is suggested where something waits on a choice, and to go on); they stay the owner's to change. Not built: the app's on the other systems, which wait for a device (§8) |
| **Phase** | W3 of [000](000-master-plan.md) §9: "every §7 budget as a failing test". The tenth of W3 |
| **Contract** | 1.51: `FakeOptions.standard_session`, for an app that is to be measured (§5, step 6). Nothing else |
| **Builds on** | [000](000-master-plan.md) §7: the budgets. [001](001-foundations.md) §11: the first measurements. [011](011-suggestions.md) §7 and [043](043-the-sidebar.md) §7: two budgets measured by hand. [051](051-a-tabs-kept-state.md), [053](053-a-tab-sleeps-by-itself.md), [059](059-a-sleeping-tabs-picture.md): what a sleeping tab keeps |
| **Last updated** | 2026-11-25 (step 6) |

## 1. User story

I chose this browser because it stays light with hundreds of tabs. I want that to be a thing that cannot quietly stop being true: a change that makes a sleeping tab cost more, or a start load two pages, should fail a test the day it is made, and not be found a year later by somebody's fan.

## 2. What is in it, and what is not

**In:** each budget of 000 §7 that can be held by a test on a machine like this one is held by one. Where a budget is a count (bytes, pages loaded, timers), the test fails at the budget. Where it is a time, the test measures and says what it measured, and fails at the budget when it is run for that. And a document that says, budget by budget, which it is.

**Not in, each with where it goes:**

- **The reference devices** (000 §7: an 8 GB Mac, an 8 GB Windows laptop, a mid-range Android phone, an iPhone). None is here. What runs here runs on this Mac, a simulator and an emulator, and says so.
- **Budgets of what is not built:** Guard's (W4), the Chromium pack's download (W5), Windows' (its shell).
- **The published run** ("50 sites, 10 runs, memory and energy", 000 §7): its own feature, the last of W3.
- **Frames.** "No dropped frame at 120 Hz" and "the next frame" cannot be counted by a test on this machine (043 §7 has what was measured instead).

## 3. The rule

A budget is one of four kinds, and its kind says where its test is.

**Counted.** Bytes on disk, bytes the core holds, pages loaded at a start, timers. The same on every machine, and the same every time. Its test is in the core's suite, runs with every other test, and fails at the budget itself.

**Timed, in the core.** A round trip, a list of suggestions. It depends on the machine and on what else the machine is doing. Its test is a measurement: a release build, run by its own command, which prints what it measured and fails at the budget. It is not part of `Tools/test.sh all`: a test that fails when a machine is busy teaches people to run it again.

**Measured in an app.** The app's own memory, a cold start. It needs the app on a device, and a script (`Tools/perf`), not a unit test. What can be scripted here is; a reference device is not here. **As built:** the Mac's, `Tools/perf/mac.sh` (§5.2).

**Not yet.** What waits for its feature.

**The standard session** (000 §7: "5 Spaces, 200 Sites, 40 temporary tabs, one live page") is made once, as something a test can ask for: a store with five Spaces, two hundred Sites among them in Groups and out, forty tabs, and one page in front. Every counted budget that names it uses that one. **And fake mode can start on it** (step 6): the same session, written into fake mode's store as the core starts, for an app that is to be measured.

## 4. The budgets, one by one

| Budget (000 §7) | Kind | Its test |
|---|---|---|
| A sleeping tab: ≤ 250 KB on disk | Counted | The store grows by no more than that for a tab that sleeps with the largest state and the largest picture the core keeps (051, 059) |
| A sleeping tab: ≤ 4 KB of memory | Counted | What the core holds for a tab that is asleep, counted by an allocator that counts. (A shell's own copy of the tab's row is not counted by a test: §8) |
| 100 sleeping tabs add ≤ 5 MB in total | Counted | The same count, for a hundred |
| A Space with nothing live: no process; ≤ 100 KB | Counted | What the core holds for a Space none of whose tabs has a web view; and that no web view was asked for |
| The core's share, with the default lists loaded: ≤ 30 MB | Counted, in part | What the core holds with the standard session. The default lists are Guard's (W4): without them, for now |
| Start-up loads: exactly one page | Counted | A start on the standard session asks the shell for one web view and one load |
| Idle with every tab asleep: no timers; at most one wake-up a second | Counted | The core counts each time one of its own timers wakes it. With nothing that could sleep and nothing to tidy: none in an hour |
| Wake a sleeping tab: snapshot within 50 ms; the engine loading within 100 ms | Timed, in the core, for the core's part | From the request that brings the tab in front to the picture's number in what the window is told, and to the load asked of the shell. The shell's and the engine's parts are measured in an app |
| A contract round trip for a 200-row sidebar page: ≤ 2 ms at p95 | Timed, in the core | 043's measurement, at 200 rows, with a ceiling |
| Address-field suggestions over 100,000 history rows: ≤ 10 ms at p95 | Timed, in the core | 011's measurement, with a ceiling |
| WebTiger's own processes in the standard session | Measured in an app | `Tools/perf/mac.sh`: the Mac app as it is released, started in fake mode on the standard session, and what the system counts against its process read some seconds later. It fails over 120 MB. The other systems when there is a device |
| Cold start to a usable window | Measured in an app, in part | The same script: from the app's process being started to a window of its own on the screen. It fails over 500 ms. What it does not measure: §8 |
| Switch to a live tab: the next frame | Not by a test here | Frames (§2) |
| Sidebar with 5,000 rows: no dropped frame at 120 Hz | Not by a test here | Frames; 043 §7 |
| A Guard decision; the page-load cost of Guard | Not yet | W4 |
| Download size ≤ 25 MB | Measured, by the build | The size of what is built, checked where it is built: with the release work |

## 5. In what order

1. **Measure first.** What the core holds for a sleeping tab, for a Space with nothing live, and for the standard session, with an allocator that counts: before any of them is a test, so that a test is not written to a number that was never true. **Done, 2026-11-24** (§5.1).
2. **The standard session,** as a fixture. **Built:** the test kit makes it through the contract, as a person would: five Spaces, each with forty Sites (four Favourites, sixteen in two Groups, twenty in none) and eight tabs; the first Space shown, one page alive.
3. **The counted budgets,** in the core's suite. **Built,** but for the timers (step 4).
4. **The core's wake-ups,** counted, and the idle test. **Built:** the core has three timers of its own (the look for tabs to put to sleep, the look for tabs to tidy, and fake mode's word that memory is short); each counts when it wakes the core, and each stops when it has nothing to look for.
5. **The timed ones,** as measurements with ceilings, and their command. **Built:** `Tools/test.sh budgets`, on a release build, outside `all`.
6. **The app's,** scripted for the Mac. **Built** (§5.2): fake mode can start on the standard session (contract 1.51), both shells hand the option on, and `Tools/perf/mac.sh` builds the Mac app as it is released, serves the test site on the machine, starts the app on that session and measures.

### 5.1 What the core holds, measured

Counted by an allocator that counts what the core's thread has asked for and not given back, with the core driven in process by the test kit's shell and the shell's own notes thrown away before each reading. Counted: everything Rust allocates there. Not counted: what SQLite allocates for itself (its page cache), which is C's and has its own limit. The numbers are bytes, and are the same on every run.

| What | As it was found | As it is now | The budget |
|---|---|---|---|
| A tab that sleeps, in a row of 100, 200, 400 and 600 | 2,672 to 2,899 | 1,981 to 2,157 | 4,096 |
| A hundred sleeping tabs | 267,250 | 198,130 | 5 MB |
| A start on nothing: one window, one empty tab | 25,944 | 16,752 | |
| A start on the standard session | 317,023 | 280,607 | 30 MB, with Guard's lists, which are not built |
| A Space that has not been shown since the start: forty Sites and eight tabs, as the session keeps them | 39,388 | 39,388 | 102,400 |
| A Space that was shown and left, nothing of it alive, its row not arranged in this run | about 134,000 (worked out from the tab's size; not read) | 99,293 with each tab in its own place; 41,433 since such a Space is put away (061) | 102,400 |
| The same, its row arranged in this run fifty times or more | 585,174 | 548,758 with each tab in its own place, which was **not met**; 41,001 since such a Space is put away (061) | 102,400 |

**What was changed on the way.** A tab is 928 bytes as the core's model has it, before its address and its name. The model kept its tabs in a map that holds the things themselves, and such a map keeps room for more than it has: about a third as much again. Each tab is now in a place of its own and the map holds where. That took 700 to 800 bytes off every tab, sleeping or not, and brought a Space that was shown and left from over its budget to under it. Nothing a person can see changed.

**What was found and not changed.** A row keeps what it was before each of the last fifty things done to arrange it (044): that is fifty copies of the row, each tab with its Site's home and name. For a Space of forty-eight tabs it is 450 KB, four times what its tabs are; for a row of two thousand it would be near 19 MB. It is the one thing that takes a Space over its budget, and it is §8's.

### 5.2 The app, measured

**The standard session in fake mode.** A shell started with `WEBTIGER_FAKE_STANDARD=1` (on Android `--ez webtiger.fake.standard true`) says so to the core (`FakeOptions.standard_session`, 1.51). The core, in fake mode and there only, writes the session into its store as it starts, before anything is read of it: four Spaces beside its own, each with logins of its own; in each, forty Sites (four Favourites, sixteen in two Groups, twenty in none) and eight tabs, the last in front. The app then finds a session, as it finds one a person left: the window opens on the first Space, its tab in front is loaded, and every other tab is asleep. That tab is at fake mode's start address, which a suite or the script makes a page of the test site; the others are at the addresses the test kit's session has, and nothing is fetched from them while they sleep. Fake mode's store is in memory: nothing of anybody's is in the app, and nothing is left behind.

Outside fake mode the option is nothing: a person's own store is never written to by it (tested).

**The script.** `Tools/perf/mac.sh [runs]` makes the core's library and the Mac app as they are released, serves the test site on 127.0.0.1, and has `Tools/perf/mac-app.swift` start the app's own program so many times (five, unless told), each time with nothing put back from the run before. Each run: the time from starting the process to a window of the app's being on the screen, as the system's list of windows has it (who owns a window and how large it is, and nothing else: no names and no contents, so no permission is asked for); then eight seconds for the page to load and the app to settle; then what the system counts against the app's own process (its footprint); then the app is ended, by its number. It says each run and the middle one, on what machine, and fails when the middle start is over 500 ms or the most memory over 120 MB. It will not start an app at an address that is not on the machine.

**Measured, 2026-11-25,** on this Mac (Mac18,5: Apple M6, twelve cores, 16 GB; macOS 27.2.1; **not** the 8 GB reference machine), a release build, three measurings of five runs each:

| What | Measured | The budget (000 §7) |
|---|---|---|
| From the process being started to a window on the screen: the middle run of five | 197, 208 and 196 ms | ≤ 500 ms |
| The same, every run but the first after a build | 162 to 221 ms | ≤ 500 ms |
| The same, the first run after the app was built (two of the measurings began with one) | 395 ms, and **673 ms: over** | ≤ 500 ms |
| The app's own process, on the standard session with its one page loaded | 48.3 to 53.2 MB | ≤ 120 MB |

**The first start of a program that has just been built is slow,** and was over the budget once of the twice it was seen. The system looks a new program over before it runs it for the first time, and that is what a first start after an update is too. The script says every run and holds the middle one to the budget: held to the slowest, it would fail for what the system does and the app does not. It is written here as found, and in §8 as not shown to be met.

The app's own process is the shell and the core. The engine's processes (WebKit's, for the page, the network and drawing) are others, and the plan counts them separately: they are not in this number. **Since [063](063-the-published-run.md) the script says them beside it:** three, holding 30.6 MB with the session's one page, which is a page of the test site. (And the first start after a build was over the budget a second time of three: 581 ms.) (001 §11 had the same process at 36 MB with one page and no session.)

## 6. Decisions

**All of these are recommendations** (§ Status). The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | A counted budget fails in the core's own suite, every run. A timed one is a measurement with its own command, outside `Tools/test.sh all`, that fails at the budget | A count is the same everywhere: there is no reason not to hold it always. A time on a busy machine fails for no fault of the code, and a gate that fails for no fault is run again until it passes | Times in the gate, with wide margins: they then catch only a change of ten times. Or no times at all until there is a reference device |
| D2 | "A sleeping tab: ≤ 4 KB of memory" is held for what the core holds | It is what a test can count exactly, and where the weight of a tab is decided. A shell holds the tab's row as the contract gave it, a few hundred bytes | Counted in the apps as well, by the difference between a window of a hundred sleeping tabs and one of none: possible, and a measurement of a system's allocator more than of the app |
| D3 | Where a measurement shows a budget is not met, the budget is not changed to fit: the test is written at the budget and the code is made to meet it; if it cannot be, the owner is asked (000 §7 lets W0 revise a number once, with the reason) | A budget that moves to where the code is, is not one | The number revised by whoever finds it too small |
| D4 | The core counts its own timers' wake-ups, and a test reads the count | "No timers" is not something that can be seen from outside the core; a count is cheap, and is true of the real thing | A test that knows the core's timers by name and checks each is stopped: it passes when a fourth is added |
| D5 | The standard session is made by the test kit, and by the `webtiger` command line for a store an app can be started on. **As built, otherwise for the app:** fake mode starts on it (§5.2) | One definition, for the core's tests and for the app's script. *As built:* a store made by the command line is a store on disk, and an app started on one is the app outside fake mode: it keeps a session, asks for icons, and is the real app on a folder. Fake mode is where every other run of the app by a script is, with its store in memory and nothing fetched but what a suite serves. So there are two makers of one session, the test kit's (through the contract, as a person would make it) and the core's (into the store), and a test holds the second to the first's counts and addresses | Each test makes what it needs. Or the command line, as first recommended |
| D6 | A timed test's ceiling is the budget itself, on whatever machine it runs on, and what it prints says which machine | This Mac is faster than the reference devices: passing here does not say the budget is met there, and the document says so. A ceiling scaled for a faster machine would be a guess | Scaled ceilings, by a factor measured once |

## 7. Tests

| Where | What |
|---|---|
| The core, through the contract (`budgets`, 3) | **The standard session is what the plan says:** five Spaces, two hundred Sites, forty tabs, one live page; as the window is told it, and as the store keeps it (each Space's forty-eight rows, its two Groups, its four Favourites). **A start loads exactly one page:** the app started again on the standard session and its window opened, the shell is asked for one web view and one load, and for no more in the hour and a half after. **A sleeping tab is no more than 250 KB on disk:** twenty tabs asleep, each with a state and a picture a byte under the most the core keeps, add 235,724 bytes each to the store's files |
| The core's memory (`wt-ffi`, `held`, 4, and 1 set aside) | The allocator counts what is held and what is given back. **A sleeping tab is no more than 4 KB,** in rows of 100, 200, 400 and 600, and not so little that something else was counted; **a hundred add no more than 5 MB.** **A start on the standard session is within the core's share** (30 MB), and **a Space that is not shown is no more than 100 KB.** **A Space that was shown and left, with nothing of it alive, is no more than 100 KB.** Set aside, because it fails: the same of a Space whose row was arranged in this run (§8) |

| The core's timers (`budgets`, +4) | **Idle with every tab asleep, no timer is going:** the standard session at a start, with one page alive in front, no tab that could sleep and no Space tidied: in an hour none of the core's timers wakes it, and nothing is asked of the shell. A tab that could sleep wakes the core once a minute until it sleeps, and then no more. A Space that is tidied wakes it once a minute (a sixtieth of the one a second it may have), and when none is tidied, no more. Fake mode's word that memory is short is counted as the others are |
| The counting and the looks' ends broken, a part at a time | 8 parts, 8 caught |

| The timed budgets, for the core's part (`timed_budgets`, 2, and `wt-store`'s measurement of suggestions; all three set aside from the usual run, and run by `Tools/test.sh budgets`) | Each fails at the plan's number and says what it measured and on what. On this Mac (Apple M6, twelve cores, a release build), three runs on 2026-11-25: **places offered among 100,000 visits,** for one letter to three words: 0.4 to 3.5 ms at p95, where 10 is allowed. **A change to a row of 200, told across the contract** (a Group opened: the model changed, the session written, the window told, and what it is told written as the contract carries it and read again): 0.26 to 0.43 ms at p95 (median 0.18), where 2 is allowed. **A sleeping tab woken,** with the largest state and picture the core keeps: to its picture's bytes answered, 4.2 to 6.4 ms at p95 (median 1.0), where 50 is allowed; to its load asked of the shell, the same, where 100 is allowed |
| Fake mode on the standard session, in the core (`budgets`, +1) | A core started in fake mode with the option, and a start address: its window opens on forty-eight tabs, the last in front, at the start address and loaded; one web view was asked for and one load, and every other tab is asleep. Five Spaces, each with logins of its own; each Space shown in turn is forty Sites (four Favourites, eight and eight in two Groups) and eight tabs, two hundred and forty in all, and its tab in front is its own last tab, by the title the test kit's session has there: only the first Space's is at the start address. **Outside fake mode the option does nothing:** a core on a store on disk, told the same, starts on one Space and one empty tab, and loads nothing. **Without the option fake mode is as it was** |
| The same, broken a part at a time | 17 parts, 17 caught, at the first run |
| The shells hand the option on (`DeviceTests` on Apple's systems; `LaunchOptionsTest` and `ThroughTheCoreTest` on Android: one test more, the others' existing tests longer) | Apple: the variable set to "1" is the option, and anything else is not. Android: the extra is the option in fake mode, in a build that can be debugged, and nothing otherwise; and **through the real core** (`ThroughTheCoreTest`, +1), a session started with it has forty-eight tabs, one of them awake and in front at the start address, one page made, and 239 counted asleep in the list of what is awake |
| The app, in fake mode, with the real core and the real engine (`StandardSessionTests`, 1, on the Mac and the iPhone simulator) | Started with the option at the suite's own site: its page shows; the list of what is awake has one tab and says "Asleep: 239"; the list of Spaces has a fifth and no sixth; and the site was asked for its first page once, and for nothing of the session |
| The Mac app, measured (`Tools/perf/mac.sh`) | §5.2. Not a test in the gate: it puts the app's window on the screen, and its numbers are a machine's |

The test of the core's memory is in the crate of the C functions and not with the core's others: an allocator is unsafe code, which is that crate's alone.

**A thing the timed tests taught:** the first reading of the row of 200 was 2.9 ms at p95, over the budget. It was the watching shell's own doing: the test kit's shell keeps every snapshot it was ever sent and copies them all to find the last. Its notes are now thrown away before each reading, and taking the snapshot out of them is not counted. The core's part was what 043 had measured.


## 8. What was not met, and what is not verified

**Was not met, and is met since 2026-11-25** ([061](061-a-space-put-away.md), the first of the two ways below): a Space whose row was arranged in this run, shown and left with nothing live, held 549 KB, and may hold 100. Its tabs were 99 KB of that. The rest was the row's line of things to undo (§5.1). Its test was written, failed, and was set aside by name with the reason; it is among the others again, at 41 KB. The two ways there were to meet it, which were a choice (D3 says the number is not to be moved to fit):

1. **A Space that no window shows, and that has nothing alive, is put away:** the core keeps of it what the session keeps, as it does of a Space that has not been shown since a start (39 KB), and its line of things to undo goes with it. Coming back to it after its tabs have all slept, there is nothing of its arrangement to undo. *Recommended:* it meets the budget whatever a row has been through, and it is what a start already does.
2. **The line kept, and made small:** each step keeping only what it changed. It keeps undo for a Space that was left; it does not by itself bring such a Space under the budget, whose tabs alone are 99 KB of the 100.

And, apart from the budget: **a long row's line was large** (near 19 MB for two thousand tabs, as it was worked out; 28.7 MB when it was measured). Making each step keep only what it changed is worth doing for that alone, whichever of the two is chosen. **Done since** ([062](062-a-shorter-line-to-undo.md)): 0.8 MB.

**Not verified:**

- **The reference devices.** None is here (§2). What is counted in bytes is the same on any 64-bit machine; a 32-bit one was not counted.
- **A shell's own memory for a sleeping tab** (D2).
- **The default lists' share** of the core's 30 MB: they are W4's.
- **What SQLite holds.** Its page cache is not Rust's and is not counted (§5.1).
- **A cold start is not shown to be within the budget.** The script starts a program whose files the system has read before, minutes after it was built. A start after the Mac has been turned on, or after a day of other work, is slower, and is not measured: nothing here can empty the system's memory of files without the owner's password. The nearest thing to it that was seen is the first start after a build: 395 ms once, and 673 ms once, against 500 (§5.2). Every other start was 162 to 221 ms.
- **A usable window.** What is timed is a window of the app's on the screen. That it is drawn, and takes typing, at that moment is not measured: reading that needs the system's leave to look at another app, which a script may not ask for. (The app's own tests, which have that leave, type into the window as soon as it is there.)
- **The reference Mac.** This one has twice its memory and is newer (§2).
- **The app's own processes on the other systems.** The iPhone's and Android's are not scripted: a simulator and an emulator are not devices, and their numbers would be this Mac's. Windows has no shell. Both shells hand the option on, so that a device can be measured the same way when there is one.
- **The engine's processes** are not counted, here or in the plan's number.
- **Frames:** §2.
- **The timed budgets on a reference device.** This Mac is faster than every one of them (D6): that a time is met here does not say it is met on an 8 GB laptop or a phone. And the shell's and the engine's parts of waking a tab (drawing the picture, beginning the load) are not in the core's measurement: 059 §8 has what was read once in an app.
- **A margin that was.** Before 061 a Space that was shown and left was 99,293 bytes against 102,400. It is 41 KB now; the nearest any count is to its budget is a sleeping tab's 235,724 bytes on disk against 256,000.

## 9. Done checklist

- [ ] The owner's word on §6 (to be used as suggested, on the word of 2026-11-24).
- [x] What the core holds, measured (§5.1).
- [x] The standard session.
- [x] The counted budgets: on disk, at a start, in the core's memory. The one that was not met (§8) is, since 061.
- [x] The core's wake-ups, and the idle test.
- [x] The timed ones, for the core's part.
- [x] The app's, scripted for the Mac (§5.2). The other systems wait for a device.
- [x] The gate over the first five steps: every stage of `Tools/test.sh all` green, and `android-device`, on 2026-11-25: `lint`, `core` (768, and the strings tool's 24), `unit` (400 on the Mac), `build`, the Mac's UI suite (65), `ios` (384 on the iPhone simulator), the iPhone's UI suite (54), `android` (536 and 32 on the JVM) and `android-device` (200 on the emulator).
- [x] The gate over the rest: every stage of `Tools/test.sh all` green, and `android-device`, on 2026-11-25: `lint`, `core` (776, and the strings tool's 24), `unit` (407 on the Mac), `build`, the Mac's UI suite (66), `ios` (391 on the iPhone simulator), the iPhone's UI suite (55), `android` (537 and 32 on the JVM) and `android-device` (200 on the emulator). (The Mac's UI suite was run last, after the others, once a dialog of the system's had gone from the screen).

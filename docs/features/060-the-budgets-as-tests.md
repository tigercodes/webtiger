# 060 — The budgets as tests (W3)

| | |
|---|---|
| **Status** | **Its first four steps are built, in the core: what the core holds was measured, the standard session is a thing a test can ask for, the core counts its own timers' wake-ups, and seven counted budgets are tests that fail at the budget.** One budget is not met, and its test is written and set aside (§8). On the recommendations of §6, used on the owner's word of 2026-11-24 (to use what is suggested where something waits on a choice, and to go on); they stay the owner's to change. Not built: the timed budgets, the app's |
| **Phase** | W3 of [000](000-master-plan.md) §9: "every §7 budget as a failing test". The tenth of W3 |
| **Contract** | None |
| **Builds on** | [000](000-master-plan.md) §7: the budgets. [001](001-foundations.md) §11: the first measurements. [011](011-suggestions.md) §7 and [043](043-the-sidebar.md) §7: two budgets measured by hand. [051](051-a-tabs-kept-state.md), [053](053-a-tab-sleeps-by-itself.md), [059](059-a-sleeping-tabs-picture.md): what a sleeping tab keeps |
| **Last updated** | 2026-11-25 |

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

**Measured in an app.** The app's own memory, a cold start. It needs the app on a device, and a script (`Tools/perf`), not a unit test. What can be scripted here is; a reference device is not here.

**Not yet.** What waits for its feature.

**The standard session** (000 §7: "5 Spaces, 200 Sites, 40 temporary tabs, one live page") is made once, as something a test can ask for: a store with five Spaces, two hundred Sites among them in Groups and out, forty tabs, and one page in front. Every counted budget that names it uses that one.

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
| WebTiger's own processes in the standard session | Measured in an app | `Tools/perf`: the app started on the standard session, its footprint read. Scripted for the Mac here; the others when there is a device |
| Cold start to a usable window | Measured in an app | The same script |
| Switch to a live tab: the next frame | Not by a test here | Frames (§2) |
| Sidebar with 5,000 rows: no dropped frame at 120 Hz | Not by a test here | Frames; 043 §7 |
| A Guard decision; the page-load cost of Guard | Not yet | W4 |
| Download size ≤ 25 MB | Measured, by the build | The size of what is built, checked where it is built: with the release work |

## 5. In what order

1. **Measure first.** What the core holds for a sleeping tab, for a Space with nothing live, and for the standard session, with an allocator that counts: before any of them is a test, so that a test is not written to a number that was never true. **Done, 2026-11-24** (§5.1).
2. **The standard session,** as a fixture. **Built:** the test kit makes it through the contract, as a person would: five Spaces, each with forty Sites (four Favourites, sixteen in two Groups, twenty in none) and eight tabs; the first Space shown, one page alive.
3. **The counted budgets,** in the core's suite. **Built,** but for the timers (step 4).
4. **The core's wake-ups,** counted, and the idle test. **Built:** the core has three timers of its own (the look for tabs to put to sleep, the look for tabs to tidy, and fake mode's word that memory is short); each counts when it wakes the core, and each stops when it has nothing to look for.
5. **The timed ones,** as measurements with ceilings, and their command.
6. **The app's,** scripted for the Mac.

### 5.1 What the core holds, measured

Counted by an allocator that counts what the core's thread has asked for and not given back, with the core driven in process by the test kit's shell and the shell's own notes thrown away before each reading. Counted: everything Rust allocates there. Not counted: what SQLite allocates for itself (its page cache), which is C's and has its own limit. The numbers are bytes, and are the same on every run.

| What | As it was found | As it is now | The budget |
|---|---|---|---|
| A tab that sleeps, in a row of 100, 200, 400 and 600 | 2,672 to 2,899 | 1,981 to 2,157 | 4,096 |
| A hundred sleeping tabs | 267,250 | 198,130 | 5 MB |
| A start on nothing: one window, one empty tab | 25,944 | 16,752 | |
| A start on the standard session | 317,023 | 280,607 | 30 MB, with Guard's lists, which are not built |
| A Space that has not been shown since the start: forty Sites and eight tabs, as the session keeps them | 39,388 | 39,388 | 102,400 |
| A Space that was shown and left, nothing of it alive, its row not arranged in this run | about 134,000 (worked out from the tab's size; not read) | 99,293 | 102,400 |
| The same, its row arranged in this run fifty times or more | 585,174 | 548,758 | 102,400: **not met** |

**What was changed on the way.** A tab is 928 bytes as the core's model has it, before its address and its name. The model kept its tabs in a map that holds the things themselves, and such a map keeps room for more than it has: about a third as much again. Each tab is now in a place of its own and the map holds where. That took 700 to 800 bytes off every tab, sleeping or not, and brought a Space that was shown and left from over its budget to under it. Nothing a person can see changed.

**What was found and not changed.** A row keeps what it was before each of the last fifty things done to arrange it (044): that is fifty copies of the row, each tab with its Site's home and name. For a Space of forty-eight tabs it is 450 KB, four times what its tabs are; for a row of two thousand it would be near 19 MB. It is the one thing that takes a Space over its budget, and it is §8's.

## 6. Decisions

**All of these are recommendations** (§ Status). The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | A counted budget fails in the core's own suite, every run. A timed one is a measurement with its own command, outside `Tools/test.sh all`, that fails at the budget | A count is the same everywhere: there is no reason not to hold it always. A time on a busy machine fails for no fault of the code, and a gate that fails for no fault is run again until it passes | Times in the gate, with wide margins: they then catch only a change of ten times. Or no times at all until there is a reference device |
| D2 | "A sleeping tab: ≤ 4 KB of memory" is held for what the core holds | It is what a test can count exactly, and where the weight of a tab is decided. A shell holds the tab's row as the contract gave it, a few hundred bytes | Counted in the apps as well, by the difference between a window of a hundred sleeping tabs and one of none: possible, and a measurement of a system's allocator more than of the app |
| D3 | Where a measurement shows a budget is not met, the budget is not changed to fit: the test is written at the budget and the code is made to meet it; if it cannot be, the owner is asked (000 §7 lets W0 revise a number once, with the reason) | A budget that moves to where the code is, is not one | The number revised by whoever finds it too small |
| D4 | The core counts its own timers' wake-ups, and a test reads the count | "No timers" is not something that can be seen from outside the core; a count is cheap, and is true of the real thing | A test that knows the core's timers by name and checks each is stopped: it passes when a fourth is added |
| D5 | The standard session is made by the test kit, and by the `webtiger` command line for a store an app can be started on | One definition, for the core's tests and for the app's script | Each test makes what it needs |
| D6 | A timed test's ceiling is the budget itself, on whatever machine it runs on, and what it prints says which machine | This Mac is faster than the reference devices: passing here does not say the budget is met there, and the document says so. A ceiling scaled for a faster machine would be a guess | Scaled ceilings, by a factor measured once |

## 7. Tests

| Where | What |
|---|---|
| The core, through the contract (`budgets`, 3) | **The standard session is what the plan says:** five Spaces, two hundred Sites, forty tabs, one live page; as the window is told it, and as the store keeps it (each Space's forty-eight rows, its two Groups, its four Favourites). **A start loads exactly one page:** the app started again on the standard session and its window opened, the shell is asked for one web view and one load, and for no more in the hour and a half after. **A sleeping tab is no more than 250 KB on disk:** twenty tabs asleep, each with a state and a picture a byte under the most the core keeps, add 235,724 bytes each to the store's files |
| The core's memory (`wt-ffi`, `held`, 4, and 1 set aside) | The allocator counts what is held and what is given back. **A sleeping tab is no more than 4 KB,** in rows of 100, 200, 400 and 600, and not so little that something else was counted; **a hundred add no more than 5 MB.** **A start on the standard session is within the core's share** (30 MB), and **a Space that is not shown is no more than 100 KB.** **A Space that was shown and left, with nothing of it alive, is no more than 100 KB.** Set aside, because it fails: the same of a Space whose row was arranged in this run (§8) |

| The core's timers (`budgets`, +4) | **Idle with every tab asleep, no timer is going:** the standard session at a start, with one page alive in front, no tab that could sleep and no Space tidied: in an hour none of the core's timers wakes it, and nothing is asked of the shell. A tab that could sleep wakes the core once a minute until it sleeps, and then no more. A Space that is tidied wakes it once a minute (a sixtieth of the one a second it may have), and when none is tidied, no more. Fake mode's word that memory is short is counted as the others are |
| The counting and the looks' ends broken, a part at a time | 8 parts, 8 caught |

The test of the core's memory is in the crate of the C functions and not with the core's others: an allocator is unsafe code, which is that crate's alone.

The rest is to be filled in as built.

## 8. What is not met, and what is not verified

**Not met: a Space whose row was arranged in this run, shown and left with nothing live, holds 549 KB, and may hold 100.** Its tabs are 99 KB of that. The rest is the row's line of things to undo (§5.1). Its test is written, fails, and is set aside by name with the reason; it is to be put back among the others when it passes. Two ways to meet it, which are a choice (D3 says the number is not to be moved to fit):

1. **A Space that no window shows, and that has nothing alive, is put away:** the core keeps of it what the session keeps, as it does of a Space that has not been shown since a start (39 KB), and its line of things to undo goes with it. Coming back to it after its tabs have all slept, there is nothing of its arrangement to undo. *Recommended:* it meets the budget whatever a row has been through, and it is what a start already does.
2. **The line kept, and made small:** each step keeping only what it changed. It keeps undo for a Space that was left; it does not by itself bring such a Space under the budget, whose tabs alone are 99 KB of the 100.

And, apart from the budget: **a long row's line is large** (near 19 MB for two thousand tabs). Making each step keep only what it changed is worth doing for that alone, whichever of the two is chosen.

**Not verified:**

- **The reference devices.** None is here (§2). What is counted in bytes is the same on any 64-bit machine; a 32-bit one was not counted.
- **A shell's own memory for a sleeping tab** (D2).
- **The default lists' share** of the core's 30 MB: they are W4's.
- **What SQLite holds.** Its page cache is not Rust's and is not counted (§5.1).
- **The app's own processes,** cold start, frames: steps 5 and 6, and §2.
- **A margin.** A Space that was shown and left is 99,293 bytes against 102,400: a tab that grows by seventy bytes fails the test. That is what the test is for; it is said here so that whoever meets it knows the number was this near.

## 9. Done checklist

- [ ] The owner's word on §6 (to be used as suggested, on the word of 2026-11-24).
- [x] What the core holds, measured (§5.1).
- [x] The standard session.
- [x] The counted budgets: on disk, at a start, in the core's memory. One is not met (§8).
- [x] The core's wake-ups, and the idle test.
- [ ] The timed ones.
- [ ] The app's, scripted for the Mac.
- [ ] The gate.

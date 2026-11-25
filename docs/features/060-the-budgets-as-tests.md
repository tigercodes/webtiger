# 060 — The budgets as tests (W3)

| | |
|---|---|
| **Status** | **Designed, not built.** The choices of §6 are recommendations, to be used on the owner's word of 2026-11-24 (to use what is suggested where something waits on a choice, and to go on); they stay the owner's to change. Its first step is to measure what the core holds for a sleeping tab, before any number is made a test |
| **Phase** | W3 of [000](000-master-plan.md) §9: "every §7 budget as a failing test". The tenth of W3 |
| **Contract** | None |
| **Builds on** | [000](000-master-plan.md) §7: the budgets. [001](001-foundations.md) §11: the first measurements. [011](011-suggestions.md) §7 and [043](043-the-sidebar.md) §7: two budgets measured by hand. [051](051-a-tabs-kept-state.md), [053](053-a-tab-sleeps-by-itself.md), [059](059-a-sleeping-tabs-picture.md): what a sleeping tab keeps |
| **Last updated** | 2026-11-24 |

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

1. **Measure first.** What the core holds for a sleeping tab, for a Space with nothing live, and for the standard session, with an allocator that counts: before any of them is a test, so that a test is not written to a number that was never true.
2. **The standard session,** as a fixture.
3. **The counted budgets,** in the core's suite.
4. **The core's wake-ups,** counted, and the idle test.
5. **The timed ones,** as measurements with ceilings, and their command.
6. **The app's,** scripted for the Mac.

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

To be filled in as built.

## 8. What is not verified

To be filled in as built. Known already:

- **The reference devices.** None is here (§2).
- **A shell's own memory for a sleeping tab** (D2).
- **The default lists' share** of the core's 30 MB: they are W4's.

## 9. Done checklist

- [ ] The owner's word on §6 (to be used as suggested, on the word of 2026-11-24).
- [ ] What the core holds, measured (§5.1).
- [ ] The standard session.
- [ ] The counted budgets.
- [ ] The core's wake-ups, and the idle test.
- [ ] The timed ones.
- [ ] The app's, scripted for the Mac.
- [ ] The gate.

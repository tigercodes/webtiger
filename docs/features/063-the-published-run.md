# 063 — The published run (W3)

| | |
|---|---|
| **Status** | **The script is built and tested, for the Mac, on the recommendations of §6, and tried here against the test site alone.** The owner's word of 2026-11-24 was to use what is suggested where something waits on a choice, and to go on: they are built on, and stay the owner's to change. Fake mode opens a list of addresses by itself and can have the device's own limit of live tabs (contract 1.52); `Tools/perf/mac-run.sh` starts the app on a list and says what the app's own process and the engine's processes hold, and the energy they used. **The run itself is the owner's, and has not been done:** it loads other people's sites with the app, and nothing here does that without the owner's word (§2) |
| **Phase** | W3 of [000](000-master-plan.md) §9, its last. 000 §7: "Phase W3 adds a public run of the kind Brave published in September 2026 (50 sites, 10 runs, memory and energy), scripted in `Tools/perf` so that anyone can repeat it." |
| **Contract** | 1.52: `FakeOptions.open_addresses` and `open_every_seconds`: fake mode opens a list of addresses by itself, one after another (§3, D2); `device_limits`: fake mode has the device's own limit of live tabs (D4) |
| **Builds on** | [060](060-the-budgets-as-tests.md) §5.2: the Mac app started in fake mode by a script, and its own process measured. [031](031-links-from-other-apps.md): an address handed to the app from outside opens in a new tab. [053](053-a-tab-sleeps-by-itself.md), [054](054-the-limit-of-live-tabs.md): what makes fifty tabs cost less than fifty pages |
| **Last updated** | 2026-11-25 |

## 1. User story

I read that this browser is light. I want to see the numbers, and how they were got, and be able to get them again on my own machine with one command.

## 2. What is in it, and what is not

**In:** a script that anyone can run on a Mac. It starts the app, which opens a list of sites one after another, each in a tab, as links pressed in another app are opened; it leaves them for a stated time; and it says what the app's own process holds, what the engine's processes hold, and what energy both used. So many runs, and a report of them with the machine, the system, the app's version and the day.

**The run, and what is published, are the owner's.** The script loads whatever addresses its list has. Tried here: the test site's own pages, served on this machine, and nothing else. A run over fifty sites of the public web sends this machine's requests to fifty other people's servers, from the owner's address, with the owner's app: that is done when the owner says so, on the list the owner chooses, and what is published of it is the owner's to publish.

**Not in, each with where it goes:**

- **Other browsers, measured beside it.** Brave's run compared browsers. This script starts this app and no other (001: nothing here starts another app). A comparison is somebody's run of each browser's own script, and is not claimed here (D5).
- **Guard.** Blocking changes what a page costs more than anything else does. Guard is W4: a run before it is a run of a browser that blocks nothing, and says so.
- **The other systems.** Windows has no shell; an iPhone and an Android phone need a device (060 §8). The Mac first.
- **Publishing.** Where the numbers go, and in what words.

## 3. The rule

**The app is started in fake mode, by the script, as its own program** (060 §5.2). Fake mode's store is in memory: nothing of anybody's is in the app, nothing is kept after it, and the run does not depend on what a person's own app has in it. The script starts the one copy there is, and nothing else may start another: no copy of the app can start on a person's own data.

**Sites are opened by the core itself, as links from outside are** (031). Fake mode is given the list and a number of seconds (contract 1.52): every so many seconds the core opens the next address in a new tab, in front, in the window that was opened first, exactly as it opens an address the system hands a shell from another app. When the last has been opened its timer stops. Nothing in the app is automated, nothing is asked of the system, and the same works on every system a shell is on. (It is the pattern of 055's knob, by which fake mode's core acts every so often as though the system had said memory is short.)

**The tabs behave as they would for a person:** the app's own limit of live tabs and its own sleep times are in force (053, 054), which is the point of the run. The script says which were in force. **As built:** fake mode has no limit of live tabs of its own, so that its suites may open what they like (054); a run asks it for the device's (`device_limits`, 1.52), which is the limit the same machine has outside fake mode. The sleep time is the app's in fake mode already. What fake mode does not do, and the app does: listen to the system for word that memory is short (055). A run does not either (§8).

**What is measured,** at stated moments (when the last site has been opened; and after a stated time of doing nothing), by the system's own counters for each process:

- **the app's own process:** what the system counts against it (its footprint), and the energy it has used;
- **the engine's processes:** the same, summed. On a Mac they are WebKit's (for pages, for the network, for drawing), which the system starts, and which are not the app's children. The ones that are the app's are told by this: they were not running before the app started, and they are gone when the app has ended. A process of the engine's that outlives the app is somebody else's, and is not counted;
- **both together.**

**What a run says of itself:** the machine and its memory, the system's version, the app's version and build, the engine's version, the day, the list (by its name and a count: the addresses are the list's own file), the times it waited, how many tabs were alive and how many asleep at each measure, and every run's numbers beside the middle one.

**Nothing a person visited is in a log** (the project's rule): the list is the script's input, of the script's user's choosing, and its addresses are in the report only as the list's file says them.

## 4. The contract

1.52, in fake mode's options and nowhere else:

- `open_addresses`: the addresses, in the order they are to be opened. Marked as what a person visited is: never in a log or an error.
- `open_every_seconds`: the seconds between them; nothing is opened without it.
- `device_limits`: the limit of live tabs is the device's own, as outside fake mode. A limit a suite gives comes first.

Outside fake mode neither is anything. A shell hands them on from what it was started with: `WEBTIGER_FAKE_OPEN` (the addresses, with spaces or lines between them), `WEBTIGER_FAKE_OPEN_EVERY` and `WEBTIGER_FAKE_DEVICE_LIMITS=1` on Apple's systems; `--es webtiger.fake.open`, `--ei webtiger.fake.open_every` and `--ez webtiger.fake.device_limits true` on Android.

The count of tabs alive and asleep at a measure cannot be had from outside the app without automating it: the report leaves it out and says so (§8), and a way to ask the app is a later choice.

## 5. In what order

1. **The engine's processes, found and measured,** in the script of 060 §5.2: on the standard session first, where the plan has a number for the app's own processes and says the engine's are counted separately. **Built,** and tried on this Mac with the test site (§5.1).
2. **Fake mode opens a list by itself** (1.52): the core, against the scripted engine; both shells hand the list on; tried in the app with the test site's pages. **Built.**
3. **The run:** a list, so many runs, the measures, the report. **Built:** `Tools/perf/mac-run.sh`. Tried here with lists of the test site's pages, which shows that the script works and nothing about the web (§5.1).
4. **The owner's:** the list of sites; the run; what is published. **Not done.**

### 5.1 The script, and what it said of the test site

`Tools/perf/mac-run.sh [--web] <list> [runs] [seconds between sites] [seconds idle]` (ten runs, twenty seconds and two minutes, unless told). It builds the Mac app as it is released; serves the test site on the machine when the list names it; and for each run starts the app's own program in fake mode with the list, the seconds, and the device's own limit of live tabs. It reads, every second, what the system counts against the app's own process and against each of the engine's processes that began after the app. It measures twice: when the last address has had its seconds, and again after the idle time. It ends the app by its number, and counts as the app's those of the engine's processes that went with it (D3). It says each run, and the middle one of them; and of itself: the day, the app's version, the machine and its memory, the system's version, the engine's version, the list's file and how many addresses it has, and the times.

**It loads nothing that is not on the machine unless `--web` is said,** and stops before it starts anything when a list has such an address, or one that is no address of the web, or none (tried). The list's addresses are in no line it prints.

**Tried here, 2026-11-25,** on this Mac (Mac18,5: Apple M6, 16 GB; macOS 27.2.1), a release build. These are numbers of the test site's pages, which are a few lines each: they say the script works.

| The list | When | The app's own process | The engine's processes | Both | Energy used |
|---|---|---|---|---|---|
| The standard session, its one page (`Tools/perf/mac.sh`, three runs) | Eight seconds after the window | 49.0 to 49.4 MB | 3, holding 30.6 to 30.7 MB | | not read |
| Eight pages of the test site, three seconds apart (`Tools/perf/lists/test-site.txt`, two runs) | All opened | 41.6, 42.2 MB | 10, holding 105.9, 110.8 MB | 147.5, 152.9 MB | 2.6, 2.9 J |
| | Ten seconds on | 44.5, 45.1 MB | 10, holding 102.4, 107.2 MB | 146.9, 152.3 MB | 2.6, 2.9 J |
| Forty pages of the test site, two seconds apart (one run) | All opened | 57.4 MB | 34, holding 370.6 MB | 428.0 MB | 7.6 J |
| | Ten seconds on | 54.1 MB | 34, holding 351.6 MB | 405.6 MB | 7.7 J |

**The limit of live tabs shows in the last:** forty pages were opened, and thirty-four of the engine's processes were running, which is thirty-two pages (this Mac's limit: one for each half gigabyte of its memory, 054), the network's and the drawing's. The eight that had been out of sight the longest were asleep.

## 6. Decisions

**All of these are recommendations** (§ Status). The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | The run is in fake mode | Nothing of a person's is read, written or left; the same start for everybody who repeats it | The real app on a profile of the run's own: nearer to a person's app (a session, icons, history), and it leaves a store behind to clear |
| D2 | The list is given to fake mode, and the core opens each address as one from outside is opened, a stated time apart | It needs no automation and no leave from the system; it is tested in the core like anything else; it is the same on every system; and no second copy of the app can come of it. *The first recommendation was to hand the app each address through the system, as another app does.* The system may start a second copy of an app to give it an address (the app's own UI suite found that it does, for a test's), and a copy the system starts is not one the script started in fake mode: nothing here may risk the app starting on a person's own data | Handing addresses through the system: nothing new in the contract. Or the app driven through its window by a UI test: it can wait for each page to finish, and needs Xcode and leave to control the app |
| D3 | The engine's processes are those that began after the app and ended with it | It uses only what the system tells any program. Asking the system which app a process works for is not a public thing to ask | The app's own count of its web views' processes: a new request, and WebKit does not say a web view's process in public either |
| D4 | The app's own limits and sleep times are in force, and the report says them. **As built:** an option of fake mode's, for the device's own limit (§3) | The run is of the product. Fifty tabs that all stay alive would be a run of the engine. *As built:* the script could have worked the limit out and given it as a suite does; the rule for it is the core's, and a script that knew it would be wrong the day the rule changed | Sleeping turned off, for a number that is the engine's alone, beside the other |
| D5 | The script measures this app and no other | 001: nothing here starts another app; and a fair run of another browser is that browser's people's to script | A script that starts several browsers by name |
| D6 | The list of sites is a file the run is given; the repository has the test site's list, and a list of public sites only when the owner has chosen one | A list of other people's sites in a public repository is a statement about them, and is the owner's to make | A list written here, from a published ranking |

## 7. Tests

| Where | What |
|---|---|
| The core, through the contract (`opens_by_itself`, 5) | **Each address in its turn:** a list of three, ten seconds apart: nothing before its time; then each in a new tab after the tab in front, in front, loaded at the address as it was given; the core woke itself three times for them, and not again. **What is no address of the web is passed over:** a script's address, a file's, a host with no scheme and an empty one open no tab, load nothing, are not mended or searched for, and nothing is handed to the system; the next address is opened at its time. **The list waits for a window that is not private:** with a private window alone nothing is opened and nothing is lost; with two ordinary windows the first has the list; that one closed, the next address goes to the one that is left; the private window is never touched. **A tab is in its window as soon as it is opened,** though the shell answers nothing the core asks: in the row, in front, with no page yet; and its page comes when the shell answers. **Nothing is opened** in fake mode without the seconds, with seconds and no list (no timer either), or outside fake mode with both |
| The core's limit (`live_tabs`, +1) | Fake mode with the option has the device's own limit: a phone three, a desktop of 8 GB sixteen, one of 2 GB eight, each as outside fake mode; and with a suite's limit as well, the suite's |
| The core broken, a part at a time | 13 parts, 13 caught. The first run was of 14 and caught 12. One was a check that the loop makes needless (a timer for a list of none): it was taken out. The other: with the windows not told when a tab is opened, every test passed, because they are told again when its page has come; the test of a shell that answers nothing is for that |
| The shells hand the options on (`DeviceTests` on Apple's systems; `LaunchOptionsTest` and `ThroughTheCoreTest` on Android: one test more, the others' existing tests longer) | Apple: the list, in its order, each address as it was written, whatever is between them; the seconds; the limit. Android: the same from the extras, in fake mode in a build that can be debugged, and nothing otherwise; and **through the real core** (`ThroughTheCoreTest`, +1), a session started with a list of two has the core ask for each page in its turn, in a new tab in front, with nothing asked by the shell |
| The apps, in fake mode, with the real core and the real engine (`OpensByItselfTests`, 1, on the Mac and the iPhone simulator) | Started with a list of two of the suite's own pages, four seconds apart: each is the page in front in its turn, with nothing pressed or typed; on the Mac the window has three tabs; and the site was asked for each page once |
| The script (`Tools/perf/mac-run.sh`), by hand | §5.1: two lists of the test site's pages, and the three lists it refuses. Not a test in the gate: it puts the app's window on the screen, and its numbers are a machine's |

## 8. What is not verified

- **The choices of §6** are the recommendations, used on the owner's word; the owner has not looked at them.
- **A run over the public web:** not done, and not to be done here without the owner's word (§2). Nothing in §5.1 says what a real site costs.
- **That the engine's processes are found rightly when something else on the machine uses WebKit at the same time.** One that begins during a run and outlives the app is left out, and the script says how many were. One of somebody else's that begins and ends within a run would be counted as the app's: not seen, and not ruled out.
- **Energy.** The system's own count for each process (`ri_energy_nj`), read without leave of any kind. It was not checked against a meter or against the system's own tools, and a page's process that ends between two readings loses up to a second of what it used.
- **The system short of memory.** The app listens to the system for that (055); fake mode does not, so that a busy machine puts no test's tab to sleep, and a run is in fake mode. On a machine that runs short during a run, the app as it is would put tabs to sleep and the run would not.
- **How many tabs were alive and asleep** at each measure is not in the report: it cannot be had from outside the app. The count of the engine's processes says it nearly (§5.1).
- **A page that asks for something** (to open another app, a window, a download, the camera): the run answers nothing, and what the app shows waits. The test site's list has no such page. Not tried.
- **The other systems.** Both shells hand the options on, and Android's does through the real core; nothing measures there (060 §8).
- **The reference Mac** (8 GB): its limit would be sixteen live tabs. Not here.

## 9. Done checklist

- [ ] The owner's word on §6, on the list of sites, and on the run.
- [x] The engine's processes, found and measured.
- [x] Fake mode opens a list by itself (1.52).
- [x] The run and its report, tried with the test site.
- [x] The gate: every stage of `Tools/test.sh all` green, and `android-device`, on 2026-11-25: `lint`, `core` (786, and the strings tool's 24), `unit` (407 on the Mac), `build`, the Mac's UI suite (67), `ios` (391 on the iPhone simulator), the iPhone's UI suite (56), `android` (538 and 32 on the JVM) and `android-device` (200 on the emulator).

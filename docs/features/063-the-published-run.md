# 063 — The published run (W3)

| | |
|---|---|
| **Status** | **Designed, on the recommendations of §6.** The owner's word of 2026-11-24 was to use what is suggested where something waits on a choice, and to go on: the script is to be built on them, and tried here against the test site alone. **The run itself is the owner's:** it loads other people's sites with the app, and nothing here does that without the owner's word (§2). Not built |
| **Phase** | W3 of [000](000-master-plan.md) §9, its last. 000 §7: "Phase W3 adds a public run of the kind Brave published in September 2026 (50 sites, 10 runs, memory and energy), scripted in `Tools/perf` so that anyone can repeat it." |
| **Contract** | None foreseen |
| **Builds on** | [060](060-the-budgets-as-tests.md) §5.2: the Mac app started in fake mode by a script, and its own process measured. [031](031-links-from-other-apps.md): an address handed to the app from outside opens in a new tab. [053](053-a-tab-sleeps-by-itself.md), [054](054-the-limit-of-live-tabs.md): what makes fifty tabs cost less than fifty pages |
| **Last updated** | 2026-11-25 |

## 1. User story

I read that this browser is light. I want to see the numbers, and how they were got, and be able to get them again on my own machine with one command.

## 2. What is in it, and what is not

**In:** a script that anyone can run on a Mac. It starts the app, opens a list of sites in it one after another, each in a tab, as a person would; it leaves them for a stated time; and it says what the app's own process holds, what the engine's processes hold, and what energy both used. So many runs, and a report of them with the machine, the system, the app's version and the day.

**The run, and what is published, are the owner's.** The script loads whatever addresses its list has. Tried here: the test site's own pages, served on this machine, and nothing else. A run over fifty sites of the public web sends this machine's requests to fifty other people's servers, from the owner's address, with the owner's app: that is done when the owner says so, on the list the owner chooses, and what is published of it is the owner's to publish.

**Not in, each with where it goes:**

- **Other browsers, measured beside it.** Brave's run compared browsers. This script starts this app and no other (001: nothing here starts another app). A comparison is somebody's run of each browser's own script, and is not claimed here (D5).
- **Guard.** Blocking changes what a page costs more than anything else does. Guard is W4: a run before it is a run of a browser that blocks nothing, and says so.
- **The other systems.** Windows has no shell; an iPhone and an Android phone need a device (060 §8). The Mac first.
- **Publishing.** Where the numbers go, and in what words.

## 3. The rule

**The app is started as a person's would be, in fake mode.** Fake mode's store is in memory: nothing of anybody's is in the app, nothing is kept after it, and the run does not depend on what a person's own app has in it. It is started through the system, as the app (and not as a child of the script), so that the system counts it as an app of its own; every way the script starts it carries fake mode, so that no copy of it can start on a person's own data.

**Sites are opened as links from outside** (031): the script hands the app an address, and the app opens it in a new tab in front, as it does for a link pressed in another app. The next is handed when a stated time has passed. Nothing in the app is automated, and nothing is asked of the system that needs leave to control another app.

**The tabs behave as they would for a person:** the app's own limit of live tabs and its own sleep times are in force (053, 054), which is the point of the run. The script says which were in force.

**What is measured,** at stated moments (when the last site has been opened; and after a stated time of doing nothing), by the system's own counters for each process:

- **the app's own process:** what the system counts against it (its footprint), and the energy it has used;
- **the engine's processes:** the same, summed. On a Mac they are WebKit's (for pages, for the network, for drawing), which the system starts, and which are not the app's children. The ones that are the app's are told by this: they were not running before the app started, and they are gone when the app has ended. A process of the engine's that outlives the app is somebody else's, and is not counted;
- **both together.**

**What a run says of itself:** the machine and its memory, the system's version, the app's version and build, the engine's version, the day, the list (by its name and a count: the addresses are the list's own file), the times it waited, how many tabs were alive and how many asleep at each measure, and every run's numbers beside the middle one.

**Nothing a person visited is in a log** (the project's rule): the list is the script's input, of the script's user's choosing, and its addresses are in the report only as the list's file says them.

## 4. The contract

None foreseen: fake mode, links from outside and the list of what is awake are all there. If the count of tabs alive and asleep cannot be had from outside the app without automating it, the report leaves it out and says so (§8), and a way to ask the app is a later choice.

## 5. In what order

1. **The engine's processes, found and measured,** in the script of 060 §5.2: on the standard session first, where the plan has a number for the app's own processes and says the engine's are counted separately. Tried on this Mac with the test site.
2. **The app started through the system, in fake mode,** and addresses handed to it: tried with the test site's pages.
3. **The run:** a list, so many runs, the measures, the report. Tried here with a list of the test site's pages, which shows that the script works and nothing about the web.
4. **The owner's:** the list of sites; the run; what is published.

## 6. Decisions

**All of these are recommendations** (§ Status). The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | The run is in fake mode | Nothing of a person's is read, written or left; the same start for everybody who repeats it | The real app on a profile of the run's own: nearer to a person's app (a session, icons, history), and it leaves a store behind to clear |
| D2 | Sites are opened by handing the app addresses from outside, a stated time apart | It needs no automation, no leave from the system and no new request in the contract; and a link from another app is a thing people do | The app driven through its window by a UI test: it can wait for each page to finish, and needs Xcode and leave to control the app. Or a list given to fake mode at the start: a new option, for a run alone |
| D3 | The engine's processes are those that began after the app and ended with it | It uses only what the system tells any program. Asking the system which app a process works for is not a public thing to ask | The app's own count of its web views' processes: a new request, and WebKit does not say a web view's process in public either |
| D4 | The app's own limits and sleep times are in force, and the report says them | The run is of the product. Fifty tabs that all stay alive would be a run of the engine | Sleeping turned off, for a number that is the engine's alone, beside the other |
| D5 | The script measures this app and no other | 001: nothing here starts another app; and a fair run of another browser is that browser's people's to script | A script that starts several browsers by name |
| D6 | The list of sites is a file the run is given; the repository has the test site's list, and a list of public sites only when the owner has chosen one | A list of other people's sites in a public repository is a statement about them, and is the owner's to make | A list written here, from a published ranking |

## 7. Tests

To be filled in as built.

## 8. What is not verified

To be filled in as built.

- **The choices of §6** are the recommendations; the owner has not looked at them.
- **A run over the public web:** not done, and not to be done here without the owner's word (§2).

## 9. Done checklist

- [ ] The owner's word on §6, on the list of sites, and on the run.
- [ ] The engine's processes, found and measured.
- [ ] The app started through the system, and addresses handed to it.
- [ ] The run and its report, tried with the test site.
- [ ] The gate.

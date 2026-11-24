# 053 — A tab sleeps by itself (W3)

| | |
|---|---|
| **Status** | **Designed, not built.** The choices of §6 are recommendations, put to the owner and not yet answered |
| **Phase** | W3 of [000](000-master-plan.md) §9. 000 §5.7: "A tab goes to sleep when any of these holds: it has been in the background past the sleep time (30 minutes by default on desktop); the system reports memory pressure; the number of live tabs passes the device's limit". The third of W3: the first of those three, the sleep time |
| **Contract** | 1.46, to be added (§4): only what a suite needs to see a tab sleep without waiting half an hour |
| **Builds on** | [051](051-a-tabs-kept-state.md): what a tab that sleeps keeps, and how it wakes. [052](052-what-keeps-a-tab-awake.md): the reasons a tab is not put to sleep. [039](039-sites.md): a Site sleeps at its home. [049](049-tidying-temporary-tabs.md): the core's look, once a minute |
| **Last updated** | 2026-11-24 |

## 1. User story

I have forty tabs open. Most of them I have not looked at since this morning, and each still holds a whole page in memory. They should not be costing me that; and when I go back to one, it should be where I left it.

## 2. What is in it, and what is not

**In:** a tab that has been **out of sight for the sleep time** is put to sleep by the browser. Its engine's state is kept ([051](051-a-tabs-kept-state.md)), its web view is let go, and it stays in its row where it was, as a tab that came back at a start does: asleep. Pressed, it wakes as it was. A tab with a reason to be kept awake ([052](052-what-keeps-a-tab-awake.md)) is passed over, for as long as the reason lasts.

**Not in, each with where it goes:**

- **The limit of live tabs,** and what kind of device this is: the next of W3. It needs the shells to say what they run on.
- **The system short of memory:** after that. It needs a shell to say so.
- **A setting for the sleep time,** and "never": with settings. Until then the time is the plan's, half an hour.
- **A Site marked "keep awake":** with the Sites' own settings.
- **A mark for why a tab is awake,** and a different look for a tab the browser put to sleep: none. It looks as a sleeping tab looks today.
- **The Memory view,** and the budgets as tests: later in W3.

## 3. The rule

**Out of sight.** A tab is in sight when it is the tab in front of the row its window shows. Every other tab that has a web view is out of sight: the tabs behind, and the tabs of a Space no window shows, the one in front of its row among them.

**The time is counted by the core's own look,** once a minute while the app runs, as tidying's is (049). At each look, a tab that is out of sight and has no reason to be awake has been so a minute longer; one that is in sight, or has a reason, starts again from nothing. At thirty it is put to sleep. No clock is read: a tab's time is so many looks, and the hours a machine itself is asleep are not counted.

**A reason is a reason for as long as it lasts** (052): sound, the camera or the microphone, typed text, a download, a question, a load. A tab that plays a song for two hours and then stops sleeps half an hour after the song.

**Putting a tab to sleep.** The core asks the shell for the web view's state and, with the same breath, to destroy the web view: the two are sent together, in that order, and the shell does them in that order. The tab is asleep from that moment: it has no web view, and says the address and the title it had. When the state arrives it is kept as the tab's. A state the tab had already, from when it left the front, stays the tab's when the shell has none to give now: it is of the same page.

**Waking** is 051's: brought in front, a sleeping tab has a web view made from its state, or loads its address where it has none. A tab that is woken before its state has arrived loads its address.

**A Site** that has been out of sight for the time sleeps as a Site does when it is closed (039): at its home, with no state (051 D4).

**A private window's tabs are left awake.** Nothing of a private window is written ([037](037-private-windows.md)), a state least of all: a private tab that slept would wake with nothing of where it was. They are few, and they go with their window.

**What a person sees:** the tab, in its row, with the name and the icon it had, said to be asleep where a sleeping tab is said to be. Nothing moves.

**Why the core.** When a tab has been out of sight, why it is kept awake, and what it keeps are the model's. A shell does what it does already: hands over a state, lets a web view go, makes one when a tab wakes.

## 4. The contract (1.46), to be added

Nothing a shell does is new. One thing is added for suites:

```proto
// FakeOptions: in fake mode, how long a tab is out of sight before it is put to sleep, in
// seconds, for a suite that must see one sleep; the core then looks every second. 0: the
// usual time, half an hour, looked for every minute.
uint32 sleep_after_seconds = 4;
```

## 5. In what order

1. **The core:** the count, what stops it, putting a tab to sleep and its state arriving after, a Site, a private window. Against the scripted engine, with the minutes passed by the test.
2. **The apps:** nothing to build; a UI test on each system, with a sleep time of seconds: a tab behind is seen to sleep, and pressed it wakes where it was, with its way back.

## 6. Decisions

**All of these are put to the owner.** The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | Half an hour, on every system, with no setting yet | It is the plan's time for a desktop. A phone's own limit of live tabs (the next step) does most of the work there, and one time for all is one thing less to explain | A shorter time on a phone. Or nothing sleeps until there is a setting to turn it off: safer, and the memory is not saved |
| D2 | The time is counted in the app's own minutes, not by the clock | A laptop that was shut for the night would otherwise put every tab to sleep at once on opening, the one being read before it was shut among the first to be looked for. And the core reads no clock it need not | By the clock: a tab not looked at since yesterday is asleep as soon as the app is looked at again |
| D3 | A tab with a reason starts its time again when the reason ends | The song that stops, the download that finishes: half an hour from then, as if the tab had just been left | The time runs on behind a reason, and a tab sleeps the minute its reason ends |
| D4 | A private window's tabs do not sleep | A private tab can keep nothing to wake with (037), so sleeping it would lose the person's place for a little memory | They sleep, and wake at their address; or their state is kept in memory only, which saves less |
| D5 | A Site sleeps by the time as it does when closed: at its home | One rule for a Site that sleeps, however it came to (039, 051 D4) | A Site that sleeps by the time keeps where it was, as a tab does |

## 7. Tests

To be filled in as built.

## 8. What is not verified

To be filled in as built. Known before a line is written:

- **A page that is doing something the browser cannot see** (waiting for a message, counting down, sending a file) has no reason to be kept awake by 052, and sleeps with the rest. What it was doing stops. This is the cost every browser that sleeps tabs has, and the Site marked "keep awake" is the plan's answer to it; it is not built.

## 9. Done checklist

- [ ] The owner's word on §6.
- [ ] The core and the contract.
- [ ] The apps' tests.
- [ ] The gate.

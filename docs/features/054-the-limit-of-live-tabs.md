# 054 — No more live tabs than the device can hold (W3)

| | |
|---|---|
| **Status** | **Built and tested, on the recommendations of §6, which are provisional:** they are put to the owner and not yet answered. On every system the core keeps no more tabs alive than the device holds: three on a phone, six on a tablet, by its memory on a desktop; the ones longest out of sight sleep, keep what [051](051-a-tabs-kept-state.md) has them keep, and wake as they were. Each shell says what it runs on. The system's own shortage of memory is not built |
| **Phase** | W3 of [000](000-master-plan.md) §9. 000 §5.7: "A tab goes to sleep when … the number of live tabs passes the device's limit (3 on a phone, 6 on a tablet or open foldable, set by memory on a desktop)." The fourth of W3 |
| **Contract** | 1.47 (§4): what a shell says of the device it runs on, and a suite's limit |
| **Builds on** | [053](053-a-tab-sleeps-by-itself.md): putting a tab to sleep, and the tabs that are passed over. [052](052-what-keeps-a-tab-awake.md): the reasons. [051](051-a-tabs-kept-state.md): what a sleeping tab keeps |
| **Last updated** | 2026-11-24 |

## 1. User story

On my phone I open links one after another, each in a tab of its own, and read the last. Half an hour is a long time on a phone: by then the system has shut the browser for having ten pages alive. The browser should keep alive the few I am using and let the rest sleep at once, so that it is still there when I come back to it.

## 2. What is in it, and what is not

**In:** each device has a **limit of live tabs**: tabs that have a web view. When a tab wakes, or is opened, and the tabs that are alive are then more than the limit, the browser puts to sleep the ones that have been **longest out of sight**, down to the limit, as [053](053-a-tab-sleeps-by-itself.md) puts a tab to sleep: its state kept, its web view let go, the tab where it was. The limit is the core's to work out, from what the shell says of the device: what kind it is, and how much memory it has.

**Not in, each with where it goes:**

- **The system saying it is short of memory:** the next of W3. It needs a shell to hear the system say so.
- **A foldable that is opened or shut** while the app runs, and a window that is made wide or narrow: the device is what it was when the app started. With foldables (W8).
- **A setting for the limit:** with settings.
- **The Memory view,** with its "sleep all others": later in W3.

## 3. The rule

**What is counted.** A tab is alive when it has a web view: the tab in sight, the tabs behind it, the tabs of Spaces no window shows, in every window. A private window's tabs are alive like any other, and are counted.

**The limit** is by the kind of device:

- **a phone:** three;
- **a tablet:** six;
- **a desktop:** one for each half gigabyte of the machine's memory, and no fewer than eight. Sixteen on a machine of eight gigabytes, thirty-two on one of sixteen.

A shell that does not say what it runs on is taken for a desktop with no memory to speak of: eight.

**When it is looked at.** Whenever anything has changed, and never on a timer: a tab that comes alive (opened with an address, or woken) is what makes the count grow, and a reason that ends is what lets a tab that was passed over sleep after all.

**Who sleeps.** Of the tabs that are alive and may be put to sleep, the one that has been out of sight the longest, then the next, until the tabs alive are no more than the limit. A tab has been out of sight since it left the front; one that came alive behind, and was never looked at, since it came. A tab may be put to sleep when 053 would let it: it is out of sight, it has no reason to be awake (052), and it is of no private window. The tab that came alive last is never one of them, in sight or not, until another comes alive: a tab opened behind the one being read is not put back to sleep by its own arrival.

**When there is nobody to put to sleep,** the limit is passed: a phone with four tabs that each play a sound has four alive. The limit is a wish for the memory, and never a reason to stop a song or lose a draft. When the song stops and the limit is passed still, the tab that played it sleeps then. What is done when the system itself says it is short of memory is the next step's.

**Half an hour still holds** (053): on a desktop most tabs go by the time, and the limit is seldom met; on a phone the limit does the work.

**Why the core.** Which tabs are alive, which left the front when, and which have reasons are the model's. A shell says what it runs on, once.

## 4. The contract (1.47)

```proto
// CoreConfig. The device, as the shell finds it when it starts (054): the core works out
// from it how many tabs may be alive at once. A window made wide or narrow, or a foldable
// opened, after that changes nothing.
DeviceKind device_kind = 10;
// The device's memory, in megabytes. 0 when the shell does not know.
uint32 memory_megabytes = 11;

enum DeviceKind {
  DEVICE_KIND_UNSPECIFIED = 0;   // taken for a desktop
  DEVICE_KIND_PHONE = 1;
  DEVICE_KIND_TABLET = 2;
  DEVICE_KIND_DESKTOP = 3;
}

// FakeOptions. The most tabs that are alive at once, for a suite that must see the limit
// met. 0: none. Fake mode has no limit of its own, whatever the device: a suite that opens
// five tabs on a phone is about something else.
uint32 live_tabs = 5;
```

Nothing else a shell does is new: a tab the limit puts to sleep is asked for its state and its web view let go, as 053 has it.

**What each shell says:**

- **A Mac** is a desktop. **An iPad** is a tablet; anything else the app runs on there, **an iPhone**, a phone. The memory is the device's own (`ProcessInfo.physicalMemory`), in megabytes. A suite's limit is `WEBTIGER_FAKE_LIVE_TABS=<count>`.
- **Android:** a screen whose narrower side is 600dp or more is a tablet's, as the system itself draws the line (`sw600dp`); a narrower one, a phone's. The memory is what the system says the device has (`ActivityManager.MemoryInfo.totalMem`). A suite's limit is `--ei webtiger.fake.live_tabs <count>`.
- **Windows** is not built. A shell that says nothing is taken for a desktop with no memory to speak of, and holds eight.

**As built, where the design did not say:**

- **The limit is looked at whenever anything has changed,** not only when a tab comes alive. The design had it looked at on arrival alone, which would have left a phone over its limit, once a song had stopped, until the next tab was opened.
- **The tabs that came alive last are the ones spared,** all of them when several came together (two links asked for in the same breath), and for as long as no other comes alive: a tab opened behind stays, loaded and with no reason to be awake, however many are over.
- **Several that are over sleep at once:** a window is told once, and never of one asleep and the other not.
- **A tab that has just left the front can be the one,** when no other can. It is then asked for its state twice, as it left the front and as it sleeps; the second is the one kept.
- **A tab with nowhere to wake** (one a page opened, in which nothing has arrived: 053) is passed over, and the next is put to sleep in its place.
- **A Space no window shows** has its tabs counted, and they sleep with the rest, the one in front of its row among them. Showing the Space again wakes that one, which is a tab come alive.
- **After a start** the session's tabs are asleep but the one in front, as they always are (009). Each that is woken is a tab come alive.
- **A phone's limit and a tablet's are theirs whatever the memory.** A desktop's is by whole half gigabytes: 6,500 megabytes hold twelve.
- **A kind the core has not heard of** (a shell built with a later contract) is taken for a desktop.

## 5. In what order

1. **The core:** the limit from the device, who sleeps and in what order, who is passed over, the tab that has just come alive. Against the scripted engine. **Built.**
2. **The shells:** each says what it runs on: a Mac, an iPhone or an iPad; an Android phone or tablet. A test of each that it says so; and in the apps, with a limit of two given by the suite, a third tab's arrival putting the first to sleep. **Built:** in the apps on Apple's systems, and on an iPhone with nothing said of a limit; through the real core under Android's shell on the JVM, in fake mode and started for real as a phone (Android's device tests share one core, started once in fake mode, with no limit).

## 6. Decisions

**All of these are put to the owner.** The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | Three on a phone and six on a tablet, as the plan has it | The plan's numbers. A phone's system ends an app that holds much more | Fewer on a phone with little memory; more on one with much. The memory is said to the core either way, and the rule can come to use it |
| D2 | On a desktop, one live tab for each half gigabyte of memory, and no fewer than eight | The plan says "set by memory" and no more. A page that is alive costs between a tenth and a half of a gigabyte; half the machine for pages, at the dearer end, is the rule | A flat number. Or by the memory that is free at the moment, which a shell would have to keep saying |
| D3 | The limit is passed when every other tab has a reason to be awake | A limit that stopped a song or lost a draft to keep a count would be the browser working against the person. The system's own shortage is another matter, and the next step's | The reasons are ranked, and the weakest loses |
| D4 | A private window's tabs are counted, and are never the ones put to sleep | They hold memory like any tab; and they can keep nothing to wake with (053 D4) | Not counted: a phone with three private tabs then has six alive |
| D5 | Fake mode has no limit unless a suite gives one | The apps' suites open many tabs on a phone's simulator for reasons of their own. A limit of three would put half of every such suite's tabs to sleep under it | The device's own limit in fake mode, and every suite written to it |

## 7. Tests

| Where | What |
|---|---|
| The core, through the contract (`live_tabs.rs`, 15), against the scripted engine | **The limit by the device,** started for real over a store of the test's: a phone holds three and a tablet six whatever their memory; a desktop sixteen with eight gigabytes, thirty-two with sixteen, twelve with 6,500 megabytes, and eight with two gigabytes or with none said; a shell that says no kind, or one the core has not heard of, is a desktop. On each, as many tabs as it holds are all alive, and one more puts the first to sleep. **Fake mode:** twelve tabs alive on a phone; with a suite's limit of two, the third puts the first to sleep. **Who sleeps:** of two tabs out of sight, the one that left the front first: its state asked for and then its web view let go, in that order, the tab where it was in its row with the name and the address it had; pressed, it wakes from its state where it was, with the list it had, and the tab that is then the longest out of sight sleeps in its turn. A tab that has just left the front sleeps when it is the only one that can, and each of two tabs wakes from its state as the other sleeps. **Passed over:** a tab that plays a sound and one that was typed into, so that three are alive where two are held; a fourth then puts the third to sleep, though the others have been out of sight longer; and when the song stops, its tab sleeps at once, with no tab come alive. A tab on its way is passed over, and sleeps where it arrives. **Come alive behind:** a tab opened behind the one being read stays, loaded and with no reason, whatever else happens, until another comes alive behind it; tabs opened behind sleep in the order they came; one that came alive behind after another tab left the front is the later out of sight. **Together:** two asked for in the same breath are both spared, the two longest out of sight sleep, and no window was told of one asleep and the other not. **A private window's tabs** are counted (an ordinary window's tab sleeps that would not, were they not) and are never put to sleep, the limit being passed when only they are left. **A Site** sleeps at its home, asked for no state, and wakes by being loaded there. **A Space no window shows:** its tab is counted and sleeps; shown again, it wakes from its state, and one of the other Space's sleeps. **Nowhere to wake:** a tab a page opened is passed over, though it is the longest out of sight, and the next sleeps. **Over a start:** a tab the limit put to sleep wakes from the state it slept with, and three tabs woken on a phone are as many as are alive |
| The core broken, a part at a time | 25 parts, 25 caught. The first run caught 23, and the two it did not are why two tests are as they are: tabs opened behind slept in the order they came even with nothing saying when they came (their numbers gave the same order), so one is now opened behind after another tab has left the front; and two tabs put to sleep one look apart looked the same as two put to sleep at once, so the test now reads what the window was told on the way. One line of the first writing was shown to do nothing, and is gone |
| The apps, in fake mode, with the real core and the real engine and a limit of two (`LiveTabsTests`, 1, on the Mac and the iPhone simulator) | Two tabs are awake. A third's arrival puts the first to sleep: it says so, by the name it had; the tab that has just left the front and the tab in front are awake. Pressed, the first wakes at the page it was at, the second sleeps in its turn, and Back in the first goes to the page before it slept |
| The app itself, on a profile of the suite's own, nothing said of a limit (`LiveTabsTests`, +1, on the iPhone simulator; and once by hand on an iPad's, outside the gate) | On the iPhone three tabs are awake; a fourth puts the first to sleep, and the other three are awake: the app said it runs on a phone, and the core held three. On the iPad's simulator the same test holds six, and a seventh puts the first to sleep |
| Apple's shell (`DeviceTests`, 2, on the Mac and the iPhone simulator) | A Mac says it is a desktop, the simulator's iPhone a phone; an iPad's idiom is a tablet's and any other a phone's; the memory is megabytes, and more than a gigabyte. A suite's knobs reach fake mode's options, each by its name, and what is no count is no limit |
| Android, on the JVM (`DeviceTest`, 2; `LaunchOptionsTest`, +1; `ThroughTheCoreTest`, +3) | A screen is a tablet's from 600dp on its narrower side, and one of no known size a phone's; memory is whole megabytes. A launch's numbers (the clock's step, the sleep time, the limit) are taken from its extras in a debuggable build's fake mode, and from nowhere else. The device and a suite's limit are in what the core is started with. **Through the real core:** with a limit of two, a third tab's arrival has the first asked for its state once more and its page let go, the window says it is asleep, the tab that has just left the front stays; brought in front, the first has a web view made from the state it gave, and the second sleeps in its turn. **Started for real as a phone,** over a folder of the test's: three tabs are alive, and a fourth puts the first to sleep |
| Android, on a device (`DeviceOnADeviceTest`, 2) | The kind the app will say is the one the screen is of, as the display measures it; the memory is within a twentieth of what the kernel lists |
| The shells broken, a part at a time | **Apple's:** 9 parts, 9 caught by `DeviceTests` on the Mac (a Mac said to be a phone, memory in bytes or kilobytes, each of a suite's knobs not handed on or read by another's name); and 2 of 2 by the iPhone's own UI test (the app not saying what it runs on; an iPhone said to be a tablet). **Android's:** 12 parts on the JVM, 12 caught; the first run caught 11, a size below nothing having been tried with a number that came out as nothing either way. And 3 of 3 on the emulator (the memory that is free said for the memory there is; the kind taken from the screen's height; the system not asked) |

## 8. What is not verified

- **The choices of §6** are the recommendations, not the owner's word.
- **The numbers.** Three, six, and half a gigabyte a tab are the plan's and a guess (D1, D2): nothing measured what a live tab costs on any system, or how many a phone holds before its system ends the app. The budgets are later in W3.
- **A Mac's own limit, in the app.** This Mac would hold too many tabs for a UI test to open. That the Mac says it is a desktop, with its memory, is `DeviceTests`; what the core makes of that is the core's tests; the two were not seen together in the app.
- **An iPad.** On an iPad's simulator, once, by hand: the gate runs the iPhone's. Not on an iPad.
- **A tablet and a foldable on Android.** The rule (600dp) is tested as a function, and on the emulator, which is a phone. No tablet, and no foldable, opened or shut, was tried.
- **On Android, in the app on a device.** Its device tests share one core, started once in fake mode, where there is no limit: the app's part is the JVM's tests through the real core, and the device's own account of itself.
- **A phone.** Android's part ran on the JVM and an emulator; Apple's on a simulator.
- **Memory given back.** That a tab the limit puts to sleep gives its memory back was not measured (053 §8).
- **A window made wide or narrow, and a foldable opened,** while the app runs: the device is what it was at the start (§2).

## 9. Done checklist

- [ ] The owner's word on §6.
- [x] The core and the contract.
- [x] The shells.
- [ ] The gate.

# 054 — No more live tabs than the device can hold (W3)

| | |
|---|---|
| **Status** | **Designed, not built.** The choices of §6 are recommendations, put to the owner and not yet answered |
| **Phase** | W3 of [000](000-master-plan.md) §9. 000 §5.7: "A tab goes to sleep when … the number of live tabs passes the device's limit (3 on a phone, 6 on a tablet or open foldable, set by memory on a desktop)." The fourth of W3 |
| **Contract** | 1.47, to be added (§4): what a shell says of the device it runs on |
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

**When it is looked at.** Whenever a tab comes alive: it is opened with an address, or woken. Not on a timer: the count only grows then.

**Who sleeps.** Of the tabs that are alive and may be put to sleep, the one that left the front longest ago, then the next, until the tabs alive are no more than the limit. A tab may be put to sleep when 053 would let it: it is out of sight, it has no reason to be awake (052), and it is of no private window. The tab that has just come alive is never one of them, in sight or not: a tab opened behind the one being read is not put back to sleep by its own arrival.

**When there is nobody to put to sleep,** the limit is passed: a phone with four tabs that each play a sound has four alive. The limit is a wish for the memory, and never a reason to stop a song or lose a draft. What is done when the system itself says it is short of memory is the next step's.

**Half an hour still holds** (053): on a desktop most tabs go by the time, and the limit is seldom met; on a phone the limit does the work.

**Why the core.** Which tabs are alive, which left the front when, and which have reasons are the model's. A shell says what it runs on, once.

## 4. The contract (1.47), to be added

```proto
// CoreConfig: the device, as the shell finds it when it starts.
DeviceKind device_kind = …;
// Its memory, in megabytes. 0 when the shell does not know.
uint32 memory_megabytes = …;

enum DeviceKind {
  DEVICE_KIND_UNSPECIFIED = 0;   // taken for a desktop
  DEVICE_KIND_PHONE = 1;
  DEVICE_KIND_TABLET = 2;
  DEVICE_KIND_DESKTOP = 3;
}

// FakeOptions: in fake mode, the limit of live tabs, for a suite that must see it met.
// 0: no limit. (Fake mode has none of its own: a suite that opens five tabs on a phone is
// about something else.)
uint32 live_tabs = …;
```

## 5. In what order

1. **The core:** the limit from the device, who sleeps and in what order, who is passed over, the tab that has just come alive. Against the scripted engine.
2. **The shells:** each says what it runs on: a Mac, an iPhone or an iPad; an Android phone or tablet. A test of each that it says so; and in the apps, with a limit of two given by the suite, a third tab's arrival putting the first to sleep.

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

To be filled in as built.

## 8. What is not verified

To be filled in as built.

## 9. Done checklist

- [ ] The owner's word on §6.
- [ ] The core and the contract.
- [ ] The shells.
- [ ] The gate.

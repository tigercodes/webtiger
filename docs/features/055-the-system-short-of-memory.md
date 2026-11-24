# 055 — When the system is short of memory (W3)

| | |
|---|---|
| **Status** | **Designed, not built.** The choices of §6 are recommendations, put to the owner and not yet answered |
| **Phase** | W3 of [000](000-master-plan.md) §9. 000 §5.7: "A tab goes to sleep when any of these holds: … the system reports memory pressure". The fifth of W3: the last of the three ways a tab comes to sleep by itself |
| **Contract** | 1.48, to be added (§4): a shell says the system is short of memory |
| **Builds on** | [053](053-a-tab-sleeps-by-itself.md): putting a tab to sleep. [052](052-what-keeps-a-tab-awake.md): the reasons. [054](054-the-limit-of-live-tabs.md): the limit, and the tabs that have just come alive. [051](051-a-tabs-kept-state.md): what a sleeping tab keeps |
| **Last updated** | 2026-11-24 |

## 1. User story

My laptop is busy: a video call, a build, a dozen tabs. The system says it is running out of memory. The browser should give back what it is not using at once, and not wait half an hour to do it; and it should not be the app the system ends to make room, taking the page I am reading with it.

## 2. What is in it, and what is not

**In:** a shell hears the system say it is short of memory and tells the core, with how short. The core puts to sleep, at once, the tabs that are out of sight and have no reason to be awake, as [053](053-a-tab-sleeps-by-itself.md) puts a tab to sleep: the state kept, the web view let go, the tab where it was. When the system says it is **very** short, tabs that are only on their way somewhere sleep too.

**Not in, each with where it goes:**

- **Giving back anything but tabs:** the engine's caches, the core's own. No engine here offers a way to ask, and the core keeps little. With the budgets, later in W3, if a measurement shows something worth giving.
- **Telling the person:** nothing is said. A tab that slept looks as a sleeping tab does. A mark for why is with the marks for the reasons.
- **A renderer the system has ended** (Android's `onRenderProcessGone`): built already, in 003. This is about being told before that.
- **The Memory view,** with its "sleep all others": later in W3.

## 3. The rule

**Two words from a shell: short, and very short.** Each system has its own (§4); a shell says which of the two it heard, each time it hears it.

**Short.** Every tab that is out of sight, has no reason to be awake (052) and is of no private window is put to sleep now: whatever its time out of sight, and however few tabs are alive. The tabs that have just come alive (054) are not spared: a tab opened behind a moment ago loses a load, and the memory is wanted now.

**Very short.** The same, and also a tab out of sight whose only reason to be awake is that it is **on its way** somewhere: it sleeps without its state and wakes where it was going, as a tab whose load failed does (053). A tab that plays a sound, holds the camera or the microphone, was typed into, has a download coming or a question waiting stays: those are the person's, and are not the browser's to take, however short the system is.

**A private window's tabs stay,** as in 053 and 054: they could keep nothing to wake with.

**The tab in sight stays,** always.

**Nothing lasts.** A report is acted on once, when it comes. The limit of live tabs is what it was, and a tab that is woken a moment later wakes. A system that is short still says so again.

**Why the core.** Which tabs are out of sight, which have reasons, and how one is put to sleep are the model's. A shell passes on what the system said.

## 4. The contract (1.48), to be added

```proto
// Request. The system has said it is short of memory. A notification: it gets no answer
// worth waiting for.
message MemoryShort {
  MemoryLevel level = 1;
}

enum MemoryLevel {
  MEMORY_LEVEL_UNSPECIFIED = 0;   // taken for SHORT
  MEMORY_LEVEL_SHORT = 1;
  MEMORY_LEVEL_VERY_SHORT = 2;
}

// FakeOptions: in fake mode, the core acts as though it had been told the system is short
// of memory, every so many seconds, for a suite that must see what follows. 0: never.
uint32 memory_short_every_seconds = …;
```

**What each shell hears** (read from each system's documents on 2026-11-24; to be tried as it is built):

- **A Mac:** the system's memory-pressure source (Dispatch's): its warning is short, its critical is very short. It tells of a change, not of a state: an app started while the system is already short is told nothing until it changes. Apple's own answer is that pressure cannot be made by a program for a test: the tests stand in for the source.
- **An iPhone and an iPad:** the same source, and the app's memory warning, which has one word and after which the system ends apps: very short.
- **Android:** less than the plan supposed. The system's documents say an app that is running is **not told** it is short of memory from Android 14 on: the levels `TRIM_MEMORY_RUNNING_LOW` and `RUNNING_CRITICAL` are "not notified … since API level 34", and `onLowMemory` "is never called". Before Android 14 the shell hears them: low is short, critical is very short. From Android 14 the only thing to hear is the answer to a question: the system will say, when asked, whether it considers itself low on memory (`ActivityManager.MemoryInfo.lowMemory`). The shell asks once a minute while the app is in front, and says short when the answer turns to yes (D7).
- **Windows:** not built.

## 5. In what order

1. **The core:** who sleeps at each level, who stays, the tabs that have just come alive, a tab on its way. Against the scripted engine.
2. **The shells:** each hears its system and says so. A test of each that what the system says reaches the core as the right word, made without making the system short: the app's own notification posted in the test's process on an iPhone, the Mac's source stood in for, the Application's own callback called on an Android device. And in the apps, in fake mode with the core told every few seconds: tabs left behind are asleep seconds later, and the one in front is not.

## 6. Decisions

**All of these are put to the owner.** The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | Short: every tab out of sight with no reason to be awake sleeps at once | The system asked. A sleeping tab wakes as it was (051), so what is given up is a second's wait on coming back | Only the half that has been out of sight longest; or down to half the device's limit |
| D2 | The tabs that have just come alive are not spared, as they are by the limit (054) | The limit spares them because sleeping them gains nothing it was asked for. Here memory was asked for, now | Spared when short, not when very short |
| D3 | Very short: a tab that is only on its way sleeps too; a song, a call, a draft, a download and a question stay | A load can be begun again and loses nothing. The others are the person's. If the system ends the app after all, they are lost with it: but that is the system's doing, and may not happen | Very short takes everything out of sight but a draft; or everything |
| D4 | A private window's tabs stay | They can keep nothing to wake with (053 D4) | They sleep when very short, and wake at their address |
| D5 | A report is acted on once, and changes nothing after | Simple to say and to test. A system still short says so again | For some minutes after, the limit of live tabs is halved |
| D6 | In fake mode the core is told every few seconds, by a knob of the suite's | A UI suite cannot make a system short of memory, and must not try: it is everybody's memory | A test-only way to tell a running app, from outside it |
| D7 | On Android 14 and later, where a running app is told nothing, the shell asks the system once a minute while the app is in front, and says short when the answer turns to yes | It is the one thing the system offers there. A minute is as often as the core looks for tabs to put to sleep (053). The limit of three (054) does most of the work on a phone in any case | Nothing on Android 14 and later: the limit alone. Or tabs behind sleep whenever the app leaves the screen, which the system does still tell an app: it saves the most, and costs a reload on coming back |

## 7. Tests

To be filled in as built.

## 8. What is not verified

To be filled in as built.

## 9. Done checklist

- [ ] The owner's word on §6.
- [ ] The core and the contract.
- [ ] The shells.
- [ ] The gate.

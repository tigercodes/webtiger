# 055 — When the system is short of memory (W3)

| | |
|---|---|
| **Status** | **Built and tested, on the recommendations of §6, which are provisional:** they are put to the owner and not yet answered. A shell that hears its system say it is short of memory says so to the core, and the core puts to sleep at once the tabs that are out of sight with no reason to be awake; when the system is very short, the tabs that are only on their way too. Built in the core and in both shells. What could not be tried is in §8: no system was made short of memory |
| **Phase** | W3 of [000](000-master-plan.md) §9. 000 §5.7: "A tab goes to sleep when any of these holds: … the system reports memory pressure". The fifth of W3: the last of the three ways a tab comes to sleep by itself |
| **Contract** | 1.48 (§4): a shell says the system is short of memory, and how short |
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

**Very short.** The same, and also a tab out of sight whose only reason to be awake is that it is **on its way** somewhere. One the person sent there sleeps without its state and wakes where it was going, as a tab whose load failed does (053). One whose page set off by itself sleeps at the page it is at, with its state, and wakes there: where it was going was never the tab's to say. A tab that plays a sound, holds the camera or the microphone, was typed into, has a download coming or a question waiting stays, and so does one that is on its way and has one of those besides: they are the person's, and are not the browser's to take, however short the system is.

**A private window's tabs stay,** as in 053 and 054: they could keep nothing to wake with.

**The tab in sight stays,** always.

**Nothing lasts.** A report is acted on once, when it comes. The limit of live tabs is what it was, and a tab that is woken a moment later wakes. A system that is short still says so again.

**Why the core.** Which tabs are out of sight, which have reasons, and how one is put to sleep are the model's. A shell passes on what the system said.

## 4. The contract (1.48)

```proto
// Request. The system has said it is short of memory, and how short. Answered with Ack,
// which a shell need not wait for; it may be sent as a notification.
message MemoryShort {
  MemoryLevel level = 1;
}

enum MemoryLevel {
  MEMORY_LEVEL_UNSPECIFIED = 0;   // taken for short, as a level this core does not know is
  MEMORY_LEVEL_SHORT = 1;
  MEMORY_LEVEL_VERY_SHORT = 2;
}

// FakeOptions: in fake mode, the core acts as though it had been told the system is short
// of memory, every so many seconds, for a suite that must see what follows. 0: never.
uint32 memory_short_every_seconds = 6;
```

A shell hands the suite's seconds on from what it was started with: `WEBTIGER_FAKE_MEMORY_SHORT_EVERY=<seconds>` on Apple's systems, `--ei webtiger.fake.memory_short_every <seconds>` on Android.

**A shell in fake mode does not listen to its system.** A UI suite's run must not go by how busy the machine it runs on is: a Mac that builds, runs a simulator and an emulator at once is short of memory often enough, and a tab that slept for that would fail a test about something else. Fake mode has the knob; the app itself listens.

**What each shell hears, as built:**

- **A Mac:** the system's memory-pressure source (Dispatch's): its warning is short, its critical is very short, and the worse of the two when it says both. Its word that the shortage is over is passed on to nobody: nothing lasted. It tells of a change, not of a state: an app started while the system is already short is told nothing until it changes.
- **An iPhone and an iPad:** the same source, and the app's memory warning. The design took the warning for the last word before the system ends apps, and so for very short. Built, it is **short**: the warning has one word, and it comes with the source's warning as well as with its critical (and when the app alone is near what it may use). Were it very short, every warning would be, and the two levels would be one. The worse word is the source's to say. (What was seen: on the simulator, the simulator's warning reaches the source as a warning. That the app's memory warning is sent for the same event is how Apple describes the simulator's warning; it was not looked for by itself.)
- **Android:** less than the plan supposed. The system's documents say an app that is running is **not told** it is short of memory from Android 14 on: the levels `TRIM_MEMORY_RUNNING_LOW` and `RUNNING_CRITICAL` are "not notified … since API level 34", and `onLowMemory` "is never called". Before Android 14 the shell hears them: running low is short; running critical is very short, and so is "low memory", which comes when every process in the background has been ended. That the window is hidden, or that the process is one of those kept in the background, is no word about memory, and nothing is said for it. From Android 14 the only thing to hear is the answer to a question: the system says, when asked, whether it considers itself low on memory (`ActivityManager.MemoryInfo.lowMemory`). The shell asks once a minute while the app is in front, and says short **each time the answer is yes** (D7): the answer is a state and not an event, and a system that is still short is to say so again (§3).
- **Windows:** not built.

**As built in the core, where the design did not say:**

- **A level that is not said, or that this core does not know,** is taken for short: the lesser of the two.
- **All the tabs that sleep for one report sleep together:** nobody is shown a row with some of them asleep and the rest about to be.
- **A tab with nowhere to wake** (one a page opened, in which nothing has arrived) is left as it is, as by the time (053), and stops nobody after it.
- **The limit of live tabs is what it was** (054): which tabs have just come alive is counted from the tabs that are alive after the report, and a tab that wakes next is a tab come alive like any other.
- **In fake mode the knob says short,** and not very short; from the first thing a shell sends, until the core stops.

## 5. In what order

1. **The core:** who sleeps at each level, who stays, the tabs that have just come alive, a tab on its way. Against the scripted engine. **Built.**
2. **The shells:** each hears its system and says so. **Built,** each with a test that what the system says reaches the core as the right word, made without making a system short: on an iPhone the app's own notification posted in the test's process; on the simulator the system's own source heard for real, when the simulator gives its warning (which is a warning and no shortage); on a Mac the source's words handed to the listener as the source hands them; on Android the system's callbacks called as the system calls them, and on a device the application told by its own way of being told. And in the apps, in fake mode with the core told every three seconds: tabs left behind are asleep seconds later, and the one in front is not.

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
| D7 | On Android 14 and later, where a running app is told nothing, the shell asks the system once a minute while the app is in front, and says short each time the answer is yes | It is the one thing the system offers there. A minute is as often as the core looks for tabs to put to sleep (053). The limit of three (054) does most of the work on a phone in any case | Short only when the answer turns to yes, and not again while it stays yes. Or very short, since the system's threshold for "low" is where it begins to end services. Or nothing on Android 14 and later: the limit alone. Or tabs behind sleep whenever the app leaves the screen, which the system does still tell an app: it saves the most, and costs a reload on coming back |
| D8 | An iPhone's memory warning is short, and very short is left to the source's critical | The warning comes with the source's warning too (§4): as very short it would make every warning very short | Very short, as first designed: the last word before the system ends apps, whatever else was said |
| D9 | A shell in fake mode does not listen to its system | A suite's run must not go by how busy its machine is (§4) | It listens in fake mode too, and a suite that cannot bear it says so |

## 7. Tests

| Where | What |
|---|---|
| The core, through the contract (`memory_short.rs`, 16), against the scripted engine | **Short:** with no minute passed, every tab behind is asleep at the report, each asked for its state once more and its web view let go, all together (nobody is shown a row with some of them asleep), each where it was with the name and the address it had; the tab in sight is awake. Woken, a tab is made from its state. **Nothing lasts:** the tab it was woken over is awake a minute on, and sleeps when the system says so again; a limit of live tabs is what it was after a report. **Very short:** a tab the person sent somewhere, on its way, stays when short and sleeps when very short, asked for no state, saying where it was going, deaf to what its web view says late, and wakes by being loaded there; a tab whose page set off by itself sleeps at the page it is at, with its state, and wakes there. **What is the person's:** a tab that plays a sound, holds the camera, was typed into, has a question waiting or a download coming stays at both levels, and so does one that plays a sound and is on its way; when the song stops the tab sleeps at the next report and not before. **A level** that is not said, or not known (four tried), is taken for short. **The tabs that have just come alive** are not spared, where the limit spared them a moment before. **A private window's tabs** stay at both levels, one on its way among them. **A Site** sleeps at its home, asked for no state. **A Space no window shows:** its tabs sleep, the one in front of its row among them. **A tab with nowhere to wake** is left as it is, and the tab after it sleeps. **Over a start:** asleep, with its state and its way back. **Answered** with Ack; with nothing out of sight nobody is told anything; sent as a notification it is acted on and gets no answer. **A suite's seconds:** not after one second and after two; again every two, said by the core itself with nothing asked of it; short and not very short; and nothing outside fake mode |
| The core broken, a part at a time | 25 parts, 25 caught. The first run caught 23, and the two it did not are why two tests are as they are: a tab with nowhere to wake now has a tab after it (one such tab stopped every tab after it from sleeping, and no test had one); and what the suite's knob put to sleep is now seen with nothing asked of the core for three seconds (the test's own asking had published it) |
| The scripted shell | It can let seconds pass in which it only listens: what is seen to have changed after, the core said by itself |
| Apple's listener (`MemoryWatchTests`, 4 on the Mac and 6 on the iPhone simulator) | The source's warning is short and its critical very short, the worse of the two when both are said, and its word that a shortage is over is none. What the source says reaches the core as that word, each time, and nothing else is asked of the core. A listener that was not told to listen says nothing. Fake mode does not listen, the app itself does, and a session in fake mode over the real core has no listener. On the iPhone simulator besides: the app's memory warning, posted in the test's process to a centre of the test's own, is short each time, is heard once however often the listener is told to listen, and not once it is stopped; and **the system's own source is heard for real**: the simulator gives its warning (by the file it names to every process it runs, whose time the test changes), and the core is told short |
| Apple's listener broken, a part at a time | 17 parts, 17 caught: 11 on the Mac, and on the iPhone simulator the six a Mac does not have or cannot make speak (an iPhone's memory warning; the source heard for real). Five of those six by a test that fails. The sixth, a source that is never set going, by the test's process ending in the test of the real source: a failed run, and a crash, which the Mac says in a notice on its screen. That part is not to be broken by a run again |
| The apps, in fake mode, with the real core and the real engine, the core told every three seconds (`MemoryTests`, 1, on the Mac and the iPhone simulator) | A tab goes to a second page and two more come in front of it, one after the other; seconds on, both that were left behind say they are asleep, by the names they had (nothing else would put them there: fake mode has no limit, and the time is half an hour); the tab in front is awake seven seconds and two reports on; the first, pressed, wakes at the page it was at with Back offered, and Back goes back; and the tab it was woken over sleeps in its turn |
| The app itself on the iPhone simulator, on a profile of the suite's own (`MemoryTests`, +1) | Nothing is said of memory by the suite. Three tabs, as many as an iPhone holds; seconds on, all are awake. The simulator's warning is given: the two left behind say they are asleep, the one in sight does not, and the first, pressed, wakes where it was. The app's own listener heard its system |
| Android's listener (`MemoryWatchTest`, 5, on the JVM) | Running low is short and running critical very short, and eight other levels are no word (a hidden window and a process kept in the background among them). What the system tells a running app is passed on each time, and "low memory" is very short. Where the system tells nothing: not asked while the app is not in front; in front, asked a minute on and every minute; nothing said while the answer is no; short each minute while it is yes; told twice that it is in front it asks once a minute still; out of sight it asks no more, and in front again it asks a minute on. Where the system tells, it is not asked. The time is a virtual one: no test waits a minute |
| Android, through the real core (`ThroughTheCoreTest`, +3, on the JVM) | A suite's seconds are handed on, are nothing outside fake mode, and fake mode does not listen. The session says short: the tab left behind is asked for its state once more and let go, the window says it is asleep by the name it had, and the tab in front is awake, also when very short is said after; brought in front, it has a web view made from the state it gave. With a suite's second: asleep with the shell having said nothing |
| Android's listener broken, a part at a time | 26 parts, 26 caught. The first run caught 25: the session passed on short whatever word it was handed, and no test had a tab that only very short puts to sleep. One is now sent on its way and left behind |
| Android, on an emulator (`MemoryOnADeviceTest`, 2) | A watch registered with the application hears what the application is told, on the main thread where the system tells it, as the right word (running low short, running critical very short, running moderate none), and nothing once it is registered no longer. And the emulator's own system (Android 16) is asked its question for real, every fifty milliseconds while the watch is told the app is in front: between five and forty times in a second and a half, not once more when it is out of sight, and short is said as often as the answer was yes (it was no) |
| By hand, the app itself on the emulator | 2026-11-24, Android 16, the app started for real on the emulator's own data, with the test site served from the Mac. Three tabs opened from outside, each at a page: the app has three web views (the system's own count of them). The system is had to tell the app its memory is running low (`am send-trim-memory`): one web view is left, the tab in front's. A tab that was left behind, pressed, shows its page again: two. Running critical: one again, and the row has its four tabs still. (Android 14 and later do not say these of themselves to a running app; said by hand, they are delivered) |

## 8. What is not verified

- **The choices of §6** are the recommendations, not the owner's word.
- **No system was made short of memory,** and none may be by a test: it is everybody's memory. What was tried is that each system's word, given as the system gives it, reaches the core as the right word, and what the core then does.
- **A Mac's source, on a Mac.** The listener is the same code as on the iPhone simulator, where the system's source was heard for real. On a Mac nothing can make the source speak without making the Mac short: its hearing there was not seen.
- **The source's critical,** anywhere: the simulator's warning is a warning. That critical is very short is the listener's test with the source's word handed to it.
- **That an iPhone's memory warning comes with the source's warning,** which is why it is short (D8). On the simulator the simulator's warning was seen to reach the source as a warning; that the app's memory warning is sent for that same event is Apple's description of it, and was not looked for by itself. Nothing was tried on a device.
- **Android before 14.** The emulator here is Android 16. What the system tells a running app before Android 14 is called by the tests as the system calls it; no such system told it.
- **Android's answer of yes.** The emulator was not low on memory: the question was asked of it for real and answered no. Short for a yes is the JVM's test.
- **Whether memory comes back.** That a tab put to sleep gives its memory back was not measured (053 §8). The budgets are later in W3.
- **Windows:** not built.

## 9. Done checklist

- [ ] The owner's word on §6.
- [x] The core and the contract.
- [x] The shells.
- [x] The gate: every stage of `Tools/test.sh all` green, and `android-device`, on 2026-11-24, over one commit and in two runs: `lint`, `core` (740, and the strings tool's 24), `unit` (376 on the Mac), `build`, the Mac's UI suite (62), `ios` (360 on the iPhone simulator), the iPhone's UI suite (51), `android` (500 and 32 on the JVM) and `android-device` (192 on the emulator). The Mac's UI suite ran last and by itself: two notices of the system's were on the Mac's screen until then, and that suite does not start while one is there.

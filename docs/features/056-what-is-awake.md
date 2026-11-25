# 056 — What is awake, and "sleep all others" (W3)

| | |
|---|---|
| **Status** | **Built and tested in the core and in the shells of Apple's systems and Android, on the recommendations of §6.** The owner's word of 2026-11-24 was to use what is suggested where something waits on a choice, and to go on: they are built on, and stay the owner's to change. The core lists what is awake and why, and puts the others to sleep when asked; each shell shows the list as the fifth of the Library's ([057](057-the-library.md)), with "Sleep All Others" as its button |
| **Phase** | W3 of [000](000-master-plan.md) §9. 000 §7: "A Memory view lists live tabs and why each is awake, with a figure per tab where the engine provides one, and a 'sleep all others' command." The sixth of W3 |
| **Contract** | 1.49 (§4): the list of what is awake, and the command |
| **Builds on** | [052](052-what-keeps-a-tab-awake.md): the reasons a tab is kept awake. [053](053-a-tab-sleeps-by-itself.md): putting a tab to sleep. [055](055-the-system-short-of-memory.md): every tab that can sleep, at once. [024](024-several-spaces.md): a row for each Space. [037](037-private-windows.md): what a private window keeps to itself |
| **Last updated** | 2026-11-24 |

## 1. User story

The fan is on, or the battery is going. I want to see what the browser has alive, and why, and to put the rest to sleep with one press: without hunting through five Spaces for the tab that is playing something.

## 2. What is in it, and what is not

**In:** a list of every tab that is alive (that has a web view), in every window and every Space, each with why it is awake: that it is on screen, or one of the reasons of [052](052-what-keeps-a-tab-awake.md), or nothing, in which case it is only waiting for its time. With the list, one command: **sleep all others**, which puts to sleep at once every tab that can be, as the system's shortage of memory does ([055](055-the-system-short-of-memory.md)).

**Not in, each with where it goes:**

- **A figure of memory for each tab.** The plan says "where the engine provides one". Neither engine built here does: WebKit gives an app no count of what a web view uses, and Android's System WebView gives none either. The contract has the place for one; nothing fills it. With Chromium on the Mac (W6), which counts.
- **Putting one tab to sleep by hand,** and waking one from the list: with the list's rows, after this. A tab is woken as it always is, by going to it.
- **Marks in the row for why a tab is awake** (000 §5.7: "The sidebar shows why"): the next of W3. They are the same reasons, said where the tab is.
- **How long until a tab sleeps by itself:** not said. The time is the core's count of its own looks (053), and is not worth a clock in a list.
- **The budgets as tests,** and the published run: later in W3.

## 3. The rule

**What is listed.** Every tab that has a web view: the one in front of each window, the tabs behind it, and the tabs of Spaces no window shows. A tab that is asleep is not listed: it costs a row in the store and nothing else. The list says how many are asleep, as one number.

**The order.** By window, the asking window first; within a window by Space, the one it shows first and then the others in the Spaces' own order; within a Space, in the row's order. So the row the person is looking at comes first, as they see it.

**Why each is awake.** Each line has: that it is **in sight** (the tab in front of the row its window shows), and the reasons of 052 that hold now: sound, the camera or the microphone, typed text, a download, a question, a load. A line with neither is a tab that is only waiting for its half hour.

**A private window's tabs are not named outside it.** Asked from an ordinary window, the list counts them ("2 tabs in private windows") and says nothing else of them: a private window's page names are not for a list in another window, which somebody else may be looking at. Asked from a private window, that window's own tabs are listed, and the ordinary ones with them.

**Sleep all others.** Every tab that is out of sight, has no reason to be awake and is of no private window is put to sleep now, as 055's "short" does it, with the same code: the state kept, the web view let go, the tab where it was. The tabs in sight stay, in every window. A tab with a reason stays: the list has just shown the person why, and it is theirs. Nothing lasts: a tab that is left behind a moment later is awake until its time.

**Why the core.** Which tabs are alive, in which Spaces, and why, are the model's; a window's snapshot has only the row it shows. And what "all others" are is a rule about browsing.

## 4. The contract (1.49)

```proto
// Request. Asks what is awake: every tab that has a web view, in every window and Space.
// Answered with AwakePage; "window.unknown".
message QueryAwake {
  // The window that asks: its tabs come first, and it decides whether private tabs are named.
  uint64 window_id = 1;
}

message AwakePage {
  repeated AwakeTab tabs = 1;
  // Tabs that are alive in private windows and not listed: asked from an ordinary window.
  uint32 private_tabs = 2;
  // Tabs that are asleep, in every window and Space.
  uint32 asleep = 3;
}

message AwakeTab {
  uint64 tab_id = 1;
  uint64 window_id = 2;
  uint64 space_id = 3;
  string space_name = 4 [(wt.redact) = true];
  string title = 5 [(wt.redact) = true];
  string host = 6 [(wt.redact) = true];
  // It is the tab in front of the row its window shows.
  bool in_sight = 7;
  // Why it would not be put to sleep (052), as in TabSummary.
  repeated KeepAwake keeps_awake = 8;
  // It is a Site (039).
  bool kept = 9;
  // What its web view uses, in bytes, where the engine says. 0: it does not say.
  uint64 memory_bytes = 10;
  // It is a private window's tab: listed for that window only.
  bool private_window = 11;
}

// Request. Puts to sleep every tab that can be: out of sight, with no reason to be awake,
// of no private window. Answered with Ack; "window.unknown".
message SleepOthers {
  uint64 window_id = 1;
}
```

Nothing is asked of a shell that it does not do already: a tab that is put to sleep gives its state and lets its web view go, as in 053.

**As built, where the design did not say:**

- **A private window's tab is marked as one.** Its row has no name, and neither has the Space the app made itself (a shell has its own word for that one): the name alone would not tell them apart.
- **The tab in front of a row that is not shown is not in sight.** Only the tab a window is showing is.
- **A tab that the person sent somewhere is listed by where it is going,** as its row says it.
- **How many are asleep** counts the tabs asleep in the rows windows have, and the tabs the session keeps of Spaces its window has not shown since the app started, which are nowhere but in the store until it does. Each is counted once. An empty tab is not asleep: it has nothing to wake to.
- **"Sleep all others" is short's rule to the letter** (055): a tab that is only on its way stays, and so does every tab with a reason. It is refused for a window that is not there, and then nothing sleeps.

## 5. In what order

1. **The core:** the list (what is on it, its order, a private window's tabs, the count of those asleep), and the command. Against the scripted engine. **Built.**
2. **The shells:** the list drawn, with the reasons in words; the command as its one button. Asked for when it is opened and again whenever the window's own row changes while it is open: no timer. **Built,** in the Library ([057](057-the-library.md)), which had first to have room for a fifth list: the sheet's switch has a line of its own on every system now.

**As built in the shells, where the design did not say:**

- **A row is three lines:** the tab's name (its page's title, or its site, or the shell's words for a tab with no page); where it is (its Space by name, the shell's own word for the Space the app made, or "Private Window", and its site when the name is not that); and why it is awake.
- **Why, in words:** "On screen" first, then the core's reasons in the core's order. A tab out of sight with no reason says "Will sleep by itself". A reason a shell has no words for (a newer core's) is not said; on Apple's systems a tab whose only reason is such a one says nothing of why. (On Android such a reason does not reach the shell at all, and the tab is said to sleep by itself: its contract's reader drops a value it does not know.)
- **A row is not a button.** A tab is gone to where it is.
- **The button is dimmed when no tab that is listed could sleep:** none is out of sight with no reason and of no private window. The core decides which sleep; the shell only says whether there is anything to ask.
- **Under the rows:** "Asleep: 12"; and, where some are, "Awake in private windows: 2". With no tab awake: "No tab is awake.", and the count of the asleep.
- **Asked again when the window's own row changes.** A tab that sleeps by itself in a Space the window does not show changes nothing the window is told, and the list is as it was until it is opened again or the row changes.

## 6. Decisions

**All of these are put to the owner.** The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | The list is of the whole app: every window, every Space | Memory is the app's, not a window's. The tab that is costing the most is as likely in a Space that is not shown | Only the Space the window shows: shorter, and it hides what the person is looking for |
| D2 | A private window's tabs are counted, and named only in that window's own list | A page's name in a private window is not for another window's list (037) | Never listed anywhere, only counted; or listed everywhere, since the person opened them |
| D3 | "Sleep all others" leaves a tab with a reason awake | The list has just said why it is awake. One press should not end a call or lose a draft | It sleeps everything that is out of sight but a draft; or it asks first when something would be lost |
| D4 | No figure of memory until an engine gives one | Neither engine here gives one. A figure that is guessed is worse than none | The whole app's memory as one figure at the top, which the system does give: true, and no help in choosing a tab |
| D5 | The list is one of the Library's parts, the fifth ([057](057-the-library.md)) | It is a list about the browser, opened now and then, as History, Downloads, Permissions and the Archive are. Their sheet's switch had no room for a fifth name on a Mac; the Library gave it a line of its own | Apart from the Library: a command in the Mac's Window menu and a row in a phone's menu, opening the list alone |
| D6 | It is called "Awake", and its button "Sleep All Others" | It lists what is awake, and "Memory" promises figures it does not have (D4) | "Memory", as the plan has it |
| D7 | A tab cannot be put to sleep by itself from the list yet | One command first. A row's own button needs its own rule: what of its reasons? | Each row has "Sleep", which sleeps it whatever its reasons, the person having pressed it |

## 7. Tests

| Where | What |
|---|---|
| The core, through the contract (`awake.rs`, 8), against the scripted engine | **The list:** three tabs alive are listed in their row's order, each with its window, its Space, its name and its host, the one in front in sight, none with a reason, no figure; one that plays a sound, one that sets off somewhere and one that is a Site each say so; an empty tab that comes in front is not listed, and is not asleep. **The order:** a window with three Spaces and another window: asked from the first, its shown Space is first, then its others in the Spaces' own order, then the other window; the tab in front of a row that is not shown is not in sight; asked from the second, its own is first; with another Space shown, that one is first. **A private window:** an ordinary window is told its own tab and that three are alive in private windows; a private window is told its own two, marked and with no Space's name, and the ordinary one, and counts the other private window's. **Sleep all others:** the tabs behind sleep and leave the list, one that was typed into stays, two are counted asleep; one that is woken is listed again, the tab it was woken over is awake a minute on, and the command done again takes it; in two ordinary windows and a private one, asked from the second, the tabs behind in both sleep, each asked for its state, the one in front of each stays, and the private window's stay; a tab on its way is listed by where it is going and is left by the command. **A window that is not there** is refused the list and the command, and nothing sleeps. **Over a start,** with a Space not shown since: its three tabs are counted asleep with the shown Space's one, its empty tab is not; when it is shown, its tab in front wakes and is listed first, and each tab asleep is counted once |
| The core broken, a part at a time | 26 parts, 26 caught, at the first run |
| Apple's model and words (`AwakeModelTests`, 8, on the Mac and the iPhone simulator), against a scripted core | The list is asked of its window and shown in the core's order with its two counts; nothing is said before the core has answered. "Sleep All Others" is sent and then the list is asked for again, in that order, and what is shown is what the core left: the shell takes nothing off the list itself. The button is worth pressing only when some tab is out of sight with no reason and of no private window (five cases). An answer that arrives after a later question's is not shown. Why a tab is awake, for each reason and for several, on screen first; a reason with no words is not said and is not taken for none. Every reason the contract has has words. A tab's name and its place, for each case. And with the real core: its answer is one the shell reads, an empty tab is not awake, and a window that is not there leaves the list unloaded |
| Apple's shell broken, a part at a time | 25 parts, 25 caught. The first run caught 21. Two of the four it did not were no breaks (they did not build, and were made again as ones that do). The other two are why two tests are as they are: the test of the button read the list of the case before while its own was on its way, and so passed whatever the button did (each case's answer now says a number of its own, and is waited for by it); and the test of an answer that comes late could not make one come late (a core that holds its answers until the test lets each go now does) |
| The apps, in fake mode, with the real core and the real engine (`AwakeListTests`, 1, on the Mac and the iPhone simulator) | Three tabs, each at a page. The Library, its fifth list: three rows in the row's order, the two behind saying they will sleep by themselves and the third that it is on screen; "Asleep: 0"; the button can be pressed. On the Mac, Apple's audit of the list. "Sleep All Others": one row is left, the one on screen; "Asleep: 2"; the button is dimmed. Done: in the strip the two behind say they are asleep, and the third does not. (The first gate over this found the test wrong for a Mac in two ways, where it was right for an iPhone: a row's words are a Mac's value and an iPhone's label, as the other lists' tests know; and Apple's audit looks through the sheet at the page under it, where the suite's tall page has a field with no name. The page in front is now one with nothing in it) |
| Android's model, words and list (`AwakeTest`, 11, on the JVM), against a scripted core | As Apple's model's and words' tests, the answer that comes late among them; a core that will not answer leaves the list as it was; and the list drawn: each row by its name, its place and why; the counts, reached at the list's end; the button and Done; and with no tab awake |
| Android, through the real core (`ThroughTheCoreTest`, +1, on the JVM) | The real core's answer reaches the list: both tabs in their row's order, the one in front on screen. "Sleep all others": the core lets the one behind go, the list is of the one left, one is counted asleep, and the window says the same |
| Android's shell broken, a part at a time | 29 parts, 29 caught. The first run caught 27: no test pressed the new name in the switch, and none could make an answer come late (a core that holds its answers now does) |
| Android, on an emulator (`AppAwakeTest`, 1), in fake mode, with the real core and the real WebView | Three tabs at pages of the suite's site. The Library's fifth list names them in order, each with its place and why; "Sleep All Others" takes the two behind off the list and counts two more asleep; the strip then says they are asleep, and the page on screen is the one it was |
| Pictures | 057 §7: what is awake with tabs and with none, in both themes, on the Mac, the iPhone and Android |

## 8. What is not verified

- **The choices of §6** are the recommendations, used on the owner's word; the owner has not looked at the list.
- **A figure of memory:** none, on either engine (D4).
- **Many tabs.** The list is built by going through every window's rows each time it is asked for. With thousands of tabs alive that is still a walk through memory; it was not timed.
- **A tab that sleeps while the list is open,** in a Space the window does not show: the list is not asked for again, and names it until it is (§5). Not a wrong word for long, and not tried.
- **Several windows, in the apps.** That another window's tabs are listed, and a private window's only counted, is the core's test and the models'; no test of an app has two windows open with this list.
- **Apple's audit of the list on an iPhone:** 057 §8.

## 9. Done checklist

- [ ] The owner's word on §6 (used as suggested, on the word of 2026-11-24).
- [x] The core and the contract. They were in the gate of 2026-11-24 with [055](055-the-system-short-of-memory.md): every stage green, the core's suite at 740.
- [x] The shells.
- [x] The gate, with the shells: every stage of `Tools/test.sh all` green, and `android-device`, on 2026-11-24: `lint`, `core` (740, and the strings tool's 24), `unit` (385 on the Mac), `build`, the Mac's UI suite (63), `ios` (369 on the iPhone simulator), the iPhone's UI suite (52), `android` (516 and 32 on the JVM) and `android-device` (193 on the emulator).

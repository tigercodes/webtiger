# 056 — What is awake, and "sleep all others" (W3)

| | |
|---|---|
| **Status** | **Built and tested in the core, on the recommendations of §6, which are provisional:** they are put to the owner and not yet answered. The core lists what is awake and why, and puts the others to sleep when asked. **The shells are not built:** no window shows the list yet |
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
2. **The shells, not built:** the list drawn, where History, Downloads and the Archive are (D5), with the reasons in words; the command as its one button. Asked for when it is opened and again whenever the window changes while it is open: no timer.

## 6. Decisions

**All of these are put to the owner.** The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | The list is of the whole app: every window, every Space | Memory is the app's, not a window's. The tab that is costing the most is as likely in a Space that is not shown | Only the Space the window shows: shorter, and it hides what the person is looking for |
| D2 | A private window's tabs are counted, and named only in that window's own list | A page's name in a private window is not for another window's list (037) | Never listed anywhere, only counted; or listed everywhere, since the person opened them |
| D3 | "Sleep all others" leaves a tab with a reason awake | The list has just said why it is awake. One press should not end a call or lose a draft | It sleeps everything that is out of sight but a draft; or it asks first when something would be lost |
| D4 | No figure of memory until an engine gives one | Neither engine here gives one. A figure that is guessed is worse than none | The whole app's memory as one figure at the top, which the system does give: true, and no help in choosing a tab |
| D5 | The list is with History, Downloads and the Archive | It is a list about the browser, opened now and then. A fourth place for one more list would be one more thing to find | A window of its own on the Mac; a command in the menu only |
| D6 | It is called "Awake", and its button "Sleep All Others" | It lists what is awake, and "Memory" promises figures it does not have (D4) | "Memory", as the plan has it |
| D7 | A tab cannot be put to sleep by itself from the list yet | One command first. A row's own button needs its own rule: what of its reasons? | Each row has "Sleep", which sleeps it whatever its reasons, the person having pressed it |

## 7. Tests

| Where | What |
|---|---|
| The core, through the contract (`awake.rs`, 8), against the scripted engine | **The list:** three tabs alive are listed in their row's order, each with its window, its Space, its name and its host, the one in front in sight, none with a reason, no figure; one that plays a sound, one that sets off somewhere and one that is a Site each say so; an empty tab that comes in front is not listed, and is not asleep. **The order:** a window with three Spaces and another window: asked from the first, its shown Space is first, then its others in the Spaces' own order, then the other window; the tab in front of a row that is not shown is not in sight; asked from the second, its own is first; with another Space shown, that one is first. **A private window:** an ordinary window is told its own tab and that three are alive in private windows; a private window is told its own two, marked and with no Space's name, and the ordinary one, and counts the other private window's. **Sleep all others:** the tabs behind sleep and leave the list, one that was typed into stays, two are counted asleep; one that is woken is listed again, the tab it was woken over is awake a minute on, and the command done again takes it; in two ordinary windows and a private one, asked from the second, the tabs behind in both sleep, each asked for its state, the one in front of each stays, and the private window's stay; a tab on its way is listed by where it is going and is left by the command. **A window that is not there** is refused the list and the command, and nothing sleeps. **Over a start,** with a Space not shown since: its three tabs are counted asleep with the shown Space's one, its empty tab is not; when it is shown, its tab in front wakes and is listed first, and each tab asleep is counted once |
| The core broken, a part at a time | 26 parts, 26 caught, at the first run |

## 8. What is not verified

- **The choices of §6** are the recommendations, not the owner's word.
- **No window shows the list.** The shells are not built: nothing here was seen by a person.
- **A figure of memory:** none, on either engine (D4).
- **Many tabs.** The list is built by going through every window's rows each time it is asked for. With thousands of tabs alive that is still a walk through memory; it was not timed.

## 9. Done checklist

- [ ] The owner's word on §6.
- [x] The core and the contract.
- [ ] The shells.
- [ ] The gate.

# 052 — What keeps a tab awake (W3)

| | |
|---|---|
| **Status** | **Built and tested in the core, on the recommendations of §6, which are provisional:** they are put to the owner and not yet answered. The core knows each tab's reasons and tells the shells. **No shell's adapter reports what a page is doing yet:** so in an app a tab is kept awake only by what the core knows itself (a download, a question, a load). Nothing is put to sleep by any of this |
| **Phase** | W3 of [000](000-master-plan.md) §9. 000 §5.7: "A tab is kept awake while it plays audio, holds the camera or microphone, is in a call, has a download or upload running, has unsaved form text, has developer tools open, or belongs to a Site marked 'keep awake'. The sidebar shows why." The second of W3, before anything is put to sleep by the browser itself |
| **Contract** | 1.45 (§4) |
| **Builds on** | [051](051-a-tabs-kept-state.md): what a tab that sleeps keeps, and what it does not (what was typed into a form, on WebKit). [013](013-downloads.md): a download and the web view it came from. [014](014-permissions.md): the camera and the microphone. [016](016-page-dialogs.md), [017](017-sign-in-prompts.md): what waits for the person in a tab |
| **Last updated** | 2026-11-24 |

## 1. User story

I started a long message in one tab and went to look something up in another. A song is playing in a third. Whatever the browser does later to save memory, it must not take the message from me, and it must not stop the song.

## 2. What is in it, and what is not

**In:** the browser **knows why a tab must not be put to sleep**, for each tab, at every moment, and says so to its shells. The reasons, each a thing the browser can know:

- **Sound**: the page is playing something that can be heard.
- **Camera or microphone**: the page was given one, and has it still.
- **Typed text**: the person typed into the page, and the page is still the one they typed into.
- **A download**: one that this tab's page started is running.
- **A question**: the tab is asking the person something (a prompt waits in it).
- **Loading**: the tab is on its way somewhere.

**Not in, each with where it goes:**

- **Putting a tab to sleep.** Nothing here sleeps a tab. The sleep time, the limit of live tabs and the system's shortage of memory are the next of W3, and are built on these reasons: a tab with one is passed over.
- **A mark in the tab for each reason** ("The sidebar shows why"): what it looks like is §6's to settle. The reasons are in what the core tells a shell from the start; the marks come with the owner's word.
- **An upload.** No engine tells an app that a page is sending a file. A page that is sending one was given the file by the person a moment before, and is in front; one left to send in the background is not known to be sending. Said in §8.
- **A call** is the camera or the microphone, or sound: there is no third thing to know it by.
- **A Site marked "keep awake"**: a choice of the person's about a Site, with the Sites' own settings.
- **Developer tools**: there are none yet.
- **Sound that no element of the page plays** (the Web Audio interface by itself): §8.

## 3. The rule

**A reason is of a page.** Sound, a camera, typed text: each is true of the page a tab shows, and goes when that page goes. A tab that commits another page starts with none. A page that only changes its own address is the same page, and keeps what it had.

**Who knows what.**

- **The engine's adapter knows three**, and reports them when they change: that the page is playing sound, that it holds the camera or the microphone, and that the person has typed into it. It reports what is so now, all three together, whenever one changes: the core holds no half of an answer.
- **The core knows the other three** by itself: a download it started for that web view is running; a prompt waits in the tab; the tab is loading.

**Typed text** is the one a person loses most by. It is true from the first time the person types into a field of the page (a line of text, a larger field, or a part of the page that can be edited), and stays true for as long as that page is the tab's. The browser does not try to know whether the text was sent, saved or deleted again: a page that keeps a draft by itself is kept awake a little longer than it need be, and no page loses a draft because the browser guessed. What was typed is never looked at: the adapter is told that something was, and nothing of what.

**Sound** is true while an element of the page (a sound or a film) is playing and is not muted. It is false again when the last of them stops.

**The camera and the microphone** are true from the moment the page is given one until the engine says the page has let go of it, where the engine says (WebKit does); where it does not (the System WebView), until the page goes.

**Nothing is decided by a page's word alone that costs the person anything.** A page can make itself seem awake-worthy (it can play silence). What that costs is memory, which the limits of the next step still hold: under the system's shortage of memory, sound and typed text are passed over last, not never.

**What a shell is told.** Each tab's summary, and the tab in front, say the reasons that hold, as a list of kinds. A shell that knows no mark for a kind shows none.

**Why the core.** What may be put to sleep is the core's to decide; these are what it decides by. An adapter reports what its engine can see and decides nothing.

## 4. The contract (1.45)

Added; nothing is changed or removed.

```proto
// EngineEvent: what the adapter knows of the page that keeps its tab awake. Sent when any
// of it changes, with all of it as it now is; a page that has just committed has none until
// the adapter says otherwise.
message PageActivityChanged {
  bool plays_sound = 1;
  bool captures = 2;      // holds the camera or the microphone
  bool typed_into = 3;
}

// TabSummary and TabView: why the tab would not be put to sleep. Empty when nothing would
// keep it.
repeated KeepAwake keeps_awake = 14;   // TabSummary; 21 in TabView
enum KeepAwake {
  KEEP_AWAKE_UNSPECIFIED = 0;
  KEEP_AWAKE_SOUND = 1;
  KEEP_AWAKE_CAPTURE = 2;
  KEEP_AWAKE_TYPED = 3;
  KEEP_AWAKE_DOWNLOAD = 4;
  KEEP_AWAKE_QUESTION = 5;
  KEEP_AWAKE_LOADING = 6;
}
```

Nothing a person typed, played or was asked crosses the contract by this: three yes-or-noes, and a list of kinds.

**As built in the core:**

- **The reasons are said in the order of the kinds,** whatever order they began in.
- **What a page was doing goes when another page commits, and when its renderer dies.** A page that only changes its own address keeps it.
- **A download is its web view's:** the tab that started one is kept awake by it until it finishes, fails or is stopped, also when the tab has gone on to another page meanwhile; with two coming, until the last ends.
- **A load that failed is over:** only a load that is on its way is a reason.
- **A tab that is asleep has none:** it is made with no page to be doing anything.
- **The windows are told when a reason begins or ends,** and not when an adapter says again what it said.

## 5. In what order

1. **The core:** the reasons a tab has, from the engine's reports and from what the core knows itself; when each begins and ends; what the shells are told. Against the scripted engine. **Built.**
2. **Apple's shell:** how the WebKit adapter knows each of the three, and the adapter suite against real WebKit: a field typed into, a sound played and stopped, each reported, and gone with the page.
3. **Android's shell:** the same, as far as the System WebView lets an app know.

## 6. Decisions

**All of these are put to the owner.** The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | Typed text keeps a tab awake from the first key until the page goes, whatever became of the text | The browser cannot know that a draft was sent or saved, and guessing wrong loses a person's words: WebKit hands nothing typed back to a tab that wakes (051). The cost of erring this way is a tab kept awake | True only while a field holds text the page was not loaded with: tabs sleep sooner, and a page that keeps its text somewhere a field's value does not show loses it |
| D2 | The adapter learns of typing and of sound from a small script of the browser's own in the page, which says yes or no and nothing else | No engine WebTiger uses tells an app, by a public way, that a page is playing sound or that a field was typed into. The script reads nothing: it hears that a key went into a field, and that a sound started or stopped | Sound and typing not known at all: the next step would then have to leave every tab awake that a person had ever touched, or lose drafts |
| D3 | A download keeps awake the tab whose page started it | An engine's download may end with the web view it came from. A file half fetched is the plainest thing to lose | Downloads moved out of their web views before a tab sleeps: possible on some engines, not on all, and nothing that needs doing while a tab can simply stay |
| D4 | A tab with a question waiting, or on its way somewhere, is kept awake | To sleep it would answer the question for the person, with no; and a load that is cut off is a page that never arrived | None worth having |
| D5 | The marks are for later: this step tells the shells the reasons and draws nothing | What a tab that is playing sound should look like, and whether the others are shown at all, is a matter of the row's design, which is the owner's. A speaker for sound is what every browser has, and is the recommendation when it is built | A speaker now, and nothing for the rest |

## 7. Tests

| Where | What |
|---|---|
| The core, through the contract (`keep_awake.rs`, 6), against the scripted engine | An empty tab and a page that does nothing have no reason. Typed into, a tab says so in its view and in the row; with sound and a camera too, all three in the order of the kinds; the sound stops and the camera is let go, and what was typed is typed still. The windows are told when a reason changes and not when the same is said again. **Of the page:** kept when the page writes its own address; gone when another page commits, by the person or by itself, and when the renderer dies. **Each tab its own:** a tab behind with sound and one in front typed into say each their own; a web view no tab has changes nothing and tells nobody. **The core's own:** a tab on its way is kept awake by that until it arrives, and one whose load failed is not; a tab with a question, until it is answered. **A download:** the tab whose page started it, and not the other; how far it has got changes nothing; the tab going on to another page changes nothing; finished, failed and stopped each end it; two at once last as long as either. **Asleep:** a tab that comes back at a start says nothing of what its page did before the quit |
| The core broken, a part at a time | 19 parts, 19 caught: what a page was doing kept over a commit, and over its renderer's death; each of the three not heard of; sound and typing taken for each other; each of the six reasons left out; a tab with no question saying it has one; a failed load counted as on its way; the order of the kinds; the row not saying the reasons, and the tab in front not; every tab kept awake by any download; the tabs never told of downloads. (A twentieth, a tab's window not told when its download began or ended, was not caught: the same windows are told by the count of their downloads, every time. The line was taken out) |

## 8. What is not verified

To be filled in as built. Known before a line is written:

- **An upload** is not known to the browser (§2). A tab that is sending a file in the background has no reason by it. It is passed over by the others when the person typed something in the page, which a form with a file in it most often had.
- **Sound made with no element** (the Web Audio interface alone: some games, some players) is not heard by the script. Such a page is not known to be playing.
- **Muted and silent are different things.** A film that plays with its sound turned all the way down by the page's own control is playing a sound that cannot be heard; whether the script can tell is to be seen on each engine.

## 9. Done checklist

- [ ] The owner's word on §6.
- [x] The core and the contract.
- [ ] Apple's shell, and the adapter suite.
- [ ] Android's shell, and the adapter suite on a device.
- [ ] The gate.

# 052 — What keeps a tab awake (W3)

| | |
|---|---|
| **Status** | **Built and tested in the core and both shells, on the recommendations of §6, which are provisional:** they are put to the owner and not yet answered. The core knows each tab's reasons and tells the shells; each shell's adapter reports that a page plays sound, holds the camera or the microphone, and was typed into. **Nothing is put to sleep by any of this, and nothing is drawn:** the reasons are what the next step decides by, and what a mark in the tab will show when its look is settled |
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

## 4a. Apple's shell, as built

- **The camera and the microphone are WebKit's own word** (`cameraCaptureState`, `microphoneCaptureState`), which the adapter watches. One that is held and muted is held: the page has it still.
- **Sound and typing come from a script of the adapter's,** in every page and every frame, in a world of the adapter's own: a page's scripts cannot see it, call what it calls, or say anything in its name (tested: the page finds no such thing to call).
- **Typing** is a key the person pressed going into a line, a larger field, or a part of the page that can be edited: an event the engine made. A box that is ticked, a slider and a button are not typing; nor is what a page's own script puts in a field, whatever the script then says happened.
- **Sound** is a player of the page (a sound or a film) that is playing and is neither turned off nor all the way down. The script hears it begin, stop, end, and its loudness change; a frame that goes takes its sound with it.
- **On an iPhone a page cannot turn its own sound down:** the loudness there is the device's. A page that sets it changes nothing that is heard, WebKit says no change happened, and the sound is a sound still. The gate's first run over this found it: the suite had held the iPhone to what a Mac does. It now holds each to its own.
- **Each frame says for itself,** by a name it makes up, all it knows as it now is. The page plays sound while any of its frames does, and was typed into once any of them was.
- **Of the page:** the adapter begins again at each commit, and tells the core only what has changed. A camera the new page holds already is said.

## 4b. Android's shell, as built

- **The same script,** put in every page and every frame as its document starts. The System WebView has no world of its own for an app's script: it runs in the page's. So the script takes its ear for itself and takes the ear's name out of the page before any script of the page's runs (tested: the page finds no such name).
- **What a frame says is read strictly:** a name, and a yes or no for each of sound and typing. Anything else is nothing.
- **The camera and the microphone** are held from the moment the adapter hands one over until the page goes: the WebView does not say when a page lets go of one.
- **A WebView that cannot be given a script, or an ear for one** (an old one), leaves the adapter deaf. Every page is then taken to have been typed into: no page is to lose a draft because the browser could not know (D1).
- **What a frame says is taken up in its turn,** like what a page asks ([014](014-permissions.md) §6): after the commit of the page that says it.

## 5. In what order

1. **The core:** the reasons a tab has, from the engine's reports and from what the core knows itself; when each begins and ends; what the shells are told. Against the scripted engine. **Built.**
2. **Apple's shell:** how the WebKit adapter knows each of the three, and the adapter suite against real WebKit: a field typed into, a sound played and stopped, each reported, and gone with the page. **Built** (§4a).
3. **Android's shell:** the same, as far as the System WebView lets an app know. **Built** (§4b).

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
| The WebKit adapter (`WebKitActivityTests`, 10, on the Mac and the iPhone simulator), against real WebKit and the test site | **Typing:** into a line, said once, with nothing of what was typed in anything reported; more of it says nothing more. A larger field, and words that can be edited, are typed into; a box that is ticked is not. What a page's own script puts in a field, saying an input happened, is not typing; and the page finds nothing of the adapter's to call. A frame typed into is the page typed into. A page that follows starts with none, and is said to be typed into in its turn, after its own commit. **Sound:** a sound that plays is said, says nothing more while it plays on, and is said to have stopped; with typing, all of it as it now is. Turned off before it plays it is no sound, and turned on it is; all the way down it is none, and up again it is. A sound goes with its page. **The camera and the microphone:** the adapter's reading of what WebKit says, held also when muted. **What is told** is all that is known, and only when it changes: two frames' sound, the last to stop, what was typed staying typed, a commit |
| Apple's shell broken, a part at a time | 21 parts, 20 caught (two of them by tests that waited out their time). One was not: **that the adapter watches what WebKit says of the camera and the microphone.** No page can be given one on this Mac, which has none (014 §10), so nothing here makes WebKit say it |
| What a page is doing, on the JVM (`PageDoingTest`, 6), with no WebView | A page that does nothing has nothing to tell. What is told is all that is known, and only when it changes. One frame's sound is the page's, and the last to stop ends it. A page given the camera holds it until it goes, and the page that follows starts with none. An adapter that cannot listen takes every page to have been typed into. What a frame says is read, and anything else is nothing |
| The System WebView adapter suite (`SystemWebViewActivityTest`, 9, on the emulator), against the real WebView and the test site | As WebKit's, with a finger on the page's buttons where the WebView wants one to start a sound: typing said once and nothing of what; a larger field, words that can be edited, and a box that is ticked; a page's own script, and the ear's name gone from the page; a frame; a page that follows; a sound that plays, plays on, and stops; turned off, on, down and up; a sound that goes with its page. And a page that is given the camera is said to hold it, and one that is refused is not |
| Android's shell broken, a part at a time | 25 parts, 24 caught. On the JVM, 11 of 11. On the emulator, 13 of 14. One was not: **that the script hears a sound's loudness change.** The sound the suite plays is a quarter of a second, played again and again, and the WebView says it is playing at each turn: the script reads the loudness then, so the test passes without the change being heard. With a sound that plays on without turning, only the change says it. The same part, broken in Apple's shell, was caught: WebKit says nothing at a turn |

## 8. What is not verified

- **The choices of §6** are the recommendations, not the owner's word.
- **On Apple's systems, a camera or a microphone that is held, against the engine.** No page can be given one here (014 §10). The adapter's reading of WebKit's word is tested; that it hears the word is not.
- **A key on a keyboard.** What is typed in the suites is typed as an editing command, which each engine reports as the person's own input. No hardware key and no keyboard on a screen was pressed.
- **A sound that was heard.** What the suites play is silence. That the script can tell a film with no sound in it from one with sound is not so: such a film, playing and not turned off, counts as sound.
- **A sound from a player that is not in the page:** one a script makes and never puts in the document says what it does to nobody the adapter's script can hear. Such a page is not known to be playing. Nor is a player inside a part of a page that keeps its events to itself.
- **On Android, a WebView that cannot be given the script:** none is here. The rule for it is tested with no WebView.
- **A phone.** Android's part ran on an emulator.
- **Nothing is drawn** (D5), so no screen was looked at.
- **An upload** is not known to the browser (§2). A tab that is sending a file in the background has no reason by it. It is passed over by the others when the person typed something in the page, which a form with a file in it most often had.
- **Sound made with no element** (the Web Audio interface alone: some games, some players) is not heard by the script. Such a page is not known to be playing.
- **Turned off, and all the way down,** are told from a sound that can be heard, on both engines (tested). Silence that is played at full loudness is not.

## 9. Done checklist

- [ ] The owner's word on §6.
- [x] The core and the contract.
- [x] Apple's shell, and the adapter suite.
- [x] Android's shell, and the adapter suite on a device.
- [ ] The gate.

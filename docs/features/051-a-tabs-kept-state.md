# 051 — A sleeping tab wakes where it was (W3)

| | |
|---|---|
| **Status** | **Built and tested in the core and the store, on the recommendations of §6, which are provisional:** they are put to the owner and not yet answered. The core asks for a tab's state when it leaves the front, keeps it, and wakes the tab from it. **No shell gives a state yet:** each answers that its engine has none, and loads the address when it is asked to make a web view from one. So nothing in an app has changed: a tab wakes at its address, as before |
| **Phase** | W3 of [000](000-master-plan.md) §9: "The lifecycle of §5.7; snapshots; keep-awake reasons; live-tab limits …". The first of W3: what a sleeping tab keeps. 000 §5.7: "Asleep. The web view is gone. A snapshot, the saved state and the scroll position remain"; "WKWebView saves `interactionState` and is released. Android WebView saves its state" |
| **Contract** | 1.44 (§4). The store's schema is 16: a tab's kept state |
| **Builds on** | [009](009-session-restore.md): the tabs come back at a start, asleep, each at its address. [039](039-sites.md): a Site that is closed sleeps. [049](049-tidying-temporary-tabs.md): the look, which knows when a tab was last in front. [008](008-history.md): what is forgotten is gone from the store's files |
| **Last updated** | 2026-11-24 |

## 1. User story

I had forty tabs open yesterday. Today each comes back when I press it: at the top of its page, and its Back button is grey, though I had followed five links to get there. I would like it as I left it: scrolled where I was reading, and Back going back.

And before the browser puts tabs to sleep by itself to save memory (the rest of W3), a tab that sleeps must lose as little as this.

## 2. What is in it, and what is not

**In:** a tab's **kept state**: what its engine can save of a web view and make another web view from. For WebKit and for Android's WebView that is the list Back and Forward go through, and, as far as the engine keeps them, where each page was scrolled to and what a form held. It is kept when a tab stops being the one in front, and when its page is let go; and a tab that wakes is made from it. Where there is none, or the engine will not take it, the tab wakes as it does today: at its address.

**Not in, each with where it goes:**

- **When a tab goes to sleep by itself** (the sleep time, the limit of live tabs, the system short of memory), **and what keeps one awake** (sound, a call, a download, a form with text in it): the next of W3. Nothing here puts a tab to sleep that does not sleep today.
- **A picture of the page as it was** (000 §5.7's "snapshot"), for the switcher's cards ([047](047-the-phones-switcher.md)) and for a tab that is waking: its own feature.
- **The tab in front when the app is quit.** It was never out of sight, so nothing asked for its state: it wakes at its address, as today. Asking as the app goes to the background, where a system says so in time, is with the lifecycle's other signals.
- **A closed tab that is reopened** ([033](033-a-closed-tab-comes-back.md)) with its state: it comes back at its address, as today.
- **A Site.** A Site sleeps at its home (039), not where it had wandered: there is nothing of where it was to keep.
- **The Windows shell.**

## 3. The rule

**The state is the engine's.** To the core it is bytes: it keeps them, hands them back, and never reads them. They are good for the engine that made them, and a shell that is handed bytes its engine will not take (another engine's, an older version's, a damaged file's) says so, and the tab is loaded at its address instead. That is never an error a person sees.

**When it is asked for.** The core asks the shell for a web view's state:

- when its tab **leaves the front** of its row (another tab is brought in front, or its window shows another Space): once, then; and
- before its web view is **destroyed** while the tab stays (a tab put to sleep).

It is not asked for on every page: a tab that is being used is in front, and a tab in front is asked nothing.

**What is kept.** One state for a tab, the last. A tab with none (it has never left the front; its page is no page of the web's; the engine gave none) has none. A state larger than the limit (D3) is not kept, and the one before it is dropped: an old state for a new page would wake the tab somewhere it no longer is. **A tab's state goes with the tab**: when it is closed, tidied (049) or its Space's session is no longer kept, the state is deleted. A private window's tabs have none written, as nothing of theirs is ([037](037-private-windows.md)).

**Forgetting.** A state holds the list of pages its tab goes back through: it is what was visited. When history is forgotten for a Space's logins (008), whatever is named to be forgotten, the states of every tab with those logins go with it: from the store's files, as the visits do, and not only from its table. A tab that is open keeps its web view, and its web view its list, so it is asked for no state again until it has another web view. Such a tab comes back at the next start at its address, as every tab did before there were states, and Back in it then goes nowhere: what was forgotten has not outlived the quit.

**Waking.** A tab that wakes (it is brought in front, asleep) has a web view made for it, as today; then, where it has a state, the shell is asked to **restore** it, and otherwise to load its address. A restore that the shell refuses is followed by the load. What the engine does with a state is its own: WebKit goes to the page the list was at and puts back what it kept of it; Android's WebView puts the list back and loads the page it was at. Either way the page arrives as any page does: the core is asked whether it may load (the navigation policy), told when it has committed, and only then shows its address as the page's. **The address field never lies**: a restored page is not shown as arrived before it is.

**While it sleeps** a tab says what it said: its name and its address, from the session, as today. The state changes nothing a person sees until the tab wakes.

**Why the core.** Which tab has left the front, which is asleep, what is kept and for how long are the model's and the store's. The shells do two things: hand over a web view's state when asked, and make a web view from one.

**Nothing leaves the device.** A state holds what a page showed and may hold what was typed into it: it is kept in the store with the session, is in no log (the contract's fields are redacted), and goes when its tab goes, and when the history of its logins is forgotten.

## 4. The contract (1.44)

Added; nothing is changed or removed.

```proto
// Host requests, core → shell (17 and 18).
message WebViewSaveState { uint64 web_view_id = 1; }
//   Answered with WebViewState (a fourth kind of answer): the engine's state of the web
//   view, or none (empty) when it has none to give. "webview.unknown".
message WebViewState { bytes state = 1; }               // redacted in logs
message WebViewRestore { uint64 web_view_id = 1; bytes state = 2; string url = 3; }
//   Makes the web view what the state says, and loads the page it was at. Where the engine
//   will not take the state, `url` is loaded instead, as WebViewLoad would load it. Answered
//   with Ack once one or the other has begun; "webview.unknown".
```

The store (schema 16): `tab_state(id, engine, state)`, and `tab.state_id`: a saved tab names its state. A state is written when it changes and not with every write of the session.

**As built, where the design said otherwise or did not say:**

- **A shell that will not take a state loads the address itself.** The design had it refuse (`webview.state_refused`) and the core load. But a host request is never sent after waiting on an earlier one (the contract's rule): by the time a refusal came back, the person might have sent the tab somewhere else, and the load that followed the refusal would have undone it. So the address goes with the state, and falling back is the shell's, at once.
- **A state is asked for, and kept, only of a tab at rest on its page.** A tab that is on its way somewhere, or whose load failed, is shown where it was going, and its web view is still where it was: its state would wake it at the wrong page. (An older test of the session found this: the tab came back at the page before the one that had failed.) Such a tab is not asked, and an answer that arrives after its tab was sent elsewhere is not kept.
- **A state is of the page it was asked on.** When its tab commits another page, or is sent to another by the person, the state is dropped at once: the tab has none until it next leaves the front. A tab that wakes from its state has none once it has arrived.
- **The engine a state is of is kept with it,** and a state is handed only to that engine: a tab that is to be made with another is loaded.
- **Before a web view is destroyed** nothing is asked yet: nothing destroys a web view and keeps its tab but a Site's closing (D4). It is for the sleep time, when that is built.
- **States that no saved tab names go when the app starts:** what a run that was killed between keeping a state and writing its session left behind.
- **No clock is read for a state,** and none is kept with it.
- **The window the session is of keeps its tabs' states when it closes** (as it keeps its tabs, 009); another window's go with it.
- **Forgetting history takes the states** (D6, which the design did not have). It was seen while the core was built: before there were states, nothing of a forgotten visit outlived a quit, and a state would have carried the list of pages across it. `ForgetHistory`'s comment in the contract says so since 1.44.
- **A tab that becomes a Site loses the state it had** as a tab (D4), and an answer that arrives after it became one is not kept.
- **An address that changes without a load** (a page that writes its own address) counts as going on to another page: the state is dropped.
- **The tab in front at a quit may have a state all the same:** one that had left the front before, and came back to the front on the same page, has the state of then, and wakes from it at the next start: the same page and the same list, scrolled where it was when it left the front. D5 is of a tab that never left the front.
- **The scripted shell answers a request for a state on a channel of its own:** it is not among the host requests a test counts, holds or answers for. Thirteen older tests count what the shell is asked, and a tab that leaves the front is now asked one thing more.

## 5. In what order

1. **The core and the store:** when a state is asked for, what is kept and dropped, waking from one, a refusal; against the scripted engine, which hands out and takes states a test can read. **Built.**
2. **Apple's shell:** `interactionState`, and the adapter suite: a page scrolled and reached by links comes back scrolled, with Back going back.
3. **Android's shell:** the WebView's saved state, and the same behaviours as far as that engine keeps them.

## 6. Decisions

**All of these are put to the owner.** They are built in the core as recommended. The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | The state is asked for when a tab leaves the front, and before its web view is destroyed | A tab in front is being used, and changes with every scroll; one that has just left the front is as the person left it, and is asked once. It is then there if the app is quit, or killed, an hour later | After every page that loads: the state is always fresh, and a page is saved many times that is never slept. On a timer: neither as fresh nor as cheap |
| D2 | It is kept in the store, with the session | It is what makes a tab that comes back at a start come back where it was: the first thing a person notices. The store is where the tabs are | In memory only: a tab that sleeps while the app runs wakes where it was, and one that comes back at a start does not |
| D3 | A state larger than 512 KB is not kept | A list of pages and where they were scrolled is small; a state that is not small is a page that put a great deal into a form or its history, and forty of those are a store that is slow to open. The tab wakes at its address | No limit. Or a limit on all the states together, dropping the oldest |
| D4 | A Site sleeps at its home, with no state | It is what a Site is (039): it comes back where it lives, not where it wandered | A Site's state kept as a tab's is |
| D5 | The tab in front at a quit wakes at its address | Nothing asked for its state, and a quit is no time to ask a web view anything: the app is going | The state of the tab in front asked for every so often while it is used |
| D6 | Forgetting history drops the states of the tabs with those logins, and a tab that is open is asked for none until it has another web view | A state holds the list a tab goes back through, which is what was visited. The person asked for it not to be kept, and before there were states nothing of it outlived a quit. All of them, whatever was named to be forgotten, as with closed tabs ([033](033-a-closed-tab-comes-back.md) D5): the core never reads a state, and could not say which holds a page that was named | The states stay: the tabs are open, and Back in them goes to those pages until they are closed. The cost of D6 is that such a tab comes back at the top of its page with nowhere to go back to, once |

## 7. Tests

| Where | What |
|---|---|
| The store (`state.rs`, 4) | A state is kept by an id of its own, never nought and never used twice, with the engine it is of, and dropped; one that is not there is no error. A saved tab names its state over a start; a state no tab names goes at a start, and only it. A state that is forgotten is gone from the database and from its write-ahead log, read as files, while another stays. A store from before states gains them and keeps its tabs, each with none |
| The core, through the contract (`kept_state.rs`, 14) | **Asked:** a tab that leaves the front, once, when another tab comes in front and when the window shows another Space; the tab in front is asked nothing, wherever it goes. **Never asked:** an empty tab, a Site, a tab whose load failed, one on its way, a private window's. **Over a start:** the state is in the store with its engine, and its tab names it; the tab wakes from it with its address to fall back on, arrives where it was, and has the list it had (it can go forward, as a tab that was loaded could not); the tab that was in front at the quit, which never left it, is loaded. **Not taken:** an engine that will not take the state loads the address, and the tab wakes all the same; another engine is not handed it. **Dropped:** when the tab has woken from it; when its page goes on to another by itself, or changes its address without a load, or the tab is sent where it never arrives; when a larger one than the limit follows it (one of 400,000 bytes is kept, one of 600,000 is not), or the engine gives none; when its tab is closed, or becomes a Site; when its window closes, but not the session's window. **Said late:** not kept for a tab that has gone on by then, whose load has failed, that has become a Site, or whose history was forgotten. **Forgetting history:** the states of that login set's tabs are gone from the store's files with the answer, another Space's stays; the tabs that were open are asked for none, a tab opened since is; a tab asleep with a state loses it and wakes by being loaded; after a start they are asked as any are. **At a start:** a state no saved tab names goes, and the one a tab names stays |
| The core's older tests | Thirteen count what the shell is asked, and pass as they were: the scripted shell answers a request for a state apart from the rest. Four files' helper for "what was loaded" (`session.rs`, `sites.rs`, `spaces.rs`, `step.rs`) counts a restore as a load at its address: a tab that left the front before a quit is now made from its state. One of the session's (a tab on its way, and one whose load failed, come back where they were going) failed against the first build, and is why a state is asked for and kept only of a tab at rest |
| The core broken, a part at a time | 39 parts. 37 caught. Two were not. **That a tab whose history was forgotten is asked again once it has another web view:** within a run nothing yet takes a tab's web view and leaves the tab (it is the sleep time's); after a start the tabs are new, and that is tested. **That the session is written when a state arrives:** the scripted shell publishes after everything it delivers, which writes the session too; the line is for a shell whose answer arrives between two publishings. The first run of these found three things the tests did not hold, and they do now: that a state goes when its page goes on by itself, that it goes when its tab is sent where it never arrives, and that a state under the limit is kept. (One part, a tab in front asked for its state, made the tests never end: the scripted shell now fails a core that asks ten thousand times) |
| The shells, as they are for now | Each answers that its engine has no state, and loads the address when it is asked to make a web view from one. Nothing new is tested in them: their suites pass as they were |

## 8. What is not verified

- **The choices of §6** are the recommendations, not the owner's word. D6 was not in the design that was put: it was added while the core was built.
- **No engine's state.** Neither shell gives one or takes one yet, so nothing a person sees has changed, no screen was looked at, and no test of an app was added. What each engine's state is was put to real engines in W0 ([001](001-foundations.md) §11.3) and is held by the adapter suites since (`WebKitRestoreTests`, `SystemWebViewRestoreTest`): data, small, bringing back the list and the scroll position in another web view. Neither is yet handed to the core.
- **The Windows shell** has none of it: it is not built.
- **Before a web view is destroyed** nothing is asked, because nothing destroys one and keeps its tab but a Site's closing. It is written in §3 and not built.
- **A state over the wire between a real shell and the core.** The scripted shell handed over one of 600,000 bytes; no shell of an app has handed over any.
- **How large the store grows** with forty tabs' states from a real engine, and how long it then takes to open: not measured. The limit of D3 is a guess, far above what W0 measured for three pages of history (001 §11.3: 1 KB from WebKit, 2.4 KB from the System WebView).
- **An app killed between a state being kept and its session being written.** The state is then one that no saved tab names, and goes at the next start: tested with such a state put into the store by hand, not by killing anything.
- **Tidying** (049) closes a tab by the way a person does, so its state goes with it; no test tidies a tab that has a state.
- **Why the gate over this found a fault of the Android adapter's.** One of the app's device tests failed on this commit, on the first run after the emulator's cold start, three times of three, and passed on the commit before: a page that asked for the camera as it loaded was reported before its own commit ([014](014-permissions.md) §6). The adapter's order was wrong before this change and is mended. What this change did to bring it out is not known: in that test no tab leaves the front, so nothing is asked of the shell that was not asked before.
- **A store that refuses** while a state is forgotten: `ForgetHistory` is then answered with the store's code, after the visits are gone. Not tested: the store in a test does not refuse.

## 9. Done checklist

- [ ] The owner's word on §6.
- [x] The core, the store and the contract, with D6.
- [ ] Apple's shell, and the adapter suite.
- [ ] Android's shell, and the adapter suite on a device.
- [ ] The core: the gate.
- [ ] The shells: the gate.

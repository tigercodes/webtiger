# 051 — A sleeping tab wakes where it was (W3)

| | |
|---|---|
| **Status** | **Designed; nothing built.** The choices of §6 are put to the owner |
| **Phase** | W3 of [000](000-master-plan.md) §9: "The lifecycle of §5.7; snapshots; keep-awake reasons; live-tab limits …". The first of W3: what a sleeping tab keeps. 000 §5.7: "Asleep. The web view is gone. A snapshot, the saved state and the scroll position remain"; "WKWebView saves `interactionState` and is released. Android WebView saves its state" |
| **Contract** | To be 1.44 (§4). The store's schema to be 16: a tab's kept state |
| **Builds on** | [009](009-session-restore.md): the tabs come back at a start, asleep, each at its address. [039](039-sites.md): a Site that is closed sleeps. [049](049-tidying-temporary-tabs.md): the look, which knows when a tab was last in front |
| **Last updated** | 2026-11-23 |

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

**Waking.** A tab that wakes (it is brought in front, asleep) has a web view made for it, as today; then, where it has a state, the shell is asked to **restore** it, and otherwise to load its address. A restore that the shell refuses is followed by the load. What the engine does with a state is its own: WebKit goes to the page the list was at and puts back what it kept of it; Android's WebView puts the list back and loads the page it was at. Either way the page arrives as any page does: the core is asked whether it may load (the navigation policy), told when it has committed, and only then shows its address as the page's. **The address field never lies**: a restored page is not shown as arrived before it is.

**While it sleeps** a tab says what it said: its name and its address, from the session, as today. The state changes nothing a person sees until the tab wakes.

**Why the core.** Which tab has left the front, which is asleep, what is kept and for how long are the model's and the store's. The shells do two things: hand over a web view's state when asked, and make a web view from one.

**Nothing leaves the device.** A state holds what a page showed and may hold what was typed into it: it is kept in the store with the session, is in no log (the contract's fields are redacted), and goes when its tab goes.

## 4. The contract (to be 1.44)

Added; nothing is changed or removed.

```proto
// Host requests, core → shell.
message WebViewSaveState { uint64 web_view_id = 1; }
//   Answered with WebViewState: the engine's state of the web view, or none (empty) when it
//   has none to give. "webview.unknown".
message WebViewState { bytes state = 1; }               // redacted in logs
message WebViewRestore { uint64 web_view_id = 1; bytes state = 2; }
//   Makes the web view what the state says, and loads the page it was at. Answered with Ack
//   once that has begun; "webview.state_refused" when the engine will not take the state,
//   and nothing has been loaded; "webview.unknown".
```

The store (schema 16): a table of kept states, one for a saved tab, with the engine it is of; a saved tab names its state. A state is written when it changes and not with every write of the session.

## 5. In what order

1. **The core and the store:** when a state is asked for, what is kept and dropped, waking from one, a refusal; against the scripted engine, which hands out and takes states a test can read.
2. **Apple's shell:** `interactionState`, and the adapter suite: a page scrolled and reached by links comes back scrolled, with Back going back.
3. **Android's shell:** the WebView's saved state, and the same behaviours as far as that engine keeps them.

## 6. Decisions

**All of these are put to the owner; none is built.** The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | The state is asked for when a tab leaves the front, and before its web view is destroyed | A tab in front is being used, and changes with every scroll; one that has just left the front is as the person left it, and is asked once. It is then there if the app is quit, or killed, an hour later | After every page that loads: the state is always fresh, and a page is saved many times that is never slept. On a timer: neither as fresh nor as cheap |
| D2 | It is kept in the store, with the session | It is what makes a tab that comes back at a start come back where it was: the first thing a person notices. The store is where the tabs are | In memory only: a tab that sleeps while the app runs wakes where it was, and one that comes back at a start does not |
| D3 | A state larger than 512 KB is not kept | A list of pages and where they were scrolled is small; a state that is not small is a page that put a great deal into a form or its history, and forty of those are a store that is slow to open. The tab wakes at its address | No limit. Or a limit on all the states together, dropping the oldest |
| D4 | A Site sleeps at its home, with no state | It is what a Site is (039): it comes back where it lives, not where it wandered | A Site's state kept as a tab's is |
| D5 | The tab in front at a quit wakes at its address | Nothing asked for its state, and a quit is no time to ask a web view anything: the app is going | The state of the tab in front asked for every so often while it is used |

## 7. Tests

To be filled in as built.

## 8. What is not verified

To be filled in as built.

## 9. Done checklist

- [ ] The owner's word on §6.
- [ ] The core, the store and the contract.
- [ ] Apple's shell, and the adapter suite.
- [ ] Android's shell, and the adapter suite on a device.
- [ ] The gate.

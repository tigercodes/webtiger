# 037 — Private windows (W2)

| | |
|---|---|
| **Status** | **The core and the contract are built and tested** (part 1 of §5): a window opened as private has a login set of its own that the engine is told to keep nothing of, and the core writes nothing of it. No shell opens one yet, so nobody can use it. The rule's choices (§4) are the owner's, confirmed 2026-11-21 (WT-D26) |
| **Phase** | W2 of [000](000-master-plan.md) §9: "private windows". 000 §5.6: "A throwaway login set. Nothing is written; closing it wipes it" |
| **Contract** | 1.26: `OpenWindow.private_window`; `WindowSnapshot.private_window`. `LoginSet.ephemeral` has been there since W0, and is now said |
| **Builds on** | [001](001-foundations.md): login sets, and the adapters' isolation tests. [008](008-history.md), [009](009-session-restore.md), [013](013-downloads.md), [014](014-permissions.md), [012](012-zoom.md), [033](033-a-closed-tab-comes-back.md): each writes something a private window must not |
| **Last updated** | 2026-11-21 |

## 1. User story

I open a private window to look something up that I do not want kept: a present, a diagnosis, another account for a moment. I am signed in to nothing there. When I close it, nothing of it is left on this device: no history, no cookies, nothing offered back to me later. The window says plainly what it is while it is open.

And what it is not: it does not hide me from the sites I visit, from the network, or from whoever runs it. The window says that too, once, where it starts.

## 2. What is in it, and what is not

**In:** a window that is private; its logins are its own and thrown away with it; nothing it visits is written to the store; it says what it is.

**Not in:**

- **Hiding from sites or the network.** Guard (W4) blocks trackers in every window; a private window promises nothing more about the network.
- **A private Space** among the Spaces. A private window has no Space to switch: it is one row of tabs.
- **A private tab in an ordinary window.**
- **Locking a private window** behind the device's lock while the app is in the background.
- **Extensions and plugins** (W6): whether they run in a private window is theirs to settle.

## 3. The rule

**Its logins are its own.** A private window has a login set made when it opens, that no Space has and no other window shares. Its web views share nothing with any other login set (the adapters' isolation tests, which are the bar for every engine). Two private windows are two login sets.

**The engine keeps nothing of it.** Where the engine can keep a login set in memory (WebKit: a store that is not persistent) it does, and closing the window's last web view is the end of it. Where it cannot (Android's WebView: a login set is a profile on disk), the profile is cleared and deleted when the window closes, and, should the app be killed first, when it next starts, before anything else is loaded.

**The core writes nothing of it.** Each of these is a place the core writes for an ordinary window, and does not for a private one:

| What | Ordinary window | Private window |
|---|---|---|
| Places visited (008) | Written to history | Not written. Nothing of it is ever offered back |
| What is offered while typing (011) | From the Space's history | Nothing from history: a search, and what is typed |
| The session (009) | Written; comes back after a quit | Not written. A private window never comes back |
| Closed tabs (033) | Journalled; can be brought back | Not journalled, and not brought back |
| A site's settings: zoom (012), what it may use (014), a certificate gone past (026) | Kept for the login set | Kept while the window is open, in memory; gone with it |
| Downloads (013) | Listed, and the list is kept | The file is the person's and stays where it was saved. Its entry is in the private window's list while the window is open, and is not written |
| The Space the window showed (024) | Kept | There is none |

**Nothing crosses.** A link from another app never opens in a private window. A page in a private window that opens a window gets a tab in the same private window. Nothing a private window did is visible in an ordinary one, and the other way round, but for files the person saved.

**It says what it is.** The window is told it is private (`WindowSnapshot`), and a shell shows it in words, not by colour alone: where the Space's chip is in an ordinary window, a private one says "Private". An empty tab in a private window says what private means and what it does not (§1).

**Passwords.** The system's password manager is the person's and is not WebTiger's to switch off; where the system lets a browser say "do not offer to save here" (Android's Autofill, [032](032-passwords-android.md)), a private window says it.

## 4. Decisions

> **The owner's decision (2026-11-21, WT-D26): "private choices are fine".** Each of these could have gone the other way, and other browsers differ on them. They were written down before any code and put to the owner, who confirmed them as written.

| # | Decision | The other course | Why this one |
|---|---|---|---|
| D1 | Nothing from history is offered while typing in a private window | Offer the ordinary history, as Safari and Chrome do | The core has one rule: a private window reads and writes nothing of the store's. A window opened so as to leave no trace is also one a person may be showing to somebody |
| D2 | A closed private tab cannot be brought back | Keep closed tabs in memory for the window | Nothing kept is nothing to get wrong. A person who closes a private tab by mistake loses a page, not a secret |
| D3 | A download's entry is listed in the private window, while the window is open | Not listed at all; or listed until the app quits | The person asked for the file and must be able to find it. The entry names an address: it is never written. **Narrower than what was put to the owner** ("while the app runs"): the list of downloads is a login set's (013), a private window's login set is its own, and so no other window could show the entry; once the private window is closed there is nowhere it would be seen. The file itself stays |
| D4 | Two private windows share nothing | One private login set for all private windows, as Chrome has | "Its logins are its own" is easy to say and to test. Signing in twice is the cost |
| D5 | The words "Private", and the explanation in an empty tab | A dark window, a mask, a colour | 000 §8.6: nothing is said by colour alone |
| D6 | On Android the profile is deleted at close and again looked for at start | Refuse private windows on Android | The WebView has no login set in memory. A profile that is certainly gone before anything loads next time is the same promise, kept one step later after a crash; the document for it will say so |

## 5. In what order

1. **The core and the contract** (built): a window opened as private has a login set of its own that is `ephemeral`; nothing of §3's table is written; the window is told. Tested through `FakeShell`, with the store's files read afterwards for what they hold.
2. **The Apple shell**: New Private Window (⇧⌘N on the Mac; where on an iPhone is to be settled), the words, the empty tab's explanation. The adapter suite's isolation tests for a login set that is not persistent.
3. **Android**: the throwaway profile, and its deletion at close and at start, as an isolation test on a device. Until then the Android adapter refuses a login set that is `ephemeral`, as it always has: a private window there opens and loads nothing.

### 5.1 How the core keeps nothing (part 1)

**The contract (1.26).** `OpenWindow.private_window` opens one; `WindowSnapshot.private_window` tells the window; a private window's snapshot names no Space (`space_id` is 0). Refusals: `window.no_private` when a private window cannot be made; `window.private` for what a private window does not do (switching Space; taking a link or a search from outside the browser).

**A row and a login set in memory only.** A private window has one row of tabs and a login set made as it opens. Neither is in the store on disk, in the list of Spaces, or among what a window can be switched to; both go when the window closes, with the certificates gone on past in it.

**A second store, in memory.** What an ordinary window keeps for its login set (a site's zoom, what a site may use, its downloads' entries), a private window keeps the same way, by the same code, in a store that is never a file. Everything the core keeps by login set goes to the store of that login set: the one on disk, or the one in memory. The login set is taken out of the memory store, with everything of it, when its window closes.

- **By construction, not by care alone.** A private login set has no row in the store on disk, and every table there names its login set or its Space: a write for a private window that reached the disk by mistake would be refused by the store itself. The tests break the rule on purpose to see that they notice.
- **Numbers that name one thing.** A download is named by a number that a shell may send back; the memory store's downloads (and visits, of which there are none) are numbered from far above any a store on disk reaches.
- **What is not kept even in memory:** places visited (nothing is offered back, D1) and closed tabs (D2). The session never has a private window in it, and a private window opened first never becomes the window whose tabs are kept.

## 6. Tests

The bar for the whole: after a private window has visited a site, signed in, downloaded, been asked for a permission and closed, **the store on disk and the engine's stores are searched for the site's name and hold none of it**. Part 1 meets it for the core's store.

| Where | What |
|---|---|
| The store (`wt-store`, 3: `private.rs`) | A private store's downloads and visits are numbered far above a real store's. A login set is added once and dropped with everything of it (visits, a site's zoom, what a site may use, downloads), another's left as it was, and nothing can be kept for it afterwards. A login set a Space has is not dropped |
| The core (`wt-app/tests/private.rs`, 9, through `FakeShell`; six over a store on disk whose files are then read whole) | **Its own login set**: the windows are told which they are; a private one names no Space and has one tab; the engine is told to keep nothing of each private window's login set, each is its own, and none is the ordinary window's. **Nothing written or offered back**: a private window has no history and is offered nothing from any, its own visits or the ordinary window's; the ordinary window is offered nothing of the private one's; left open as a crash leaves it, the files hold the ordinary site's name and not the private one's. **The session**: a private window opened first is not the one whose tabs are kept, and never comes back; the first ordinary window gets the session. **Closed tabs**: not journalled, nothing to undo there or anywhere. **Zoom**: kept for the site while the window is open, across its tabs; not the ordinary window's; gone with the window. **What a site may use**: what was allowed in an ordinary window is asked afresh in a private one, kept there while it is open, and gone with it; the ordinary window's answer is untouched. **Downloads**: each window lists its own; a number names one download; the private one's entry is written nowhere, by its address or its file's name. **No Space**: in no list of Spaces; cannot be switched; no window can be switched to its row; a link and a search from outside are refused and open nothing. **A window a private page opens** is a tab of the same private window, in its login set |
| By breaking it | Eighteen parts of the rule, each then run against these tests. Seventeen caught, one of them only after its test was made to open a second tab in each private window: with two private windows given one row number, the second window's next tab took the first one's logins, and nothing looked. The eighteenth is no break that can be seen: asking the store on disk for a private window's history finds none either, because none is ever recorded, there or in memory |

## 7. What is not verified

- **Anybody using it.** No shell opens a private window yet.
- **The engines.** That WebKit's store that is not persistent keeps nothing, and is gone when the window closes, is part 2's to show; Android's WebView refuses such a login set today.
- **That the memory store's rows are deleted when the window closes**, through the app. The store's own test shows a login set dropped with everything of it; nothing can ask the core for a closed private window's things, so no test through the contract can see them gone or still there.
- **A download still coming when its private window closes.** The file goes on being saved; its entry is gone with the window. Not tested.
- **Signing in** (017) and **a certificate gone on past** (026) in a private window: both are kept by login set in memory already, and go with the window; no test of this feature does either.

## 8. Done checklist

- [x] The owner's word on §4 (2026-11-21).
- [x] The core and the contract (contract 1.26).
- [ ] The Apple shell.
- [ ] Android.
- [x] Part 1: `Tools/test.sh all` green, and `android-device`, on 2026-11-21: `lint`, `core` (537), `unit` (268 on the Mac), `build`, the Mac's UI suite (36), `ios` (250 on the iPhone simulator, one skipped as before), the iPhone's UI suite (27), `android` (366 and 32 on the JVM) and `android-device` (150 on the emulator).
- [ ] Parts 2 and 3: the same again.

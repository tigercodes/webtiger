# 037 — Private windows (W2)

| | |
|---|---|
| **Status** | Written before the code. Nothing is built. The rule's choices (§4) are the owner's, confirmed 2026-11-21 (WT-D26) |
| **Phase** | W2 of [000](000-master-plan.md) §9: "private windows". 000 §5.6: "A throwaway login set. Nothing is written; closing it wipes it" |
| **Contract** | To come: a window opened as private; the window told that it is. `LoginSet.ephemeral` has been there since W0 |
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
| Downloads (013) | Listed, and the list is kept | The file is the person's and stays where it was saved. Its entry is in the list while the app runs, and is not written |
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
| D3 | A download's entry is listed while the app runs | Not listed at all; or listed only while the private window is open | The person asked for the file and must be able to find it. The entry names an address: it is never written, and goes when the app quits |
| D4 | Two private windows share nothing | One private login set for all private windows, as Chrome has | "Its logins are its own" is easy to say and to test. Signing in twice is the cost |
| D5 | The words "Private", and the explanation in an empty tab | A dark window, a mask, a colour | 000 §8.6: nothing is said by colour alone |
| D6 | On Android the profile is deleted at close and again looked for at start | Refuse private windows on Android | The WebView has no login set in memory. A profile that is certainly gone before anything loads next time is the same promise, kept one step later after a crash; the document for it will say so |

## 5. In what order

1. **The core and the contract**: a window opened as private has a login set of its own that is `ephemeral`; nothing of §3's table is written; the window is told. Tested through `FakeShell`, with the store read afterwards for what it holds.
2. **The Apple shell**: New Private Window (⇧⌘N on the Mac; where on an iPhone is to be settled), the words, the empty tab's explanation. The adapter suite's isolation tests for a login set that is not persistent.
3. **Android**: the throwaway profile, and its deletion at close and at start, as an isolation test on a device.

## 6. Tests

To be written with each part. The bar for the whole: after a private window has visited a site, signed in, downloaded, been asked for a permission and closed, **the store on disk and the engine's stores are searched for the site's name and hold none of it**.

## 7. What is not verified

Nothing is built.

## 8. Done checklist

- [x] The owner's word on §4 (2026-11-21).
- [ ] The core and the contract.
- [ ] The Apple shell.
- [ ] Android.
- [ ] `Tools/test.sh all` green, and `android-device`.

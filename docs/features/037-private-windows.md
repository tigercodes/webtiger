# 037 — Private windows (W2)

| | |
|---|---|
| **Status** | **The core, the contract and the Android adapter are built and tested** (parts 1 and 3 of §5): a window opened as private has a login set of its own that the engine is told to keep nothing of, and the core writes nothing of it; on Android that login set is a profile cleared with its last page and deleted at the next start. **On the Mac, on Android and on the iPhone a private window can be opened and used** (§5.3, §5.4): in the Mac's app, on an emulator and on the simulator. **On Android it asks the keyboard not to learn from what is typed in it** (§5.5). The rule's choices (§4) are the owner's, confirmed 2026-11-21 (WT-D26) |
| **Phase** | W2 of [000](000-master-plan.md) §9: "private windows". 000 §5.6: "A throwaway login set. Nothing is written; closing it wipes it" |
| **Contract** | 1.26: `OpenWindow.private_window`; `WindowSnapshot.private_window`. `LoginSet.ephemeral` has been there since W0, and is now said |
| **Builds on** | [001](001-foundations.md): login sets, and the adapters' isolation tests. [008](008-history.md), [009](009-session-restore.md), [013](013-downloads.md), [014](014-permissions.md), [012](012-zoom.md), [033](033-a-closed-tab-comes-back.md): each writes something a private window must not |
| **Last updated** | 2026-11-22 |

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

**The keyboard.** What is typed in a private window is typed through the system's keyboard, and a keyboard may learn words from what it is given: a place where what was typed is kept, outside WebTiger. Where the system lets an app ask a keyboard not to learn (Android), a private window asks, for its own fields and for a page's (§5.5). It is asked, not enforced: the keyboard is the person's own choice, and what it does with the request is its own.

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
2. **The Apple shell** (built): New Private Window (⇧⌘N) on the Mac, a window of its own (§5.4); on the iPhone the list of Spaces' last row (§5.3); the words; the empty tab's explanation.
3. **Android** (built): the throwaway profile (§5.2); the window's part, a way to open one and the words (§5.3).
4. **Android: the keyboard is asked not to learn** (§5.5).

### 5.1 How the core keeps nothing (part 1)

**The contract (1.26).** `OpenWindow.private_window` opens one; `WindowSnapshot.private_window` tells the window; a private window's snapshot names no Space (`space_id` is 0). Refusals: `window.no_private` when a private window cannot be made; `window.private` for what a private window does not do (switching Space; taking a link or a search from outside the browser).

**A row and a login set in memory only.** A private window has one row of tabs and a login set made as it opens. Neither is in the store on disk, in the list of Spaces, or among what a window can be switched to; both go when the window closes, with the certificates gone on past in it.

**A second store, in memory.** What an ordinary window keeps for its login set (a site's zoom, what a site may use, its downloads' entries), a private window keeps the same way, by the same code, in a store that is never a file. Everything the core keeps by login set goes to the store of that login set: the one on disk, or the one in memory. The login set is taken out of the memory store, with everything of it, when its window closes.

- **By construction, not by care alone.** A private login set has no row in the store on disk, and every table there names its login set or its Space: a write for a private window that reached the disk by mistake would be refused by the store itself. The tests break the rule on purpose to see that they notice.
- **Numbers that name one thing.** A download is named by a number that a shell may send back; the memory store's downloads (and visits, of which there are none) are numbered from far above any a store on disk reaches.
- **What is not kept even in memory:** places visited (nothing is offered back, D1) and closed tabs (D2). The session never has a private window in it, and a private window opened first never becomes the window whose tabs are kept.

### 5.2 Android: a profile to throw away (part 3)

The WebView has no login set in memory: a login set is a profile, and a profile is on disk. What the WebView allows ([003](003-android-shell.md) §5.2) decides the rest: a profile's data can be removed while the app runs; the profile itself can be deleted only by a run that has not used it.

- **Written down first.** Before a private login set's profile is made, its name is added to a list in the app's own files (never backed up). A profile the app is killed with is therefore known to be one to delete. If the list cannot be written, the web view is refused (`engine.no_private`).
- **Cleared with its last page.** When the last web view of a private login set is destroyed, its cookies and everything its pages stored are removed. (On a WebView older than the one that clears everything in one call, caches and service workers stay until the profile is deleted.)
- **Deleted at the next start.** Before the first web view of a run is made, of any login set, the profiles on the list are deleted and taken off it. One that will not go stays on the list.

**What this is, plainly.** While a private window is open on Android its cookies are in a file, as any login set's are. If the app is killed with a private window open, that file stays until WebTiger is next started, and is deleted then before anything loads. On Apple's systems nothing of a private window is ever in a file.

**The window's last page, not the window.** The adapter knows web views, not windows: a private window whose every tab has been closed, and which then has an empty tab, has had its logins removed, though the window is still open. The same is so in WebKit's adapter. On the safe side; a person who closes every page of a private window is signed out there.

### 5.3 On a phone: one window at a time (part 2, Android first)

A phone's app has one window on the screen. A private window there is a second window of the core's that the app shows in place of the first, which stays as it was behind it.

- **Opened from the list of Spaces**: its last row is "Private Window". It opens one, or shows the one that is open, and says "Open" then.
- **In a private window the chip says "Private"**, the whole word, on a narrow strip too ("Private Window" to a screen reader). As first built it was one letter where the strip is narrow, as a Space's chip is: seen on the emulator, a private window then looked like a Space whose name begins with P, and said nowhere on the screen what it was. The word costs the tabs some room; they scroll. It opens a page that says what private means and what it does not (§1), with three ways on: **Close Private Window** (it is gone, with everything of it, and the other window is shown); **Back to** the Space the other window shows (the private window stays open behind); **Done**.
- **An empty tab in a private window** says the same two sentences.
- **A link from another app** is shown in the other window, which comes to the front: never in the private one.
- There is no list of Spaces in a private window: it has none.

> **Provisional (2026-11-21):** where a phone's private window is opened from, and the page behind its chip, were not among the choices put to the owner (§4 said "where on an iPhone is to be settled"). Built first on Android, to be the same on the iPhone. The owner's to change.

### 5.4 The Apple shell (part 2)

**WebKit's adapter was ready.** Since W0 it has given a login set that is `ephemeral` a data store that is not persistent, and let go of it with the login set's last web view. Nothing in it changed.

**The Mac: a window of its own.** File → New Private Window (⇧⌘N) opens one, with any window in front or none. It is a window like any other on the screen, so there is no way "back" to offer: the page behind its chip has Close Private Window and Done. The system is told not to put a private window back when the app starts again, and to hand it nothing from other apps; a link that waited for a window is not opened by a private one.

Two things the Mac's app test found when it first ran, a day after the code was written (a dialog of the system's had been on the screen, and the Mac's suite does not start under one):

- **The menu bar is said on the last kind of window.** Private windows are a second kind of window of the app's (a second `WindowGroup`), and each kind brings the system's own File menu with it. With the browser's commands said on the first kind, as they had been (034), the system's own Close and Close All came back in the place of Close Tab and Close Window: ⌘W closed a window with all its tabs, and six tests of the menus and of ⌘W failed. Said on the last kind, the menu bar is the browser's again, once.
- **Close Private Window waits for the sheet to be gone.** A Mac's window does not close while a sheet is over it, and it lets go of the sheet a moment after SwiftUI says the sheet was dismissed: asked then, the window stayed. What the page was asked for is now done when the window itself says its sheet has ended (`NSWindow.didEndSheetNotification`); on a phone, when the sheet is dismissed.

**The iPhone and the iPad: one window at a time**, as Android (§5.3): the list of Spaces' last row; the app keeps both windows and draws the one that is shown. As first built it kept the other one drawn and invisible behind: it was then still there for a screen reader, and for the app's test, which found it.

**The chip** says "Private", the whole word, on every width; "Private Window" to a screen reader.

### 5.5 Android: the keyboard is asked not to learn (part 4)

Android has a word for it: an app that gives a keyboard a field to type in can say, of that field, that nothing typed there is to be learned from (`IME_FLAG_NO_PERSONALIZED_LEARNING`; Chrome's incognito tabs say it). A private window says it of every field it has:

- **A page's fields.** The web view tells the keyboard about a page's field itself. A private login set's web view is one that adds the request to whatever it tells. The adapter knows which login sets are private: it is told as the web view is made.
- **The window's own fields**: the address field, the find bar, the fields of a page's prompt and of a sign-in. Everything a private window draws is drawn inside one wrapper that adds the request to whatever Compose tells the keyboard (`InterceptPlatformTextInput`, which Compose still marks as experimental).

An ordinary window's fields are told about as before.

**Apple has no such word in public.** Nothing is asked on the Mac or the iPhone. What WebKit itself tells the keyboard about a page whose store is not persistent was not looked at.

## 6. Tests

The bar for the whole: after a private window has visited a site, signed in, downloaded, been asked for a permission and closed, **the store on disk and the engine's stores are searched for the site's name and hold none of it**. Part 1 meets it for the core's store.

| Where | What |
|---|---|
| The store (`wt-store`, 3: `private.rs`) | A private store's downloads and visits are numbered far above a real store's. A login set is added once and dropped with everything of it (visits, a site's zoom, what a site may use, downloads), another's left as it was, and nothing can be kept for it afterwards. A login set a Space has is not dropped |
| The core (`wt-app/tests/private.rs`, 9, through `FakeShell`; six over a store on disk whose files are then read whole) | **Its own login set**: the windows are told which they are; a private one names no Space and has one tab; the engine is told to keep nothing of each private window's login set, each is its own, and none is the ordinary window's. **Nothing written or offered back**: a private window has no history and is offered nothing from any, its own visits or the ordinary window's; the ordinary window is offered nothing of the private one's; left open as a crash leaves it, the files hold the ordinary site's name and not the private one's. **The session**: a private window opened first is not the one whose tabs are kept, and never comes back; the first ordinary window gets the session. **Closed tabs**: not journalled, nothing to undo there or anywhere. **Zoom**: kept for the site while the window is open, across its tabs; not the ordinary window's; gone with the window. **What a site may use**: what was allowed in an ordinary window is asked afresh in a private one, kept there while it is open, and gone with it; the ordinary window's answer is untouched. **Downloads**: each window lists its own; a number names one download; the private one's entry is written nowhere, by its address or its file's name. **No Space**: in no list of Spaces; cannot be switched; no window can be switched to its row; a link and a search from outside are refused and open nothing. **A window a private page opens** is a tab of the same private window, in its login set |
| By breaking it | Eighteen parts of the rule, each then run against these tests. Seventeen caught, one of them only after its test was made to open a second tab in each private window: with two private windows given one row number, the second window's next tab took the first one's logins, and nothing looked. The eighteenth is no break that can be seen: asking the store on disk for a private window's history finds none either, because none is ever recorded, there or in memory |
| Android's adapter on the JVM (`SystemWebViewEngineTest`, 3 more) | What an earlier run left is looked for first, then the login set is written down, then its page is made; a second page keeps it, and it is cleared once when the last is gone; an ordinary login set is neither written down nor cleared; each private login set is cleared by itself; one that cannot be written down is refused and nothing is made. Eight parts checked by breaking them: all caught |
| Android's adapter on the emulator (`SystemWebViewIsolationTest`, 3 in place of the one that expected a refusal) | A private login set has its cookie and stored value while a page of it is there, through a second page after the first is gone; with the last gone its jar empties, and a page made in it afterwards finds no cookie, no stored value and no database. It and an ordinary login set share nothing, either way. What an earlier run left is deleted before anything is made |
| By hand (2026-11-21, the emulator, the app and its tests installed once and the isolation tests run three times as three processes, the app's data kept between them) | The first run found nothing left. The second found the three private login sets the first had left and deleted all three profiles as it started; the third the same for the second's. (Run by Gradle the last test finds nothing to delete: Gradle removes the app, and its data, after each run.) |
| Android's window on the JVM (`PrivateWindowTest`, 7, drawn by Compose on Robolectric over a scripted core) | A private window is asked for as one, and is one from the start; a window the core says is private is one. It says what it is: the chip to a screen reader, the two sentences in an empty tab; its chip opens the page that says the same, not the list of Spaces, and the core is asked for none; Close and Back to the other window ask what they say, Done asks nothing. An ordinary window says nothing of private; its list of Spaces ends with the row, which asks for the private window and closes the list; the row says when one is open; a window drawn with no way to one has no row. The holder shows one window at a time: the first window is still the one other apps' links go to; left, the private one stays and is the same one when shown again; closed, the core is asked to close that window and no other, and the next is another. Eleven parts checked by breaking them: all caught |
| Android's app on the emulator (`AppPrivateTest`, 1, fake mode: the real core and the real WebView) | A private window from the list of Spaces; it signs in to the test site and the site says who it is. Left open: the other window is where it was, and is not signed in. The list says a private window is open; shown again, it is as it was left, signed in. Closed: the other window is shown, and the list no longer says one is open. Another private window is not signed in. The history the other window lists has its own pages and neither of the private window's. Passed three runs one after another, and in the whole suite twice |
| By hand (2026-11-21, the emulator, the app in fake mode) | The list of Spaces with its last row; the private window with a second, empty tab: the chip says "Private", the empty tab says the two sentences; the page behind the chip, with its three buttons. All well inside a phone's width. This is where the one-letter chip was seen and changed |
| The Apple shell's model (`PrivateModelTests`, 3, on the Mac and the iPhone simulator) | A private window is asked for as one and is one from the start; a window the core says is private is one; a link from another app that waited for a window is not opened by a private one, and is by an ordinary one |
| The iPhone's app (`PrivateTests`, 1, fake mode: the real core and real WebKit) | From the list of Spaces' last row: the window says it is private and names no Space; it signs in to the test site. The page behind the chip says what private means; Back shows the other window, which is not signed in. The list says a private window is open; shown again, it is as it was left. Closed: the other window, and the list no longer says one is open. Another private window is not signed in. Passed three runs one after another. Its first runs found the other window still there behind, and the page's name given to everything in it |
| The Mac's app (`PrivateTests`, 1, fake mode: the real core and real WebKit) | New Private Window from the File menu opens a second window that says it is private; signed in to the test site there, the ordinary window is not signed in and the private one still is; the page behind the chip says what private means, offers no way back, and Close Private Window closes that window and leaves the other; another, by ⇧⌘N, is not signed in. Broken three ways, each seen to fail: the menu bar said on the first kind of window (this test passes, six others fail: Close Tab is not in the File menu and ⌘W closes the window); the window asked to close at once, and when SwiftUI says the sheet was dismissed (the window stays) |
| The keyboard, Android's adapter on the JVM (`SystemWebViewEngineTest`, 1 more) | The page of a private login set is made as one whose keyboard is asked not to learn, and an ordinary login set's is not; a window a private page asks for is made so too, and one an ordinary page asks for is not; a second tab of each likewise |
| The keyboard, Android's window on the JVM (`PrivateWindowTest`, 2 more) | With the address field touched, the window's view is asked what the system asks it for the keyboard: a private window's field says nothing typed there is to be learned from, and still says it is a field that goes somewhere; an ordinary window's says nothing of the kind |
| The keyboard, the real WebView on the emulator (`SystemWebViewIsolationTest`, 1 more) | The sign-in form in a private login set and in an ordinary one, the cursor put in its first field by a finger: the private one's web view tells the keyboard not to learn, the ordinary one's does not |
| The keyboard, Android's app on the emulator (`AppPrivateTest`, 1 more, fake mode: the real core and the real WebView) | In the ordinary window neither the address field nor a field of the page asks; in a private window both do; back in the ordinary window the address field does not. It is the real core that says which login set is a private window's |
| The keyboard, by breaking it | Ten parts (the wrapper adding nothing; every window wrapped, and none; the engine saying no page is private, and every page; the same for a window a page opens; no web view the kind that asks, every web view, and that kind adding nothing): all caught, seven on the JVM and three on the emulator |

## 7. What is not verified

- **The iPad.** The iPhone's code; no test at its size.
- **That WebKit's store that is not persistent leaves nothing on disk.** Its adapter's isolation tests hold for it as for any login set; nobody looked at the disk.
- **The system not putting a private window back** when the Mac app starts again: asked of the system (`restorationBehavior`), not tested: the app's tests start with nothing kept.
- **"Do not offer to save here"** (§3, passwords): not built. On Android the system's Autofill is asked about a form in a private window as in any other (032).
- **A link from another app while a private window is shown**: the first window is shown again with it, by the activity's code; no test hands a link over with a private window shown.
- **The system making the window again** (a rotation, the app brought back from the background after its process was ended): the private window's model is kept with the first window's, by the same holder; that a private window ended with the process does not come back is the core's rule and is tested there, not on a device.
- **What a keyboard does with the request** (§5.5). The tests read what the keyboard is told. Whether the emulator's keyboard, or any other, then learns nothing was not looked at, and cannot be from here: a keyboard's learned words are its own.
- **The find bar's field, and the fields of a prompt and of a sign-in, in a private window**: drawn inside the same wrapper as the address field; only the address field was asked about.
- **Apple and the keyboard**: nothing is asked (§5.5).
- **Keys** (036) in a private window, and its page behind the chip while a list is open.
- **Android killed with a private window open.** The profile's file stays until the next start (§5.2). That the next start deletes it was run by hand, with profiles whose pages had been closed; a kill with pages open was not tried.
- **An older WebView**, which cannot remove everything in one call: caches and service workers of a private login set stay until its profile is deleted. Not run on one.
- **A phone.** An emulator.
- **That the memory store's rows are deleted when the window closes**, through the app. The store's own test shows a login set dropped with everything of it; nothing can ask the core for a closed private window's things, so no test through the contract can see them gone or still there.
- **A download still coming when its private window closes.** The file goes on being saved; its entry is gone with the window. Not tested.
- **Signing in** (017) and **a certificate gone on past** (026) in a private window: both are kept by login set in memory already, and go with the window; no test of this feature does either.

## 8. Done checklist

- [x] The owner's word on §4 (2026-11-21).
- [x] The core and the contract (contract 1.26).
- [x] The Apple shell.
- [x] Android: the adapter (the window's part comes with the shells').
- [x] Part 1: `Tools/test.sh all` green, and `android-device`, on 2026-11-21: `lint`, `core` (537), `unit` (268 on the Mac), `build`, the Mac's UI suite (36), `ios` (250 on the iPhone simulator, one skipped as before), the iPhone's UI suite (27), `android` (366 and 32 on the JVM) and `android-device` (150 on the emulator).
- [x] Part 3 (Android's adapter): `lint`, `android` (369 and 32 on the JVM) and `android-device` (152 on the emulator, twice from a cold start), on 2026-11-21. Nothing of the core's or of Apple's changed.
- [x] Part 2 on Android (the window): `lint`, `android` (376 and 32 on the JVM) and `android-device` (153 on the emulator, twice), with the Apple package's own tests (268 on the Mac) and the apps' build, because the shared words gained eight; on 2026-11-21. The Mac's UI suite could not be run: a dialog of the system's was on the screen.
- [x] Part 2 on the Mac and the iPhone: `Tools/test.sh all` green, and `android-device`, on 2026-11-22: `lint`, `core` (538), `unit` (271 on the Mac), `build`, the Mac's UI suite (37), `ios` (253 on the iPhone simulator, one skipped as before), the iPhone's UI suite (28), `android` (376 and 32 on the JVM) and `android-device` (153 on the emulator).
- [x] Part 4 (Android: the keyboard is asked not to learn): `lint`, `android` (379 and 32 on the JVM) and `android-device` (155 on the emulator), on 2026-11-22. Nothing of the core's or of Apple's changed, and no words: those stages were not run again.

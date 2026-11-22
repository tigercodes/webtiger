# 034 — The Mac's menus: the window's commands in the menu bar (W2)

| | |
|---|---|
| **Status** | Built and tested in the Mac's app: the window's commands are in the menu bar with their keys, grey when they can do nothing; ⌘R, ⌘[, ⌘] and ⇧⌘W are new. An iPad's keys are carried as before, by the same code, and were not run (§6) |
| **Phase** | W2 of [000](000-master-plan.md) §9: "keyboard maps per platform", the Mac's, for the commands there are so far |
| **Contract** | No change |
| **Builds on** | [005](005-tabs.md): the keys for tabs, which were carried by buttons nobody sees ("the app's menus are W2's"). [008](008-history.md), [010](010-find-in-page.md), [012](012-zoom.md): their keys, carried the same way. [033](033-a-closed-tab-comes-back.md): Reopen Closed Tab |
| **Last updated** | 2026-11-22 |

## 1. User story

On the Mac I look in the menu bar for what a browser can do, and it is there, each command with its key beside it: a new tab, the tab I closed, back and forward, reload, find, zoom, history. A command that can do nothing now is grey. The keys I know from every other browser work, among them ⌘R, ⌘[ and ⌘], which did nothing until now.

## 2. What is in it, and what is not

**In:** on the Mac, the commands of the window in front are in the menu bar, with their keys; each is grey when it can do nothing; Back, Forward, Reload and Close Window have keys.

**Not in:**

- **The iPhone, the iPad and Android.** An iPad with a keyboard keeps the keys it has (the buttons nobody sees); its own menu bar is not looked at here.
- **Commands there are not yet**: bookmarks, a new private window, print, save, the developer's tools. Each comes with its feature.
- **A Space's commands** (make, rename, switch): with the keyboard map for Spaces.
- **Changing a key.** The keys are fixed.

## 3. The rule

**The menu bar speaks for the window in front.** A browser window says what it offers; the menu's items ask it. With no browser window in front (none open, or the page that says the app could not start), every item of these is grey, but for the two that close: those close whatever window is in front, which would otherwise have no key to close it by.

**Nothing is decided in the menu.** An item calls what the window's button or key called before: the model asks the core, as ever.

**The keys move from the window to the menu.** On the Mac the buttons nobody sees are gone, but for one: ⌘= beside ⌘+ for Zoom In (a menu item has one key). On an iPad the buttons nobody sees carry every key, as before; they and the menu are written once, over one value that the window offers (`WindowCommands.swift`).

| Menu | Item | Key | Grey when |
|---|---|---|---|
| File | New Private Window ([037](037-private-windows.md)) | ⇧⌘N | |
| File | New Tab | ⌘T | |
| File | Open Location | ⌘L | |
| File | Reopen Closed Tab | ⇧⌘T | there is none to bring back (033) |
| File | Keep as Site, or Stop Keeping for a tab that is one (since [039](039-sites.md)) | ⌘D | the tab in front has no page and is no Site |
| File | Add to Favourites, or Remove from Favourites for a tab that is one (since [042](042-favourites.md)) | | the tab in front has no page and is no Site, the window is a private one, or the row has as many Favourites as it can |
| File | Move to Group, a menu: the Groups of the row by name and "No Group", the one that is the tab's marked, and "New Group…" (since [041](041-groups.md)) | | the tab in front has no page and is no Site, or the window is a private one |
| History | Return to the Site, by its name ("Return to Mail"; since 039) | ⇧⌘H | the tab in front is no Site, or is at its home: the item then says "Return to Site" |
| File | Close Tab | ⌘W | |
| File | Close Window | ⇧⌘W | |
| Edit | Find on Page | ⌘F | the tab has no page |
| Edit | Next Match | ⌘G | no find is open |
| Edit | Previous Match | ⇧⌘G | no find is open |
| View | Reload | ⌘R | the tab has no page |
| View | Stop | ⌘. | no page is loading |
| View | Zoom In, Zoom Out, Actual Size | ⌘+, ⌘−, ⌘0 | the page cannot be zoomed; there is no level further that way; for Actual Size, the page is at 100% |
| History | Back | ⌘[ | there is no way back |
| History | Forward | ⌘] | there is no way forward |
| History | History and Downloads | ⌘Y | |

**Close Tab on a window's last tab closes the window**, as ⌘W did (005). Since [039](039-sites.md) a Site that is asleep does not count as a tab to be left with: when the tab in front is the only one that is not a sleeping Site, the window closes. **Close Window** closes it however many tabs it has. They take the place of the system's own Close, which had ⌘W.

**New Window** (⌘N) is the system's own item, and stays.

## 4. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | The window offers its commands as one value, and the menu reads the window's model | The menu must be grey by what the model says, and the model is the window's. Nothing new is kept anywhere |
| D2 | The same words as the buttons | One name for one thing: what a button's label says, the menu says |
| D3 | The Mac only | The menu bar is the Mac's. An iPad's keys work as they did |
| D4 | The commands are said on the last kind of window the app has (since [037](037-private-windows.md)) | Each kind of window brings the system's own File menu with it. Said on the first of two kinds, the system's Close and Close All came back in the place of Close Tab and Close Window; the tests of §5 found it the first time they ran with private windows |

## 5. Tests

| Where | What |
|---|---|
| The Mac's app (`CommandTests`, 4, fake mode: the real core and real WebKit; only the app's own menus are asked about, each item by its name) | **The menus.** With one page and nothing closed: Reopen Closed Tab, Back, Forward and Next Match are grey. New Tab, from the File menu, opens a tab; in the empty tab Reload, Find on Page and Zoom In are grey. Close Tab closes the tab in front; Reopen Closed Tab brings it back, its page loaded again; Reload asks the site for the page once more; Find on Page opens the find bar, and Next Match can then be chosen; History and Downloads opens the lists. **The keys.** ⌘[ goes back, and Back is then grey; ⌘] goes forward; Back and Forward from the History menu do the same; ⌘R asks the site for the page again; ⇧⌘W closes a window that has two tabs. **A window that is no browser's** (the design catalogue) still closes with ⇧⌘W. **Two windows**: ⌘N opens a second, in front; sent to another page, it has a way back and the menu says so; New Tab opens a tab in it and not in the first; ⌘W closes that tab, ⇧⌘W the window; the menu then speaks for the first window, which has no way back, and New Tab opens a tab there |
| The keys there were | The app's older tests of ⌘T, ⌘W, ⇧⌘T, ⌘L, ⌘F and the rest pass unchanged, with the keys now the menu's |
| By breaking it | Six parts, each then run against `CommandTests` in the app: Reopen Closed Tab, Back, Reload and Next Match never grey; Close Window closing a tab; Forward going back. All caught |

## 6. What is not verified

- **An iPad with a keyboard.** Its keys are carried by the same code as before, moved to another file; no test presses a key on an iPad, before or now.
- **The page that says the app could not start.** Its window has nothing of a browser's; Close Tab and Close Window close whatever is in front, by AppKit's own close. The design catalogue was run, which has windows of its own inside it; a window with none was not.
- **Stop (⌘.), Zoom Out, Actual Size, Previous Match, and ⌘G.** In the menu, calling what their buttons call; no test chooses them from the menu. Their buttons and their older keys are tested.
- **Open Location from the menu.** ⌘L is pressed by many tests; the item was not clicked.
- **Other languages.** The menus File, Edit and View are the system's and take its language; the items are the app's words, of which there is English only.
- **Which window is in front when a person clicks one.** The test's second window was in front because it had just opened; no test brings a window forward by clicking it.
- **What a second window keeps.** It starts on the page a new window starts on and is not in the session ([009](009-session-restore.md)): several windows as a feature of their own are still to come.

## 7. Done checklist

- [x] The window offers its commands; the menu bar has them, with their keys.
- [x] Each is grey when it can do nothing.
- [x] ⌘R, ⌘[ and ⌘], and ⇧⌘W.
- [x] The keys there were still work: the app's tests of them pass unchanged.
- [x] `Tools/test.sh all` green, on 2026-11-21, in one run: `lint`, `core` (519), `unit` (267 on the Mac), `build`, the Mac's UI suite (34), `ios` (249 on the iPhone simulator, one skipped as before), the iPhone's UI suite (27) and `android` (354 and 32 on the JVM). `android-device` (148 on the emulator) is not part of `all`: in the same run one of its tests failed, a certificate test of the WebView adapter's, in code this feature does not touch ([003](003-android-shell.md) §8.1); run twice more, the whole suite passed both times.
- [x] Two windows (added after): lint and the Mac's whole UI suite (35) run again, 2026-11-21; nothing else changed since the run above.

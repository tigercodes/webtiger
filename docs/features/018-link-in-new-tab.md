# 018 — A link opens in a new tab (W1)

| | |
|---|---|
| **Status** | Built and tested in the core, the Android shell and the Apple shell. Both shells report the wish: Android by the menu it draws for a long press on a link, run on an emulator; Apple by ⌘-click, by the menu an iPhone shows for a long press, and by the Mac's own menu, run against real WebKit in the adapter's suite and in the apps on the Mac and the iPhone simulator. Windows is not started |
| **Phase** | W1 of [000](000-master-plan.md) §9: "context menus". The first of them, and the one a day's browsing cannot do without: the menu of a link |
| **Contract** | 1.14: `LinkOpenRequested` |
| **Builds on** | [005](005-tabs.md): tabs, and a tab opened behind another. [006](006-new-windows.md): a tab that remembers the tab it was opened from |
| **Last updated** | 2026-11-20 |

## 1. User story

I am reading a page and want one of its links without leaving it. I press and hold the link (on a phone) or right-click it (on a Mac), and choose "Open in New Tab": a tab opens behind the one I am reading, and loads the link while I read on. "Copy Link" puts the link's address where I can paste it. On a Mac, a click with ⌘ held does the first without a menu.

## 2. What is in it, and what is not

**In:** the person asks that a link open in a new tab, by the link's menu or, with a keyboard on Apple's systems, by ⌘-click (in front with ⇧ as well); the core opens the tab; "Copy Link" in the menus WebTiger draws.

**Not in:**

- **Menus of anything but a link**: a picture, a selection, the page itself. Each engine keeps what it shows for those today (on Android, the system's own bar for selected text; on Apple, WebKit's).
- **A preview of the link** on a long press. A preview loads the page before anybody asked for it.
- **"Open in New Window"**: there is one window until W2. **"Download Linked File"** and **sharing**: later.
- **The middle button**, and modifier clicks on Android: the System WebView does not say how a link was pressed.

## 3. Who does what

The shell notices that the person asks for a link in a new tab, and says so; it leaves the page where it is. The core decides whether a tab opens, where, and what it is allowed. The shell's own part is the menu and the clipboard.

**The rule.** For `LinkOpenRequested` the core opens nothing when:

- the web view is no tab's;
- the person did not do it: a page that only makes it look as if its link was pressed with a key held is not obeyed, as a page that asks for a window by itself is not (006 §3);
- the address is not one a tab starts from. Only the web's own addresses are, `http` and `https`. A script (`javascript:`), a page of no address (`data:`, `about:`, `blob:`) and a file are not; nor is an address for the system (`mailto:`), which is opened by pressing the link itself (007).

Otherwise it opens **a new tab, after the tab that has the link**: behind it, or in front when the shell says so; in the same login set. Links opened one after another from one tab stand in that order after it. The new tab remembers the tab it was opened from, and closing it brings that tab forward (006).

**The new tab is a link's, not the person's own address.** So:

- its address field is empty until its page commits (001 §4.2): where a link leads is the page's claim, however the link was opened;
- a first download there asks (015 §3): the person asked for a link, not for a file;
- a page cannot close it (`window.close()`): no page opened it.

## 4. The contract (1.14)

Added; nothing is changed or removed.

```proto
// engine.proto
message LinkOpenRequested {
  string url = 1;                           // redacted. The link's address
  bool background = 2;                      // the tab in front stays in front
  bool user_gesture = 3;                    // the person did it: a choice from a menu, a press
}
```

`EngineEvent.web_view_id` is the web view whose page has the link. The shell has refused or left alone whatever the press would otherwise have done there.

## 5. The engines and the shells

| | Mac | iPhone and iPad | Android |
|---|---|---|---|
| The menu | WebKit's own, which WebKit builds and shows. Its "Open Link in New Window" asks for a window (006), and the core opens a tab in front: the item is renamed "Open Link in New Tab". "Copy Link" is WebKit's | WebTiger's, given to WebKit when it asks what a long press on a link shows: "Open in New Tab", "Copy Link". No preview | WebTiger's, drawn by the shell when the WebView reports a long press on a link: the link's address, "Open in New Tab", "Copy Link" |
| ⌘-click | The navigation is refused in its own tab and reported as `LinkOpenRequested`, behind; with ⇧, in front | The same, with a keyboard | Not told apart from a click |

**As built on Apple** (`WebKitLinks.swift`: the page's navigation delegate, `LinkMenu`, `MenuWebView`; `LinkClipboard` in the shell), and as seen against real WebKit:

- **⌘-click.** WebKit says with each navigation which keys were held. A link of the page, pressed with ⌘ held, that would have loaded in the page's own frame at the top, is reported as `LinkOpenRequested` (behind; in front with ⇧ as well) and refused in its own tab; the page stays, and nothing is said to have ended there. A click with no key held follows the link as before.
- **A click a page makes by itself**, saying in its script that ⌘ was held: WebKit followed the link in its own tab and said nothing of a key. A page cannot ask for a tab this way.
- **The iPhone's menu** is the shell's, given to WebKit when it asks what a long press on a link shows: "Open in New Tab", which reports the wish, behind; and "Copy Link". No preview is given, and the site was asked for nothing while the menu was open. Anything that is not a link is left to WebKit.
- **The Mac's menu** is WebKit's own: WebKit builds it and does not tell an app what link it is for. Its "Open Link in New Window" asks for a window, which the core opens as a tab in front (006); the item is given the shell's words, "Open Link in New Tab", by the name WebKit gives the item. That name is not a declared constant: should WebKit change it, the item keeps WebKit's words and the app's test says so. "Copy Link" on a Mac is WebKit's.
- **The clipboard**: the system's, or in fake mode and in every session a test makes, one of the app's own by a name nothing else pastes from. (An iPhone simulator shares its clipboard with the Mac it runs on.)

**As built on Android** (`WebViewPage`, `SystemWebViewEngine.onLinkPressed` and `openLink`; `LinkMenu`, `BrowserModel`), and as seen on the emulator (System WebView 133):

- **A long press on a link** is the adapter's: the WebView is asked where the link leads (`requestFocusNodeHref`), answers in a message of its own, and the adapter tells the window. For a link in words and for a picture that is a link alike, the answer was the link's own address; what the WebView names at once for such a picture is the picture's address, which is why it is asked. A long press anywhere else is left to the WebView, which selects the words under the finger. A tap still follows the link.
- **The menu is the window's**, over the page and at its foot: where the link leads, as the engine says it, on two lines at most, its beginning first; "Open in New Tab"; "Copy Link"; "Cancel". A touch beside the menu puts it away and is not the page's; so does the system's Back. It is the menu of the page in front: a press said of a page behind shows nothing, and the menu goes when another tab comes forward.
- **"Open in New Tab"** tells the core, through the engine that saw the press, with "behind" and "the person did it". Nothing is said to the core of a press, a menu, or a copy.
- **"Copy Link"** copies to the system's clipboard; in fake mode, and in every session a test makes, to a clipboard of the session's own.

**The clipboard** is the system's, and the person's: in fake mode a shell copies to a clipboard of its own, so that no test changes what somebody had copied.

## 6. Tests

| Where | What |
|---|---|
| Core (`wt-app/tests/link_in_new_tab.rs`, 10 through `FakeShell`) | A link opens in a tab after the one that has it, behind it, in a web view of the core's own naming that is sent to the link; the page that has the link is as it was; brought forward, the new tab is the link's page, in the same login set. In front when the shell says so. Links opened one after another stand in that order after their tab, before what stood there already; a link of one of them opens after that one. A page that only makes it look so opens nothing. Only an address a tab starts from opens one: not a script, a page of no address, a file, nor what is for the system, which is not handed to the system either; a page not over `https` does. A web view that is no tab's opens nothing. The new tab's field is empty until its page commits. A first download there asks, in the name of where the file comes from. A page cannot close it, and the person's closing it brings back the tab it came from, not the one after. A load that fails shows in the new tab. Eleven parts of the rule checked by breaking them |
| The test site | A page with a link in words, a link that is a picture, words that are no link, and a link for the system |
| System WebView adapter, on the JVM (`LinksTest`, 3) | A long press on a link is told to whoever shows its menu, and not to the core; a link the WebView could say nothing of is not told of. With nobody to tell, nothing comes of the press. A link asked for in a new tab is reported for its web view, with its address, behind or in front as chosen, as the person's doing |
| System WebView adapter suite (`SystemWebViewLinkTest`, 5, on the emulator) | With a finger pressed and held on the test site's page, against the real WebView: a link in words is told with where it leads, and the page stays where it is; a picture that is a link is told with the link's address, not the picture's; words that are no link are the WebView's own; a tap still follows the link; and a link chosen is reported for the core to open |
| The Android menu (`LinkMenuTest`, 5, drawn on the JVM against `ScriptedCore`) | A link pressed and held shows where it leads and the three choices, and nothing is asked or copied until one is made. "Open in New Tab" asks the core for a tab behind, as the person's doing, and puts the menu away. "Copy Link" copies the address and asks nothing of the core. "Cancel" and a touch beside the menu put it away and do nothing. The menu is of the page in front, and goes with it. Pictures of the menu, light and dark, each looked at. Twelve of the rules, the adapter's among them, checked by breaking them |
| The Android app, on the emulator | With the real core and WebView, in fake mode: a finger is held on a link; the menu says where it leads, Android's accessibility checks pass over it, and the page stays; "Open in New Tab", and a second tab is there behind the first, which is still in front; brought forward, it is the link's page; a picture that is a link has the link's menu; a touch beside the menu puts it away and is not the page's |
| WebKit adapter suite (`WebKitLinkTests`, 6 on the Mac, 3 on the iPhone simulator) | Against real WebKit and the test site's page of links. A plain click, made by a script, still follows the link. A link asked for in a new tab is reported as the person's doing, behind or in front. A click a page makes by itself with the ⌘ key said to be held is only a click. On the Mac, with the mouse's own events given to a web view in a window that is not on the screen: a click with ⌘ held is reported with the link's address, behind, and the page stays with nothing said to have ended; with ⇧ as well, in front; a click with no key held follows the link. And the Mac's own item for a link in a new window is given the shell's words, that item and no other, and none without words of the shell's. Six of the rules checked by breaking them |
| The Apple shell (`LinkMenuTests`, 2) | A session gives its engine the menu's words for the system it runs on, and where to copy; and the app's own clipboard is not the system's, whose count of changes stays as it was |
| The Apple apps | With the real core and WebKit, in fake mode. On the Mac: a click on the page's link with ⌘ held opens a second tab behind, and the page being read stays in front; the link's menu has the shell's words for its item, and choosing it opens the link's page in a tab of its own. On the iPhone simulator: a finger held on the link shows the shell's menu, with nothing asked of the site meanwhile; "Open in New Tab" opens a second tab behind, and brought forward it is the link's page |

## 7. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | The shell reports the wish; the core opens the tab | One rule for a menu, a key and every engine; and a tab that only the core makes is one whose web view the core named |
| D2 | Behind, by default | The person said they are not done with the page they are on. Safari and Chrome do the same |
| D3 | It is an engine's report, not a request of the window | An adapter knows the web view, not the tab; and it is said of a page's link, by the engine's shell, like a page's wish for a window |
| D4 | No preview | It loads a page nobody chose to load |
| D5 | On the Mac, WebKit's own item is renamed rather than replaced | WebKit builds that menu and does not tell an app what link it is for; its item already goes through the core (006) |

## 8. What is not verified

- **The login set of the new tab.** It is the asking tab's. With one Space every tab has the same one, and a core that used the window's instead passes every test: it is a test for W2.
- **The system's clipboard.** No test copies to it: every session a test makes has a clipboard of its own, and the two lines that copy to the system's were read, not run.
- **A long press with a mouse, a pen or a keyboard's menu key** on Android: not tried. A finger was.
- **A link inside a frame**, and one that a page's script follows instead of the browser: the WebView is asked the same way; not tried.
- **Android: a phone.** It ran on an emulator.
- **⌘-click on an iPad**, with a keyboard: the same code as the Mac's, where it ran; no test holds a key on the simulator.
- **A link in a frame**, with ⌘ held: left to load in its frame, as before. Not tried.
- **A link that asks for a window of its own** (`target="_blank"`), with ⌘ held: WebKit asks for a window, and the core opens a tab in front (006), not behind.
- **"Copy Link" on an iPhone.** The app's test opens the menu and chooses "Open in New Tab"; that the other item copies is the shell's test of where it copies to, not a press on the item.
- **The iPhone's menu keeps a UI test waiting.** While the system's menu is open on the simulator, the test runner is never told that the app has come to rest, and waits a minute before each thing it does: the app's test of the menu takes over two minutes. Nothing of the shell's animates without end; what does was not found.

## 9. Done checklist

- [x] Contract 1.14 and its version in every place it is written.
- [x] The core: the rule of §3. 10 tests through `FakeShell`.
- [x] Android: the adapter's report of a long press on a link (3 on the JVM, 5 on the device); the menu and its tests (5); pictures; a device test of the app.
- [x] Apple: ⌘-click in the adapter and its suite (6 on the Mac); the iPhone's menu; the Mac's renamed item; the shell's words and clipboard (2); UI tests on the Mac and the iPhone simulator.
- [x] `Tools/test.sh all` green, and `android-device` (126 on the emulator).
- [ ] Windows shell.

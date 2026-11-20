# 018 — A link opens in a new tab (W1)

| | |
|---|---|
| **Status** | Designed. Nothing is built yet: a link can be followed in its own tab, and a page can ask for a window (006), but a person cannot ask for a link in a new tab on Android or an iPhone at all; on a Mac WebKit's own menu offers "Open Link in New Window", which opens a tab |
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
- the address is not one a tab may be sent to. The navigation policy is asked as for any page's navigation at the top level: `http` and `https` open. `javascript:`, `data:` and `file:` do not; nor does an address for the system (`mailto:`), which is opened by pressing the link itself (007).

Otherwise it opens **a new tab, after the tab that has the link**: behind it, or in front when the shell says so; in the same login set. The new tab remembers the tab it was opened from, and closing it brings that tab forward (006).

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

What each does in fact, as observed, is written here when built.

**The clipboard** is the system's, and the person's: in fake mode a shell copies to a clipboard of its own, so that no test changes what somebody had copied.

## 6. Tests

| Where | What |
|---|---|
| Core (through `FakeShell`) | The rule of §3, each part of it, and each checked by breaking it |
| Adapter suites | A ⌘-click on a link of the test site is reported and the page stays; what a plain click does is unchanged; what each engine says of a press a page made by itself |
| The shells | Android: the menu drawn against a scripted core, and what each choice does; pictures. In the apps, with the real core and engines: the link's menu is opened as a person opens it, "Open in New Tab" is chosen, and a second tab is there behind the first, with the link's page in it |

## 7. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | The shell reports the wish; the core opens the tab | One rule for a menu, a key and every engine; and a tab that only the core makes is one whose web view the core named |
| D2 | Behind, by default | The person said they are not done with the page they are on. Safari and Chrome do the same |
| D3 | It is an engine's report, not a request of the window | An adapter knows the web view, not the tab; and it is said of a page's link, by the engine's shell, like a page's wish for a window |
| D4 | No preview | It loads a page nobody chose to load |
| D5 | On the Mac, WebKit's own item is renamed rather than replaced | WebKit builds that menu and does not tell an app what link it is for; its item already goes through the core (006) |

## 8. What is not verified

Written when built.

## 9. Done checklist

- [ ] Contract 1.14 and its version in every place it is written.
- [ ] The core: the rule of §3, through `FakeShell`.
- [ ] Android: the adapter's report of a long press on a link; the menu; pictures; a device test.
- [ ] Apple: ⌘-click in the adapter and its suite; the iPhone's menu; the Mac's renamed item; UI tests.
- [ ] `Tools/test.sh all` green, and `android-device`.
- [ ] Windows shell.

# 006 — A page that asks for a new window (W1)

| | |
|---|---|
| **Status** | Built and tested in the core, the Apple shell (the WebKit adapter) and the Android shell (the System WebView adapter, on an emulator). Windows is not started. What it leaves out is in §8 |
| **Phase** | W1 of [000](000-master-plan.md) §9: "navigation, new windows and popups" |
| **Contract** | 1.2: `NewWindowRequested`, `CloseRequested`; a web view the shell names. 1.3: `NavigationFailed.refused` |
| **Builds on** | [005](005-tabs.md): tabs |
| **Last updated** | 2026-11-19 |

## 1. User story

I click a link that opens "in a new window" and it opens in a new tab beside the one I was in, in front of me. A site's sign-in opens its little window, finishes, and closes itself, and the page that opened it knows. When I close a tab a link opened, I am back in the tab I came from. A page that tries to open windows by itself, with nobody clicking, gets none.

Until now such a link's address loaded in the tab that asked ([001](001-foundations.md) F5), which loses the page the user was on and breaks every sign-in that opens a window and waits for it.

## 2. Why this is not just `OpenTab`

The page that asks for a window keeps a hold on it: `window.open` returns the new window, the new page has an `opener`, and they talk to each other. A sign-in window hands its result to the page that opened it that way. For the hold to exist, **the engine itself must make the new web view**, at the moment the page asks, from what the engine hands the shell for it. WebKit asks for it in a call that must be answered before it returns.

So here, and only here, a web view exists before the core has asked for one. The contract's rule that the core chooses every web view's id (host.proto) needs one exception, made narrowly:

- **The shell names the web view it had to make**, with an id whose top bit is set. The core's ids come from one mint that counts up from 1 and never gets there, so the two can never be the same id.
- The shell makes it in the asking web view's login set, shows it nowhere, and reports `NewWindowRequested` for the web view that asked.
- **The core then says what becomes of it, by what it does next.** It adopts it: a new tab has that web view, and the next snapshot says so. Or it refuses it: `WebViewDestroy`, the request that already exists.

Nothing else changes: the new web view's loads are reported like any other's, and the navigation policy is asked about them.

## 3. What the core decides

| The page asked… | The core |
|---|---|
| while the user was doing something to it (a click, a key) | Adopts the web view into a **new tab, after the tab that asked, in front**. The new tab remembers the tab that opened it |
| by itself | **Refuses it**: the web view is destroyed. A pop-up nobody asked for is not opened |
| from a web view the core does not have (its tab was closed meanwhile) | Refuses it |
| naming a web view by an id the shell had no right to choose (not top-bit, or one already known) | Ignores the report; destroys nothing it did not expect |

- **The new tab's address field is empty until its page commits.** Nobody sent the tab anywhere: where the page that opened it wants it to go is that page's claim, and is not shown as an address (001 §4.2). Its name in the strip is "New Tab" until then.
- **A tab a page opened may be closed by a page.** `CloseRequested` (`window.close()`) closes the tab when a page opened it, and is ignored for a tab the user opened: a page cannot close the window it was typed into.
- **Closing a tab a page opened brings its opener forward**, when the opener is still there, and otherwise the tab after it as before (005, T4).
- **A window that is refused the only place it was going is closed.** A link that opens a window on a letter to write (`mailto:`) or a number to ring is common, and the address is for the system, not for a browser: the navigation policy refuses it. When the shell reports that refusal for a tab a page opened, in which no page has ever committed and which the user has sent nowhere, the core closes the tab and the opener is in front again, as if the window had not been opened. A page that is there stays; so does a tab the user opened; and so does a window whose load was only stopped or replaced, which may yet show a page (§4).
- Engines block most windows asked for without a gesture before the shell ever hears of them. The core's rule is the same on every engine all the same; the shells set their engines to block, and report honestly what gets through.

## 4. The contract (1.2)

Added to `engine.proto`, as two more kinds of `EngineEvent`; nothing is changed or removed.

```proto
// A page asked for a new window. `EngineEvent.web_view_id` is the web view that asked.
message NewWindowRequested {
  uint64 new_web_view_id = 1;   // made and named by the shell: top bit set
  string url = 2;               // where the page wants it to go, when the engine says (redacted)
  bool user_gesture = 3;
}

// The page asked for its own window to be closed.
message CloseRequested {}
```

`host.proto`'s `WebViewCreate.web_view_id` comment gains the exception of §2. `WebViewDestroy` is unchanged and is how a refusal is said.

**1.3** adds one field, for the last rule of §3:

```proto
message NavigationFailed {
  string code = 1;
  string url = 2;
  bool refused = 3;   // the shell refused the navigation on the core's verdict,
                      // and with that the web view is left loading nothing
}
```

A refused navigation was already reported as `net.cancelled` (engine.proto's second rule), and so is a load the user stopped, and a load another load replaced. The core could not tell them apart, and closing a window on any of them would close one whose page had only gone on to somewhere else. An adapter knows which it is, and now says. With another load still on its way the refusal is not marked: that load's end says how things stand.

## 5. The adapters

| | WebKit | System WebView |
|---|---|---|
| The engine asks | `createWebViewWith:configuration:` | `onCreateWindow`, with `setSupportMultipleWindows(true)` |
| The new web view | `WKWebView` made with the configuration WebKit hands over, which is what keeps the hold | A `WebView` in the same profile, given to the transport WebView hands over |
| A gesture | WebKit asks only for windows its own blocker let through (`javaScriptCanOpenWindowsAutomatically` is set off): reported as a gesture | `isUserGesture`, as the WebView says it |
| The address | The request's | Not given: empty |
| `window.close()` | `webViewDidClose` | `onCloseWindow` |

**As built on WebKit** (`WebKitEngine.newWindow`, and `Page`'s `WKUIDelegate`), and what the adapter suite found there:

- The web view is made from the configuration WebKit hands over, which carries the asking page's data store: the new web view is in the same login set without the adapter choosing one.
- `window.open` keeps the hold: the new page has an `opener` and reaches it. **A link with a target does not**: WebKit gives the page such a link opens no opener, as if the link said `noopener`.
- WebKit's blocker is switched on for every web view (it is off by default on a Mac). A page that calls `window.open` as it loads is given nothing, and a page that clicks its own link is given nothing either: in both cases WebKit never asks the adapter, so nothing is reported and no web view is made. WebKit does not tell an app whether somebody clicked, so every window it does ask for is reported with a gesture.
- The navigation policy is asked about a link with a target in the web view that asked, before WebKit asks for a window. A refusal opens no window, and says nothing of the page that asked, which went nowhere.
- For a window script opens, WebKit asks for the web view first and about the address after, in the new web view: a refusal there is reported as one (§3's last rule). When the load had begun and was sent on to a refused address, WebKit ends it as "interrupted", and that end is what is marked.
- The shell needed no change: a window shows the web view its tab names, whoever made it.

**As built on the System WebView** (`SystemWebViewEngine.newWindow`, and `WebViewPage`'s `onCreateWindow`), and what the adapter suite found there:

- The WebView is told that its pages may ask for windows (`setSupportMultipleWindows`); without that it loads the address in the web view that asked, which is what it did until now. Its blocker is on (`javaScriptCanOpenWindowsAutomatically` is false, and stays so).
- The new web view is made in the asking page's profile, by the adapter, and handed to the WebView through the transport it gives. The engine keeps which login set each web view is in, to know.
- Ids of the shell's own have the top bit set, which in Kotlin's `Long` is the sign: they are the negative ones.
- The same as on WebKit: `window.open` keeps the hold both ways; a link with a target gives the page it opens no opener; a page that asks as it loads, or clicks its own link, gets nothing and nothing is reported. The WebView says whether somebody touched the page, and the adapter reports what it says.
- **Not the same as on WebKit:** the WebView asks for the window before it says where the window goes, and never tells the adapter the address. So the navigation policy is asked in the new web view, after it is made: a refusal there is in a web view that already exists, and that the core may already have put in a tab. On WebKit a refused link opens no window at all (though a refused `window.open` does, there too). This is what the last rule of §3 is for: the adapter says the failure was a refusal (`PageTracker.navigationRefused`), and the core closes the tab.
- A page is given a window only when somebody touched it, and a WebView takes nothing a test's script does for a touch. The device tests put the web view in a window on the screen and send it a touch down and up at the link's middle (`PageTouch`).

## 6. Tests

| Where | What |
|---|---|
| Core (`wt-app/tests/new_windows.rs`, 19 tests through `FakeShell`) | A window asked for with a gesture becomes a tab after the asking tab, in front, with the shell's web view and no `WebViewCreate`; it is a tab like any other, and can open another. Without a gesture, or from a web view the core no longer has: no tab, and the web view destroyed. Under an id that is not a shell's, or one reported twice: nothing, and nothing destroyed. The new tab's address is empty until it commits. `CloseRequested` closes a tab a page opened and not one the user opened; on a window's only tab it leaves an empty one. Closing an opened tab brings its opener forward; with the opener gone, the next. Closing the window destroys the shell's web views too. The refusal rule: a window whose only navigation is refused is closed and its opener is in front; a load stopped or replaced closes nothing, and its page may still arrive; a window that has had a page stays; a tab the user opened stays; a window the user has sent somewhere stays |
| Core (`wt-core`) | The mint never gives out an id in the shell's half, even at the end of its own |
| WebKit adapter suite (`WebKitNewWindowTests`, 11) | A link with a target reports `NewWindowRequested` with an id of the shell's own, and its address loads in the new web view and not in the one that asked. `window.open` keeps the hold both ways. The new web view is in the asking page's login set, and another login set's is in its own; the core's requests about it are answered. `window.close()` reports `CloseRequested` and destroys nothing. A page that asks as it loads, or clicks its own link, gets nothing. A refused web view is destroyed and released, and the next has a new id. A navigation the policy refuses opens no window. A window script opens on a refused address says it was refused; so does one redirected to a refused address; a load that is stopped does not |
| System WebView adapter, on the JVM (`SystemWebViewEngineTest`, 3 more) | The web view for a window is made in the asking page's login set, named with an id of the shell's own, never twice, and reported with what the WebView said of a gesture; the core's requests about it are answered; none is made for a web view that is gone, or when the WebView has no profiles or cannot be made |
| System WebView adapter suite, on the device (`SystemWebViewNewWindowTest`, 10) | The same behaviours as the WebKit run, with a finger on the page; that the navigation policy is asked about where a new window goes, and its refusal is reported as one, also after a redirect; a load that is stopped is not a refusal. `PageTrackerTest` has the same rules on the JVM |
| The apps (Mac, iPhone simulator, Android emulator) | With the real core and the real engine: click or touch a link that opens a new window; its page is in front, in a tab of its own beside the tab it came from; close it; the tab it came from is in front again, not the tab after it. And: a window opened on an address for the system (a number to ring, a letter to write) leaves no tab behind, and the page that asked is in front and still works |

## 7. Decisions

| # | Decision | Why |
|---|---|---|
| N1 | The shell names the web view it had to make, in a range of ids of its own | §2. The alternative, the core handing out spare ids ahead of time, is state on both sides for the same result |
| N2 | The core answers by adopting or destroying, with no new request | The snapshot already says which web view a tab has, and `WebViewDestroy` already exists |
| N3 | No gesture, no window | What every browser does. The mark in the address field that says a pop-up was blocked, and the way to allow one, come with site permissions (W1, later) |
| N4 | A new tab opened by a page is in front | It is what the user clicked for. "Open in a background tab" is a gesture of its own (a modifier, a menu) and arrives with context menus |
| N5 | A window refused the only place it was going is closed, and the adapter says a failure was a refusal | §3, §4. Without it every `mailto:` link that opens a window left an empty tab in front on Android. The other way to tell, the core guessing from "cancelled with nothing committed", closes windows whose first load was only replaced |

## 8. What is not verified

- Sign-in windows of real sites. The tests' pages hold on to each other the way those do; no real provider was tried.
- A physical Android device, and any WebView but the emulator's (133). What a finger does there was a test's touch on an emulator.
- Whether WebKit ever asks for a window nobody clicked for. The two ways tried (asking as the page loads; the page clicking its own link) were both stopped by its blocker, and WebKit gives an app no way to tell, so anything it does let through is reported as clicked for.
- Windows asked for with a size and a place (`window.open` with features): every one becomes a tab.
- An address for the system is still only refused: handing it to the system (the mail app, the dialler) is the System host service's, later in W1. The tab that would have been left behind is closed; nothing is opened in its place yet.
- A refusal while another load is on its way in a window that has never had a page: not marked, by design, and not seen in any test page.
- Windows: not started.

## 9. Done checklist

- [x] Contract 1.2 and its version in every place it is written.
- [x] The core: adopt or refuse; the opener; `CloseRequested`.
- [x] Core tests through `FakeShell`: 14, and four of its rules checked by breaking them (no gesture, the opener, who may close, whose ids).
- [x] The test site's pages for it: `/opener`, `/opened`, `/pops`, `/clicks`.
- [x] WebKit adapter and its suite: 8 tests, and two of its rules checked by breaking them (the blocker; a refused navigation).
- [x] A UI test in the Apple app, on the Mac and the iPhone simulator.
- [x] System WebView adapter: 3 tests on the JVM, and its suite on the emulator (8), one of its rules checked by breaking it (the blocker).
- [x] A device test in the Android app.
- [x] `Tools/test.sh all` green, and `android-device` (54 on the emulator).
- [x] Contract 1.3 and the rule it is for: the core closes a tab a page opened that is refused the only place it was going. 5 core tests, each of the rule's conditions checked by breaking it; both adapters say a refusal is one (checked by breaking it, both ways, on WebKit); an end-to-end test in each app, and the Mac's seen to fail without the rule.
- [ ] Windows shell.

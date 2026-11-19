# 006 — A page that asks for a new window (W1)

| | |
|---|---|
| **Status** | In progress. The contract (1.2) and the core are built and tested; no shell reports a window asked for yet, so in the apps such a link still loads in the tab that asked (§9) |
| **Phase** | W1 of [000](000-master-plan.md) §9: "navigation, new windows and popups" |
| **Contract** | 1.2: `NewWindowRequested`, `CloseRequested`; a web view the shell names |
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

## 5. The adapters

| | WebKit | System WebView |
|---|---|---|
| The engine asks | `createWebViewWith:configuration:` | `onCreateWindow`, with `setSupportMultipleWindows(true)` |
| The new web view | `WKWebView` made with the configuration WebKit hands over, which is what keeps the hold | A `WebView` in the same profile, given to the transport WebView hands over |
| A gesture | WebKit asks only for windows its own blocker let through (`javaScriptCanOpenWindowsAutomatically` is set off): reported as a gesture | `isUserGesture`, as the WebView says it |
| The address | The request's | Not given: empty |
| `window.close()` | `webViewDidClose` | `onCloseWindow` |

## 6. Tests

| Where | What |
|---|---|
| Core (`wt-app/tests/new_windows.rs`, 14 tests through `FakeShell`) | A window asked for with a gesture becomes a tab after the asking tab, in front, with the shell's web view and no `WebViewCreate`; it is a tab like any other, and can open another. Without a gesture, or from a web view the core no longer has: no tab, and the web view destroyed. Under an id that is not a shell's, or one reported twice: nothing, and nothing destroyed. The new tab's address is empty until it commits. `CloseRequested` closes a tab a page opened and not one the user opened; on a window's only tab it leaves an empty one. Closing an opened tab brings its opener forward; with the opener gone, the next. Closing the window destroys the shell's web views too |
| Core (`wt-core`) | The mint never gives out an id in the shell's half, even at the end of its own |
| WebKit adapter suite | A link with a target and `window.open` each report `NewWindowRequested` with a top-bit id and make a web view in the same login set; the new page has an `opener` and can reach it; `window.close()` reports `CloseRequested`; a refused web view is destroyed |
| System WebView adapter suite | The same, on the device |
| The apps | A UI test: click a link that opens a new window; a second tab is in front with the page; close it; the first is in front again |

## 7. Decisions

| # | Decision | Why |
|---|---|---|
| N1 | The shell names the web view it had to make, in a range of ids of its own | §2. The alternative, the core handing out spare ids ahead of time, is state on both sides for the same result |
| N2 | The core answers by adopting or destroying, with no new request | The snapshot already says which web view a tab has, and `WebViewDestroy` already exists |
| N3 | No gesture, no window | What every browser does. The mark in the address field that says a pop-up was blocked, and the way to allow one, come with site permissions (W1, later) |
| N4 | A new tab opened by a page is in front | It is what the user clicked for. "Open in a background tab" is a gesture of its own (a modifier, a menu) and arrives with context menus |

## 8. What is not verified

- Sign-in windows of real sites. The tests' pages hold on to each other the way those do; no real provider was tried.
- Windows asked for with a size and a place (`window.open` with features): every one becomes a tab.
- Windows: not started.

## 9. Done checklist

- [x] Contract 1.2 and its version in every place it is written.
- [x] The core: adopt or refuse; the opener; `CloseRequested`.
- [x] Core tests through `FakeShell`: 14, and four of its rules checked by breaking them (no gesture, the opener, who may close, whose ids).
- [ ] The test site's pages for it.
- [ ] WebKit adapter and its suite.
- [ ] System WebView adapter and its suite, on a device.
- [ ] A UI test in each app.
- [ ] `Tools/test.sh all` green, and `android-device`.

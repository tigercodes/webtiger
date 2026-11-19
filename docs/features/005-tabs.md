# 005 — Tabs (W1)

| | |
|---|---|
| **Status** | In progress. The contract (1.1) and the core are built and tested; the shells still draw one tab, the one in front, and their tab strips are next (§9) |
| **Phase** | W1 of [000](000-master-plan.md) §9: "tabs and windows" |
| **Contract** | 1.1: `OpenTab`, `CloseTab`, `ActivateTab`, `TabOpened`, `TabSummary`; `WindowSnapshot.tabs` |
| **Last updated** | 2026-11-19 |

## 1. User story

I can have several pages open in one window. I open a new tab, and it is in front of me, empty, ready for an address. I switch between tabs and each is where I left it. I close a tab and the one beside it comes forward. The window never ends up with nothing in it.

W0's window had exactly one tab ([001](001-foundations.md) §5). Everything else in W1 (history, downloads, session restore, links that open tabs) is about tabs in the plural, so this comes first.

## 2. What is in it, and what is not

**In:** several tabs in a window; opening one (empty, or on an address; in front, or behind); closing one; bringing one forward; the window's snapshot listing its tabs; a tab strip in each shell, enough to use them.

**Not in, each with where it goes:**

- **A page asking for a new window** (a link with a target, `window.open`). Today its address still loads in the tab that asked (001 F5). It needs a web view the engine itself made, for the page that asked to keep its hold on the one it opened, and so a way for the core to adopt a web view it did not ask for: its own feature, next.
- **Moving a tab** within a window or to another window: with the tree and drag and drop (W2).
- **Sleeping tabs and the live-tab limit** (W3). Every tab that has had a page keeps its web view until it is closed.
- **Restoring tabs** after a quit or a crash: the store and the journal come first (W1, later).
- **The real layouts.** The sidebar, the tab strip proper, the phone's switcher and the keyboard maps are W2 (000 §6.1). What is built here is one plain strip, the same on every size of window.

## 3. The model

A window has an ordered list of tabs and one of them in front. **A window always has a tab.**

| Request | What the core does | Answer |
|---|---|---|
| `OpenTab { window_id, text, background, after_tab_id }` | Makes a tab: after `after_tab_id` when that tab is in the window, at the end otherwise. With `text`, loads it as if typed in the tab's address field (address or search: the core decides). Brings it in front unless `background` | `TabOpened { tab_id }`; `window.unknown` |
| `CloseTab { tab_id }` | Removes the tab and has its web view destroyed. If it was in front, the tab that came after it comes forward, or the one before when it was the last. If it was the window's only tab, an empty tab takes its place | `Ack`; `tab.unknown` |
| `ActivateTab { tab_id }` | Brings the tab in front in its window | `Ack`; `tab.unknown` |

- **Closing the last tab does not close the window.** Whether ⌘W on a window's last tab closes the window is a shell's idiom (it sends `CloseWindow`); the core's rule is only that a window is never empty.
- **A tab behind the others is as alive as the one in front.** Its web view exists, its loads go on, and what its engine reports changes its tab. Nothing is slept here (W3).
- **An address is still never the page's before it commits**, tab by tab: the rule of 001 §4.2 is each tab's own.
- **Requests keep their order.** A tab opened with text has its web view asked for, then its load, before `OpenTab` returns, as `SubmitAddress` does (001 §4.1).

## 4. The contract (1.1)

Added to `browser.proto`; nothing is changed or removed.

```proto
message WindowSnapshot {
  uint64 window_id = 1;
  TabView tab = 2;                 // the tab in front, whole (as in 1.0)
  repeated TabSummary tabs = 3;    // every tab, in the order they are shown
}

message TabSummary {
  uint64 tab_id = 1;
  string title = 2;    // the page's title; empty: the shell shows `host`
  string host = 3;     // the host of the address the tab shows, in ASCII
  bool loading = 4;
}
```

- **The tab in front is `tab`, whole; the rest are summaries.** A shell draws one page and its address field, and a strip of names. Sending every tab whole would send forty tabs' progress to draw one progress line. The tab in front is `tab.tab_id`; there is no separate field to disagree with it.
- **A shell built for 1.0 still works**: it draws `tab`, which is now whichever tab is in front.
- A summary's `host` follows the same rule as the address field: it is where the *user* sent the tab until a page commits, and otherwise the page that owns the web view.

`WindowOpened.tab_id` stays: it is the window's first tab. A shell no longer keeps it as "the window's tab": the tab its controls act on is the one in front in the last snapshot.

## 5. The shells

Each shows a strip of the window's tabs above the toolbar: a tab's title (or its host, or "New Tab"), a close button, the tab in front marked, and a button for a new tab at the end. The strip scrolls when the tabs do not fit. It is the same on a phone as on a desktop for now (§2).

- The window's controls (back, forward, reload, the address field) act on the tab in front.
- **Only the tab in front has its web view in the window.** The others' web views stay in the engine, out of every window, as a sleeping tab's will not.
- A new tab is empty and in front, and the keyboard goes to the address field: there is nothing else to do in an empty tab.
- Mac: ⌘T opens a tab; ⌘W closes the one in front.

## 6. Tests

| Where | What |
|---|---|
| Core (`wt-app/tests/tabs.rs`, 21 tests through `FakeShell`) | A new window has one tab and lists it. Opening: empty, on an address, behind, after a given tab, in an unknown window. Activating. Closing: a tab behind, the tab in front (which neighbour comes forward), the only tab, an unknown tab; the web view is destroyed. A tab behind goes on loading and its summary follows. A tab behind cannot put an address in the field. Closing the window destroys every tab's web view. One snapshot for changes that arrive together |
| Contract | The version is 1.1 in its three places |
| Apple | The window's model over `ScriptedCore`: what it sends for each gesture, and that its controls follow the tab in front. A UI test: open a second tab, load in it, switch back, close. The catalogue's pictures, with a strip |
| Android | The same, on the JVM and on the device |

## 7. Decisions

| # | Decision | Why |
|---|---|---|
| T1 | A window always has a tab | An empty window is a state with no gesture that makes sense in it. A shell that wants ⌘W to close the window says so with `CloseWindow` |
| T2 | The snapshot carries the tab in front whole and the others as summaries | §4 |
| T3 | A new tab goes at the end unless the request says after which | ⌘T opens at the end in every browser; a link's tab opens beside the tab it came from, which the request can say when that feature arrives |
| T4 | Closing the tab in front brings forward the one after it | What most browsers do when a tab has no opener to go back to. The opener's turn comes with the feature that makes tabs have one |
| T5 | One plain strip on every size of window, for now | The layouts are W2's. A switcher for phones built here would be built twice |

## 8. What is not verified

- More tabs than a strip can show at once, by a person. The strip scrolls; nobody has used forty tabs.
- Memory with many tabs alive: every tab keeps its web view until W3.
- Windows: not started.

## 9. Done checklist

- [x] Contract 1.1 and its version in three places (and Kotlin's constant).
- [x] The core: several tabs, their order, the one in front; open, close, activate.
- [x] Core tests through `FakeShell`: 21, and two of its rules checked by breaking them.
- [ ] Apple shell: the strip, ⌘T and ⌘W, the model's tests, a UI test, the pictures.
- [ ] Android shell: the strip, the model's tests, the device suite, the pictures.
- [ ] `Tools/test.sh all` green.

# 010 — Find on the page (W1)

| | |
|---|---|
| **Status** | In progress. The contract (1.6) and the core are built and tested; no shell finds anything yet (§9) |
| **Phase** | W1 of [000](000-master-plan.md) §9: "find". The `WebViews` host service's find (000 §5.5) |
| **Contract** | 1.6: `FindInPage`; the host request `WebViewFind`; `FindResult`; `TabView.find` |
| **Last updated** | 2026-11-19 |

## 1. User story

I want a word on a long page. I open Find, type it, and the page shows me the first place it is; I go to the next and the one before; I am told how many there are where the engine counts them, and that there is none when there is none. When I close Find the marks are gone.

## 2. What is in it, and what is not

**In:** finding text in the page of the tab in front, forwards and backwards; what was found, in the tab's snapshot; a find bar in each shell.

**Not in, each with where it goes:**

- **Matching case, whole words, regular expressions.** The engines' own find is what is asked: it ignores case and wraps round. Options arrive with settings.
- **A count on Apple.** WebKit's public find says whether the text was found and shows it; it does not count. The bar says so little there on purpose (§5).
- **Finding in every tab, or in history** (the command bar, W2; history has its own words, 008).
- **Marks in the scroll bar.** The engines' own.

## 3. Who does what

The shell draws the bar and sends what is typed. The core passes it to the tab's web view and keeps what was found, for the snapshot. The engine finds.

| Request | What the core does | Answer |
|---|---|---|
| `FindInPage { tab_id, text, backwards }` | Asks the tab's web view to find `text`: the first match, or with the same text again the next one, or the one before with `backwards`. **Empty text stops finding**: the marks are cleared, and the tab has no find | `Ack`; `tab.unknown`; `tab.no_page` for a tab with no web view |

- **What was found is the tab's** (`TabView.find`): the text looked for, whether it is there, and, where the engine counts, how many matches there are and which one is shown. A shell draws it; it keeps no count of its own.
- **A result is for the text now looked for.** One that arrives after finding has stopped is dropped: the engine reports late.
- **A new page ends the find.** When the tab's page commits, the marks went with the old page: the tab has no find, and the shell's bar shows no result until something is looked for again.
- **Whether the bar is open is the shell's.** The core holds no bar. A shell that closes its bar, or brings another tab in front, sends empty text for the tab it was finding in.
- **Nothing typed is logged**: the text is marked for redaction wherever it travels, like an address.

## 4. The contract (1.6)

Added; nothing is changed or removed.

```proto
// browser.proto
message FindInPage {
  uint64 tab_id = 1;
  string text = 2;        // redacted. Empty: stop finding
  bool backwards = 3;
}

// What a tab's find has found: TabView.find. Not set while the tab has no find.
message FindView {
  string text = 1;        // redacted
  bool answered = 2;      // the engine has said something about this text: until then, nothing is known
  bool found = 3;         // with `answered`: the text is on the page
  bool counted = 4;       // the engine counts: `matches` and `current` mean something
  uint32 matches = 5;
  uint32 current = 6;     // which match is shown, from 1; 0 when none
}

// host.proto: WebViews. Finds text in a web view, or with empty text clears what was found.
message WebViewFind {
  uint64 web_view_id = 1;
  string text = 2;        // redacted
  bool backwards = 3;
}

// engine.proto: what a WebViewFind found. Reported after each, and again if the engine goes on counting.
message FindResult {
  bool found = 1;
  bool counted = 2;
  uint32 matches = 3;
  uint32 current = 4;
}
```

## 5. The shells

A find bar under the toolbar: a field, "previous" and "next", what was found, and a button to close. It is opened by a button beside the tabs (⌘F on a Mac). Return goes to the next match. Closing the bar, or another tab coming in front, stops the find.

| | WebKit | System WebView |
|---|---|---|
| Finds | `WKWebView.find(_:configuration:)`, which shows the match by selecting it | `findAllAsync` for new text, `findNext` for the same text again |
| Counts | No: found or not | Yes: how many, and which one (`FindListener`) |
| Stops | Clears the selection the find made | `clearMatches` |

What the bar says: "3 of 12" where the engine counts; "Not found" when the text is not there; nothing otherwise.

## 6. Tests

| Where | What |
|---|---|
| Core (`wt-app/tests/find.rs`, 10 through `FakeShell`) | The request reaches the tab's web view with its text and direction; the same text again, and backwards. A tab with no page, and no such tab. What the engine reports is the tab's find: found, counted, how many, which; not found; an engine that does not count. Other text starts from nothing known. A result after finding stopped is dropped. Empty text stops, and asks the web view to clear. A new page ends the find, and the page changing its own address does not. A tab behind keeps its own find. A count the engine cannot mean (the seventh of three) is not passed on. Four of the rules checked by breaking them |
| WebKit adapter suite | Text that is there is found, and is the page's selection; again finds the next; backwards the one before; text that is not there is not found; stopping clears the selection |
| System WebView adapter | The same, on the JVM (what is asked of the page) and on the device, with the counts |
| The shells' models and bars | What the bar sends for typing, Return, previous, next and closing; what it says for a count, for not found, and for nothing; pictures |
| The apps | With the real core and engine: open Find, look for a word that is there and one that is not, close |

## 7. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | Find goes through the core, though the core decides nothing about it | It is the `WebViews` service's (000 §5.5): one request for every shell, the command bar and plugins later, and a tab's find in its snapshot like everything else a window draws |
| D2 | Empty text is "stop" | One request and one host request instead of two of each |
| D3 | No count on Apple, rather than one of our own | Counting needs script run in the page. The engine's own word, found or not, is the honest one until WebKit says more |

## 8. What is not verified

- Written when built.

## 9. Done checklist

- [x] Contract 1.6 and its version in every place it is written.
- [x] The core: the request, the tab's find, the rules of §3.
- [x] Core tests through `FakeShell`: 10.
- [ ] Apple: the adapter and its suite; the bar; pictures; a UI test.
- [ ] Android: the adapter, on the JVM and the device; the bar; pictures; a device test.
- [ ] `Tools/test.sh all` green, and `android-device`.

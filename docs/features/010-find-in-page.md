# 010 — Find on the page (W1)

| | |
|---|---|
| **Status** | Built and tested in the core, the Apple shell and the Android shell (on an emulator). Windows is not started. What it leaves out is in §2 |
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

**As built on Android** (`WebViewPage.find`, `PageTracker.findResult`; the bar in `BrowserWindow`, its state in `BrowserModel`), and what the device suite found:

- New text is `findAllAsync`; the same text again is `findNext`, forwards or back; empty text is `clearMatches`. A new page, and a web view made anew, start from nothing: the same word in a new page is a new find, from its first match.
- **The WebView counts**, and the bar says "2 of 3". The WebView counts from 0 and the contract from 1; the tracker turns one into the other, and passes on no number the WebView could not mean.
- **The WebView finds nothing in a page that is not on screen**: asked the same things of a web view in no window, it answered "none" every time. The bar finds only in the tab in front, which is shown, so nothing follows from it yet.
- The count is its own thing beside the field, not part of it: inside the field's own box, a screen reader was given it as part of what was typed.
- The system's Back closes the bar before it does anything to the page.
- With a third thing beside the tabs, the strip has less room: on a narrow phone with three tabs the button for a new tab is past the strip's end, and the strip is scrolled to reach it. The strip is W2's to lay out again.

**As built on Apple** (`Page.find` in the WebKit adapter; the bar in `BrowserWindowView`, its state in `BrowserModel`):

- WebKit's own find, as the adapter suite found it: it ignores case, goes round from the last match to the first and back, and shows a match by making it the page's selection. Stopping lets go of that selection, asked in a world of the app's own where no script of the page's runs. It says found or not, and no more: the bar says "Not found", or nothing.
- ⌘F opens the bar, ⌘G and ⇧⌘G go to the next match and the one before, Return goes to the next, Escape closes. The bar takes the keyboard when it opens.
- The find is the tab's it was begun in: another tab coming in front closes the bar and stops the find there, not in the tab that came forward. There is nothing to find in a tab with no page, and the button says so by being off.
- A picture of the bar must not depend on when the window got round to opening it: the first time, one of four pictures was taken before the bar was there. The catalogue's model starts with its bar open.
- On an iPhone Apple's audit, run with the bar open, faulted the test page's own small text and the system keyboard's row of suggestions. Neither is the app's: the helper passes over what WebKit's elements and the keyboard's say they are, with the reason beside each.

## 6. Tests

| Where | What |
|---|---|
| Core (`wt-app/tests/find.rs`, 10 through `FakeShell`) | The request reaches the tab's web view with its text and direction; the same text again, and backwards. A tab with no page, and no such tab. What the engine reports is the tab's find: found, counted, how many, which; not found; an engine that does not count. Other text starts from nothing known. A result after finding stopped is dropped. Empty text stops, and asks the web view to clear. A new page ends the find, and the page changing its own address does not. A tab behind keeps its own find. A count the engine cannot mean (the seventh of three) is not passed on. Four of the rules checked by breaking them |
| WebKit adapter suite (`WebKitFindTests`, 6) | Text that is there is found, not counted, and is the page's selection; the same text again goes to the next, whatever its case, and round to the first; backwards goes to the one before, and round to the last; text that is not there is not found and nothing is shown; empty text clears what was shown and reports nothing; a web view that is not there is refused with a code |
| System WebView adapter | On the JVM: a find reaches the page of the web view the core named, and a web view that is not there is refused (1); what a find found is counted from one, and a number the WebView could not mean is not passed on (1). On the device (`SystemWebViewFindTest`, 8): found and counted, whatever its case; next, and round to the first; backwards, and round to the last; not found; other text starts from its first match; empty text stops, and the next find starts again; a new page is looked in from its first match; a web view that is not there |
| The shells' models and bars | Apple (`BrowserModelTests`, 6 more): what is typed is looked for at once, and nothing while the bar is shut; next and previous; closing stops the find and empties the bar; emptying the field stops it and leaves the bar open; another tab in front stops it in the tab that had it; nothing to find in a tab with no page; what the bar says is what the core says, for a count, for an engine that does not count, for not found, and for text the core has not spoken of yet. Pictures of the bar with a count and with "Not found", light and dark, on the Mac and the iPhone. Android: the same six of the model, and the bar drawn (3: shut until asked for and off with no page; what it sends for typing, next, previous, Return and Done; what it says); its pictures, with no caret in them |
| The apps | Apple, on the Mac and the iPhone simulator, with the real core and engine: the bar is shut until asked for; a word that is there is found without a word from the bar; next and previous; the bar passes Apple's audit; closed and opened again it is empty; a word that is not there is said not to be. Android, on the emulator: the same, with the count following next and previous ("1 of 3", "2 of 3", "1 of 3"), Android's accessibility checks over the bar, and the system's Back closing it |

## 7. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | Find goes through the core, though the core decides nothing about it | It is the `WebViews` service's (000 §5.5): one request for every shell, the command bar and plugins later, and a tab's find in its snapshot like everything else a window draws |
| D2 | Empty text is "stop" | One request and one host request instead of two of each |
| D3 | No count on Apple, rather than one of our own | Counting needs script run in the page. The engine's own word, found or not, is the honest one until WebKit says more |

## 8. What is not verified

- **That the match is shown where a person looks.** The adapter suite reads the page's selection, which is how WebKit shows a match; that it is scrolled into view and drawn was not looked at by a test.
- **A page in frames**, and text across elements: the engines' own find decides, and was not tried.
- **A physical Android device.** The find was used on an emulator.

## 9. Done checklist

- [x] Contract 1.6 and its version in every place it is written.
- [x] The core: the request, the tab's find, the rules of §3.
- [x] Core tests through `FakeShell`: 10.
- [x] Apple: the adapter and its suite (6); the bar and the model's tests (6); pictures; a UI test on the Mac and the iPhone simulator.
- [x] Android: the adapter, on the JVM (2) and the device (8); the bar and the model's tests (9); pictures; a device test of the app.
- [x] `Tools/test.sh all` green, and `android-device` (75 on the emulator).
- [ ] Windows shell.

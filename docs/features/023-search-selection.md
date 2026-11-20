# 023 — Searching for words that are selected (W1)

| | |
|---|---|
| **Status** | In progress. The contract (1.17), the core's rule and the Android shell (on an emulator) are built and tested. The Apple shells are not: there the item is still the system's, which hands the words to the system's choice of search engine, in another app (§5, as found) |
| **Phase** | W1 of [000](000-master-plan.md) §9: "context menus". After a link's ([018](018-link-in-new-tab.md)) and a picture's ([022](022-picture-menu.md)): the menu of words that are selected |
| **Contract** | 1.17: `SearchRequested` |
| **Builds on** | [020](020-search-engine.md): the person's choice of where searches go. 018: a tab opened from another. 022: the Mac's script that says what a right-click was on |
| **Last updated** | 2026-11-20 |

## 1. User story

I am reading a page and want to know more about something it says. I select the words and choose, from the menu that selecting gives me, to search for them. A new tab opens in front with what the search engine I chose has to say, and the page I was reading is still there behind it.

## 2. What is in it, and what is not

**In:** the menu of selected words has one item that searches for them; it tells the core, which opens the search in a new tab, with the search engine the person chose; no item of that menu sends the words anywhere else. Where the shell can put an item of its own (Apple's systems), it is named for the engine and the system's is not shown; on Android the WebView's own item is kept, and what it asks for is answered by the core.

**Not in:**

- **The rest of the menu of selected words**: copying, looking up, translating, sharing, speaking. They are the system's own, and stay.
- **Searching from anywhere else** than a page's selected words: the address field does that already (001, 020).
- **Words selected in the app's own fields** (the address field): the system's menu there is the system's.

## 3. The rule

**Why.** 020 §3 says: what is searched for goes to the chosen engine and nowhere else. The system's item breaks that in two ways at once: the words go to the engine the system is set to, and they are opened in whatever app the system calls its browser. A person who chose an engine in WebTiger, and then searches from inside WebTiger, has both go wrong without being told.

**The shell** notices that the person asks to search for words of a page, and says so (`SearchRequested`), with the words. It decides nothing: not where the search goes, and not whether a tab opens.

**The core** opens nothing when:

- the web view is no tab's;
- the person did not do it: the shell says so only of a choice in a menu;
- there are no words: what is left when the white space at the ends is taken off, and every run of white space inside is one space, is empty.

Otherwise it opens **a new tab, after the tab that has the words, in front**, in the same login set, and loads there the chosen engine's search for the words (020). The words are always searched for: they are never taken for an address, whatever they look like, as words typed in the address field can be. Words longer than 500 characters are cut there: a search for a whole page is nobody's wish.

**The new tab is the person's own search**, as one typed in the address field is: its field shows the search's address from the start, since the person asked for it and its item named where it goes. It remembers the tab it was opened from, and closing it brings that tab forward (006).

**Nothing is sent until the item is chosen.** Selecting words, and opening the menu, tell nobody anything.

## 4. The contract (1.17)

Added; nothing is changed or removed.

```proto
// engine.proto
message SearchRequested {
  string text = 1;                          // redacted. The words that were selected
  bool user_gesture = 2;                    // the person did it: a choice from a menu
}
```

`EngineEvent.web_view_id` is the web view whose page has the words.

## 5. The engines and the shells

| | Mac | iPhone and iPad | Android |
|---|---|---|---|
| The system's item | "Search with Google" (or whichever engine the system is set to), in WebKit's menu | "Search Web", in the menu of selected text | "Web search", in the WebView's bar for selected text; offered only when the WebView can see something that answers a web search |
| What becomes of it | Replaced by the shell's | Taken out, and the shell's put in | Kept. What it asks the system for is heard by the adapter, and told to the core: nothing is started |
| The item's words | "Search with DuckDuckGo", naming the engine the person chose | The same | The WebView's own, "Web search" |
| The words searched for | From the adapter's own script, as the right-click happens (022) | Asked of the page when the item is chosen | Named by the WebView, in what it asks for |

**As found on the Mac, before this** (macOS 27, in the app in fake mode): WebKit's menu for selected words has "Copy", "Copy Link with Highlight", "Look Up …", "Search with Google", "Share…", "Speech" and "Translate …". "Search with Google" was not chosen, and what it does was not seen: a test that chose it would start another app and send its words to the internet. As WebKit's published source has it, the item hands the words to the system's search service, which opens them in the system's browser with the system's engine. The item's own name says as much: Google, where this app's searches went to DuckDuckGo.

**The item names the engine** (020 D4: what leaves the device, and to whom, is said where it is about to happen). The shell knows the name from the core's answer about where searches go, and gives the adapter the item's words again whenever the choice changes.

**Where the words cannot be had** (the Mac's script said nothing of the click), the item is not shown: an item that does nothing is not shown (022 D5).

**As found on Android, before this** (System WebView 133, on the emulator): with a finger held on a word, the WebView's bar offered "Copy", "Share", "Select all" and "Read aloud", and no search. The WebView offers "Web search" only when it can see an app that answers a web search; an app sees only its own parts and what it has asked to see, and WebTiger asks to see no such apps. So nothing left the app on Android, and nothing could be searched for either.

**As built on Android** (`SearchingContext`, `SearchSelectionActivity`, `WebViewPage`, `SystemWebViewEngine`), and as seen on the emulator:

- **The manifest names an activity of the app's own as something that answers a web search.** It is not exported: no other app sees it or can start it, and the app gains no entry in any other app's menus. With it there, the WebView's bar offered "Web search".
- **The WebView starts its search by asking the context it was made with** to start whatever answers a web search, with the words. The adapter's WebViews are made with a context of the adapter's own, which takes that one request for what it is and tells the adapter the words; nothing is started, the app's own activity included. Every other request goes on as it was.
- **The adapter reports** `SearchRequested` for the web view, with the words as the WebView named them and as the person's doing. Seen: a finger held on a word of the test site's page selected it; the item, chosen as a press chooses it, reported that word; nothing asked the system to start anything; the page stayed where it was.
- **The WebView decides** when the bar is shown and whether a search is offered for what is selected, and names the words: the adapter reads nothing of the page.
- **The item's words are the WebView's**, "Web search", in the WebView's own translation: they do not name the engine. Where searches go is said on every empty tab (020), and the new tab's address field shows it from the start.
- **The activity is never shown.** Started some other way, it does nothing and goes.

What the Apple shells do in fact, as observed, is written here when built.

## 6. Tests

| Where | What |
|---|---|
| Core (through `FakeShell`) | Words searched for open a tab after the one that has them, in front, loading the chosen engine's search for them; the page that has the words is as it was. They go to the engine chosen, and to another after another is chosen. Words that look like an address are searched for all the same. No words, and a page that only makes it look as if the person asked, open nothing. Long words are cut. The new tab's field shows the search's address from the start. A web view that is no tab's opens nothing |
| Core, as built (`wt-app/tests/search_selection.rs`, 7; `wt-core`, 1) | All of the row above; and the words' tidying and cutting by itself. Six parts of the rule checked by breaking them |
| System WebView adapter, on the JVM (`SearchingContextTest`, 3; `LinksTest`, 1 more) | A request for a web search is heard, with its words, however it is asked, and starts nothing; one with no words is heard as none; every other request goes on as it was. The engine reports a search for a web view's words as the person's doing. Six of the rules checked by breaking them |
| System WebView adapter suite (`SystemWebViewSearchTest`, 1, on the emulator) | Against the real WebView, with a finger held on a word of the test site: the word is selected; the WebView's bar offers "Web search"; that item, chosen as a press chooses it, is reported with the word, as the person's doing; nothing asked the system to start anything (the test stops and counts whatever does); the page is where it was. Without the manifest's activity the item was not offered |
| The adapters' own rules, Apple | What the shell's item is called; that the system's item is not in the menu; that choosing the shell's reports the words as the person's doing |
| Adapter suites, Apple | The words selected in a page are the ones reported |
| The apps | The menu of selected words has the shell's item, named for the engine, and not the system's. **No test chooses it**: a search goes to the internet |

## 7. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | No item of the menu sends the words anywhere but to the core. The system's item is replaced where the shell can put its own; on Android it is kept, and answered | Two items that search, one of which leaves the app for another engine, is a trap. On Android the WebView's item, heard by the adapter, is the one way that keeps the WebView's own judgement of when a search may be offered and of what the words are |
| D5 | On Android nothing is offered to other apps | An exported entry ("search with WebTiger" in every app's menu of selected text) is what the system's other way of adding to that bar needs. It is a thing a person may want, and a part of the app that other apps can start: the owner's to decide, and not needed for this |
| D2 | A new tab, in front | The person asked to see the search now, and has not finished with the page: Chrome does the same. Safari replaces the page |
| D3 | Always a search | The item says "Search". Words that happen to look like an address are still what the person wants to know about |
| D4 | The core cuts long words; the shell does not | One rule for every shell |

## 8. What is not verified

- **Android: any other WebView.** That the WebView asks the context it was made with, and no other, was seen on WebView 133. A WebView that asked the system some other way would have the words go to whatever the system starts for a web search, which could be another app, or a chooser with the app's own answerer in it, which does nothing.
- **Android: the search itself, in the app.** No test chooses the item with a core behind it: the tab would load the search engine. The adapter suite chooses it with no core; the core's own tests open the tab against a scripted engine.
- **Android: words selected in a frame, in a field, and a password's field.** The WebView's own judgement, not looked at.
- **Android: a phone.** It ran on an emulator.
- **The Apple shells**: not built.

## 9. Done checklist

- [x] Contract 1.17 and its version in every place it is written.
- [x] The core: the rule of §3 (7 tests through `FakeShell`).
- [ ] The Mac.
- [x] Android: the WebView's own item, heard by the adapter; the adapter suite on the emulator.
- [ ] iPhone.
- [ ] `Tools/test.sh all` green, and `android-device`.
- [ ] Windows shell.

# 023 — Searching for words that are selected (W1)

| | |
|---|---|
| **Status** | In progress. The contract (1.17), the core's rule, the Android shell (on an emulator) and the Mac are built and tested, the Mac's app among them: its own test of its menu first ran on 2026-11-21, found the search missing from the app's menu, and passes since that was put right (§5). The iPhone is not built: there the item is still the system's, which hands the words to the system's choice of search engine, in another app |
| **Phase** | W1 of [000](000-master-plan.md) §9: "context menus". After a link's ([018](018-link-in-new-tab.md)) and a picture's ([022](022-picture-menu.md)): the menu of words that are selected |
| **Contract** | 1.17: `SearchRequested` |
| **Builds on** | [020](020-search-engine.md): the person's choice of where searches go. 018: a tab opened from another. 022: the Mac's script that says what a right-click was on |
| **Last updated** | 2026-11-21 |

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

**What the Mac app's test found** (2026-11-21, the first time the Mac's UI suite ran over this). In the app, the menu of a word that was right-clicked had no search at all: WebKit's item was there, the script had said the word, and the shell had no words for the item, so it was taken out, as the rule above has it. The words had been given and then taken away. They are the session's, shared by every window's model; a model gave them to the adapter whenever what it knew of where searches go changed, and a model that is new knows nothing. SwiftUI makes a window's view again whenever it likes, each time with a model that is thrown away: each of those took the words away, and the model that is kept, knowing what it knew, never gave them back. Now the words change only when the core has said where searches go, and never because a model was made. No test of the model made a second one over the same session; one does now, and fails without the change.

**As found on Android, before this** (System WebView 133, on the emulator): with a finger held on a word, the WebView's bar offered "Copy", "Share", "Select all" and "Read aloud", and no search. The WebView offers "Web search" only when it can see an app that answers a web search; an app sees only its own parts and what it has asked to see, and WebTiger asks to see no such apps. So nothing left the app on Android, and nothing could be searched for either.

**As built on Android** (`SearchingContext`, `SearchSelectionActivity`, `WebViewPage`, `SystemWebViewEngine`), and as seen on the emulator:

- **The manifest names an activity of the app's own as something that answers a web search.** It is not exported: no other app sees it or can start it, and the app gains no entry in any other app's menus. With it there, the WebView's bar offered "Web search".
- **The WebView starts its search by asking the context it was made with** to start whatever answers a web search, with the words. The adapter's WebViews are made with a context of the adapter's own, which takes that one request for what it is and tells the adapter the words; nothing is started, the app's own activity included. Every other request goes on as it was.
- **The adapter reports** `SearchRequested` for the web view, with the words as the WebView named them and as the person's doing. Seen: a finger held on a word of the test site's page selected it; the item, chosen as a press chooses it, reported that word; nothing asked the system to start anything; the page stayed where it was.
- **The WebView decides** when the bar is shown and whether a search is offered for what is selected, and names the words: the adapter reads nothing of the page.
- **The item's words are the WebView's**, "Web search", in the WebView's own translation: they do not name the engine. Where searches go is said on every empty tab (020), and the new tab's address field shows it from the start.
- **The activity is never shown.** Started some other way, it does nothing and goes.

**As built on the Mac** (`WebKitMenus.swift`: the script, `MenuWebView`, `search`; `BrowserModel`), and as seen against real WebKit:

- **The words** come from the adapter's script (022), which now says with each right-click what is selected in the page, up to 2,000 characters: a selected word of the test site's page was told as that word, and words typed in an ordinary field as those words.
- **What is typed in a password's field is not read.** The script says no words when the click was in such a field, or one has the keyboard, wherever in the page the field is: in the script's own world it sees into the parts of a page that are closed to the page's own scripts, and a password's field there was told as nothing. Without that rule, what WebKit gave the script for a password's field was its dots, one for each character, never the characters themselves; with it, dots are not offered for a search either.
- **The menu**: in place of WebKit's item that searches stands one of the adapter's own, under WebKit's name for it and the shell's words, "Search with DuckDuckGo", which reports `SearchRequested` with the words as they were selected. It is there only when words were selected and the shell has said where searches go. **Otherwise WebKit's item is taken out**: it is never left in the menu, since what it does is go elsewhere.
- **The item's words follow the choice**: the shell gives the adapter "Search with" and the engine's name when the core says where searches go, and again when another engine is chosen. Until the core has said, there are none, and no search is offered.

> **The owner's decision (2026-11-21): the iPhone's own search for selected words is left as it is for now.** The question put to them was whether taking out only that item, which may mean knowing it by a name Apple does not publish, is within the rule against private interfaces; the other courses were to take out the whole of the system's group of items it is in, or to leave it.

**The iPhone**: not built, and left so for now. What its menu of selected words offers was not established: with the app's test holding a finger on a word in the simulator, and tapping it twice, nothing of a menu of selected words was in what the app shows a test (no "Copy", no "Look Up", no search). Whether the simulator showed no such menu, or shows it where a test cannot see, is not known.

## 6. Tests

| Where | What |
|---|---|
| Core (through `FakeShell`) | Words searched for open a tab after the one that has them, in front, loading the chosen engine's search for them; the page that has the words is as it was. They go to the engine chosen, and to another after another is chosen. Words that look like an address are searched for all the same. No words, and a page that only makes it look as if the person asked, open nothing. Long words are cut. The new tab's field shows the search's address from the start. A web view that is no tab's opens nothing |
| Core, as built (`wt-app/tests/search_selection.rs`, 7; `wt-core`, 1) | All of the row above; and the words' tidying and cutting by itself. Six parts of the rule checked by breaking them |
| System WebView adapter, on the JVM (`SearchingContextTest`, 3; `LinksTest`, 1 more) | A request for a web search is heard, with its words, however it is asked, and starts nothing; one with no words is heard as none; every other request goes on as it was. The engine reports a search for a web view's words as the person's doing. Six of the rules checked by breaking them |
| System WebView adapter suite (`SystemWebViewSearchTest`, 1, on the emulator) | Against the real WebView, with a finger held on a word of the test site: the word is selected; the WebView's bar offers "Web search"; that item, chosen as a press chooses it, is reported with the word, as the person's doing; nothing asked the system to start anything (the test stops and counts whatever does); the page is where it was. Without the manifest's activity the item was not offered |
| WebKit adapter suite (`WebKitSearchTests`, 5 on the Mac, 1 on the iPhone simulator) | A search is reported with the words as they were selected, as the person's doing, and the page stays. On the Mac, against real WebKit with the mouse's own right-click: a selected word is told as that word; a password's field as nothing, in the page and inside a part of it that is closed to scripts, though there was something typed there; words typed in an ordinary field as those words. WebKit's item that searches is replaced by the shell's, that item and no other, and choosing it tells the adapter the words; it is taken out when the shell has no words for it, when no words or only white space were selected, and when nothing was said of the click. The menu that opens after a right-click on a selected word searches for that word. Nine of the rules checked by breaking them |
| The Apple shell (`SearchModelTests`, 2 more) | The menu has no words for a search until the core has said where searches go; then it names the engine in use, and the one chosen after. A model that is made and thrown away over the same session, as SwiftUI makes them, leaves the menu's words as they are: a window's view made again, a model alone, and a picture's model that is handed where searches go. Checked by breaking it: with the words following the model again, the test fails |
| The Mac app (`MenuTests`, 1) | With the real core and real WebKit, in fake mode: the menu of a word that is right-clicked has one item that searches, the shell's, named for the engine, and not WebKit's; put away with nothing chosen, no tab opens and nothing was asked of anybody. **No test chooses it**: a search goes to the internet. Where it fails, it says which of WebKit's own items the menu had, and nothing of the menu bar's. It failed the first time it ran (§5), and passes since |

## 7. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | No item of the menu sends the words anywhere but to the core. The system's item is replaced where the shell can put its own; on Android it is kept, and answered | Two items that search, one of which leaves the app for another engine, is a trap. On Android the WebView's item, heard by the adapter, is the one way that keeps the WebView's own judgement of when a search may be offered and of what the words are |
| D5 | On Android nothing is offered to other apps (in this feature: the owner has since decided it is, and it is [028](028-search-from-other-apps.md)) | An exported entry ("search with WebTiger" in every app's menu of selected text) is what the system's other way of adding to that bar needs. It is a thing a person may want, and a part of the app that other apps can start: the owner's to decide, and not needed for this |
| D2 | A new tab, in front | The person asked to see the search now, and has not finished with the page: Chrome does the same. Safari replaces the page |
| D3 | Always a search | The item says "Search". Words that happen to look like an address are still what the person wants to know about |
| D4 | The core cuts long words; the shell does not | One rule for every shell |

## 8. What is not verified

- **Android: any other WebView.** That the WebView asks the context it was made with, and no other, was seen on WebView 133. A WebView that asked the system some other way would have the words go to whatever the system starts for a web search, which could be another app, or a chooser with the app's own answerer in it, which does nothing.
- **Android: the search itself, in the app.** No test chooses the item with a core behind it: the tab would load the search engine. The adapter suite chooses it with no core; the core's own tests open the tab against a scripted engine.
- **Android: words selected in a frame, in a field, and a password's field.** The WebView's own judgement, not looked at.
- **Android: a phone.** It ran on an emulator.
- **The Mac: the item chosen, in the app.** No test chooses it: the tab would load the search engine. That choosing it tells the adapter the words is the adapter suite's, with a menu the test made; that the core then opens the search is the core's.
- **The Mac: that the script's word of what is selected always comes before the menu** (as 022 §8). Late, the search would be missing from that menu, not wrong.
- **The Mac: words selected in a frame.** The script is in every frame; no test has one.
- **The iPhone**: not built, by the owner's decision for now (§5), and what its menu offers not established. Words selected on an iPhone and searched for from the system's menu go to the system's choice of engine, in another app.

## 9. Done checklist

- [x] Contract 1.17 and its version in every place it is written.
- [x] The core: the rule of §3 (7 tests through `FakeShell`).
- [x] The Mac: the script's words; the menu's item; the adapter suite (5); the app's test, which found the menu's search missing in the app and passes since.
- [x] Android: the WebView's own item, heard by the adapter; the adapter suite on the emulator.
- [ ] iPhone.
- [x] `Tools/test.sh all` green, stage by stage, and `android-device`, on 2026-11-21: `lint`, `core` (498), `unit` (261 on the Mac), `build`, `ios` (243 on the iPhone simulator, one skipped as before), `ui` (28 on the Mac, 25 on the iPhone), `android` (341 and 32 on the JVM) and `android-device` (144 on the emulator). The Mac's UI suite ran that day for the first time since [021](021-certificate-errors.md): all of it passed but this feature's test, which found the fault of §5; with that put right, the whole of it passed.
- [ ] Windows shell.

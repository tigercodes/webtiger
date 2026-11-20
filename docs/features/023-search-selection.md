# 023 — Searching for words that are selected (W1)

| | |
|---|---|
| **Status** | Designed. Nothing is built yet. Today the item is the system's on every platform: it hands the words to the system's choice of search engine, in another app (§5, as found) |
| **Phase** | W1 of [000](000-master-plan.md) §9: "context menus". After a link's ([018](018-link-in-new-tab.md)) and a picture's ([022](022-picture-menu.md)): the menu of words that are selected |
| **Contract** | 1.17: `SearchRequested` |
| **Builds on** | [020](020-search-engine.md): the person's choice of where searches go. 018: a tab opened from another. 022: the Mac's script that says what a right-click was on |
| **Last updated** | 2026-11-20 |

## 1. User story

I am reading a page and want to know more about something it says. I select the words and choose, from the menu that selecting gives me, to search for them. A new tab opens in front with what the search engine I chose has to say, and the page I was reading is still there behind it.

## 2. What is in it, and what is not

**In:** the menu of selected words has one item that searches for them, named for the search engine the person chose; it tells the core, which opens the search in a new tab; the system's own item for a search, which goes elsewhere, is not shown.

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
| The system's item | "Search with Google" (or whichever engine the system is set to), in WebKit's menu | "Search Web", in the menu of selected text | "Web search", in the bar of selected text |
| What becomes of it | Replaced by the shell's | Taken out, and the shell's put in | Turned off, and the shell's put in |
| The shell's item | "Search with DuckDuckGo", naming the engine the person chose | The same | The same |
| The words | From the adapter's own script, as the right-click happens (022) | Asked of the page when the item is chosen | Asked of the page when the item is chosen |

**As found on the Mac, before this** (macOS 27, in the app in fake mode): WebKit's menu for selected words has "Copy", "Copy Link with Highlight", "Look Up …", "Search with Google", "Share…", "Speech" and "Translate …". "Search with Google" was not chosen, and what it does was not seen: a test that chose it would start another app and send its words to the internet. As WebKit's published source has it, the item hands the words to the system's search service, which opens them in the system's browser with the system's engine. The item's own name says as much: Google, where this app's searches went to DuckDuckGo.

**The item names the engine** (020 D4: what leaves the device, and to whom, is said where it is about to happen). The shell knows the name from the core's answer about where searches go, and gives the adapter the item's words again whenever the choice changes.

**Where the words cannot be had** (the Mac's script said nothing of the click), the item is not shown: an item that does nothing is not shown (022 D5).

What each does in fact, as observed, is written here when built.

## 6. Tests

| Where | What |
|---|---|
| Core (through `FakeShell`) | Words searched for open a tab after the one that has them, in front, loading the chosen engine's search for them; the page that has the words is as it was. They go to the engine chosen, and to another after another is chosen. Words that look like an address are searched for all the same. No words, and a page that only makes it look as if the person asked, open nothing. Long words are cut. The new tab's field shows the search's address from the start. A web view that is no tab's opens nothing |
| The adapters' own rules | What the shell's item is called; that the system's item is not in the menu; that choosing the shell's reports the words as the person's doing |
| Adapter suites, against the real engines | The words selected in a page are the ones reported |
| The apps | The menu of selected words has the shell's item, named for the engine, and not the system's. **No test chooses it**: a search goes to the internet |

## 7. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | The system's item is replaced, not kept beside the shell's | Two items that search, one of which leaves the app for another engine, is a trap |
| D2 | A new tab, in front | The person asked to see the search now, and has not finished with the page: Chrome does the same. Safari replaces the page |
| D3 | Always a search | The item says "Search". Words that happen to look like an address are still what the person wants to know about |
| D4 | The core cuts long words; the shell does not | One rule for every shell |

## 8. What is not verified

Written when built.

## 9. Done checklist

- [ ] Contract 1.17 and its version in every place it is written.
- [ ] The core: the rule of §3.
- [ ] The Mac.
- [ ] Android.
- [ ] iPhone.
- [ ] `Tools/test.sh all` green, and `android-device`.
- [ ] Windows shell.

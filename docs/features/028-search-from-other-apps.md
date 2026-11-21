# 028 — "Search with WebTiger" in other apps (Android, W1)

| | |
|---|---|
| **Status** | Built and tested in the core and the Android shell, and seen by hand on an emulator with its network off (no search is ever made by a test). Not run on a phone, nor from a real app's menu (§8) |
| **Phase** | W1 of [000](000-master-plan.md) §9: "context menus". After a search for words selected in a page ([023](023-search-selection.md)): the same search, for words selected in another app |
| **Contract** | 1.21: `SearchWords` |
| **Builds on** | [020](020-search-engine.md): the person's choice of where searches go. 023: words that are always searched for, and how they are tidied |
| **Last updated** | 2026-11-20 |

> **The owner's decision (2026-11-20): WebTiger offers "Search with WebTiger" in other apps' menus of selected text.** 023 left it open (023 D5): it makes a part of the app that other apps can start.

## 1. User story

I am reading a message, a note, a document, in some other app. I select a few words, and the menu that selecting gives me has "Search with WebTiger". I choose it: WebTiger comes forward with a new tab, and in it what the search engine I chose has to say about those words. My other tabs are as they were.

## 2. What is in it, and what is not

**In:** Android's menu of selected text, in every app that shows the system's, has "Search with WebTiger"; choosing it opens a new tab in front, in the window WebTiger has, with the chosen engine's search for the words (020); the words are always searched for, whatever they look like.

**Not in:**

- **Opening an address from another app** (a link pressed in a message opens in WebTiger). That is being the system's browser, or one of them: a larger promise, with the default-browser question (000 §6.3). Not here; since built as [031](031-links-from-other-apps.md).
- **The system's "web search" asked for by another app** (`ACTION_WEB_SEARCH`), which shows the system's chooser of apps. The same thing by another road; later if wanted.
- **Apple's systems.** The Mac's Services menu and an iPhone's share sheet can carry the same; neither is built.
- **Handing anything back.** The menu can let an app replace the selected words with others; WebTiger gives none back.

## 3. The rule

**The shell** hears that another app hands it words to search for, and hands them to the core with the window they are for (`SearchWords`). It decides nothing: not where the search goes, and not what the words are.

**The core** does with them what it does with words selected in a page (023 §3), and by the same code:

- the words are tidied: white space at the ends goes, and every run of it inside is one space; words longer than 500 characters are cut there;
- **they are always searched for**: never taken for an address, whatever they look like. Another app's words are not typed text: `example.com` is searched for, and so is `javascript:…`;
- no words, no search: the request is refused (`search.no_words`), and nothing opens. A window the core does not have is refused (`window.unknown`);
- otherwise **a new tab opens, after the tab in front, in front**, in the Space the window shows and its login set, and loads the chosen engine's search for the words. Its field shows the search's address from the start, as a search typed there does.

**Words from another app are another app's.** The entry is one any app on the device can start, with any words: that is what being in the menu means. So nothing is done with them but a search, by the engine the person chose; they are never run, never loaded as an address, never written in a log, and never kept but as the visit the search itself is (008).

**Nothing is sent until the item is chosen.** Selecting words in another app tells WebTiger nothing.

## 4. The contract (1.21)

Added; nothing is changed or removed.

```proto
// browser.proto
// A search for words the person chose somewhere a shell can hear of, outside
// any page: another app's selected text. Always a search (023): never an
// address. Answered with TabOpened; "window.unknown", "search.no_words".
message SearchWords {
  uint64 window_id = 1;
  string text = 2;                          // redacted
}
```

## 5. Android

- **The menu's item** is an activity of the app's that other apps can start (`SearchWordsActivity`, exported), which answers the system's "process this text" (`ACTION_PROCESS_TEXT`, plain text). Its name in the menu is its label: "Search with WebTiger".
- **It shows nothing.** It reads the words, starts the app's window with them (bringing the window the app has to the front, or starting it), and is gone. With no words it starts nothing.
- **The window** (`MainActivity`) is told of the words when it is started with them, or when they arrive while it is there; it hands them to the window's model, which asks the core once the window is open. Words that arrive with the start of the app wait for the window. When the system makes the window again from the list of recent apps it brings the request the window was first started with, and says so: the words are not searched for a second time (added with [031](031-links-from-other-apps.md), whose test it is).
- The window is not made a second time for it: the one there is comes forward.

**As built, and what the emulator showed.** Two things about how Android keeps its tasks had to be said in the manifest, and were found by trying:

- **The window is `singleTask`.** Started a second time with a different request (words, then its icon), Android puts a second window on top of the first unless the activity says there is only ever one. Now the one there is comes forward and is told (`onNewIntent`); its icon tells it nothing, and it stays as it was.
- **The item has no task of WebTiger's** (`taskAffinity=""`). Started as the first thing in WebTiger's task, the item became what that task was "for": the next request, being the same request with other words, was taken by the system for the first one over again, and nothing was started. WebTiger came forward and the words were never searched for. With a task of its own, which ends with it, each request is new.

The item is also kept out of the list of recent apps, and out of the history of the task it runs in. It carries at the most 4,000 characters (the core cuts what is searched for at 500): another app's whole document is not handed on.

## 6. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | The words are always a search | They are another app's. Taking them for an address would let any app send WebTiger anywhere, under an item that says "search" |
| D2 | The core's own request, not "open a tab with this text" | A tab opened with text takes the text as typed: an address if it looks like one. The rule that it is always a search is the core's to keep |
| D3 | An activity that shows nothing, in front of the window | The menu starts its item inside the other app's task. Answered by the window itself, that would be a second WebTiger window living in another app |

## 7. Tests

| Where | What |
|---|---|
| Core (`wt-app/tests/search_words.rs`, 6, through `FakeShell`) | Words open a search in a new tab, after the tab that was in front and in front of it, at the chosen engine, with the search's address in its field; the tab that was in front is as it was. **What looks like an address, or like something to run, is searched for**: a host, an `https:` and an `http:` address, `javascript:`, `about:`, `file:`, `data:` and `intent:` are each sent to the engine and nowhere else. The words are tidied, and a long run of them is cut at 500. No words (none, a space, only white space) is refused and opens nothing; so is a window there is not, which is said first. The search goes to the engine the person chose. It opens in the Space the window shows, with that Space's logins, and the other Space's tabs are as they were. Seven parts of the rule checked by breaking them: all caught |
| The real core from Kotlin (`ThroughTheCoreTest`, 1 more) | The window's model hands the words on; the real core, through JNI, opens a new tab in front and asks its web view for the engine's search; words that look like an address are searched for all the same. The web view is the test's: nothing is fetched |
| Android (`SearchWordsTest`, 7, on the JVM) | The words are read only from the system's own request, as they are, and only when there are some; a whole document is not carried. Chosen in another app, the item starts the window with the words, in the window's own task, with no address riding along, hands nothing back, and is gone; with no words, or started some other way, it starts nothing. The window hands the words to the core with its window, and asks nothing else: no tab of the shell's, nothing loaded. An intent that carries no words asks the core nothing. Words that come with the start of the app wait for the window: the last ones, once. **The system is told of the item**: it answers "process this text" for plain text, can be started by other apps, is called "Search with WebTiger", has no task of WebTiger's and is not in recent apps; the window is single, and answers nothing of another app's but its own launch and, since [031](031-links-from-other-apps.md), a link of the web (no text to send). Thirteen parts of this checked by breaking them: all caught (one only after a case was added: a window closed and opened again does not search a second time) |
| By hand (2026-11-20, the emulator, the app started for real, **the emulator's network switched off** so that nothing could be sent) | The system listed the item for selected plain text. Started as the system starts it, with WebTiger not running: WebTiger opened with a tab at the engine's search for the words (which failed, the device being offline, and said so). From the home screen with WebTiger behind: it came forward, the same one window, with a search for the new words; they were `example.com`, and were searched for, not gone to. With WebTiger in front, words `javascript:alert(1)`: searched for. The app's icon afterwards: the same window came forward, unchanged. The item was not in recent apps. The network was switched back on and the app's data cleared |

## 8. What is not verified

- **The item chosen from a real app's menu.** By hand it was started the way the system starts it (`am start`, with the system's action and the words), not by selecting text in another app and pressing it: so the item's place in a real menu, and its name there, were not seen. The system's own list of who answers was.
- **A search itself**, on a device: no test makes one, and the by-hand check was offline on purpose. That the address the web view is sent to is the engine's search is the core's test.
- **A phone**, and other makers' versions of the menu of selected text. Some apps draw a menu of their own and leave the system's items out.
- **Words chosen while a system picker of WebTiger's is open** (a file being picked for a page): the window coming forward ends the picker. Not tried.
- **Apple's systems**: not built (§2).

## 9. Done checklist

- [x] Contract 1.21 and its version in every place it is written.
- [x] The core: the rule of §3 (6 tests).
- [x] Android: the item, what it hands on, and what the window asks of the core (7 tests, and 1 through the real core).
- [x] On the emulator: the system lists the item for selected text; the whole road by hand, offline.
- [x] `Tools/test.sh all` green, stage by stage, and `android-device` (144 on the emulator), on 2026-11-21. The Mac's UI suite, which had not run when this was built (a dialog of the system's was on the Mac's screen), ran with it.

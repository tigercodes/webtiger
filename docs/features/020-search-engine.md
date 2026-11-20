# 020 — Where searches go (W1)

| | |
|---|---|
| **Status** | Built and tested in the store, the core, the Android shell and the Apple shell. DuckDuckGo is the default (the owner's decision, 2026-11-20): both shells show the choice once at a first run, with the default to keep, and say on an empty tab where searches go: Android on an emulator, Apple in the apps on the Mac and the iPhone simulator. Windows is not started |
| **Phase** | W1 of [000](000-master-plan.md) §9: "the first-run flow". Of the first run's four choices (000 §6.3) this is the one that W1 can make: importing and the style need W2's Spaces, and the default browser is the system's to set |
| **Contract** | 1.15: `QuerySearch`, `SearchSettings`, `ChooseSearchEngine`; `FakeOptions.first_run` |
| **Builds on** | [001](001-foundations.md): typed text is an address or a search. The store |
| **Last updated** | 2026-11-20 |

## 1. User story

The first time I open WebTiger it shows where my searches will go: DuckDuckGo, unless I pick another of a handful of search engines. I keep it, or pick one. From then on, what I type in the address field that is not an address is sent there, and nowhere else. A new tab says where searches go, and lets me change it.

## 2. What is in it, and what is not

**In:** the list of engines the core knows; the person's choice, kept in the store; the first run's question, in each shell; "Searches go to …" on an empty tab, with the way to change it.

**Not in:**

- **Suggestions from the engine** as one types: off ([011](011-suggestions.md)); they are a thing a person turns on, later.
- **An engine of the person's own** (an address they write): with Settings.
- **An engine for each Space**: W2.
- **The first run's other choices**: importing (W2, [002](002-arc-import.md)), a style (W2), the default browser.

## 3. The rule

**The core has a list of search engines**: for each, a name, and the address a search is sent to. It is short, the engines are ones anybody can use without an account, and it is in the order of their names: no engine has a place in it that it paid for, and none pays to be in it (000 WT-D17).

> **The owner settled it (2026-11-20): DuckDuckGo is the default, and Google and Bing are among the engines to choose.** The list is: Bing, Brave Search, DuckDuckGo, Ecosia, Google, Qwant, Startpage. Which engines are named, and which is the default, are a few lines in one file (`wt-core/src/search.rs`).

**DuckDuckGo is the default**: until the person chooses another, searches go there (it needs no account and no agreement, and it pays nothing to be the default), and the core says that nobody has chosen yet. A shell that is told so shows the choice once, before anything else, **with the default marked and a button that keeps it**: the first run's choice. Keeping the default is choosing it (`ChooseSearchEngine` with its id): it is written down, and the choice is not offered again. Closing the list any other way (a swipe, the system's Back) keeps it too.

This was first built as a question nobody could put off, with no default (D1 and D3 as they then stood). The owner's decision replaced that: a first run takes one press at the most, and nothing if the list is simply closed.

**The choice** (`ChooseSearchEngine`) names one of the list. It is written in the store, is the whole app's (every window, every tab), and holds until another is chosen. A name the list does not have is refused (`search.unknown_engine`), and nothing changes.

**What is searched for goes to the chosen engine and nowhere else**: the words, in the engine's address, as a load of the tab. Nothing is sent while a person types (011). What is typed that is an address is loaded as before: the choice changes nothing about it.

**Fake mode** has nobody to ask: its store is in memory and every run would be a first run. There the default is taken as chosen, unless the run asks for the first run's question (`FakeOptions.first_run`), as a test of that question does.

## 4. The contract (1.15)

Added; nothing is changed or removed.

```proto
// browser.proto
message QuerySearch {}                      // answered with SearchSettings

message SearchSettings {
  repeated SearchEngine engines = 1;        // the list, in its order
  string chosen_id = 2;                     // where searches go now
  bool chosen = 3;                          // the person chose it. False: nobody has been asked yet
}

message SearchEngine {
  string id = 1;                            // the core's name for it, which a choice names it by
  string name = 2;                          // its own name, which is not translated
  string host = 3;                          // where searches are sent, for a shell to show under the name
}

message ChooseSearchEngine {                // answered with Ack, or "search.unknown_engine"
  string id = 1;
}

// config.proto
message FakeOptions {
  ...
  bool first_run = N;                       // fake mode: nobody has chosen a search engine yet
}
```

## 5. The store

A table of the app's own settings, each a name and a value: `setting(key, value)`. Where searches go is the first (`search.engine`, the engine's id). A value the list no longer has is read as no choice.

## 6. The shells

- **The first run's question**: over the window, before it is used: "Where should your searches go?", a sentence that says what a search is and that nothing is sent until one is made, and the engines, each with its name and where it is. One press chooses. The default is marked "Default", and a button under the list keeps it by its name ("Keep DuckDuckGo"); closed any other way, the default is kept too.
- **An empty tab** says where searches go, under its hint, with a button that opens the same list, where the engine in use is marked.

**As built on Apple** (`SearchEnginesView` and the model's part in `SearchEngines.swift`; `EmptyTabView`): the same list, as a sheet over the window. At the first run the default says "Default" and the button under the list is "Keep DuckDuckGo"; swiping the sheet away keeps it too. Opened again from an empty tab's "Change…", the button is "Cancel", and the engine in use says "In use". (Kept, the list goes at once and does not come back while the core takes the choice down.) The words in the list are all in the colour of its heading, with each engine's name in a heavier weight: a quieter grey for the sentence and the addresses was too faint on an iPhone's sheet for Apple's audit. Started with `WEBTIGER_FAKE_FIRST_RUN=1` beside fake mode, the app's core says that nobody has chosen.

**As built on Android** (`SearchEngines`; `BrowserModel`; `EmptyTab`): the window asks the core where searches go as it opens. While the core says that nobody has chosen, the list is over the window: its heading, the sentence, and each engine with its name and where it is. A press chooses, and the list goes when the core has taken the choice; a choice the core refuses leaves it open. At the first run the default is marked "Default", and "Keep DuckDuckGo" or the system's Back keeps it. Opened again from an empty tab's "Change…", the engine in use is marked "In use", and "Cancel" or Back closes the list with nothing changed. A core that says nothing of search engines (an older one) is asked nothing more: no list, and no line on the empty tab.

Started with `--ez webtiger.fake.first_run true` beside fake mode, the app's core says that nobody has chosen.

## 7. Tests

| Where | What |
|---|---|
| Store (`wt-store`, 3) | A setting written is read back, and replaced, and is not another's; it is still there when the store is opened again; a store from before settings gains them and keeps what it had |
| The list (`wt-core`, 3) | It is in the order of the names and names each engine once; every engine makes an address over `https` with the words in it, written so that they stay words; the default is one of the list, DuckDuckGo, and Google and Bing are in it; an engine is found by its id as written |
| Core (`wt-app/tests/search_engine.rs`, 7 through `FakeShell`) | The engines are listed by name, with where each sends a search. Nobody has chosen at a first run, and fake mode takes the stand-in as chosen. Searches go to the stand-in until somebody chooses and then to the choice, in every tab, and another can be chosen after. A name the core does not have is refused and changes nothing. An address is loaded as before whatever was chosen. The choice is kept when the app starts again, and nobody is asked twice. A choice the list no longer has is no choice. Fourteen parts of the rule checked by breaking them |
| The Android shell (`SearchEnginesTest`, 7, drawn on the JVM against `ScriptedCore`) | At a first run the list is over the window, with each engine and where it is, the default marked "Default" and a button "Keep DuckDuckGo"; one press on another chooses it, and the empty tab then says where searches go. One press on the button keeps the default: the core is told, the list goes, the empty tab says DuckDuckGo, and opened to change it is "In use" with "Cancel" under it. A choice the core refuses leaves the question where it is; a change it refuses leaves the list open. When somebody has chosen, nothing is asked. "Change…" opens the list with the engine in use marked, and it can be kept or changed. Until the core has said, nothing is asked and nothing is said. Pictures of the question and of the list opened to change, light and dark, and the empty tab's again; each looked at. Eleven of the rules checked by breaking them |
| The Android app, on the emulator | With the real core, in fake mode: an empty tab says where searches go, and Android's accessibility checks pass over it and over the list; "Change…" opens the list with the engine in use marked; a press on another changes it, and every empty tab then says so. The tests share one core, which is left as it was found |
| By hand, on the emulator | The app installed afresh and started for real: the question was there before anything else, with the seven engines; a press on one closed it, and the empty tab said so; stopped and started again, the app did not ask, and said the same. Uninstalled afterwards |
| The Apple shell (`SearchModelTests`, 6, against `ScriptedCore`) | At a first run the list is shown and a choice answers it; closing it keeps the default, which the core is told, and it is gone at once and not shown again, while closing a list opened to change tells the core nothing; a choice the core refuses leaves the list as it is, at a first run and when changing; when somebody has chosen nothing is asked until they ask to change, and the list can be kept or changed; until the core has said, nothing is asked and nothing is said; the empty tab's sentence names the engine. Pictures of the question and of the list opened to change, light and dark, on the Mac and the iPhone, and the empty tab's again; each looked at. Seven of the rules checked by breaking them |
| The Apple apps | With the real core, started as for a first run, on the Mac and the iPhone simulator: the list is there before anything else, with the default marked and "Keep DuckDuckGo" under it, and Apple's audit passes over it; one press on another answers it; the empty tab then says where searches go, the audit passing again; "Change…" opens the list with that engine marked, the audit passing, and "Cancel" leaves things as they were. A second test keeps the default with one press: the empty tab says DuckDuckGo, and the list opened again has it in use. **Since the default was made one (2026-11-20) these ran on the iPhone simulator only: the Mac's UI suite has not run** (024 §9). And with the app started for real on a profile of the suite's own (the test of the tabs coming back, [009](009-session-restore.md)): a fresh profile is asked before anything else; ended and started again, the app does not ask, and a new tab says where searches go. No test searches: a search goes to the internet |

## 8. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | DuckDuckGo is the default; the person is shown the choice once, and keeps it or picks another | The owner's decision, 2026-11-20, in place of "nothing is chosen for them" (000 WT-D17 as first written). No engine pays for the place: that part of WT-D17 stands |
| D2 | The list is the core's, in the order of the names | One list for every shell, and an order nobody can buy |
| D3 | The first run's choice takes one press at the most, and closing it keeps the default | Under a minute (000 §6.3), with a default for each of the first run's choices, as 000 §6.3 has it. It first could not be skipped, when there was no default to fall back on |
| D4 | An empty tab says where searches go | What leaves the device, and to whom, is said where it is about to happen |

## 9. What is not verified

- **The engines' addresses.** Each is the address the engine itself gives for a search. No test opens one: a test that did would send words to the internet.
- **The first run's question in the Android app, by a test.** The app's tests on a device share one core, started in fake mode; the question is tested against a scripted core, and was seen by hand with the real one.
- **A search itself**, in an app: typing words and seeing them go to the chosen engine is the core's test (which checks the address the web view is sent to) and nobody's in a shell.
- **Android: a phone.** It ran on an emulator.
- **Changing the engine in an Apple app**: the app's test opens the list again and leaves it; choosing another there is the model's test.
- **The engines beyond the one chosen**, in an app: the tests choose Ecosia; the others are the same row.

## 10. Done checklist

- [x] The store's table, and its migration (7).
- [x] Contract 1.15 and its version in every place it is written.
- [x] The core: the rule of §3. 7 tests through `FakeShell`, 3 of the list's own.
- [x] Android: the question and the empty tab's line (6 tests); pictures; a device test of the app; the first run by hand on the emulator.
- [x] Apple: the question and the empty tab's line (5 tests of the model); pictures; a UI test on the Mac and the iPhone simulator.
- [x] `Tools/test.sh all` green, and `android-device` (131 on the emulator).
- [ ] Windows shell.

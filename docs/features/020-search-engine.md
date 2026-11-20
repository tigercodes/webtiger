# 020 — Where searches go (W1)

| | |
|---|---|
| **Status** | Designed. Nothing is built yet: searches go to one engine that nobody chose (001 F3, "a placeholder, not a product decision") |
| **Phase** | W1 of [000](000-master-plan.md) §9: "the first-run flow". Of the first run's four choices (000 §6.3) this is the one that W1 can make: importing and the style need W2's Spaces, and the default browser is the system's to set |
| **Contract** | 1.15: `QuerySearch`, `SearchSettings`, `ChooseSearchEngine`; `FakeOptions.first_run` |
| **Builds on** | [001](001-foundations.md): typed text is an address or a search. The store |
| **Last updated** | 2026-11-20 |

## 1. User story

The first time I open WebTiger it asks where my searches should go, and names a handful of search engines. I pick one. From then on, what I type in the address field that is not an address is sent there, and nowhere else. A new tab says where searches go, and lets me change it.

## 2. What is in it, and what is not

**In:** the list of engines the core knows; the person's choice, kept in the store; the first run's question, in each shell; "Searches go to …" on an empty tab, with the way to change it.

**Not in:**

- **Suggestions from the engine** as one types: off ([011](011-suggestions.md)); they are a thing a person turns on, later.
- **An engine of the person's own** (an address they write): with Settings.
- **An engine for each Space**: W2.
- **The first run's other choices**: importing (W2, [002](002-arc-import.md)), a style (W2), the default browser.

## 3. The rule

**The core has a list of search engines**: for each, a name, and the address a search is sent to. It is short, the engines are ones anybody can use without an account, and it is in the order of their names: no engine has a place in it that it paid for, and none pays to be in it (000 WT-D17).

> **The list is the owner's to settle.** It starts as: Bing, Brave Search, DuckDuckGo, Ecosia, Google, Qwant, Startpage. Which engines are named, and which stands in until a person has chosen, are a few lines in one file (`wt-core/src/search.rs`).

**Until the person has chosen**, searches go to the engine that stood in before there was a choice (DuckDuckGo: it needs no account and no agreement), and the core says that nobody has chosen yet. A shell that is told so asks, before anything else: the first run's question.

**The choice** (`ChooseSearchEngine`) names one of the list. It is written in the store, is the whole app's (every window, every tab), and holds until another is chosen. A name the list does not have is refused (`search.unknown_engine`), and nothing changes.

**What is searched for goes to the chosen engine and nowhere else**: the words, in the engine's address, as a load of the tab. Nothing is sent while a person types (011). What is typed that is an address is loaded as before: the choice changes nothing about it.

**Fake mode** has nobody to ask: its store is in memory and every run would be a first run. There the stand-in is taken as chosen, unless the run asks for the first run's question (`FakeOptions.first_run`), as a test of that question does.

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

- **The first run's question**: over the window, before it is used: "Where should your searches go?", a sentence that says what a search is and that nothing is sent until one is made, and the engines, each with its name and where it is. One press chooses. It cannot be put away unanswered: it is the one question, and takes one press.
- **An empty tab** says where searches go, under its hint, with a button that opens the same list, where the engine in use is marked.

## 7. Tests

| Where | What |
|---|---|
| Store | The migration; a setting written is read back, and replaced; one that was never written |
| Core (through `FakeShell`) | The rule of §3, each part of it, and each checked by breaking it: the list and its order; where searches go before a choice and after; that the choice is kept when the app starts again; a name the list does not have; fake mode and its first run |
| The shells | Against a scripted core: the question, what it lists, what a press sends; the empty tab's line. Pictures. In the apps, started as for a first run: the question is there before anything else, an engine is chosen, and the empty tab says so. No test searches: a search goes to the internet |

## 8. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | The person is asked; nothing is chosen for them | 000 WT-D17. The engine that stands in until then is not a default anybody was given |
| D2 | The list is the core's, in the order of the names | One list for every shell, and an order nobody can buy |
| D3 | The question takes one press and cannot be skipped | Under a minute (000 §6.3); and a choice put off is a choice made by the stand-in |
| D4 | An empty tab says where searches go | What leaves the device, and to whom, is said where it is about to happen |

## 9. What is not verified

- **The engines' addresses.** Each is the address the engine itself gives for a search. No test opens one: a test that did would send words to the internet.
- The rest: written when built.

## 10. Done checklist

- [ ] The store's table, and its migration.
- [ ] Contract 1.15 and its version in every place it is written.
- [ ] The core: the rule of §3, through `FakeShell`.
- [ ] Android: the question; the empty tab's line; pictures; a device test.
- [ ] Apple: the question; the empty tab's line; pictures; UI tests.
- [ ] `Tools/test.sh all` green, and `android-device`.
- [ ] Windows shell.

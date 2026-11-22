# 011 — Places offered as I type (W1)

| | |
|---|---|
| **Status** | Built and tested in the store, the core, the Apple shell and the Android shell (on an emulator). Windows is not started. What it leaves out is in §2 |
| **Phase** | W1 of [000](000-master-plan.md) §9: "the address field with fix-up and search". Local suggestions (000 §5.10: "local suggestions always") |
| **Contract** | 1.7: `SuggestAddresses`, `AddressSuggestions`, `AddressSuggestion` |
| **Builds on** | [008](008-history.md): history and its words |
| **Last updated** | 2026-11-19 |

## 1. User story

I start typing in the address field and WebTiger offers the places I have been that match: by a word of their title or their address, the likeliest first. I choose one and I am there. If none is what I want I go on typing and press Return, which does what it always did.

## 2. What is in it, and what is not

**In:** suggestions from this device's own history; a list of them under the address field in each shell.

**Not in, each with where it goes:**

- **Suggestions from a search provider.** Off by default (000 §5.10, WT-D17): what is typed would leave the device with every letter. They arrive with the first-run flow's choice of a provider, as something a person turns on.
- **Open tabs, Sites, commands** in the same list: the command bar (W2). (Open tabs are there since [038](038-an-open-tab-is-offered.md).)
- **Completing the address in the field itself** as it is typed (inline completion). The list comes first.
- **What Return will do**, shown as a row ("Search for …"): with the provider's choice, when there is a name to show.

Nothing typed leaves the device, and nothing typed is logged: a suggestion is an answer from the user's own store.

## 3. What is offered, and in what order

The core is asked with what is typed and answers with a few places, best first.

- **What matches:** a place in whose title or address every word typed is found, as a word or the start of one, whatever its case (the words of 008 §4). "rus boo" finds "The Rust Programming Language — doc.rust-lang.org/book/".
- **A place is offered once**, however often it was visited: by its address.
- **The order:**
  1. places whose site begins with what is typed ("exa" → `example.com`), before the rest: it is what a person typing an address means;
  2. then the more often visited;
  3. then the more lately visited.
- **How many:** six, unless the shell asks for another number; never more than twenty.
- **Only lately visited places are looked at:** the newest five hundred visits that match. That is what keeps an answer quick however much history there is (§7), and a place not been to in that long is seldom what is meant.
- Text with no word in it (nothing, or only punctuation) offers nothing.

## 4. The contract (1.7)

```proto
// Asks what to offer for what is typed in a window's address field. Answered with
// AddressSuggestions, or "window.unknown".
message SuggestAddresses {
  uint64 window_id = 1;
  string text = 2;          // redacted
  uint32 limit = 3;         // 0: six; more than twenty is twenty
}

message AddressSuggestions {
  repeated AddressSuggestion suggestions = 1;   // best first
}

message AddressSuggestion {
  string url = 1;           // redacted: where choosing it goes
  string title = 2;         // redacted; empty: the shell shows `display`
  string display = 3;       // redacted: the address as the address field shows it
}
```

Asked for, not subscribed to, like history: each letter typed is a question, and only the answer to what is now in the field is shown.

## 5. The shells

While the address field has the keyboard and something has been typed in it, the places offered are listed under the toolbar, over the page: each by its title and its address. Choosing one loads it in the tab in front, as if it had been typed. The list goes when the field loses the keyboard, when the text is sent, when another tab comes in front, and when nothing is offered.

On a Mac the arrow keys move through the list and Return goes to the one marked; with none marked, Return sends what was typed. Escape puts the list away.

**As built on Apple** (`SuggestionList`; the model's part in `BrowserModel`):

- Every change to what is typed asks the core, and only the answer to the last question is shown: an answer that arrives after the field has moved on, or after the list was put away, is dropped.
- The arrow keys go round: down from nothing marked to the first place, and from the last to nothing marked again, where Return sends what was typed. Typing again unmarks.
- **The padlock is the page's, and never what was typed's.** The first picture of the list had the page's padlock beside the letters "exa". The mark beside the field now says what the page is only while the field shows the page's address; once something else is typed there it is the plain magnifying glass, in both shells. It is the address field's rule (the field never lies) applied to its mark.
- **What is typed before the window's first snapshot stays.** The model took the first snapshot for another tab coming in front, and put the page's address over what had been typed. The first snapshot names the first tab, and is no switch.
- A place's address is cut to one line, in its middle; the audit is told so, as it is of a visit's address in the History list.

**As built on Android** (`SuggestionList` in `BrowserWindow`; the model's part in `BrowserModel`): the same list and the same rules, without the arrow keys. The system's Back puts the list away before it does anything to the page, and leaves what was typed. A model made as a picture of the list keeps what it was made with: a field that was never given the keyboard cannot lose it.

- **A long press on the field at rest was taken for typing**, and the list showed what the page's own address matched. The gesture that brings the keyboard also acts on the text the field showed until then: a tap places a caret there, a long press selects a word, a third tap all of it. The field knew to drop the caret (003) and took the rest for the person's own text: the short form of the address went into the model as typed, the field stopped following the page, and, once there was a list, the list appeared. Whatever the focusing gesture does to the at-rest text is now dropped, until the field is drawn again with the whole address. Found by the device test's own three taps in a row; a long press is how a person does it.

## 6. Tests

| Where | What |
|---|---|
| Store (`wt-store`, 8) | What matches; a place once, with the title it last had; a site that begins with what is typed comes first, with or without "www.", a scheme or a slash; then the more often visited, then the more lately; words typed with a space are not a site's name; how many; another login set's places are not offered; text with no word. The three rules of the order checked by breaking them |
| Core (`wt-app/tests/suggestions.rs`, 6 through `FakeShell`) | Places visited are offered by a word of their title and of their address, with the address as the field shows it; the order; six unless the shell says, never more than twenty; what was forgotten is not offered; an unknown window; nothing visited, nothing offered |
| The shells' models | Apple (`SuggestionModelTests`, 8): typing asks, each letter, and what the model itself puts in the field does not; the last answer is shown; the list goes when the field is emptied or loses the keyboard, and an answer that comes after is dropped; choosing goes there and puts the list away; the arrow keys mark, go round, and Return goes to the place marked or sends what was typed; typing again unmarks; another tab in front puts the list away; what was typed before the first snapshot stays; the padlock is the page's. Android (`SuggestionModelTest`, 8): the same but the keys, with an answer held back by the test and let go late, and a picture's model keeping what it was made with |
| The list drawn | Android (`SuggestionListTest`, 3, on the JVM): nothing is listed, and nothing asked, until something is typed; what is typed is asked about and the places are listed by title and address, with no padlock beside what was typed; choosing one sends its address and the list is gone |
| Pictures | The list under the field, with a title, a title too long for the list, and a place with none: light and dark, on the Mac, the iPhone and Android |
| The apps, with the real core and engine | A page is visited, the start of a word of its title is typed, it alone is offered, and choosing it goes there. iPhone simulator: with Apple's audit over the list. Android emulator: with Android's accessibility checks, the keyboard alone offering nothing, and the system's Back putting the list away; and a long press on the field at rest, which is not typing (it fails with the field as it was). The Mac: the same, and Escape puts the list away and leaves what was typed, typing on brings it back, and the arrow key and Return go to the place marked |

## 7. Measured

The plan's budget is 10 ms at p95 over 100,000 visits (000 §7). On this Mac (Apple M6), a release build, a store on disk with 100,000 visits of 5,000 places, each question asked 200 times:

| Typed | Median | p95 |
|---|---|---|
| One letter | 2.8 ms | 3.0 ms |
| Two letters | 1.1 ms | 1.1 ms |
| A word | 1.1 ms | 1.1 ms |
| The start of a site's name | 0.33 ms | 0.35 ms |
| Two words | 1.9 ms | 2.0 ms |
| Three words | 3.3 ms | 3.4 ms |

Within the budget, on a Mac. Not measured: a phone. What bounds the cost is looking at the newest five hundred matching visits and no more (§3).

## 8. What is not verified

- **The arrow keys beyond the first place.** The Mac app's test marks the first place and goes there; going down the list, round, and up again is the model's test, not the app's. (The app's test ran a few hours after the rest: dialogs of the system's were on this Mac's screen, put there by another test, [007](007-outside-links.md) §8, and the Mac's UI suite waited for a person to answer them.)
- **A phone.** The budget was measured on a Mac (§7); the Android list ran on an emulator.
- **Much history.** The order was tested with a handful of visits and measured with 100,000 made up ones; nobody has looked at what it offers for a real year of browsing.
- **Other scripts.** The words are history's (008): what is offered for text in a script without spaces between words was not tried.

## 9. Done checklist

- [x] The store: what to offer, and its order. 8 tests.
- [x] Contract 1.7 and its version in every place it is written.
- [x] The core: the request. 6 tests through `FakeShell`.
- [x] The measurement (§7).
- [x] Apple: the list, the keys, the model's tests (8), pictures, a UI test on the Mac and the iPhone simulator.
- [x] Android: the list, the model's tests (8) and the list's (3), pictures, a device test.
- [x] `Tools/test.sh all` green, and `android-device` (78 on the emulator).
- [ ] Windows shell.

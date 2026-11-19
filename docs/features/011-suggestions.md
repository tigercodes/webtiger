# 011 — Places offered as I type (W1)

| | |
|---|---|
| **Status** | In progress. The store, the contract (1.7) and the core are built, tested and measured; no shell lists what is offered yet (§9) |
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
- **Open tabs, Sites, commands** in the same list: the command bar (W2).
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

## 6. Tests

| Where | What |
|---|---|
| Store (`wt-store`, 8) | What matches; a place once, with the title it last had; a site that begins with what is typed comes first, with or without "www.", a scheme or a slash; then the more often visited, then the more lately; words typed with a space are not a site's name; how many; another login set's places are not offered; text with no word. The three rules of the order checked by breaking them |
| Core (`wt-app/tests/suggestions.rs`, 6 through `FakeShell`) | Places visited are offered by a word of their title and of their address, with the address as the field shows it; the order; six unless the shell says, never more than twenty; what was forgotten is not offered; an unknown window; nothing visited, nothing offered |
| The shells' models | Typing asks; only the last answer is shown; choosing loads and ends the list; when the list goes |
| Pictures, and the apps | The list under the field. With the real core: visit, type part of a title, choose the place offered |

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

- Written when built.

## 9. Done checklist

- [x] The store: what to offer, and its order. 8 tests.
- [x] Contract 1.7 and its version in every place it is written.
- [x] The core: the request. 6 tests through `FakeShell`.
- [x] The measurement (§7).
- [ ] Apple: the list, the keys, the model's tests, pictures, a UI test.
- [ ] Android: the list, the model's tests, pictures, a device test.
- [ ] `Tools/test.sh all` green, and `android-device`.

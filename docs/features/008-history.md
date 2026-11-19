# 008 — History (W1)

| | |
|---|---|
| **Status** | In progress. The store, the contract (1.5) and the core are built and tested: visits are written, asked for and forgotten. No shell has a History screen yet (§9) |
| **Phase** | W1 of [000](000-master-plan.md) §9: "history". The store's first table that is written while browsing (000 §5.6: `visit`) |
| **Contract** | 1.5: `QueryHistory`, `HistoryPage`, `HistoryEntry`, `ForgetHistory` |
| **Last updated** | 2026-11-19 |

## 1. User story

I saw a page last week and want it back. I open History, type a word from its title or its address, and it is there, newest first. I can make WebTiger forget one visit, the last hour, or everything, and what is forgotten is gone from the disk, not only from the list.

## 2. What is in it, and what is not

**In, in this order:** the store keeps visits (this document's first part: the store and the core); a shell asks for a page of them, with or without words to look for; a shell asks for visits to be forgotten; a History screen in each shell.

**Not in, each with where it goes:**

- **Suggestions in the address field** from history, and their ranking: `wt-omnibox`, later in W1. They read the same table.
- **"Forget this site"**, which removes a site's history and its stored data together (000 §5.10): it needs the `LoginSets` host service's clearing, which the contract does not have yet.
- **Private windows** (W2). A private login set is never in the store, so nothing of it can be written: the rule is in the schema already (§3).
- **Sync of history** (later, opt-in; 000 §5.11).
- **A crate of its own.** The plan names `wt-history` for "visits, ranking, full-text search" (000 §5.2). Until there is ranking, the queries live in `wt-store` and the rule for what is a visit in `wt-app`; a crate with two functions in it is not worth having (as 001 F2).

## 3. What a visit is

A visit is written when a page **commits** in a tab: its address is then the page's, by the rule the address field already keeps (001 §4.2). So:

| | |
|---|---|
| A page that loads | One visit, at the address that committed: where a redirect landed, not where it began |
| A load that fails, or is refused, or is stopped before it commits | None: nothing was seen |
| A page that changes its own address (a fragment, the History API) | A visit at the new address: it is how one-page sites move |
| The same address again in the same tab, straight after (a reload) | None: it is the visit that is there already |
| Going back or forward | A visit, like any other commit: the page was seen again |
| An address that is not `http` or `https` (`about:blank`) | None |
| A private login set | None, ever: a visit names its login set, and the store has no row for a private one to name |

- **The title is the page's, when it has one.** A visit is written with no title, at the commit. The first title the page then says is written, and once more the title it has when the tab goes on to another page, if that is a different one. A page that changes its title ten times a second costs the disk two writes, not ten a second. A tab that is closed leaves its last visit with the title already written.
- **Forgetting is for good.** A page still on screen when its visit is forgotten does not write to it again; seen again afterwards (a reload), it is a visit again.
- **A store that refuses a visit loses that visit and nothing else.** There is nobody to tell while a page loads, and no log an address may go in.
- **History is per login set** (000 §5.6): two Spaces with their own logins have their own histories. A window is asked about its Space's.
- **Nothing a person visited is logged.** Addresses and titles are in the store, which is the user's own file, and in the contract's redacted fields. No error carries one.

## 4. The store

Migration 2 adds `visit` and a full-text index over it (SQLite's FTS5, which the bundled SQLite has and a test checks):

```sql
CREATE TABLE visit (
    id INTEGER PRIMARY KEY,
    login_set_id BLOB NOT NULL REFERENCES login_set (id),
    url TEXT NOT NULL,
    title TEXT NOT NULL,
    visited_at INTEGER NOT NULL          -- milliseconds since 1970, UTC
) STRICT;
```

- **Newest first is by `id`.** Rows are written in the order pages were seen, and never moved; the clock is only what a person is shown. A clock that is set back cannot reorder history.
- **Looking for words:** each word typed must be found in the title or the address, as a word or the start of one ("rus" finds "Rust"). What is typed is never handed to SQLite as a query: it is cut into words, and each is quoted.
- **Forgetting really forgets.** Deleted rows are overwritten, not only unlinked (`secure_delete`, and the index's own), and the write-ahead log is emptied afterwards. A test reads the files and does not find what was forgotten.

## 5. The contract (1.5)

```proto
// A page of a window's history: the visits of its Space's login set, newest first.
message QueryHistory {
  uint64 window_id = 1;
  string text = 2;               // words to look for; empty: everything (redacted)
  uint32 limit = 3;              // 0: the core's own page (100); never more than 500
  uint64 before_visit_id = 4;    // only visits older than this one; 0: from the newest
}

message HistoryPage {
  repeated HistoryEntry entries = 1;
  bool more = 2;                 // older visits than these match too
}

message HistoryEntry {
  uint64 visit_id = 1;
  string url = 2;                // redacted
  string title = 3;              // redacted; empty: the shell shows `display`
  string display = 4;            // the address as the address field shows it (redacted)
  int64 visited_at_ms = 5;
}

// Forgets visits of a window's Space's login set.
message ForgetHistory {
  uint64 window_id = 1;
  oneof what {
    uint64 visit_id = 2;         // this one
    int64 since_ms = 3;          // every visit at or after this time
    bool everything = 4;
  }
}
```

`QueryHistory` is answered with `HistoryPage`; `ForgetHistory` with `Ack`. Both with `window.unknown` for a window there is not.

History is asked for, not subscribed to: a list that is being read does not move under the reader. A shell asks again when its screen opens.

## 6. The shells

Written when their part is built.

## 7. Tests

| Where | What |
|---|---|
| Store (`wt-store`, 14) | The migration from version 1, with a Space already there. Visits come back newest first, a page at a time, and say when there are more. Words: all must be found, in titles and addresses, as a word's start, whatever their case; what looks like a query's own syntax is only words. Another login set's visits are not seen. A title that arrives later is found by its words, and the title before it no longer is. A login set the store does not have can have no visit. An id is never used twice. Forgetting one, since a time, and everything; and the files no longer hold what was forgotten: seen to fail with the log not emptied, with rows only unlinked, and with the index left to its own ways, each alone |
| Core (`wt-app/tests/history.rs`, 19 through `FakeShell`) | Each row of §3's table. A visit takes its page's first title and the one it is left with, and none between. The contract's requests: a page, the next page, a hundred when the shell does not say; words; forgetting one, since a time, everything, and that it must say what; a visit forgotten stays forgotten, and a page seen again is a visit again; an unknown window. Six of the rules checked by breaking them |
| A measurement, not a gate | Below |

**Measured** on this Mac (Apple M6), a release build, a store on disk with 100,000 visits of one login set, each query asked 200 times for its newest hundred matches:

| | Median | p95 |
|---|---|---|
| The newest hundred, no words | 0.02 ms | 0.02 ms |
| A common word (one visit in sixteen has it) | 0.25 ms | 0.27 ms |
| The start of a word | 0.25 ms | 0.26 ms |
| Three words together | 0.96 ms | 0.99 ms |
| A word only one visit has | 0.01 ms | 0.01 ms |

Writing the 100,000 visits one at a time, each its own transaction, took 3.8 s: 38 µs a visit. The plan's budget is for the address field's suggestions over this table, 10 ms at p95 (000 §7); these are not those queries, which rank as well as find, but they are the same index. Not measured: a phone, and a store that has been written and forgotten from for months.

## 8. What is not verified

- **A phone's disk.** The times of §7 are a Mac's.
- **A store used for months.** Whether forgetting many visits over a long time leaves the files larger than they need be was not looked at; what is forgotten is overwritten either way.
- **The History screens**: not built.

## 9. Done checklist

- [x] The store: migration 2, visits, words, forgetting. 14 tests.
- [x] Contract 1.5 and its version in every place it is written.
- [x] The core: what a visit is; the requests.
- [x] Core tests through `FakeShell`: 19.
- [ ] Apple: the History screen.
- [ ] Android: the History screen.
- [ ] `Tools/test.sh all` green, and `android-device`.

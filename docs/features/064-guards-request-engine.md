# 064 — Guard's request engine (W4)

| | |
|---|---|
| **Status** | **Designed, on the recommendations of §6.** The owner's word of 2026-11-24 was to use what is suggested where something waits on a choice, and to go on: they are to be built on, and stay the owner's to change. Not built. What is known of the blocker here is what its documents say (§3): nothing of it has been built or run on this machine yet |
| **Phase** | W4 of [000](000-master-plan.md) §9, its first: "The request engine and fast path". 000 §5.8: "One blocker, acting where each engine allows. The core holds Brave's `adblock-rust`" |
| **Contract** | To come with step 3: fake mode given rules by a suite. The fast path's `wt_guard_check` has been in the C header since [001](001-foundations.md), answering "allow" |
| **Builds on** | [001](001-foundations.md): the fast path, and each shell's request callback wired to it from the first build (Android's asks already). 000 WT-D10: the blocker and its lists, decided. [060](060-the-budgets-as-tests.md): how a budget is held by a test |
| **Last updated** | 2026-11-25 |

## 1. User story

A page I open asks for a hundred things from thirty places, and half of them are there to follow me or to sell to me. I want the browser to say no to those before they are fetched, without my having to do anything, and without the page breaking.

## 2. What is in it, and what is not

**In:** the core holds the blocker. Given rules as text, in the syntax the public lists are written in, it answers one question, for one request: may this page fetch this? The fast path asks it, from whatever thread an engine calls on. A corpus of requests with the answers expected, held as a test.

**Not in, each with where it goes:**

- **The lists** (EasyList and the rest, 000 §5.8), fetching them, and the daily bundle: the next of W4. Until there are lists the blocker has no rules, and allows everything, as now.
- **WebKit.** It has no callback for a request: its rule lists are compiled from the same rules (the WebKit compiler, later in W4). Nothing here blocks anything on Apple's systems.
- **Page fixes** (hiding what is left, scripts), **redirects** to stand-ins, and **addresses rewritten** by a rule: later in W4. A rule that asks for one is, here, a rule that does not block.
- **The switches** (this site, this Space, everywhere), **the Guard mark, its count and its panel:** later in W4. Here Guard is on for every page.
- **A page of the top frame that is itself blocked:** it needs a page that says so. Here a request for the top frame's own document is always allowed (D5).

## 3. The rule

**A decision.** A request is its address, the address of the page that asks (the top frame's), and what it is for (a script, a picture, a frame, and so on: the kinds the C header has had since 001). The answer is allow or block. Blocked: the request is not made, and the page goes on without it.

**It fails open.** Whatever goes wrong in deciding (an address that is none, a core that is stopping, a fault in the blocker), the answer is allow: a blocker that fails must not break the page. As the header has said since 001.

**The first party and the third.** Whether a request goes to the page's own site or another's is decided by the core's own registrable domains (`wt-core`, on the public suffix list it has), as everything else in the app decides it: there is one list of public suffixes in the app, and not the blocker's beside it (D2).

**Rules change by a new blocker.** The blocker is made from rules and is not changed afterwards (its documents say so). When rules change, a new one is made, off the fast path, and put in the old one's place at a stroke; a decision that is being made is made on the one it began with.

**From any thread.** An engine asks on its own threads, many at once, and waits for the answer. One blocker is shared between them (D3).

**What the blocker's documents say, and what follows** (`adblock` 0.13.6, MPL-2.0, read on 2026-11-25; nothing of it tried here yet):

- Rules go in as a list's text; a decision is `check_network_request`, which answers whether a blocking rule matched, whether an exception did, and whether the rule is one no exception undoes.
- **As it is built by default it may not be shared between threads, or even moved to another** (its documents list it as neither). It has a switch of its build named for one thread, on by default. Whether it may be shared with that switch off, and what a decision then costs, is the first thing to try (§5, step 1). If it may not be, D3 has the other ways.
- It can be saved as bytes and read back, as a cache: the format is not kept from one of its versions to the next. So its version is pinned exactly (D1), and a saved blocker is thrown away when the version changes.
- A request can be handed over with its hosts already worked out, which is what lets the core use its own registrable domains.

**The header's promise, restated.** The header says of the fast path's two calls that they "never queue, lock or allocate". That is true of the navigation check and of the answer Guard gives today, which reads nothing. It cannot be true of a blocker that parses an address and looks rules up: a decision allocates, and the blocker may lock within itself. What stays true, and is what the promise was for: a decision never queues behind the core's own thread, never waits for the store, and never waits for a shell. The header is to say that, and the time is to be held by the plan's budget (000 §7: "A Guard decision: ≤ 25 µs at p95 on the phone").

## 4. The contract

Nothing a shell does changes at steps 1 and 2. At step 3:

- `wt_guard_check` answers from the blocker. Its declaration does not change; its comment does (§3).
- Fake mode can be given rules by a suite (`FakeOptions`, the next minor): so that a test of an app can see a request refused, with rules of the test's own, before there are lists.

## 5. In what order

1. **Try it,** outside the repository: the blocker built with the switches wanted; whether it may be shared between threads; what a decision costs in time and what it allocates; what a blocker of 100,000 rules holds; what it adds to the core's size. Before anything is designed further on what its documents only half say.
2. **The core:** a crate of Guard's own, with the blocker inside it and nothing of the blocker in what it offers the rest of the core: rules in, a decision out. The corpus.
3. **The fast path:** `wt_guard_check` asks it; fake mode's rules; Android's adapter, which asks already, tested on the emulator with a page of the test site that fetches something.
4. **The budgets:** a decision's time as a measurement that fails at the budget (`Tools/test.sh budgets`); what the blocker holds as a counted test, within the core's share.

## 6. Decisions

**All of these are recommendations** (§ Status). The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | The blocker is `adblock` (Brave's `adblock-rust`), at one exact version | 000 WT-D10 and §5.8 chose it; W0 converted two lists with this version (001 §11.1). Exact, because its saved form and its WebKit conversion are a version's | A range of versions: a saved blocker that a newer build cannot read |
| D2 | The core's own registrable domains, handed to the blocker with each request; the blocker's own resolver left out of the build | One list of public suffixes in the app, one answer to "is this the same site" everywhere, and one dependency fewer | The blocker's resolver: what it is tested with by its makers, and a second list of suffixes that can disagree with the first |
| D3 | One blocker, shared by every thread that asks, swapped at a stroke when rules change | A blocker of the default lists is some 15 MB (000 §5.8): it cannot be one a thread. The core swaps such things already | If it may not be shared (§3): one blocker behind a lock, which queues every engine thread behind the slowest decision; or the decisions sent to one thread of Guard's own, which every request then waits on |
| D4 | It fails open | 001: a blocker that fails must not break the page | Fail closed: safer against a tracker, and a blank page for a fault of ours |
| D5 | A request for the top frame's own document is always allowed, at this step | Blocking a whole page needs a page that says so and a way past it, which are not built. The lists' rules of that kind are few | Blocked, with the engine's own error page |
| D6 | The corpus's rules are written here, in the lists' syntax, for hosts of the test's own. No public list is checked in until the owner has said which, and under what terms | The public lists are other people's work under their own licences (the GNU GPL, Creative Commons), and this repository is MPL-2.0: what of them may sit in it is a decision, and the owner's. A corpus of our own rules says the blocker and the fast path work; it does not say what the public lists block | The lists checked in, pinned by hash (000 §8 foresaw it) |
| D7 | Fake mode can be given rules by a suite | An app's test can then see a request refused without any list, on Android now and wherever a request callback comes later | No test of an app until the lists are there |
| D8 | The header says what the fast path keeps to (§3), and no longer "never … allocate" | A promise that the code cannot keep is worse than none | The words left, and untrue |

## 7. Tests

To be filled in as built. Planned:

| Where | What |
|---|---|
| Guard's crate | Rules in, decisions out: a blocking rule, an exception, a rule for the third party alone, a rule for one kind of request, a rule no exception undoes; what is no rule is ignored; no rules, everything allowed; the top frame's document always allowed |
| The corpus | A file of requests with the answers expected, against rules of this repository's own: the same file for every way of deciding there will be (this one now; WebKit's rule lists later) |
| The fast path | From many threads at once, while the rules are changed under them: every answer is one the old rules or the new would give; a bad address is allowed |
| Android, on the emulator | A page of the test site that fetches something, with a suite's rule against it: the page loads, and the thing was not asked of the site |
| The budgets | A decision's time; what the blocker holds |

## 8. What is not verified

To be filled in as built.

- **The choices of §6** are the recommendations; the owner has not looked at them.
- **Everything §3 says of the blocker** is from its documents. Step 1 is to try it.

## 9. Done checklist

- [ ] The owner's word on §6.
- [ ] The try (step 1).
- [ ] Guard's crate, and the corpus.
- [ ] The fast path; fake mode's rules; Android.
- [ ] The budgets.
- [ ] The gate.

# 064 — Guard's request engine (W4)

| | |
|---|---|
| **Status** | **Built and tested, in the core and through Android's request callback, on the recommendations of §6.** The owner's word of 2026-11-24 was to use what is suggested where something waits on a choice, and to go on: they are built on, and stay the owner's to change. The core holds the blocker; given rules it answers for a request whether a page may fetch it; the fast path asks it from any thread; a corpus of this repository's own holds the answers. A decision is a microsecond here, where the phone's budget is 25. **There are no lists yet, so outside a suite Guard has no rules and allows everything, as before:** the lists are the next of W4, and which public lists may be checked in is the owner's (D6). Nothing blocks on Apple's systems, which wait for WebKit's rule lists |
| **Phase** | W4 of [000](000-master-plan.md) §9, its first: "The request engine and fast path". 000 §5.8: "One blocker, acting where each engine allows. The core holds Brave's `adblock-rust`" |
| **Contract** | 1.53: `FakeOptions.guard_rules`, a suite's rules for fake mode's blocker. `wt_guard_check` has been in the C header since [001](001-foundations.md), answering "allow": its declaration is as it was, and its answer is now Guard's |
| **Builds on** | [001](001-foundations.md): the fast path, and each shell's request callback wired to it from the first build (Android's asks already). 000 WT-D10: the blocker and its lists, decided. [060](060-the-budgets-as-tests.md): how a budget is held by a test |
| **Last updated** | 2026-11-25 |

## 1. User story

A page I open asks for a hundred things from thirty places, and half of them are there to follow me or to sell to me. I want the browser to say no to those before they are fetched, without my having to do anything, and without the page breaking.

## 2. What is in it, and what is not

**In:** the core holds the blocker. Given rules as text, in the syntax the public lists are written in, it answers one question, for one request: may this page fetch this? The fast path asks it, from whatever thread an engine calls on. A corpus of requests with the answers expected, held as a test.

**Not in, each with where it goes:**

- **The lists** (EasyList and the rest, 000 §5.8), fetching them, and the daily bundle: the next of W4. Until there are lists the blocker has no rules, and allows everything, as now.
- **WebKit.** It has no callback for a request: its rule lists are compiled from the same rules (the WebKit compiler, later in W4). Nothing here blocks anything on Apple's systems.
- **Page fixes** (hiding what is left, scripts), **stand-ins** for what is blocked, and **addresses rewritten** by a rule: later in W4. As built: rules that hide or mend a page's own content are not taken in at all; a rule that asks for a stand-in is a rule that blocks, with no stand-in; a rule that takes a part out of an address refuses nothing, and takes nothing out.
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

- `wt_guard_check` answers from the core's blocker. Its declaration does not change; its comment does (§3), and so does the header's account of the fast path.
- **1.53:** `FakeOptions.guard_rules`: rules, a rule to a line, of which fake mode's blocker is made as the core starts. So that a test of an app can see a request refused, with rules of the test's own, before there are lists. Outside fake mode it is nothing. Android hands it on from `--es webtiger.fake.guard`; Apple's shell has no such knob yet, since nothing there asks the fast path.

**As built, where the design did not say:**

- **The blocker is made as the core starts and is not changed after:** there is nothing yet that changes rules. Putting a new one in its place at a stroke (§3) comes with the lists.
- **With no rules nothing is parsed or looked up:** the answer is "allow" at once, which is what every app has until there are lists.
- **The method of a request is not known to the fast path:** a rule for one method alone is read as a rule for a GET.

## 5. In what order

1. **Try it,** outside the repository: the blocker built with the switches wanted; whether it may be shared between threads; what a decision costs in time and what it allocates; what a blocker of 100,000 rules holds; what it adds to the core's size. Before anything is designed further on what its documents only half say. **Done, 2026-11-25** (§5.1), but for the size, which is measured when it is in the core.
2. **The core:** a crate of Guard's own (`wt-guard`), with the blocker inside it and nothing of the blocker in what it offers the rest of the core: rules in, a decision out. The corpus. **Built.**
3. **The fast path:** `wt_guard_check` asks it; fake mode's rules; Android's adapter, which asks already. **Built,** and seen by hand in the app on the emulator (§7).
4. **The budgets:** a decision's time as a measurement that fails at the budget (`Tools/test.sh budgets`); what the blocker holds as a counted test. **Built** (§5.2).

### 5.1 The try

A program of its own, outside the repository, with `adblock` 0.13.6 built without its switch for one thread and without its own resolver of domains. On this Mac (Apple M6, twelve cores), a release build. **The rules were made up for the try:** 100,000 of five ordinary shapes (a host, a path with a kind and "third party", a host with a kind, an exception, a piece of an address), and 2,000 requests of which three in ten match one. They are not the public lists, and say nothing of what those cost.

| What | Found |
|---|---|
| May it be shared between threads | **Yes,** built so: the blocker's own source asserts it. (Built by default, for one thread, it may not, as its documents say) |
| Requests handed over with their hosts worked out, and no resolver in the build | Works: every request of the try was |
| A blocker of 100,000 rules | Made in 40 ms. Holds 4.6 MB. Saved, 4.6 MB |
| What a decision leaves held | Nothing: after a hundred more, not a byte more |
| A decision, one thread | The middle 0.4 µs; p95 0.5 to 0.7 µs; p99 0.6 to 0.7 µs; the slowest of 40,000, 7 to 16 µs |
| **How it is shared** | **One at a time.** Built for many threads, the blocker keeps what it has compiled of its rules behind a lock, and holds the lock for the whole of a decision. Threads that ask together wait for each other |
| A decision, eight threads asking without a pause | The middle 3.3 µs; **p95 44 µs**; p99 74 µs |
| The same, each thread doing 5 µs of other work between its requests | The middle 2.0 µs; p95 29 µs; p99 61 µs |
| The same, 50 µs between | The middle 1.0 µs; p95 5.6 µs; p99 11 µs |
| The same, 500 µs between | The middle 1.0 µs; p95 3.7 µs; p99 8 µs |

**What it says for D3.** One blocker shared is right as long as the threads that ask are not asking without a pause: at 50 µs between a thread's requests, which is twenty thousand requests a second from each of eight threads and far more than a page's loading, p95 is 6 µs against the budget's 25. Eight threads that do nothing but ask are over it, on this Mac. A phone has slower cores and fewer: the budget is the phone's, and is not measured here. If a phone shows it over, the other way is a few blockers, each with its own lock, at the cost of the rules' memory once for each (the rules themselves cannot be shared between two blockers through anything the blocker offers).

### 5.2 In the core, measured

Through Guard's own interface, which parses the request's address and the page's with the core's own parser before the blocker is asked: so a decision here is the whole of what the fast path does. The same made-up rules as the try (§5.1), and the same caution: they are not the public lists.

| What | Measured | Held to |
|---|---|---|
| A decision, one thread (`Tools/test.sh budgets`, a release build, this Mac) | The middle 0.9 µs; p95 1.0 to 1.1 µs; p99 1.1 to 1.2 µs | 25 µs at p95 (000 §7, which is the phone's) |
| The same, four threads at once with 50 µs between a thread's requests | The middle 1.8 µs; p95 2.9 to 3.0 µs; p99 7.0 to 7.4 µs | 25 µs at p95 |
| A blocker of 100,000 rules, counted (a test that runs every time) | 4,634,976 bytes | 6 MB, a ceiling of this test's own |
| 100,000 rules more that hide a page's content | Nothing more: 4,632,320 bytes | No more than a twentieth over |
| Two hundred decisions | 64 bytes the first time they are made; nothing when asked again | Nothing when asked again |
| The blocker gone | 2,664 bytes left | 64 KB |
| The released Mac app, as it is built here (two architectures, not stripped) | 32.0 MB before; 35.0 MB with the blocker: 3.0 MB more, about 1.5 MB an architecture | Not held |

## 6. Decisions

**All of these are recommendations** (§ Status). The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | The blocker is `adblock` (Brave's `adblock-rust`), at one exact version | 000 WT-D10 and §5.8 chose it; W0 converted two lists with this version (001 §11.1). Exact, because its saved form and its WebKit conversion are a version's | A range of versions: a saved blocker that a newer build cannot read |
| D2 | The core's own registrable domains, handed to the blocker with each request; the blocker's own resolver left out of the build | One list of public suffixes in the app, one answer to "is this the same site" everywhere, and one dependency fewer | The blocker's resolver: what it is tested with by its makers, and a second list of suffixes that can disagree with the first |
| D3 | One blocker, shared by every thread that asks, swapped at a stroke when rules change | A blocker of the default lists is some 15 MB (000 §5.8): it cannot be one a thread. The core swaps such things already. *As tried (§5.1):* it may be shared, and its decisions are then made one at a time behind its own lock, which at any rate of asking a page has is well within the budget here | A few blockers, each with its own lock, a thread taking one by its number: less waiting, and the rules' memory several times. For a phone that shows the one over its budget |
| D4 | It fails open | 001: a blocker that fails must not break the page | Fail closed: safer against a tracker, and a blank page for a fault of ours |
| D5 | A request for the top frame's own document is always allowed, at this step | Blocking a whole page needs a page that says so and a way past it, which are not built. The lists' rules of that kind are few | Blocked, with the engine's own error page |
| D6 | The corpus's rules are written here, in the lists' syntax, for hosts of the test's own. No public list is checked in until the owner has said which, and under what terms | The public lists are other people's work under their own licences (the GNU GPL, Creative Commons), and this repository is MPL-2.0: what of them may sit in it is a decision, and the owner's. A corpus of our own rules says the blocker and the fast path work; it does not say what the public lists block | The lists checked in, pinned by hash (000 §8 foresaw it) |
| D7 | Fake mode can be given rules by a suite | An app's test can then see a request refused without any list, on Android now and wherever a request callback comes later. *As built:* Android's shell hands them on; no test of the app in the gate uses them (§8) | No test of an app until the lists are there |
| D8 | The header says what the fast path keeps to (§3), and no longer "never … allocate" | A promise that the code cannot keep is worse than none | The words left, and untrue |

## 7. Tests

| Where | What |
|---|---|
| The corpus (`Fixtures/guard/rules.txt`, `Fixtures/guard/corpus.txt`; `wt-guard`, `corpus`, 3) | Thirty-three requests, each with the answer expected, against seventeen rules written here: a host and what is under it, whatever it is for, and not the host above it or one that only begins like it; a rule for scripts of another site alone, and the same site by another host; a path for pictures alone; a piece of an address; a host but not from one site; an exception, and what it does not except; a rule no exception undoes; frames; the top frame's own document, always allowed; a web socket; a page that is nobody's site; what has no host, and what is no address. Every kind of request the header has is in it, and both answers. **Every request has its answer.** With no rules, in each of the three ways there are to have none, every one is allowed. Eight threads asking the whole corpus at once, two hundred times each, are each given its answers |
| Guard's decisions, a rule at a time (`wt-guard`, `decisions`, 7) | A kind is the header's by its number, and "other" when it is not known. Each kind is asked about by its own name in the rules: a rule for one kind alone blocks that kind and no other, for all ten. The rules of several lists are one blocker, and an exception of one list undoes a rule of another. The first party is the site and not the host, by the public suffix list (two hosts under one registration are one; two registrations under `co.uk` are not). An address is matched as it is normalised, whatever its case and port. Rules that hide or mend a page's content block nothing. A rule that asks for a stand-in blocks; one that mends an address does not |
| The fast path, through the C functions (`wt-ffi`, `core`, +3 and one renamed) | A core in fake mode with a suite's rules: blocked and allowed as the rules say, with the status "ok"; the top frame's document allowed; a kind this core has not heard of asked about as one it cannot name; what is no address allowed, and what is not text allowed with the status that says so; a page that is no address asks as a third party. Another core, with no rules, allows the same: each core has its own blocker. A stopped core answers nothing, and the request is allowed. **Outside fake mode a launch's rules are nothing.** Eight threads at once, two thousand requests each: every answer right |
| What the blocker holds (`wt-ffi`, `held`, +1) | §5.2, each at its ceiling |
| A decision's time (`wt-guard`, `timed_budgets`, 1, set aside from the usual run; `Tools/test.sh budgets`) | §5.2: fails at 25 µs at p95, alone and with four threads asking |
| Guard's crate broken, a part at a time | 18 parts, 18 caught, at the first run |
| The fast path broken, a part at a time | 10 parts, 10 caught, at the first run: the last of them (rules that hide a page's content taken in as well) by the count of what the blocker holds, and by nothing else |
| Android, on the JVM, through the core's own library (`NativeCoreTest`, +1; `LaunchOptionsTest` and `ThroughTheCoreTest` longer) | A core started with a suite's rules blocks and allows as they say, through the function Android's adapter calls, with what a WebView can say of a request (that it is the page, or that it is not); another core with no rules allows the same. The extra is the rules as they were written, in fake mode in a build that can be debugged, and nothing otherwise |
| Android's adapter (`SystemWebViewAdapterTest`, as it was since 001) | A request the core blocks never leaves the device: with a scripted core |
| **The app on the emulator, by hand, 2026-11-25** | The app started in fake mode on the test site's page that shows a picture of the site's own, the site served on this Mac and recording what it was asked for. With a rule against the picture's address: the site was asked for the page, and not for the picture. With no rule, and with a rule for something else: it was asked for both. (Each time the WebView also asked for the site's icon, which is its own doing) |
| Every other test of the core | 786 before this, and all pass with it |

## 8. What is not verified

- **The choices of §6** are the recommendations, used on the owner's word; the owner has not looked at them.
- **The public lists.** Every rule here was written here (D6), and every number is of made-up rules (§5.1). What the public lists block, hold and cost is not known from any of it. 000 §5.8 has Brave's figure, some 15 MB for 100,000 of theirs; the 100,000 here held 4.6 MB.
- **A phone.** The budget for a decision is the phone's. It was measured on a Mac, where there is twenty times the room; and the app was seen on an emulator, which is not a phone.
- **On Android a rule for one kind of request alone does not match.** The System WebView says of a request only whether it is the page itself: everything else is asked about as a kind the engine cannot name, and a rule for scripts alone, or pictures alone, is then no rule. Most of the public lists' rules name no kind; many name one. The by-hand check used a rule that names none. Telling a request's kind from what the WebView does say (what the request accepts) is the Android adapter's next step. **Built since** ([065](065-a-requests-kind-on-android.md)): the adapter tells four kinds, and the core guesses the rest from the address.
- **No test in the gate is of the real core with a real WebView.** The adapter's test is with a scripted core, the core's with no WebView; the two together were seen by hand (§7). The app's device tests share one core, whose rules would be the first launch's (003).
- **Apple's systems:** nothing asks the fast path there, and nothing is blocked. WebKit's rule lists are later in W4.
- **Rules that change while requests are asked:** nothing changes rules yet (§4).
- **A request's method** (§4), and **requests made by a frame of another site:** the page that asks is the top frame's, as the header has always said, and a rule that depends on the frame's own site is read against the top frame's.
- **Many threads asking without a pause** are over the budget here (§5.1). The measurement that fails at the budget has four threads with a pause between requests.
- **The download's size.** The blocker adds 3.0 MB to the Mac app as it is built here; what it adds to what is downloaded is the release work's to measure.

## 9. Done checklist

- [ ] The owner's word on §6; and, before the lists, on which public lists may be checked in (D6).
- [x] The try (step 1).
- [x] Guard's crate, and the corpus.
- [x] The fast path; fake mode's rules; Android.
- [x] The budgets.
- [x] The gate: every stage of `Tools/test.sh all` green, and `android-device`, on 2026-11-25: `lint`, `core` (800, and the strings tool's 24), `unit` (407 on the Mac), `build`, the Mac's UI suite (67), `ios` (391 on the iPhone simulator), the iPhone's UI suite (56), `android` (538 and 33 on the JVM) and `android-device` (200 on the emulator).

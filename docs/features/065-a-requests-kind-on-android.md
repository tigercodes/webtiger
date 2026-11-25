# 065 — What a request is for, on Android (W4)

| | |
|---|---|
| **Status** | **Designed, on the recommendations of §6.** The owner's word of 2026-11-24 was to use what is suggested where something waits on a choice, and to go on: they are to be built on, and stay the owner's to change. Not built. What the System WebView says of a request is to be read on the emulator first (§5, step 1): nothing here has been tried |
| **Phase** | W4 of [000](000-master-plan.md) §9. What [064](064-guards-request-engine.md) §8 found: "On Android a rule for one kind of request alone does not match" |
| **Contract** | None |
| **Builds on** | [064](064-guards-request-engine.md): Guard's decision, and the kinds a request has. [003](003-android-shell.md): the System WebView's adapter, whose request callback asks Guard |
| **Last updated** | 2026-11-25 |

## 1. User story

A list has a rule that says "scripts from this host, on other people's sites". On my phone the browser must know that a request is for a script, or the rule is no rule.

## 2. What is in it, and what is not

**In:** on Android, a request is put to Guard with what it is for, as far as that can be told: by the adapter from what the WebView says, and by the core from the address where the engine cannot name a kind.

**Not in, each with where it goes:**

- **The other engines.** WebKit has no request callback (its rule lists carry the kinds themselves). The engines to come say a request's kind outright.
- **A request's method,** which the WebView does say and the fast path has no place for: with whatever next changes the fast path's request.
- **The frame that asks.** The page that asks is the top frame's (064 §8), here as there.

## 3. The rule

**What the WebView says of a request:** its address; whether it is for the top frame; its method; and the headers the page's engine has set by then, among them what the request accepts. It does not say what the request is for.

**What the adapter tells from that** (it is the engine's knowledge, and the adapter's to read):

- the top frame's own document: as now;
- **a frame's document:** not for the top frame, and it accepts a page before anything else;
- **a picture:** it accepts pictures before anything else;
- **a stylesheet:** it accepts a stylesheet before anything else;
- anything else: a kind the engine cannot name.

**What the core tells from the address,** for a request of a kind the engine cannot name (it is a guess, and a decision, so it is the core's; and it is then the same for any engine that cannot say): by the end of the address's path, a script, a stylesheet, a picture, a font or a film or sound, for the endings everybody uses for them. Whatever else stays a kind that cannot be named: a rule that names no kind matches it, and a rule for one kind alone does not.

**A guess can be wrong.** A script at an address with no ending is not known for a script, and a rule for scripts alone misses it: that is how it is today for every request. An address that ends like a picture and is fetched as a script is taken for a picture: a rule for pictures alone can then block a script, and a rule for scripts alone miss it. The lists' rules for hosts that only follow people mostly name no kind, and are untouched by any of this.

## 4. The contract

None. The kinds have been in the C header since 001.

## 5. In what order

1. **Read what the WebView says,** on the emulator: what each kind of request accepts, for a page of the test site that fetches one of each. Before the rule of §3 is built on what is only expected.
2. **The test site:** a page that fetches one of each kind, from addresses of its own.
3. **The adapter:** the kinds it can tell. The adapter suite, on the emulator: each request of that page is put to the core with its kind.
4. **The core:** the guess from the address, in Guard's crate. The corpus: requests of a kind that cannot be named, with the answers the guess gives.

## 6. Decisions

**All of these are recommendations** (§ Status). The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | The adapter tells what the WebView's headers tell, and no more | What a request accepts is the engine's own word about it. It names a frame, a picture and a stylesheet; it does not tell a script from a fetch, a font or a film | Nothing told (as now): every rule for one kind is lost |
| D2 | The core guesses the rest from the address's ending | A script's address mostly ends as one. It is a decision, so the core's; and one rule for every engine that cannot say | The guess in the adapter: one for each shell, in three languages. Or no guess: rules for scripts alone, which are many, never match on Android |
| D3 | What cannot be told or guessed is asked about as a kind that cannot be named | A wrong block breaks a page; a missed one is where every request is today | Taken for a script, as the likeliest: more of the lists' rules for scripts match, and a fetch of a page's own data is sometimes blocked as one |
| D4 | The guess is used only where the engine could not name the kind | An engine that says "a picture" knows | The address's ending over the engine's word |

## 7. Tests

To be filled in as built. Planned:

| Where | What |
|---|---|
| The adapter suite, on the emulator | A page that fetches one of each kind: each request is put to the core as the kind the adapter can tell, and as "cannot be named" for the rest |
| The adapter's rule, on the JVM | What is told from what a request accepts, for what each kind is seen to accept, and for what is not there |
| Guard's crate | The guess, ending by ending; that it is not made for a kind the engine named; the corpus, with requests of a kind that cannot be named |

## 8. What is not verified

To be filled in as built.

- **The choices of §6** are the recommendations; the owner has not looked at them.
- **What the WebView says** is what its documents and other people's accounts say: step 1 is to read it.

## 9. Done checklist

- [ ] The owner's word on §6.
- [ ] What the WebView says, read (step 1).
- [ ] The test site's page.
- [ ] The adapter.
- [ ] The core's guess, and the corpus.
- [ ] The gate.

# 065 — What a request is for, on Android (W4)

| | |
|---|---|
| **Status** | **Built and tested, on Android and in the core, on the recommendations of §6.** The owner's word of 2026-11-24 was to use what is suggested where something waits on a choice, and to go on: they are built on, and stay the owner's to change. Android's adapter tells a frame's page, a picture, a stylesheet and a film from what the WebView says of a request; the core guesses a script, a font and the rest from how the address ends, for a request of a kind the engine could not name. A rule for one kind of request alone now matches on Android, where it did not |
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

**What the WebView says of a request:** its address; whether it is for the top frame; its method; and the headers the page's engine has set by then. It does not say what the request is for.

**What it was seen to say** (§5.1), which is what the adapter's rule is built on:

| A request for | Accepts, first | Besides |
|---|---|---|
| The top frame's page | `text/html` | It is for the top frame |
| A frame's page | `text/html` | Not for the top frame |
| A picture (and the site's icon) | `image/avif`, then other pictures | |
| A stylesheet | `text/css` | |
| A film | anything (`*/*`) | It asks for a part of the thing: `Range: bytes=0-` |
| A script | anything | Nothing that tells |
| A fetch by the page's own script | anything | Nothing that tells |
| A font | anything | It says where it is asked from (`Origin`), as a fetch from another site does too |

**What the adapter tells from that** (it is the engine's knowledge, and the adapter's to read), by what a request accepts first, whatever the case it is written in:

- the top frame's own document, whatever it says: as before;
- **a frame's page:** not for the top frame, and it accepts a page first;
- **a picture:** it accepts a picture first;
- **a stylesheet:** it accepts a stylesheet first;
- **a film or a sound:** it accepts none of those first, and asks for a part of the thing;
- anything else: a kind the engine cannot name.

**What the core tells from the address,** for a request of a kind the engine could not name (it is a guess, and a decision, so it is the core's; and it is the same for any engine that cannot say): by how the address's path ends, whatever its case, and whatever query follows it.

| The path ends | Taken for |
|---|---|
| `.js`, `.mjs` | A script |
| `.css` | A stylesheet |
| `.png`, `.jpg`, `.jpeg`, `.gif`, `.webp`, `.avif`, `.svg`, `.ico`, `.bmp` | A picture |
| `.woff2`, `.woff`, `.ttf`, `.otf`, `.eot` | A font |
| `.mp4`, `.m4v`, `.webm`, `.mp3`, `.m4a`, `.ogg`, `.wav` | A film or a sound |
| Anything else, or no ending | A kind that cannot be named |

A kind that cannot be named is matched by a rule that names no kind, and by no rule for one kind alone. **What the engine did name is never guessed at.**

**A guess can be wrong.** A script at an address with no ending is not known for a script, and a rule for scripts alone misses it: that is how it was for every request. An address that ends like a picture and is fetched as a script is taken for a picture: a rule for pictures alone can then block a script, and a rule for scripts alone miss it. The lists' rules for hosts that only follow people mostly name no kind, and are untouched by any of this.

## 4. The contract

None. The kinds have been in the C header since 001.

## 5. In what order

1. **Read what the WebView says,** on the emulator: what each kind of request accepts, for a page of the test site that fetches one of each. Before the rule of §3 is built on what is only expected. **Done** (§5.1).
2. **The test site:** a page that fetches one of each kind, from addresses of its own. **Built:** `/kinds`, whose things are at addresses none of which ends as its kind's do, so that what the adapter tells is not what the core would guess.
3. **The adapter:** the kinds it can tell. The adapter suite, on the emulator: each request of that page is put to the core with its kind. **Built.**
4. **The core:** the guess from the address, in Guard's crate. The corpus: requests of a kind that cannot be named, with the answers the guess gives. **Built.**

### 5.1 What the WebView said

The System WebView 133, on an Android 16 emulator, 2026-11-25: the app's own adapter, with a line added for the reading and taken out again, writing what the WebView handed it for each request of `/kinds`. The table of §3 is what it wrote. Two things were not as the design expected: **a film can be told** (it asks for a part of the thing, which nothing else on the page did), and **a font cannot** (what it says of itself, a fetch from another site says too).

## 6. Decisions

**All of these are recommendations** (§ Status). The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | The adapter tells what the WebView's headers tell, and no more | What a request accepts is the engine's own word about it. It names a frame, a picture and a stylesheet; it does not tell a script from a fetch or a font. *As built:* a film too, by its asking for a part of the thing (§5.1) | Nothing told (as it was): every rule for one kind is lost |
| D2 | The core guesses the rest from the address's ending | A script's address mostly ends as one. It is a decision, so the core's; and one rule for every engine that cannot say | The guess in the adapter: one for each shell, in three languages. Or no guess: rules for scripts alone, which are many, never match on Android |
| D3 | What cannot be told or guessed is asked about as a kind that cannot be named | A wrong block breaks a page; a missed one is where every request is today | Taken for a script, as the likeliest: more of the lists' rules for scripts match, and a fetch of a page's own data is sometimes blocked as one |
| D4 | The guess is used only where the engine could not name the kind | An engine that says "a picture" knows | The address's ending over the engine's word |

## 7. Tests

| Where | What |
|---|---|
| The adapter suite, on the emulator, with the real WebView and a scripted core (`SystemWebViewAdapterTest`, +1) | The page that fetches one of each kind: its own document is put to the core as the page's, its stylesheet as a stylesheet, its picture as a picture, its frame as a frame's page, its film as a film; its script, its fetch and its font as a kind the engine cannot name |
| The adapter's rule, on the JVM (`RequestKindTest`, 4) | With the headers the WebView was seen to give: each kind told by what it accepts first; what accepts anything, says nothing of what it accepts, has no headers at all, or accepts a kind of data, is a kind that cannot be named; the top frame's document is one whatever it says; a header is read whatever the case of its name and of what it says; and what a request accepts first is what tells: a picture that asks for a part of itself is a picture |
| The core's guess (`wt-guard`, `decisions`, +2) | A rule for each kind alone, and for a kind that cannot be named: a request of no named kind is blocked by the rule for the kind its address ends as, and by no other, for all twenty-four endings, before a query and a fragment that end otherwise; an ending that names no kind, no ending, a dot earlier in the path, and a path that ends in a dot, are a kind that cannot be named. **What the engine named is never guessed at:** a picture, a fetch, a font and the rest at an address that ends as a script's are not scripts, and a script at an address that ends as a picture's is a script |
| The corpus (`Fixtures/guard/corpus.txt`, seven requests more: forty) | Requests of a kind that cannot be named: ending as a script, in capitals and before a query; with no ending; with the ending in an earlier part of the path; ending as a picture; with an ending that names no kind; and a picture at an address that ends as a script's |
| The adapter's rule and the core's guess broken, a part at a time | 17 parts, 17 caught, at the first run |
| **The app on the emulator, by hand, 2026-11-25,** with the real core and the real WebView, the test site on this Mac recording what it was asked for, and the app's own data cleared before each start | With a rule for its own kind alone, each of these was not asked of the site: a picture, a frame's page, a film, a stylesheet (told by the adapter); a script at an address that ends `.js` (guessed by the core). With a rule for another kind, or a rule that names something else, each was asked for |
| Every other test of the core | 800 before this, and all pass with it |

**Two things the by-hand check taught,** about checking and not about the app. A rule with a `$` in it, handed to the app through the device's shell, has what follows the `$` taken out by that shell unless it is quoted for it: the first run "passed" with every rule shorn of its kind. And the test site's one script that ends `.js` may be kept by the WebView for an hour: started twice, the second start does not ask the site for it, rule or no rule. The app's data on the emulator is cleared before each start now.

## 8. What is not verified

- **The choices of §6** are the recommendations, used on the owner's word; the owner has not looked at them.
- **One WebView, on an emulator.** What a request says of itself is Chromium's doing and could change with a version; the test in the adapter suite fails if it does. A phone was not tried.
- **A sound,** as against a film; **a web socket,** which the WebView does not put to the callback at all; **a ping** (a beacon); **a script fetched as a module from another site,** which may say what a font says: none is on the test page. Each is a kind that cannot be named, or the core's guess.
- **A request that asks for a part of something that is no film** (a page's own script fetching a file in parts) is taken for a film. Not seen; rules for films alone are few.
- **How often the guess is wrong** on real pages, and how many of the public lists' rules for one kind it gains: there are no public lists here (064 D6).
- **The frame that asks, and the method** (064 §8), are as they were.

## 9. Done checklist

- [ ] The owner's word on §6.
- [x] What the WebView says, read (step 1).
- [x] The test site's page.
- [x] The adapter.
- [x] The core's guess, and the corpus.
- [ ] The gate.

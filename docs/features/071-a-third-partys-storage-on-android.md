# 071 — A third party's storage, on Android (W4)

| | |
|---|---|
| **Status** | **Designed, on the recommendations of §6; nothing built.** The owner's word of 2026-11-24 was to use what is suggested where something waits on a choice, and to go on: they are to be built on, and stay the owner's to change. It mends what [070](070-third-party-cookies.md) found: on Android a site in a frame of another site's page reads there what it stored as itself. **What is designed stops the plain use of storage to follow a person, and is not a wall** (§3, §8) |
| **Phase** | W4 of [000](000-master-plan.md) §9, with "third-party cookies" and "page fixes": the engine is mended where it falls short of the others. The owner's word: "make sure cookies don't leak across sites in any system" |
| **Contract** | Nothing in the contract's messages. One question more on the fast path, in the C header: `wt_storage_check` (§4) |
| **Builds on** | [070](070-third-party-cookies.md): the finding, the test site's pages, and the tests that say it as it is. [067](067-guards-switches.md): Guard's switches, and the fast path that heeds them. [069](069-global-privacy-control.md): a script of the adapter's own, run before a page's, in every frame. `wt-core`: which site an address is of |
| **Last updated** | 2026-11-28 |

## 1. User story

My browser no longer hands a site its cookies when the site is only a piece of the page I am reading. On my phone the same site can still leave itself a note in the page's storage, and read it again from the next page it is a piece of. I want that stopped too.

## 2. What is in it, and what is not

**In:** on Android, a frame of another site than the page's has no storage of its site's own. What its script keeps in the page's storage there is kept for that document and goes with it; the other stores are refused, as an engine refuses them where it will not give them. The core says which frames: by its rule for sites, and by Guard's switches.

**Not in, each with where it goes:**

- **WebKit:** it keeps a third party's storage apart by itself (070 §5.1). The engines to come (Chromium, as itself) do too.
- **A worker the frame starts:** it has stores of its own, and no script of the adapter's runs in a worker. Said (§8), and not mended.
- **Storage that lasts, kept apart for each pair of page and frame:** what Chrome and WebKit do. Doing it from a script would put a third party's notes, under other names, in the storage its own site reads. Not built.
- **Messages between a site's own documents** (a broadcast channel), by which a frame could talk to its own site open in another tab at that moment: not touched.
- **A third party that asks to be let in** (the Storage Access API): the engine's own.
- **The lists:** a known follower's frame is best not loaded at all, and that is Guard's, when there are lists ([064](064-guards-request-engine.md) D6).

## 3. The rule

**Who is kept apart.** A frame whose site is another than the site of the page in the address field. Site is the core's word (what somebody registered, or the host where nothing is registered): a frame of another host of the page's own site is not apart. A frame within a frame is judged against the page, whatever is between them, as cookies are. A frame with no address of its own (an empty one, one written by a script) is of whoever made it, as the engine has it.

**Under Guard's switches** (067). Where Guard is off for the page, by the nearest switch that says anything, nothing is kept apart: the WebView is as it is. "This page is broken, stop protecting here" stops this too. (Third parties' cookies have no such switch, 070 D1: there the engines give none. Here the decision is asked of the core for each frame, so it can.)

**What "apart" means,** for a frame that is:

| What a script reaches for | What it gets |
|---|---|
| The page's storage (`localStorage`, `sessionStorage`) | A store of this document's own: empty at first, working as a store does, and gone with the document. Nothing it holds is the site's own storage, and nothing put in it reaches there |
| A database (`indexedDB`) | Refused: opening one fails as it does where an engine denies it |
| A cache (`caches`) | Refused, the same way |
| A worker that serves pages (`navigator.serviceWorker`), a shared worker | Not offered: to a script that looks, this is an engine without them |
| The private file system (`navigator.storage.getDirectory`) | Refused |
| Cookies | None: 070 |

**Decided by the core, at the start of each frame's document.** The adapter's script runs before any script of the page's, in every frame. In a frame it asks one question, and waits for the answer: is this frame, in this page, to be kept apart? The question goes to the core's fast path, which answers from the core's rule for sites and from Guard's switches, on whatever thread asks. Nothing is asked for the page itself, or for a frame of the page's own origin.

**When the question fails** (no core, nonsense asked), the answer is "apart": a third party is let at its storage only on the core's word.

**It is not a wall.** A script that runs before a page's can take away what the page reaches for; it cannot take away what an engine would give by another door. A worker has databases of its own. A script that goes looking for a way round a script may find one. What this stops is a site keeping a note in storage in the plain way, which is how it is done; what stops a site that works at it is not loading its frame (the lists), or an engine that keeps storage apart itself. The documents say so wherever this is described.

## 4. The fast path

One function more, beside `wt_guard_check` and `wt_nav_check`:

- **`wt_storage_check(core, frame, out)`:** `frame` is the frame's origin, the page's origin, and the web view. The verdict is `WT_STORAGE_OWN` (the frame has its site's storage) or `WT_STORAGE_APART`. Apart when the two origins are of different sites and Guard is on for the page in that web view. On any error, apart.

It parses two origins and reads the switches: it allocates, as Guard's check does, and locks nothing of its own.

No message of the contract changes: nothing is said to a shell or by one. Android's adapter asks through the core's library, as it asks Guard.

## 5. In what order

1. **The core:** the rule (two origins, one site or not), and the fast path's question with Guard's switches. Tested in the core and through the C functions.
2. **The test site:** `/keep` and `/kept` (070) also keep and tell a record in a database; a page that tries each store and says what became of it.
3. **Try the WebView:** that a script of the adapter's runs before a page's in a frame of another origin, in a frame with no address of its own, and can ask a question and wait for the answer. On the emulator, before the rest is built on it.
4. **Android's adapter:** the script, the question, and the suite: the tests of 070 that say "its storage it has" then say it has not.
5. **In the app,** with the real core: a third party in a frame has nothing; with Guard off for the page, it has what the WebView gives.

## 6. Decisions

**All of these are recommendations** (§ Status). The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | Mend it with a script of the adapter's, though that is not a wall | The WebView gives an app nothing else, and the gap is in what the owner asked for by name. The plain use is the common one | Leave it to the lists: nothing until there are lists, and nothing for a follower no list knows. Wait for the WebView to keep storage apart itself: no date is known |
| D2 | The core says which frames, asked through the fast path at each frame's start | Which site an address is of is the core's knowledge, and is not to be written again in a script. Asked as the document starts, the answer cannot be late | The adapter told once which site the page is of: a frame that starts before the telling lands is judged against the page before. A rule of thumb for sites in the script: a third copy of the core's rule, and wrong where it matters |
| D3 | Under Guard's switches | A page this breaks needs a remedy, and Guard's switch is the remedy a person has | Always on, like cookies: no remedy |
| D4 | The page's storage is replaced by a store that goes with the document; the other stores are refused | A frame that only wants to remember something while it is on the screen goes on working. A database cannot be stood in for by a few lines, and pages already meet engines that refuse one | Everything refused: more frames break. Everything stood in for: a great deal of code, to be wrong in |
| D5 | When the question fails, apart | The harm of being wrong that way is a frame that forgets; the other way is a person followed | Not apart, as Guard's check allows on an error: that rule is for a blocker, which breaks pages when it is wrong |
| D6 | Only on Android | The other engines do it themselves, and better: their storage lasts, apart | Everywhere, for sameness: it would make WebKit worse |

## 7. Tests

To be written with each step. Foreseen:

| Where | What |
|---|---|
| The core's rule | Two origins of one site (the same, another host of it, another port, another scheme) and of two; an origin that is none (`null`, empty, not an address) |
| The fast path, through the C functions | Apart for two sites, not for one; not apart where Guard is off for the page (everywhere, the Space, the site), and apart again when it is on; apart on an error; many threads asking while a switch is turned |
| Android's adapter, with the real WebView on the emulator | 070's tests, with the storage now apart: a site in a frame of another site's page finds nothing stored and what it stores there is not there when it is visited as itself; each store, tried: the page's storage works and is its own, a database and a cache are refused, a worker that serves pages is not offered; a frame of the page's own site has that site's storage; a frame the core says is not apart (Guard off) has what the WebView gives; a frame within a frame; a frame with no address of its own |
| The script and the question broken, a part at a time | On the emulator |
| The apps | On Android, with the real core: nothing stored reaches a frame of another site; with Guard off for the page, it does |

## 8. What is not verified

To be filled in as it is built. Known now:

- **The choices of §6** are the recommendations, used on the owner's word; the owner has not looked at them.
- **It is not a wall** (§3): a worker's own databases, and whatever way round a script a script can find. No test here plays the part of a site that works at it, beyond the ones named in §7.
- **Pages it breaks:** a frame that needs its site's storage to work (a sign-in in a frame, a chat that remembers) forgets, on Android, until Guard is turned off for the page; and no shell has a place to turn Guard off yet (067). Not tried on a real site.
- **A WebView that keeps storage apart itself,** should one come: this would then stand in its way, giving a store that goes with the document where the engine would give one that lasts. The suite's tests of what the WebView does by itself (with the question answered "not apart") would show it.
- **A phone:** an emulator is not one.

## 9. Done checklist

- [ ] The owner's word on §6.
- [ ] The core's rule and the fast path's question.
- [ ] The test site's pages.
- [ ] The WebView tried (step 3).
- [ ] Android's adapter, and its suite.
- [ ] In the app.
- [ ] The gate.

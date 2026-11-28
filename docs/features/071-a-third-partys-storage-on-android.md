# 071 — A third party's storage, on Android (W4)

| | |
|---|---|
| **Status** | **Built and tested, in the core and on Android, on the recommendations of §6.** The owner's word of 2026-11-24 was to use what is suggested where something waits on a choice, and to go on: they are built on, and stay the owner's to change. It mends what [070](070-third-party-cookies.md) found: on Android a site in a frame of another site's page read there what it stored as itself. Now such a frame finds nothing its site stored, what it stores goes with its document, and the stores that last are refused it: in the adapter's suite and in the app, on the emulator. **It stops the plain use of storage to follow a person, and is not a wall: two ways round it are known, tried, and said as they are** (§5.1, §8) |
| **Phase** | W4 of [000](000-master-plan.md) §9, with "third-party cookies" and "page fixes": the engine is mended where it falls short of the others. The owner's word: "make sure cookies don't leak across sites in any system" |
| **Contract** | 1.57: no message changes. One question more on the fast path, in the C header: `wt_storage_check` (§4) |
| **Builds on** | [070](070-third-party-cookies.md): the finding, the test site's pages, and the tests that say it as it is. [067](067-guards-switches.md): Guard's switches, and the fast path that heeds them. [069](069-global-privacy-control.md): a script of the adapter's own, run before a page's, in every frame. `wt-core`: which site an address is of |
| **Last updated** | 2026-11-28 |

## 1. User story

My browser no longer hands a site its cookies when the site is only a piece of the page I am reading. On my phone the same site can still leave itself a note in the page's storage, and read it again from the next page it is a piece of. I want that stopped too.

## 2. What is in it, and what is not

**In:** on Android, a frame of another site than the page's has no storage of its site's own. What its script keeps in the page's storage there is kept for that document and goes with it; the other stores are refused, as an engine refuses them where it will not give them. The core says which frames: by its rule for sites, and by Guard's switches.

**Not in, each with where it goes:**

- **WebKit:** it keeps a third party's storage apart by itself (070 §5.1). The engines to come (Chromium, as itself) do too.
- **A worker the frame starts:** it has stores of its own, and no script of the adapter's runs in a worker. Tried, said (§8), and not mended.
- **An empty frame the frame makes for itself:** the WebView runs no script of the adapter's there. Tried, said (§8), and not mended.
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

**Which page.** The frame's script says who the frame is and who the top frame above it is, as its own document has them. The adapter adds the page's address as it knows it: the one whose arrival the engine last told it of. The core answers "its own storage" only when the two agree: when the top frame the frame's document has is of the origin of the page the adapter knows. An adapter hears of a new page a moment after its engine has it, and a frame that starts in that moment is in a page nothing is known of: it is kept apart, and is judged neither against the page before nor by that page's switches. So what a frame says of its top frame can only keep it apart.

**When the question fails** (no core, nonsense asked, nobody to ask), the answer is "apart": a third party is let at its storage only on the core's word.

**As built, where the design did not say:**

- **The question's name is taken out of the page** by the script, before any script of the page's runs, in the page itself as in its frames: a page cannot ask it. (But for an empty frame a script makes, §8.)
- **A frame within a frame** is judged against the page: a frame of another site two frames down is apart; a frame of the page's own origin has the page's storage, whatever is between them, and nothing is asked for it.
- **The session's storage** is stood in for as the page's is: in one tab, the WebView gives a site in a frame the session's storage it had as itself there.
- **A frame that is nobody's** (an origin of none: a frame shut in a sandbox) is kept apart where Guard is on, like any other that is not of the page's site.
- **An empty frame a script makes** is of whoever made it, and by the rule is apart with its maker. It is not: the WebView runs no script of the adapter's in one, and its storage is the engine's own (§5.1, §8).
- **A top frame that is no page of the web** (an empty tab a script wrote into) has no origin the core can hold a frame's word against: its frames of other origins are kept apart, whatever Guard's switches say.

**It is not a wall.** A script that runs before a page's can take away what the page reaches for; it cannot take away what an engine would give by another door. A worker has databases of its own. An empty frame a site makes for itself has the storage of its maker, and no script of the adapter's runs in it. What this stops is a site keeping a note in storage in the plain way, which is how it is done; what stops a site that works at it is not loading its frame (the lists), or an engine that keeps storage apart itself. The documents say so wherever this is described.

## 4. The fast path

1.57. One function more, beside `wt_guard_check` and `wt_nav_check`:

- **`wt_storage_check(core, frame, out)`:** `frame` (`wt_frame`) is the frame's origin, the origin of the top frame above it (both as the frame's document has them), the page's address as the shell knows it, and the web view. The verdict is `WT_STORAGE_OWN` (the frame has its site's storage) or `WT_STORAGE_APART`. Apart when the top frame is not of the origin of the page the shell knows; otherwise own where Guard is off for the page in that web view; otherwise apart unless the frame is of the page's site. On any error, apart.

It parses its three addresses and reads the switches: it allocates, and locks nothing.

No message of the contract changes: nothing is said to a shell or by one. The minor goes up all the same, so that a shell can tell from the library's version that the function is there. Android's adapter asks through the core's library, as it asks Guard; Apple's shells ask nothing (D6).

## 5. In what order

1. **The core:** the rule, and the fast path's question with Guard's switches. **Built.**
2. **The test site.** **Built:** `/stores`, which tries every store a page has, says what each had, and keeps a name in each it was let into; `/storage-works`, which uses the page's storage as a page does and says what did not work; `/kept-through-a-frame` and `/kept-through-a-worker`, the two ways round (§8); `/holds-a-holder`, a frame within a frame.
3. **Try the WebView.** **Done,** by building it: the adapter's suite is the trial (§5.1).
4. **Android's adapter:** the script, the question, and the suite. **Built.**
5. **In the app,** with the real core. **Built.**

### 5.1 What the engines do

As tried, 2026-11-28: the System WebView 133 on an Android 16 emulator; WebKit on macOS 27.2 and on the iOS 27 simulator. A site that kept the name "ada" in every store it has, as itself, then met in a frame of another site's page:

| | The System WebView, by itself | The System WebView, kept apart by the adapter | WebKit, by itself |
|---|---|---|---|
| The page's storage | "ada" | Nothing: a store of the document's own | Nothing |
| The session's storage, in the tab the site was itself in | "ada" | Nothing: a store of the document's own | Nothing |
| A database | "ada" | Refused | Nothing |
| A cache | "ada" | Refused | Nothing |
| The private file system | Given | Refused | Given |
| A worker that serves pages | Offered | Not there | Offered |
| A shared worker | Not there (the WebView has none) | Not there | Offered |
| What the frame keeps, when the site is next visited as itself | There, in every store | Not there | Not there |
| The store the frame is given, used as a page uses storage | Works | Works | Works |
| The site's storage, reached through an empty frame of the frame's own making | (its own anyway) | **"ada": a way round** | Nothing |
| The site's database, from a worker the frame starts | (its own anyway) | **"ada": a way round** | Nothing |

**The WebView shares every store, not only the page's.** 070 looked at the page's storage alone; a database, a cache and the session's storage are shared the same way.

**A script of the adapter's runs before a page's own in a frame of another origin, and can ask and wait.** The script is added to the web view for every origin (`addDocumentStartJavaScript`); the question is an object of the adapter's the WebView puts in each frame (`addJavascriptInterface`), whose one method answers yes or no on a thread of the WebView's own. The script takes the object's name out of the page first of all.

**The WebView runs no such script in an empty frame a script makes.** A frame with no address of its own is of whoever made it, and has their storage: the engine's own, untouched. It is the first way round (§8).

**WebKit keeps every store apart by itself,** and through an empty frame and from a worker too: the same pages say so there, so that a WebKit that stopped would be noticed.

## 6. Decisions

**All of these are recommendations** (§ Status). The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | Mend it with a script of the adapter's, though that is not a wall | The WebView gives an app nothing else, and the gap is in what the owner asked for by name. The plain use is the common one | Leave it to the lists: nothing until there are lists, and nothing for a follower no list knows. Wait for the WebView to keep storage apart itself: no date is known |
| D2 | The core says which frames, asked through the fast path at each frame's start | Which site an address is of is the core's knowledge, and is not to be written again in a script. Asked as the document starts, the answer cannot be late. *As built:* the frame says who its top frame is, the adapter says which page it knows, and the core lets a frame at its storage only where the two agree (§3): neither alone is enough, since the adapter is a moment behind its engine, and a page may say anything | The adapter told once which site the page is of: a frame that starts before the telling lands is judged against the page before. A rule of thumb for sites in the script: a third copy of the core's rule, and wrong where it matters. The frame's word alone for its page: a page that asked in a frame's name could learn of any site whether Guard is off for it |
| D3 | Under Guard's switches | A page this breaks needs a remedy, and Guard's switch is the remedy a person has | Always on, like cookies: no remedy |
| D4 | The page's storage is replaced by a store that goes with the document; the other stores are refused | A frame that only wants to remember something while it is on the screen goes on working. A database cannot be stood in for by a few lines, and pages already meet engines that refuse one | Everything refused: more frames break. Everything stood in for: a great deal of code, to be wrong in |
| D5 | When the question fails, apart | The harm of being wrong that way is a frame that forgets; the other way is a person followed | Not apart, as Guard's check allows on an error: that rule is for a blocker, which breaks pages when it is wrong |
| D6 | Only on Android | The other engines do it themselves, and better: their storage lasts, apart (§5.1) | Everywhere, for sameness: it would make WebKit worse |
| D7 | The two ways round that were found are said, and tested as they are; they are not closed | An empty frame can be reached for in ways no script can stand in front of (by its number in the window), and a worker's stores are out of any page's reach: closing the ways that can be closed leaves the others, with more code to be wrong in. What stops a site that works at it is not loading its frame, or an engine that keeps storage apart | Close what can be closed (an empty frame reached for by its element): the look of a wall, and not one |

## 7. Tests

| Where | What |
|---|---|
| The core's rule (`wt-core`, +3) | **A frame of another site than the page's is kept from its storage:** of the page's own site (its origin, another host of it, another port, another scheme, whatever the case) it is not; of another (another name, one that only ends alike, one under another suffix, an address of numbers) it is; an origin that is none (`null`, empty, not an address) is; and two names of one machine are two sites, each itself whatever the port. **A frame in a page the shell does not know is kept apart,** whatever the switches say: a top frame of another site than the page the shell knows, of another host of its site, of another port or scheme, or of no origin; and a page that is none of the web's. The top frame's origin is the page's however it is written. **Where Guard is off for the page no frame is kept apart:** all eighteen of the switches' ways; the switches are the page's and not the frame's own site's; a web view that is not known is under everywhere's |
| The fast path, through the C functions (`wt-ffi`, `core`, +4; the header's test, wider) | Apart for another site and for a frame that is nobody's, not for the page's own by any host; apart where the top frame is not the page the shell knows; a web view that is not known. **It fails apart:** no frame, an origin that is not text, one that is nowhere but has a length, no core of that handle and none at all, nowhere to write the verdict, and a core that has stopped; the verdict starts as "its own", so that what a failure leaves is seen to have been written. **Guard's switches, from the next question:** off everywhere, for the Space, on for the site in a Space that is off, the Space saying nothing again, off for the site alone. **Eight threads asking while everywhere's switch is turned three hundred times:** every answer is "ok" with one verdict or the other, both were seen, and the last switch set is the one in force. The header declares the function, its two verdicts and its structure's layout |
| Android, on the JVM, through the core's library (`NativeCoreTest`, +2; `ScriptedCoreTest`, +1) | Through the function the adapter calls: apart for another site, for a frame that is nobody's, for text that is no address and text that is not UTF-8; not for the page's own site; apart where the top frame is not the page the shell knows; Guard off everywhere, the very next frame has its storage, and on again, not; a core that has stopped keeps every frame apart. Only the library's own word lets a frame at its storage: a verdict this client does not know keeps it apart. The scripted core says "its own" until a test says otherwise, and keeps what was asked |
| Android's engine and the question, on the JVM (`SystemWebViewEngineTest`, +2, and two wider) | A frame's storage is the core's to say, with all that was said of the frame; with no core, apart. The question takes text and nothing else: what is not text, or is longer than an origin can be, is kept apart and nobody is asked; an adapter that fails is not an answer. The script takes its question's name out of the page first of all, asks only in a frame, and keeps apart unless told otherwise |
| Android's adapter, with the real WebView on the emulator (`SystemWebViewThirdPartyStorageTest`, 7) | The test plays the core's part: it says which frames are apart, and sees what was asked. **A site in another site's page finds nothing it stored, and what it stores there goes with the page:** the core was asked about that frame, by its origin, its top frame's and the page the adapter knows, and about no other; as itself the site has what it had; what it stores in a frame is not there as itself, nor in the next page it is a frame of. **Every store, tried** (§5.1), in the very web view the site was itself in: the page's storage and the session's empty, a database, a cache and the private file system refused, the workers not there; as itself, the name it kept, in each. **The store a frame is given works as a page's storage does:** the page that tries it passes the WebView's own storage, as itself and in a frame, and the stand-in. **A frame of the page's own site** has its storage in every store, nothing is asked, and the question's name is not in the page. **A frame the core says has its storage has what the WebView gives:** every store its site has as itself, and what it keeps there is the site's from then on: the gap of 070, store by store, so that a WebView that keeps any apart is noticed. **A frame within a frame** is judged against the page. **The two ways round, as they are** (§8) |
| WebKit, by itself (`WebKitThirdPartyStorageTests`, 3, on the Mac and the iPhone simulator) | The same pages: every store of a site in another site's page is apart from its own, in the web view it was itself in, and what it keeps there is not the site's; neither an empty frame nor a worker reaches the site's own storage; and WebKit's storage, as itself and in a frame, passes the page that tries storage. (An app's web view on an iPhone is offered no worker that serves pages; on the Mac it is. Said in the test) |
| The apps (`AppThirdPartyStorageTest`, 1, on the Android emulator; `AppThirdPartyCookieTest`, changed) | With the real core: a site keeps a name in every store; in another site's page, in the same tab, it has none of it; with Guard turned off everywhere, it has all of it; turned on again, none, from the next page. And 070's test of the app now says the frame finds nothing stored |
| Broken, a part at a time | **In the core and the fast path: 15 parts, 15 caught,** at the first run. **Through the library and in Kotlin, on the JVM: 12 parts, 12 caught;** the first run caught 11: nothing held a verdict the client does not know to "apart", since the library gives only the two; the reading is a function of its own now, with a test. **With the real WebView, on the emulator: 23 parts, 23 caught,** at the first run: the adapter adding no script, reading the verdict the wrong way round, or saying no page; the name left in the page; the page itself, or a frame of its own origin, asking; a frame the core lets through kept apart all the same; the top frame taken to be the frame above; each store left as it was, and the two stood-in stores made one; and the stand-in store broken eight ways (forgetting what it is given, keeping what is not text, not taking away, not emptying, not being a Storage, and its names as properties). 50 parts in all |
| Every other test | The core's (832 before this), the Swift package's on the Mac (420) and Android's on the JVM (577) pass with it |

## 8. What is not verified, and what is known

- **The choices of §6** are the recommendations, used on the owner's word; the owner has not looked at them.
- **Two ways round it, found and tried** (§5.1), and left as they are (D7). **An empty frame of the site's own making:** a frame with no address is of whoever made it, the WebView runs no script of the adapter's in one, and through it the site has its storage, at once. **A worker the frame starts:** it has the site's databases and caches. Either takes a site that sets out to get round this; no site is known to. The suite says both as they are.
- **Other ways, not tried:** a window the frame opens (it is a page of its own, and the site is itself there, as in every browser); messages between a site's own documents open at one moment (a broadcast channel); what the engine keeps for a site that is not storage a script reaches (its cache of what it fetched).
- **The question's name is in an empty frame a script makes,** since no script of the adapter's runs there to take it away. A page that asks by it learns what the core would say of a frame: of the page it is in, whether Guard is on there, which it can see anyway by what is blocked; and, for the moment after a new page arrives and before the adapter hears of it, the same of the page before. Nothing else: the core answers for the page the adapter knows and no other (§3).
- **The moment a new page arrives:** a frame that starts before the adapter has heard of its page is kept apart, whoever it is (§3). For a frame of the page's own site by another host, that is a frame that forgets for that one load. It was not seen in any test, and how often it happens on a busy phone was not measured. The same moment is in Guard's check of requests (064), which goes by the page the adapter knows.
- **A frame shut in a sandbox,** which is nobody's: kept apart by the core's rule, where the engine refuses it storage anyway. Not tried with the WebView.
- **Pages it breaks:** a frame that needs its site's storage to work (a sign-in in a frame, a chat that remembers) forgets, on Android, until Guard is turned off for the page; and no shell has a place to turn Guard off yet (067). Not tried on a real site.
- **What it costs:** one question, asked and waited for, for each frame of another origin as its document starts. Not measured.
- **A WebView that keeps storage apart itself,** should one come: this would then stand in its way, giving a store that goes with the document where the engine would give one that lasts. The suite's test of what the WebView gives by itself would fail, and say so.
- **A WebView too old to run a script before a page's own:** nothing is kept apart there. None that can keep login sets apart is known to be.
- **A WebView older or newer than 133,** and one on a phone: an emulator is not a phone.
- **The stand-in store** is held to the page that tries storage, which the engines' own storage passes on all three; it sends no `storage` event, which is for other documents of the same site and there are none that share it.

## 9. Done checklist

- [ ] The owner's word on §6.
- [x] The core's rule and the fast path's question.
- [x] The test site's pages.
- [x] The WebView tried (step 3).
- [x] Android's adapter, and its suite.
- [x] In the app.
- [ ] The gate.

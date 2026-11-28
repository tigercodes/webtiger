# 069 — Global Privacy Control (W4)

| | |
|---|---|
| **Status** | **Built and tested, in the core and on both engines, on the recommendations of §6.** The owner's word of 2026-11-24 was to use what is suggested where something waits on a choice, and to go on: they are built on, and stay the owner's to change. Every page a WebTiger app loads is told Global Privacy Control: its scripts read it, in every frame, and cannot make it say otherwise; and each load the browser itself begins carries the header. What a page fetches of its own accord, and a link followed inside a page, go without the header: neither engine lets a shell add to those |
| **Phase** | W4 of [000](000-master-plan.md) §9: "Global Privacy Control". 000 §5.9: "Sent where the engine lets a header be added; exposed to scripts everywhere" |
| **Contract** | 1.56: `WebViewCreate.global_privacy_control` |
| **Builds on** | The engine adapters, and the adapter suite that holds each to the same behaviour. The test site. [067](067-guards-switches.md): a thing a person may turn off |
| **Last updated** | 2026-11-28 |

## 1. User story

There is a signal a browser can send that says "do not sell or share what you learn of me", and in some places a site must heed it. I want my browser to send it for me, everywhere, without my asking each site.

## 2. What is in it, and what is not

**In:** every page is told the signal in the two ways the signal has: a page's own scripts can read it; and the requests that the browser itself begins carry it as a header, where the engine lets a header be added.

**Not in, each with where it goes:**

- **A switch for it.** It is on. Turning it off, everywhere or for a site, is a setting, with the page of settings that does not exist yet.
- **Requests the engine begins by itself** (a link pressed in a page, a picture, a script): no engine here lets a shell add a header to those. The plan says so: "where the engine lets".
- **Whether a site heeds it.** That is the site's, and the law's.

## 3. The rule

**The signal** is Global Privacy Control, as its specification has it: to a page's scripts, `navigator.globalPrivacyControl` is `true`; to a server, the request carries `Sec-GPC: 1`.

**To scripts, everywhere:** in every page and every frame, of every site, from before the page's own scripts run. A script that reads it finds `true`, and cannot change it.

**To servers, where the browser begins the request:** an address a person typed, a link from another app, a tab that wakes, a reload asked from the window: the loads the core asks a shell for. Not what the engine fetches of its own accord, which a shell cannot add to.

**The core says, the adapters do.** Whether the signal is sent is the core's to say, to each shell, for each web view it asks for: so that the day it has a switch, nothing in a shell changes.

**A private window** sends it too.

## 4. The contract

1.56. One thing a shell is told when it is asked to make a web view: `WebViewCreate.global_privacy_control`. With it, the web view's pages are told the signal: to their scripts, before any of a page's own run, in every frame, where the page cannot change it; and to servers, with `Sec-GPC: 1` on each load the shell begins for the web view (`WebViewLoad`). A web view a page opened by itself is told as the one that opened it is.

The core says it for every web view it asks for (D1, D3).

## 5. In what order

1. **Try each engine.** **Done,** by building it: the adapter suites are the trial (§5.1).
2. **The test site:** `/gpc`, which says in its title and its words what its script read first, and what it read after trying to write the property and to make it again; and `/gpc-in-a-frame`, which holds it in a frame of another origin and says what the frame read. **Built.** (The servers did not have to learn anything: each already keeps what every request said of itself, and a test reads the header from that. Android's did not, and does now.)
3. **The contract, and the core.** **Built.**
4. **Each adapter,** and the adapter suite on each: the same tests. **Built.** And a test in each app, with the real core.

### 5.1 What the engines allow

As tried, 2026-11-28: WebKit on macOS 27.2 and on the iOS 27 simulator; the System WebView 133 on an Android 16 emulator. The same on all three.

| | What was seen |
|---|---|
| A script of the shell's, before the page's own, in the page's own world | The page reads `true` at its first line |
| In a frame of another origin | The frame reads `true` |
| The page writes the property, then makes it again | It still reads `true`: the property is the page's `navigator`'s own, not to be written, taken away or made again |
| A load the shell begins | The request carries `Sec-GPC: 1`, each time |
| What the page fetches of its own accord (a picture) | Without the header |
| A link of the page, followed in the page | Without the header; the page that arrives reads `true` all the same, since the script is the web view's |
| A window a page opens by itself | Its pages read `true`, as the opener's do; and a load the shell begins in it carries the header. Opened by a page that is not told: neither |
| A web view the core did not say it for | The page reads nothing (`undefined`), what it then writes is its own, and no header |

**How it is done.** Neither engine has a setting for the signal. The property is defined by a script the adapter adds to the web view: on WebKit a user script at the start of the document, for every frame, in the page's world; on Android `addDocumentStartJavaScript`, for every origin. The header is added to the request the adapter makes for a load: a header field of WebKit's request, and `loadUrl`'s additional headers on Android. A WebView too old to run a script before a page's own (none that can keep login sets apart is, so far as is known) sends the header and leaves its pages untold.

## 6. Decisions

**All of these are recommendations** (§ Status). The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | On, for everybody, from the first run | 000 §5.9 lists it among the protections there are, not among choices. It asks a site for less, never for more | Off until asked for: nobody asks |
| D2 | The header only where a shell can add it, and the property everywhere | It is what the engines allow. A site that reads the property in its page has the signal whatever the header | Nothing sent unless both can be: the property alone is worth having |
| D3 | The core says whether, for each web view | A switch later is then the core's alone | Each shell always sends it |
| D4 | Not under Guard's switches | Guard's switch means "this page is broken, stop blocking". The signal blocks nothing and breaks nothing | Off where Guard is off |

## 7. Tests

| Where | What |
|---|---|
| The core, through the contract (`privacy_signal`, 1) | Every web view the core asks a shell for is to be told the signal: an ordinary window's tab, a second tab, a link opened in a tab of its own, a private window's tab, and a tab that slept and woke |
| The WebKit adapter, with real WebKit and the suite's pages (`WebKitPrivacySignalTests`, 7, on the Mac and the iPhone simulator) | §5.1, row by row: a page reads the signal and cannot make it say otherwise; a frame of another origin reads it; a load the shell begins carries the header, twice in one web view, and the page's picture does not; a link the page follows goes without it, and the page that arrives reads the signal; a window a page opens is told as its opener is, and a load begun in it carries the header, and neither where the opener is not told; a web view whose pages are not told has neither; and the header is on a load's request and on nothing else |
| Android's adapter, with the real WebView on the emulator (`SystemWebViewPrivacySignalTest`, 6) | The same behaviours |
| Android's engine, on the JVM (`SystemWebViewEngineTest`, +1) | A page is made as the core said of its web view, and a window a page asks for as the page that asked |
| The apps, in fake mode, with the real core and the real engine (`PrivacySignalTests`, 1, on the Mac and the iPhone simulator; `AppPrivacySignalTest`, 1, on the Android emulator) | A page the app loads says it read the signal and could not change it; and the request the app began for it carried the header |
| The signal broken, a part at a time | 16 parts, 16 caught, at the first run: the core's word (1); in WebKit's adapter (9: pages not told, only the top frame told, told after the page's own scripts, a property the page can write or make again, no load or every load with the header, and a web view the core asked for or one a page opened without it); in Android's engine on the JVM (3); and in Android's page on the emulator (3: no header, pages not told, a property the page can write) |

## 8. What is not verified

- **The choices of §6** are the recommendations, used on the owner's word; the owner has not looked at them.
- **Whether any site heeds it.** No test loads anybody's site.
- **Most requests go without the header:** everything a page fetches, and every link followed inside a page. It is what the engines allow (§5.1), and the plan said so. A site that looks for the signal in its own scripts has it everywhere; one that looks only at the header sees it on the first page of a visit and not after.
- **A reload, going back, and a redirect:** whether the engine sends the header again on its own second asking for an address the shell began. Not tried.
- **A tab that wakes from its kept state** (051): the engine begins that load from the state, and the shell adds nothing to it. Not tried; expected to go without the header.
- **Workers:** a script in a worker has a `navigator` of its own, which a script added to pages does not reach. Not told, and not tried.
- **A page that looks for the property on `Navigator.prototype`** and not on `navigator` does not find it: it is the object's own. No page that does so is known.
- **A phone:** an emulator and a simulator are not one.

## 9. Done checklist

- [ ] The owner's word on §6.
- [x] Each engine tried (step 1).
- [x] The test site's page.
- [x] The contract and the core.
- [x] Each adapter, and its suite.
- [x] The gate: every stage of `Tools/test.sh all` green, and `android-device`, on 2026-11-28: `lint`, `core` (832, and the strings tool's 24), `unit` (414 on the Mac), `build`, the Mac's UI suite (68), `ios` (398 on the iPhone simulator), the iPhone's UI suite (57), `android` (543 and 34 on the JVM) and `android-device` (208 on the emulator).

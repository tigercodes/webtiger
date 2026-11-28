# 070 — Third-party cookies (W4)

| | |
|---|---|
| **Status** | **Built and tested on both engines, on the recommendations of §6.** The owner's word of 2026-11-24 was to use what is suggested where something waits on a choice, and to go on: they are built on, and stay the owner's to change. The owner's own word, earlier: "make sure cookies don't leak across sites in any system". A site that is in another site's page is not sent its cookies, and what it sets there is not kept: on WebKit by WebKit's own doing, on Android by the WebView's setting, said outright; shown by the same tests on each, and in each app. **One thing was found that is not as it should be, and is not mended here: on Android a third party's storage is not kept apart** (§5.1). A site in a frame of another site's page reads there what it stored as itself, so a site that means to follow a person can do with its storage what it can no longer do with a cookie. WebKit keeps the two apart |
| **Phase** | W4 of [000](000-master-plan.md) §9: "third-party cookies". 000 §5.9: "Blocked by default on every engine (WebKit already does; the others are set to)" |
| **Contract** | None (D2) |
| **Builds on** | The engine adapters, and the adapter suite that holds each to the same behaviour. The test site, which answers by two names (`127.0.0.1` and `localhost`): two sites, to a browser. Login sets, which keep one identity's cookies from another's, tested as a security property since 001 |
| **Last updated** | 2026-11-28 |

## 1. User story

I am signed in to a big site. Every other page I read has a piece of that site in it: a button, a picture, a frame. I do not want the big site to be told, with my name on it, each page I read.

## 2. What is in it, and what is not

**In:** a site that is in a page as a third party (a frame, a picture, a script of another site than the page's) is not sent the cookies it has, and what it sets there is not kept. On every engine; shown by the same tests on each.

**Not in, each with where it goes:**

- **A switch for it.** The plan has it under Guard's switches (000 §5.8: "Each level can set ads, trackers, cookie notices and third-party cookies"). WebKit gives a shell no way to turn its own blocking off, for a site or at all; the System WebView has a setting for each web view, which is not a setting for each page. A switch is a later step, on the engines that can heed one (§6, D1).
- **What a third party keeps by other means** (a page's storage in a frame): the engines' own doing, which this looks at and does not change (§3).
- **A third party that asks to be let in** (the Storage Access API), and the engines' own exceptions (a site a person has signed in through, in a window it opened): the engines' own, untouched and untried.
- **Cookies between login sets:** kept apart since 001, and tested there.

## 3. The rule

**Whose a request is.** The page a person is on is the first party: the site of the address in the address field. Anything in that page that comes from another site is a third party there. A site is what somebody registered, or the host where nothing is registered, as everywhere else in WebTiger.

**A third party is not sent its cookies.** A request to it from within another site's page carries none: a frame's page, a picture, a script, a fetch.

**What a third party sets is not kept.** A cookie in its answer, there, is dropped. A script of its own, in its frame, reads no cookie and can write none.

**The site itself loses nothing.** Visited as itself, in the address field, it has the cookies it had. And a frame of the page's own site has that site's cookies: this is about other sites only.

**On every engine, whatever the engine's own habit.** WebKit does it by itself; the adapter suite says so, so that a WebKit that stopped is noticed. The System WebView does it for an app built for a recent Android; the adapter sets it all the same, so that it does not hang on a default.

**A page's storage in a frame** (what a script keeps other than cookies): as each engine has it. The trial of §5 looks at whether a third party in a frame reads what it kept as itself, and the document says what was seen (§5.1: on WebKit it does not; on Android it does).

## 4. The contract

None. Each adapter holds its engine to the rule, as it holds it to running a page's scripts (D2).

## 5. In what order

1. **The test site.** **Built:** `/keep`, which keeps a name every way a site can (a cookie that says nothing of where it may go, one that says it may go anywhere, and the page's storage), and has a button, for a site a person has used; `/kept`, which says what it was asked with, what its script reads and can write, and what its storage has; `/holds-another-site`, which holds those by the site's other name in a frame and as a picture, or, told to, by its own name.
2. **Try each engine,** by its suite. **Done** (§5.1).
3. **Android's adapter:** the setting, said outright for every web view. **Built**, and the suite run with it broken.
4. **In each app,** with the real core. **Built.**
5. **Under Guard's switches,** where an engine can: a later step, with its own design (D1). Not built.

### 5.1 What the engines do

As tried, 2026-11-28: WebKit on macOS 27.2 and on the iOS 27 simulator; the System WebView 133 on an Android 16 emulator. A site that kept the name "ada" as itself, then met in another site's page:

| | WebKit | The System WebView |
|---|---|---|
| Its frame's page, asked for | With no cookie | With no cookie |
| Its picture, asked for | With no cookie | With no cookie |
| Its script in the frame reads its cookies | None | None |
| Its script in the frame writes a cookie, of either kind, and reads it back | Not there | Not there |
| What its frame's answer sets, when it is next visited as itself | Not there | Not there |
| What a picture's answer sets, when it is next visited as itself | Not there | Not there |
| Having been used as itself first (a click, a touch, in its own page) | The same: shut out | The same: shut out |
| Visited as itself afterwards | It has what it had | It has what it had |
| A frame and a picture of the page's own site | Have that site's cookies, of both kinds | Have that site's cookies, of both kinds |
| In a private window's login set | The same | Not tried apart (there is no login set in memory on Android) |
| **Its script in the frame reads the page's storage** | **Nothing: kept apart** | **What it stored as itself** |
| **What its script stores in the frame, when it is next visited as itself** | **Not there** | **There** |

**The two kinds of cookie were both needed.** With Android's setting turned the other way (third parties' cookies accepted), the cookie that says it may go anywhere (`SameSite=None; Secure`) went to the frame and the one that says nothing did not: Chromium keeps such a cookie to its own site whatever the setting. A suite with the plain cookie alone would pass on Android with the blocking off. WebKit sends the plain one anywhere it is let. Both engines took a cookie marked `Secure` from the loopback site though it is not https.

**Storage on Android.** Chromium keeps a third party's storage apart in Chrome, and has not turned that on in the WebView; the WebView offers an app no setting for it. What this means: on Android, a site that is in many sites' pages as a frame cannot be told a person by a cookie, and can by what its script keeps in the page's storage. It is said in the suite as it is, so that a WebView that begins to keep it apart fails a test and is noticed. What to do about it is in §8.

## 6. Decisions

**All of these are recommendations** (§ Status). The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | Blocked always, with no switch yet | It is what the owner asked for and what the plan's table says. WebKit has no public way to stop its own blocking, so a switch could only ever be for some engines; and the WebView's setting is a web view's, not a page's, so a switch for a site would lag a moment behind the page, in the direction that leaks. That wants a design of its own | Under Guard's switch at once, on Android alone |
| D2 | No contract: each adapter holds its engine to it | Nothing is decided at run time yet: it is always so. A field that says "block them" which one engine cannot be told "do not" by would be a contract that lies. When there is a switch it is the core's, and comes with its own message | The core says it with each web view, as it says Global Privacy Control (069 D3) |
| D3 | The suite tries both kinds of cookie | A test with one kind alone passes on one engine for another reason than the blocking | One kind: half a test |
| D4 | A page's storage in a frame is looked at, said, and left | The engines keep a third party's storage apart or refuse it by themselves; no shell has a setting for it. What is seen goes in this document. *As found:* WebKit does, and the System WebView does not (§5.1). The design did not foresee that; what to do about it is put to the owner (§8) | Not looked at |
| D5 | The adapter sets Android's setting though it is the default | A default is the WebView's to change, and an app's build can change it without anybody saying so | Trust the default, and the test |

## 7. Tests

| Where | What |
|---|---|
| The WebKit adapter, with real WebKit and the suite's pages (`WebKitThirdPartyCookieTests`, 6 on the Mac, 5 on the iPhone simulator) | §5.1, row by row. A site that kept a name, as itself, is asked with both cookies and its script reads them; in another site's page its frame's page and its picture are asked for with none, its script there reads none, can write none and finds nothing stored; as itself again it has what it had. What it sets in a frame's answer, and in a picture's, is not there when it is visited as itself, nor what its script stored. **The other way,** so that the tests are known to see a cookie where there is one: a frame and a picture of the page's own site have its cookies. In a private window's login set, the same. On the Mac, where the suite can click: a site a person has used is shut out all the same, in another web view and in the one it was used in |
| Android's adapter, with the real WebView on the emulator (`SystemWebViewThirdPartyCookieTest`, 5) | The same behaviours, the site a person has used among them (a touch), and but for the private window's login set. With the storage as the WebView has it (§5.1), said in the test |
| The apps, in fake mode, with the real core and the real engine (`ThirdPartyCookieTests`, 1, on the Mac and the iPhone simulator; `AppThirdPartyCookieTest`, 1, on the Android emulator) | A site keeps a name in the app's tab; a page of another site then holds it in a frame and as a picture: the frame says it has no cookie, and by the site's own record neither was asked for with one; as itself, the site has what it kept |
| Android's setting broken, on the emulator | **Third parties' cookies accepted: caught,** by four of the five tests (the fifth is the other way round, and should pass). **The setting not said at all: not caught, and cannot be:** shut out is the WebView's default for an app built as this one is, so the line changes nothing a test can see today. It is there for the day the default is otherwise (D5) |
| WebKit's blocking broken | Cannot be: nothing public turns it off (§8) |
| Every other test | The Swift package's on the Mac (414 before this) and Android's on the JVM (577) pass with it |

## 8. What is not verified, and what was found

- **On Android a third party's storage is not kept apart** (§5.1). Found, said, and not mended. What could be done, for the owner to choose (000 §11.1): **(a)** a script of the adapter's own, run before a page's in every frame, that gives a frame of another site than the page's no storage, or storage of its own that goes with the page. It would stop a site that uses storage in the plain way; it is not a wall, since a script that looks for a way round a script can find one (a worker has storage of its own that no such script reaches), and telling "another site" needs the core's knowledge of sites inside the page. **(b)** Leave it to Guard's lists, which keep the known followers' frames from loading at all, once there are lists. **(c)** Both. Recommended: (c), with (a) as a feature of its own, its limits said. Until then Android is weaker here than Apple's systems, and than Chrome on the same phone.
- **The choices of §6** are the recommendations, used on the owner's word; the owner has not looked at them.
- **On WebKit the tests cannot be shown to fail:** nothing public turns WebKit's blocking off, so there is no WebKit with third parties' cookies allowed to run them against. The other way round (a cookie seen where there should be one) stands in for it, and Android's broken setting shows the same pages can tell.
- **Two names of one loopback server** are two sites to a browser, and are not two registered domains: an engine that treats loopback addresses in a way of their own would not be caught treating real sites otherwise. No test fetches from anywhere but the test's own site.
- **Other ways a third party is fetched:** a script, a stylesheet, a fetch by the page's own script, a web socket. A frame's page and a picture were tried, and the engines do not go by kind; the others were not tried.
- **A third party that asks to be let in** (the Storage Access API), and what each engine does for a site reached through a window it opened or a redirect: not tried. WebKit asks the person in a dialog of its own for the first; no test may raise one.
- **Other storage than the page's own** (a database, a cache, a worker that serves pages): only `localStorage` was looked at. On Android they are expected to be as it is, shared.
- **A WebView older or newer than 133,** and one on a phone: an emulator and a simulator are not a phone, and the storage finding is one version's.
- **No switch:** a page that needs a third party's cookies to work (a sign-in in a frame, a shop's payment frame) has no remedy but the site's own. Not tried on a real site.

## 9. Done checklist

- [ ] The owner's word on §6.
- [x] The test site's pages.
- [x] Each engine tried, by its suite.
- [x] Android's setting, and the suite with it broken.
- [x] In each app.
- [ ] A third party's storage on Android (§8): the owner's word, and then its own feature.
- [x] The gate: every stage of `Tools/test.sh all` green, and `android-device`, on 2026-11-28: `lint`, `core` (832, and the strings tool's 24), `unit` (420 on the Mac), `build`, the Mac's UI suite (69), `ios` (403 on the iPhone simulator), the iPhone's UI suite (58), `android` (543 and 34 on the JVM) and `android-device` (214 on the emulator).

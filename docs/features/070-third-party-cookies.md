# 070 — Third-party cookies (W4)

| | |
|---|---|
| **Status** | **Designed, on the recommendations of §6; nothing built.** The owner's word of 2026-11-24 was to use what is suggested where something waits on a choice, and to go on: they are to be built on, and stay the owner's to change. The owner's own word, earlier: "make sure cookies don't leak across sites in any system" |
| **Phase** | W4 of [000](000-master-plan.md) §9: "third-party cookies". 000 §5.9: "Blocked by default on every engine (WebKit already does; the others are set to)" |
| **Contract** | None foreseen for the first steps (D2) |
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

**A page's storage in a frame** (what a script keeps other than cookies): as each engine has it. The trial of §5 looks at whether a third party in a frame reads what it kept as itself, and the document says what was seen.

## 4. The contract

None for the first steps. Each adapter holds its engine to the rule, as it holds it to running a page's scripts (D2).

## 5. In what order

1. **The test site:** a page that keeps a name every way a site can; one that says what it was asked with and what its script can read and write; one that holds those, by the site's other name, in a frame and as a picture.
2. **Try each engine,** by writing the adapter suite's tests and running them: they are the trial. Two kinds of cookie are needed, since the engines differ in what they would send across sites at all: a cookie that says nothing of where it may go (which WebKit sends anywhere, and Chromium keeps to its own site), and one that says it may go anywhere (`SameSite=None; Secure`, which is what a third party that means to follow people sets).
3. **Android's adapter:** the setting, said outright for every web view; and the suite with it broken, on the emulator, to show that the tests can tell.
4. **In each app,** with the real core: a page with a third party in it, seen.
5. **Under Guard's switches,** where an engine can: a later step, with its own design (D1).

## 6. Decisions

**All of these are recommendations** (§ Status). The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | Blocked always, with no switch yet | It is what the owner asked for and what the plan's table says. WebKit has no public way to stop its own blocking, so a switch could only ever be for some engines; and the WebView's setting is a web view's, not a page's, so a switch for a site would lag a moment behind the page, in the direction that leaks. That wants a design of its own | Under Guard's switch at once, on Android alone |
| D2 | No contract: each adapter holds its engine to it | Nothing is decided at run time yet: it is always so. A field that says "block them" which one engine cannot be told "do not" by would be a contract that lies. When there is a switch it is the core's, and comes with its own message | The core says it with each web view, as it says Global Privacy Control (069 D3) |
| D3 | The suite tries both kinds of cookie | A test with one kind alone passes on one engine for another reason than the blocking | One kind: half a test |
| D4 | A page's storage in a frame is looked at, said, and left | The engines keep a third party's storage apart or refuse it by themselves; no shell has a setting for it. What is seen goes in this document | Not looked at |
| D5 | The adapter sets Android's setting though it is the default | A default is the WebView's to change, and an app's build can change it without anybody saying so | Trust the default, and the test |

## 7. Tests

To be written with each step. Foreseen:

| Where | What |
|---|---|
| The WebKit adapter, with real WebKit and the suite's pages, on the Mac and the iPhone simulator | A site that has kept a name, then met as a third party in another site's page: its frame's page and its picture are asked for with no cookie; its script in the frame reads none and can write none. What it sets as a third party, in a frame's answer and in a picture's, is not there when it is visited as itself. **And the other way, so that the tests are known to see a cookie when there is one:** visited as itself it is asked with its cookies; a frame of the page's own site has them; and after being met as a third party it still has what it had |
| Android's adapter, with the real WebView on the emulator | The same behaviours |
| Android's setting broken (third parties' cookies accepted), on the emulator | The tests fail: they can tell |
| The apps, in fake mode, with the real core and the real engine | A page with a third party in it says what the third party was asked with: nothing |

## 8. What is not verified

To be filled in as it is built. Known now:

- **The choices of §6** are the recommendations, used on the owner's word; the owner has not looked at them.
- **On WebKit the tests cannot be shown to fail:** nothing public turns WebKit's blocking off, so there is no WebKit with third parties' cookies allowed to run them against. The other way round (a cookie seen where there should be one) is what stands in for it.
- **Two names of one loopback server** are two sites to a browser, and are not two registered domains: an engine that treats loopback addresses in a way of their own would not be caught treating real sites otherwise. No test fetches from anywhere but the test's own site.
- **A phone:** an emulator and a simulator are not one.

## 9. Done checklist

- [ ] The owner's word on §6.
- [ ] The test site's pages.
- [ ] Each engine tried, by its suite.
- [ ] Android's setting, and the suite with it broken.
- [ ] In each app.
- [ ] The gate.

# 067 — Guard's switches (W4)

| | |
|---|---|
| **Status** | **Built and tested, in the core, on the recommendations of §6.** The owner's word of 2026-11-24 was to use what is suggested where something waits on a choice, and to go on: they are built on, and stay the owner's to change. Guard has three switches, the nearest that says anything winning: everywhere's, a Space's, and a site's within a login set. The fast path heeds them from the next request, a site's switch reloads its tab, they are kept, and a window is told what Guard is for the page in front and which switch said so. **No shell shows or sets them yet:** the mark, its count and the panel are a shell's design, and are not designed here |
| **Phase** | W4 of [000](000-master-plan.md) §9: "the three levels of switch". 000 §5.8: "Three levels, the nearest wins: this site (within this Space's logins), this Space, everywhere." W4's exit: "a site's switch takes effect in one reload" |
| **Contract** | 1.55: `SetGuard`; `TabView.guard`. The store is at schema 18: a site's switch has a table |
| **Builds on** | [064](064-guards-request-engine.md): the blocker and the fast path. [066](066-a-persons-own-rules.md): something put in force at a stroke for every thread that asks. [012](012-zoom.md): a thing remembered for a site, in a login set. [024](024-several-spaces.md): Spaces, and logins shared between them |
| **Last updated** | 2026-11-25 |

## 1. User story

A site I need is broken, and I suspect my browser's blocker. I want to turn the blocker off for that site, there and then, see whether it mends, and have my choice remembered. And I want my Work Space, where everything is my employer's own, not to be blocked at all.

## 2. What is in it, and what is not

**In:** three switches for Guard: everywhere; this Space; this site, within this Space's logins. The nearest that says anything wins. The fast path heeds them at once. A site's switch, set for the page a tab shows, reloads that tab. A window is told whether Guard is on for the page in front, and which switch said so.

**Not in, each with where it goes:**

- **The mark, its count and the panel** (000 §5.8): a shell's design.
- **The plan's four things a switch can set** (ads, trackers, cookie notices, third-party cookies): they are kinds of list, and there are no lists ([064](064-guards-request-engine.md) D6). One switch at each level, for Guard as a whole, until then.
- **WebKit:** nothing there asks the fast path. A site's switch on WebKit is "a small allow-list rule list" (000 §5.8), with the WebKit compiler.
- **A count of what was blocked:** with the mark.

## 3. The rule

**Three switches.** Everywhere: on or off; on until a person says otherwise. A Space's: on, off, or not said. A site's, in a login set: on, off, or not said.

**The nearest that says wins:** the site's; if it says nothing, the Space's; if that says nothing, everywhere's.

**Whose switches a request is under.** The page that asks is the top frame's (064). Its site is what somebody registered, or the host where nothing is registered (012). The web view that shows it is a tab's, in a Space, with that Space's logins. So: the site's switch is the one kept for that site in those logins, which Spaces that share logins share (as the plan says, and as a site's zoom is); the Space's switch is that Space's. A frame of another site within the page is under the page's switches.

**Off means nothing is asked of the blocker:** every request of such a page is allowed, at once.

**A page that is of no site** (an empty tab, a page made by a script) has no site's switch: the Space's, then everywhere's.

**A web view the fast path does not know** (one that has just gone) is under everywhere's switch.

**In force from the next request,** on every thread that asks, as rules are (066). And a site's switch set for a tab reloads that tab, so that what the person sees is the page under their choice: one click, and a reload. A Space's switch and everywhere's reload nothing: they are for many pages, and reloading those is a person's to do.

**Kept:** everywhere's and each Space's with the app's settings; a site's with the login set, as its zoom is, in a table of its own. ("Forget this site", 000 §5.9, is not built: when it is, a site's switch goes with the rest.)

**A private window** has a login set of its own, kept in memory (037), and no Space: it is under everywhere's switch, and a site's switch set in it holds for the window's life and is kept nowhere.

## 4. The contract

1.55.

- **`SetGuard { of, switch }`:** `of` is `everywhere`, a `space_id`, or a `tab_id` (the site of the page that tab shows, within its logins); `switch` is on, off, or nothing said. Answered with `Ack`. Refused, changing nothing: `guard.where` when none of the three is named; `guard.switch` when everywhere's switch is to say nothing, or the value is one this core does not know; `space.unknown`; `tab.unknown`; `guard.no_site` for a tab that shows no page of a site.
- **`TabView.guard`:** `on`, and `said_by` (everywhere, the Space, the site). In what a window is told of the tab in front, always.

**As built, where the design did not say:**

- **A tab is reloaded only when its site's switch changed what Guard is for its page.** A site turned off where the Space is off already changes nothing for the page, and reloads nothing.
- **Only the tab the switch was set from** is reloaded: another tab of the same site is under the new switch from its next request, and is as it was until it is reloaded.
- **Whoever answers the fast path is told before anything else happens.** A switch that is set is in force before the tab's reload is asked of the shell, and before the request is answered. A web view that is to be made is known there, with its Space and its logins, before the shell is asked to make it: its first request is under its own switches.
- **A web view a page opened by itself** (006) is made by the engine before the core hears of it: it is known to the fast path's side when the windows are next told, and under everywhere's switch for the moment before.
- **A tab's site, for its switch and for what its window is told,** is the site of the page that has arrived in it. A tab that is on its way to its first page has none yet. The fast path does not wait for that: it goes by the address the engine says the page has.
- **The refusals have no words yet:** no screen shows them.

## 5. In what order

1. **The store:** a site's switch, a Space's, everywhere's. **Built:** schema 18.
2. **The core:** the switches, the request, what a window is told, the reload. **Built.**
3. **The fast path:** the switches, and which web view is whose, put where every thread reads them, at a stroke when either changes. **Built.**
4. **The shells:** nothing, until the mark and the panel are designed. There is no new kind of answer for Android's client to know (003).

## 6. Decisions

**All of these are recommendations** (§ Status). The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | One switch at each level, for Guard as a whole | There is one kind of rule until there are lists. A switch for each of the plan's four kinds is the lists' feature, and is added beside this one | The four now, three of them switching nothing |
| D2 | "Not said" at a site and at a Space, and no such thing everywhere | The nearest that says wins only if a level can say nothing. Turning a site's switch back to "as the Space has it" is not the same as turning it on | On and off alone: a site that was ever switched is pinned for ever |
| D3 | A site's switch is kept for the login set, and Spaces that share logins share it | 000 §5.8 says "within this Space's logins"; it is how a site's zoom and its permissions are kept; and what a site may do to a person is a matter of who they are there | For the Space: two Spaces with the same logins could differ |
| D4 | A site's switch set for a tab reloads that tab; the other two reload nothing | W4's exit is "a site's switch takes effect in one reload", and the person is looking at the page | Nothing reloaded, and the person told to reload. Or every tab of the site reloaded |
| D5 | A frame of another site is under the page's switches | The person turned Guard off "here", and here is the page they see. It is also what the fast path knows: the page that asks is the top frame's | The frame's own site's switch: not known to the fast path |
| D6 | A private window: everywhere's switch, and a site's for the window's life | It has no Space, and keeps nothing (037) | A site's switch refused in a private window |
| D7 | A window is told Guard's state for the tab in front in what it is already told of that tab | A mark in the address field is drawn from it, with no question asked | A request that asks |

## 7. Tests

| Where | What |
|---|---|
| The rule (`wt-core`, 3) | **The nearest that says wins:** everywhere on or off, the Space saying nothing, on or off, the site the same: all eighteen, by the page and by the fast path's question. **A site's switch is its logins', and a Space's its own:** another site in the same logins, and the same site in other logins, are not under it; another Space with the same logins shares the site's switch and not the Space's; a page of no site has no site's switch. **The fast path's question:** the site is what somebody registered, whatever the host, the port and the path; another site, a host that only ends alike, and a page that is no address or of no site, are under the next switch; a web view that is not known is under everywhere's |
| The store (`wt-store`, +2) | A site's switch written, listed with every other in every login set, replaced in place, and taken away by saying nothing, which leaves nothing written; another login set's is its own; a login set the store does not have can have none; a site's zoom is not touched. A store from before the switches gains them and keeps what it had |
| The core, through the contract (`guard_switches`, 8) | **On until a switch says otherwise, and the nearest wins:** each switch set in turn, and what the window is told of the page in front at once, with which switch said so; a site and a Space saying nothing again; another site in the same tab, and another host of the same site. **Which tab is reloaded:** a site's switch that changes what Guard is for the page reloads that tab, once; said again, or saying what the level above says already, it reloads nothing; a Space's and everywhere's reload nothing whatever they change; another tab of the same site is not reloaded by it. **What is refused,** each by its code and the field it names, with nothing changed, nothing reloaded and nothing told. **Spaces that share logins share a site's switch and not a Space's;** a Space with logins of its own shares neither. **Kept across a start:** all three, told to whoever answers the fast path before anything is asked, and to the window of its page; said to say nothing, or on, nothing of them is left at the next start. **A web view is told before the shell is asked for it,** with its Space and its logins; one that has gone is told when the windows are. **A site's switch is told before its tab's reload is asked,** once. **A private window** is under everywhere's switch and not the Space's of another window; it shows no Space; a site's switch set in it holds there and is not the same site's in an ordinary window, nor the other way |
| The fast path, through the C functions (`wt-ffi`, `core`, +3) | A core with a suite's rules and a page in a web view: refused with Guard on; off everywhere, the very next request allowed, in this web view and one that is not known; off for the Space, allowed whatever page asks, and a web view that is not known still refused; on for the site in a Space that is off, that site's pages under Guard, by any of its hosts, and another site's in the same web view not; each change from the next request, and a site's switch asking a reload each time. **After a start with nothing asked.** **Eight threads asking while everywhere's switch is turned three hundred times:** every answer is "ok" with one verdict or the other, both were seen, and the last switch set is the one in force |
| Android, on the JVM, through the core's library (`NativeCoreTest`, +1) | Off everywhere, the very next request is allowed through the function Android's adapter calls; on again, refused |
| The switches broken, a part at a time | 23 parts, 23 caught. The first run caught 21. One of the two was a break that did not build. The other: with a switch that is set not told to the fast path's side until the windows are told, every test passed, since the windows are told a moment later; nothing checked that the switch is in force before the reload is asked of the shell, which is the one moment it matters. A test does now |
| Every other test of the core | 809 before this, and all pass with it |

## 8. What is not verified

- **The choices of §6** are the recommendations, used on the owner's word; the owner has not looked at them.
- **In an app.** No shell sets a switch or shows what Guard is: nothing here was seen in an app. On Android the fast path is the adapter's, and heeds the switches through the core's own library (tested on the JVM); with the real WebView it was not tried. On Apple's systems nothing asks the fast path (064).
- **A reload that mends a page:** that a page which was broken by a rule works after its site's switch is turned off and the tab reloads is what all of this is for, and needs a real engine and a rule that breaks a page. Not tried.
- **A web view a page opened by itself,** in the moment before the core hears of it (§4): under everywhere's switch, whatever its Space's and its site's say. Written, and not tested.
- **A store that refuses:** the switch holds for the run, and what was there before comes back at the next start. Written, and not tried.
- **Another tab of the same site,** which is as it was until it is reloaded (§4): whether a person expects that was not tried with anyone.
- **What telling the switches at every telling of the windows costs:** the switches and the web views are copied and compared each time. They are few (a web view for each live tab), and the plan's time for a row of two hundred told (060) is held by its measurement; it was not measured apart.

## 9. Done checklist

- [ ] The owner's word on §6.
- [x] The store.
- [x] The core.
- [x] The fast path.
- [x] The gate: every stage of `Tools/test.sh all` green, and `android-device`, on 2026-11-25: `lint`, `core` (825, and the strings tool's 24), `unit` (407 on the Mac), `build`, the Mac's UI suite (67), `ios` (391 on the iPhone simulator), the iPhone's UI suite (56), `android` (542 and 34 on the JVM) and `android-device` (201 on the emulator).

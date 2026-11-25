# 067 — Guard's switches (W4)

| | |
|---|---|
| **Status** | **Designed, on the recommendations of §6, for the core alone.** The owner's word of 2026-11-24 was to use what is suggested where something waits on a choice, and to go on: they are to be built on, and stay the owner's to change. Not built. The mark in the address field, its count and its panel are a shell's design, and are not designed here |
| **Phase** | W4 of [000](000-master-plan.md) §9: "the three levels of switch". 000 §5.8: "Three levels, the nearest wins: this site (within this Space's logins), this Space, everywhere." W4's exit: "a site's switch takes effect in one reload" |
| **Contract** | To come: a request that sets a switch; and what a window is told of Guard for the page in front |
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

**Kept:** everywhere's and each Space's with the app's settings and the Space; a site's with the login set, as its zoom is. Forgetting a site ("Forget this site") forgets its switch with the rest.

**A private window** has a login set of its own, kept in memory (037), and no Space: it is under everywhere's switch, and a site's switch set in it holds for the window's life and is kept nowhere.

## 4. The contract

The next minor.

- **Set a switch:** everywhere (on or off); a Space's, by the Space (on, off, or nothing said); a site's, by a tab that shows a page of it (on, off, or nothing said). Refused when the tab shows no page of a site.
- **What a window is told of the tab in front:** whether Guard is on for its page, and which of the three said so.

## 5. In what order

1. **The store:** a site's switch, a Space's, everywhere's.
2. **The core:** the switches, the request, what a window is told, the reload.
3. **The fast path:** the switches, and which web view is whose, put where every thread reads them, at a stroke when either changes.
4. **The shells:** nothing, until the mark and the panel are designed. Android's client must know anything new by its name (003).

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

To be filled in as built. Planned:

| Where | What |
|---|---|
| The core, through the contract | Each switch set and what a window is then told, for each of the three saying and not saying; Spaces that share logins share a site's switch and not a Space's; kept across a start; a site's switch reloads its tab and the others reload nothing; a tab with no site refused; a private window |
| The fast path, through the C functions | A request refused with Guard on and allowed with it off, at each level, with the nearest winning; from the next request; eight threads asking while a switch is turned |
| The core broken, a part at a time | Each level, and each "not said", left out |

## 8. What is not verified

To be filled in as built.

- **The choices of §6** are the recommendations; the owner has not looked at them.

## 9. Done checklist

- [ ] The owner's word on §6.
- [ ] The store.
- [ ] The core.
- [ ] The fast path.
- [ ] The gate.

# 068 — Link cleaning (W4)

| | |
|---|---|
| **Status** | **Designed, on the recommendations of §6.** The owner's word of 2026-11-24 was to use what is suggested where something waits on a choice, and to go on: they are to be built on, and stay the owner's to change. Not built |
| **Phase** | W4 of [000](000-master-plan.md) §9: "link cleaning". 000 §5.9: "Tracking parameters are removed from navigations in the core; 'Copy clean link'" |
| **Contract** | To come, with the steps that need it (§5) |
| **Builds on** | [031](031-links-from-other-apps.md): an address handed in from another app. [018](018-link-in-new-tab.md): a link of a page opened in a tab. `wt-core`: addresses, and which site one is of. [067](067-guards-switches.md): Guard's switches, which this is under |
| **Last updated** | 2026-11-25 |

## 1. User story

Somebody sends me a link. After the address there is a tail of letters that is there so that the site, and whoever sent me there, can tell it was me who clicked. I want the page, and not the tail.

## 2. What is in it, and what is not

**In:** the parts of an address that exist to follow a click from one site to another are taken out before the address is loaded, where the core is the one that loads it; and, in a later step, where a page sends the tab to another site. A link can be copied clean.

**Not in, each with where it goes:**

- **Rules from the public lists** that take parts out of addresses: with the lists ([064](064-guards-request-engine.md) D6). Here the core has a short list of its own.
- **Addresses within one site.** What a site puts in its own addresses for its own pages is its business, and may be needed.
- **What a page fetches** (pictures, scripts): Guard's rules, not this.
- **Shortened links and redirectors** ("unwrapping" a link that goes through a counting server): another thing, with other risks.

## 3. The rule

**What is taken out.** The parts of an address's query (after the `?`) whose names are on the core's list: names that the big advertising and mailing systems add to the links they hand out, which carry who clicked and from what campaign, and which no page needs to show itself. The list is short and of names everybody knows (D1). Nothing else of the address is touched: its other parts stay, in their order, as they were written; a query left empty goes with its `?`.

**When.** Only when the address is arrived at from elsewhere:

- from outside the browser: a link pressed in another app (031);
- from another site: a link of a page opened in a new tab (018) that leads to another site than the page's;
- (a later step) from another site in the same tab: a link pressed in a page that leads to another site.

Never within one site; and never an address a person typed or pasted, which is theirs as they gave it (D3).

**Under Guard's switches** (067): where Guard is off for the page that has the link (or, from outside, off everywhere), nothing is cleaned. Cleaning is one of Guard's protections, and "turn it off here" turns it off.

**What a person sees.** The address they arrive at is the clean one, in the address field and in history. Nothing says that something was taken out.

**Copy clean link.** Where a link's address can be copied (018's menu), it can be copied clean: the same rule, whatever site the link leads to, since what is copied is for somewhere else.

## 4. The contract

To come with each step. Foreseen: nothing for the first two (the core cleans what it loads); for a page's own navigations, an answer of the navigation check that says "go here instead"; for copying, the clean address beside the link's own in what a shell is told of a link.

## 5. In what order

1. **The rule,** in `wt-core`: the list, and an address cleaned.
2. **What the core loads by itself:** an address from another app; a link opened in a new tab that leads to another site.
3. **Copy clean link:** the clean address told to a shell with a link; the menus.
4. **A page's own navigation to another site:** the navigation check, and each engine's adapter, which must stop the navigation and begin the clean one.

## 6. Decisions

**All of these are recommendations** (§ Status). The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | A short list of the core's own, of the names the big systems use | The names are facts about those systems, known to everybody; a short list can be read and reasoned about; and it needs nobody's list (064 D6) | Other browsers' lists, or the public lists' rules for it: longer, and somebody else's work to take in |
| D2 | Only across sites, and from outside | A site's own addresses are its own; and the harm is in being followed from one site to another | Everywhere: a site that uses the same names for its own pages breaks |
| D3 | Never what a person typed or pasted | The address field never lies (the project's rule), and what a person gives is what they asked for. A pasted link can be copied clean first | Cleaned as well: most pasted links are other people's |
| D4 | Under Guard's switches | It is one of Guard's protections; a page broken by it is mended by the same switch | A switch of its own |
| D5 | Nothing says that an address was cleaned | It is the ordinary case, and a mark for it is noise. Guard's panel, when there is one, can count them | A mark in the address field |
| D6 | A page's own navigation is the last step | It needs each engine's adapter to stop a navigation and begin another, which is where pages break; the first steps need no engine | First: it is where most such links are pressed |

## 7. Tests

To be filled in as built. Planned:

| Where | What |
|---|---|
| The rule (`wt-core`) | Each name on the list taken out, whatever its case and wherever it is in the query; the rest left as written, in order; a query left empty gone; a fragment kept; an address with none of them untouched, to the byte; a name that only begins like one left |
| The core, through the contract | From another app: cleaned, shown and kept in history clean. A link in a new tab: cleaned when it leads to another site, and not within one. Typed: never. With Guard off: never |

## 8. What is not verified

To be filled in as built.

- **The choices of §6** are the recommendations; the owner has not looked at them.

## 9. Done checklist

- [ ] The owner's word on §6.
- [ ] The rule.
- [ ] What the core loads by itself.
- [ ] Copy clean link.
- [ ] A page's own navigation.
- [ ] The gate.

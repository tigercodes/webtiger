# 068 — Link cleaning (W4)

| | |
|---|---|
| **Status** | **Its first two steps are built and tested, in the core, on the recommendations of §6:** the rule, and its use where the core itself loads an address (a link from another app; a link of a page opened in a tab of its own, when it leads to another site). The owner's word of 2026-11-24 was to use what is suggested where something waits on a choice, and to go on: they are built on, and stay the owner's to change. **Not built: a link pressed in a page that takes the same tab to another site, which is where most such links are pressed; and copying a link clean, which is designed (§3, §4, D7 to D10) and is the next step.** Both need the shells |
| **Phase** | W4 of [000](000-master-plan.md) §9: "link cleaning". 000 §5.9: "Tracking parameters are removed from navigations in the core; 'Copy clean link'" |
| **Contract** | None for the steps that are built. Designed for copying a link clean: 1.58, `CleanLink` (§4). To come with the last step (§5) |
| **Builds on** | [031](031-links-from-other-apps.md): an address handed in from another app. [018](018-link-in-new-tab.md): a link of a page opened in a tab. `wt-core`: addresses, and which site one is of. [067](067-guards-switches.md): Guard's switches, which this is under |
| **Last updated** | 2026-11-28 |

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

**The list, as built** (a name is one of these whatever its case):

| Whose | Names |
|---|---|
| A family many systems use | every name that begins `utm_` |
| Google's | `gclid`, `gclsrc`, `dclid`, `gbraid`, `wbraid`, `_gl` |
| Meta's | `fbclid`, `igshid` |
| Microsoft's, X's, TikTok's, LinkedIn's, Yandex's | `msclkid`, `twclid`, `ttclid`, `li_fat_id`, `yclid` |
| The mailing systems' (Mailchimp's, HubSpot's, Marketo's) | `mc_eid`, `mc_cid`, `_hsenc`, `_hsmi`, `mkt_tok` |

**When.** Only when the address is arrived at from elsewhere:

- from outside the browser: a link pressed in another app (031);
- from another site: a link of a page opened in a new tab (018) that leads to another site than the page's;
- (a later step) from another site in the same tab: a link pressed in a page that leads to another site.

Never within one site; and never an address a person typed or pasted, which is theirs as they gave it (D3).

**Under Guard's switches** (067): where Guard is off for the page that has the link (or, from outside, off everywhere), nothing is cleaned. Cleaning is one of Guard's protections, and "turn it off here" turns it off.

**As built, where the design did not say:**

- **An address from another app** comes from no page, so no site's switch is its: it is under the switch of the Space the window shows, and everywhere's. The site of the page that happens to be in front has no say.
- **A link of a page** is under that page's switches: its site's, its Space's, everywhere's. A page of no site (an empty tab cannot have a link; a page made by a script can) leads to another site wherever it leads.
- **Fake mode's list of addresses** (063) is opened as addresses from another app are, and so is cleaned as they are.

**What a person sees.** The address they arrive at is the clean one, in the address field and in history. Nothing says that something was taken out.

**Copy clean link** (designed, not built). Where a link's address can be copied in a menu WebTiger draws or adds to (018, 022), it can be copied clean, by an item of its own under "Copy Link": "Copy Clean Link". What is copied is the link's address without the parts of the list, and otherwise to the letter.

- **Whatever site the link leads to,** the page's own among them: what is copied is for somewhere else.
- **Whatever Guard's switches say:** it is asked for by name (D9).
- **Always offered** where "Copy Link" is, whether or not the link has anything to take out (D8). A link with nothing of the kind is copied as it is.
- **The core cleans, the shell copies.** The shell asks the core for the clean address when the item is chosen, and puts the answer where "Copy Link" puts the link (018: the system's clipboard; in fake mode and in every test, one of the app's own).
- **An address that is none of the web's** (a link that writes a letter) has nothing the list names: it is copied as it is.

## 4. The contract

Nothing for the first two steps (the core cleans what it loads).

**For copying a link clean (designed): 1.58.**

- **`CleanLink { url }`:** a link's address, as the page has it. Answered with **`CleanedLink { url }`**: the address without the parts the core's list names, or as it was given when there is nothing of the kind or it is no address of the web. Never refused. Both addresses are a person's: neither is logged.

(The design first foresaw the clean address told to a shell beside the link's own. A shell is not told of links by the core: it finds them in the page. So it asks.)

**For a page's own navigations:** to come with that step. Foreseen: an answer of the navigation check that says "go here instead".

## 5. In what order

1. **The rule,** in `wt-core`: the list, and an address cleaned. **Built.**
2. **What the core loads by itself:** an address from another app; a link opened in a new tab that leads to another site. **Built.**
3. **Copy clean link:** the request; then each shell's menu: Android's and the iPhone's, which are the shell's own, and the Mac's, where the item stands under WebKit's "Copy Link". Designed; not built.
4. **A page's own navigation to another site:** the navigation check, and each engine's adapter, which must stop the navigation and begin the clean one. Not built.

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
| D7 | Copying clean is a request to the core, made when the item is chosen | The rule and its list are the core's, and are not to be written again in a shell; and a shell finds a link in the page, where the core does not see it | The list in each shell: three copies to keep alike. The fast path, with the answer written where the caller says: a C function that hands back text, for a thing nobody waits on |
| D8 | Always offered where "Copy Link" is | The Mac's menu is built at once, by WebKit, and cannot wait for the core to say whether a link has a tail; a menu that is the same on every system is the one to have; and a person who wants a link without its tail need not know whether it has one | Only where there is something to take out: the menu waits on a question, and on the Mac cannot |
| D9 | Not under Guard's switches | It is asked for by name. An item called "Copy Clean Link" that copies the tail where Guard is off would lie | Under the switch, like the cleaning nobody asked for |
| D10 | An item of its own, under "Copy Link", which stays | "Copy Link" is what a person knows, and now and then the tail is wanted (a link that signs somebody in) | "Copy Link" copies clean, and the tail cannot be had |

## 7. Tests

| Where | What |
|---|---|
| The rule (`wt-core`, +2) | **Taken out, and nothing else:** each name of the list alone, with the query then gone with its `?` and the fragment kept; the `utm_` family, whatever follows; any of them whatever its case; among other parts, the others staying in their order exactly as they were written (an encoded space, a plus, a part with no value, an empty value, a name twice); one with no value; one whose value has an `=` in it. **An address with nothing of the kind is as it is:** no query, an empty one, a name that only begins or ends like one or is the family's without its mark, the names in the path, in the fragment or as a value, and an address that is none of the web's |
| The core, through the contract (`link_cleaning`, 4) | **From another app:** what is loaded, what the address field shows and what history keeps are the address without what followed the click, the rest as written; an address with nothing of the kind is loaded to the letter. **A link opened in a tab:** cleaned when it leads to another site; within the site, by another of its hosts, as the site wrote it. **What a person typed** is loaded and shown as they gave it. **Where Guard is off:** off everywhere, neither is cleaned; off for the Space the window shows, an address from another app is not; off for the site of the page that has the link, its links are not, and an address from another app still is; on again, cleaned |
| Link cleaning broken, a part at a time | 15 parts, 15 caught, at the first run |
| Copying a link clean (to be written with it) | The core, through the contract: the clean address for a link with a tail, whatever site it leads to and whatever Guard's switches say; a link with nothing of the kind, and one that is none of the web's, as given. Each shell's menu, against `ScriptedCore` and in its pictures: the item is there under "Copy Link". With the real engines and the real core: a link with a tail, its menu, the item chosen, and the clean address in the app's own clipboard |
| Every other test of the core | 825 before this, and all pass with it |

## 8. What is not verified

- **The choices of §6** are the recommendations, used on the owner's word; the owner has not looked at them.
- **Most links are not cleaned yet.** A link pressed in a page, which takes the same tab to another site, is the engine's navigation: the core does not load it, and it is as the page wrote it (§5, step 4). What is built cleans a link from another app, and one opened in a tab of its own.
- **The list** is of names that are widely known for what they are. It was written from that, and not checked name by name against each system's own documents here; a name that is wrong takes out something harmless or misses something. It has not been compared with any other browser's list.
- **A page that needs one of these to work.** A site that reads `gclid` on its own page to credit a sale will not see it: that is the point, and for some people's own work it is a loss. Guard's switch for the page that has the link is the way round it. Not tried on a real site.
- **In an app:** not seen. Nothing of a shell changed; the apps' own tests of links from other apps (031) and of links opened in tabs (018) run with it, with addresses that have nothing to take out.
- **Copy clean link** (step 3) is not built.

## 9. Done checklist

- [ ] The owner's word on §6.
- [x] The rule.
- [x] What the core loads by itself.
- [ ] Copy clean link.
- [ ] A page's own navigation.
- [x] The gate over the first two steps: every stage of `Tools/test.sh all` green, and `android-device`, on 2026-11-25: `lint`, `core` (831, and the strings tool's 24), `unit` (407 on the Mac), `build`, the Mac's UI suite (67), `ios` (391 on the iPhone simulator), the iPhone's UI suite (56), `android` (542 and 34 on the JVM) and `android-device` (201 on the emulator).

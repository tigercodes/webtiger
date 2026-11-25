# 069 — Global Privacy Control (W4)

| | |
|---|---|
| **Status** | **Designed, on the recommendations of §6.** The owner's word of 2026-11-24 was to use what is suggested where something waits on a choice, and to go on: they are to be built on, and stay the owner's to change. Not built. What each engine lets a shell do is to be tried first (§5, step 1): nothing here has been tried |
| **Phase** | W4 of [000](000-master-plan.md) §9: "Global Privacy Control". 000 §5.9: "Sent where the engine lets a header be added; exposed to scripts everywhere" |
| **Contract** | To come: what a shell is told to say for the web views it makes |
| **Builds on** | The engine adapters, and the adapter suite that holds each to the same behaviour. The test site. [067](067-guards-switches.md): a thing a person may turn off |
| **Last updated** | 2026-11-25 |

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

To come. Foreseen: in what a shell is told when it is asked to make a web view, whether pages there are told the signal; and with a load the shell is asked to begin, the same for its header.

## 5. In what order

1. **Try each engine:** a script that runs before a page's own, in every frame, and defines the property so that a page cannot change it; a header added to a load the shell begins. On WebKit and on the System WebView. Before the contract is written to what is only expected.
2. **The test site:** a page that says, in its title, what its script read and what header its own request came with. (The site's servers, one in each suite, must learn to say a request's header.)
3. **The contract, and the core.**
4. **Each adapter,** and the adapter suite on each: the same test.

## 6. Decisions

**All of these are recommendations** (§ Status). The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | On, for everybody, from the first run | 000 §5.9 lists it among the protections there are, not among choices. It asks a site for less, never for more | Off until asked for: nobody asks |
| D2 | The header only where a shell can add it, and the property everywhere | It is what the engines allow. A site that reads the property in its page has the signal whatever the header | Nothing sent unless both can be: the property alone is worth having |
| D3 | The core says whether, for each web view | A switch later is then the core's alone | Each shell always sends it |
| D4 | Not under Guard's switches | Guard's switch means "this page is broken, stop blocking". The signal blocks nothing and breaks nothing | Off where Guard is off |

## 7. Tests

To be filled in as built. Planned:

| Where | What |
|---|---|
| The adapter suite, on each engine | The test site's page: its script reads `true`, in the page and in a frame of another origin; its own request came with the header when the shell began it; a page cannot set the property to anything else |
| The core | Every web view it asks for is to be told the signal, a private window's too |

## 8. What is not verified

To be filled in as built.

- **The choices of §6** are the recommendations; the owner has not looked at them.
- **What the engines allow** is what their documents say: step 1 is to try it.

## 9. Done checklist

- [ ] The owner's word on §6.
- [ ] Each engine tried (step 1).
- [ ] The test site's page.
- [ ] The contract and the core.
- [ ] Each adapter, and its suite.
- [ ] The gate.

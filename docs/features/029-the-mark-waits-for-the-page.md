# 029 — The mark beside the address waits for the page (W1)

| | |
|---|---|
| **Status** | Built and tested in the core. The shells draw what the core says and needed no change; their catalogues' picture of a load on its way was made again on the Mac, the iPhone and Android |
| **Phase** | W1 of [000](000-master-plan.md) §9: the address field. A matter [026](026-past-a-certificate.md) §9 left for the owner |
| **Contract** | No new message. What `AddressView.security` says while a load is on its way changes (1.21) |
| **Builds on** | [001](001-foundations.md): the address field never lies. 026: a load that failed is not shown as secure |
| **Last updated** | 2026-11-20 |

> **The owner's word (2026-11-20): "use suggestions".** 026 §9 put the question: an address over `https` that a person typed has the padlock beside it before any connection is made; should the mark wait for the page? The suggestion was that it should. This is that.

## 1. User story

I type an address and press Return. While the page is on its way, nothing beside the address tells me the connection is secure: nothing has been checked yet. When the page arrives, the mark says what the connection was. If it never arrives, there was never a padlock to take back.

## 2. The rule

**The mark speaks of a page that has arrived.** `AddressView.security` says "secure" or "not secure" of an address only once a page of that address owns the tab (it has committed). Until then, and when the load fails, it says nothing (`SECURITY_UNSPECIFIED`).

What that means in each case:

- **An address a person typed** (or chose from history, or a search): the field shows where they sent the tab, as before (001), and **no mark** until the page commits. Then the mark is the page's.
- **A page that is reloaded, or that goes somewhere by a link**: the field still shows the page that is there, and its mark, until the new page commits. Nothing changes.
- **A load that failed**: no mark (026).
- **A site whose certificate a person went on past** (026): "not secure", from the moment the tab is sent there. That is not a claim that waits on anything: it is what the person was warned of.
- **The browser's own pages and local content** (`about:`, `file:`): "local", when they have arrived; nothing before.
- **A tab that is asleep** ([009](009-session-restore.md)) remembers an address and has no page: no mark, until it wakes and its page arrives.

**Why.** The padlock is a claim: this connection is to who the address says, and nobody is listening. Before the connection is made there is no connection to make it of. Shown early, it is a promise WebTiger may have to take back a moment later, and for a certificate that is refused it was shown beside the very address that failed.

**Not changed:** what the field shows (001's rule about addresses); the shells, which draw whatever mark the core says and none for "unspecified"; a typed-in field, whose mark the shells already hide ("a padlock must not vouch for what a person typed").

## 3. Tests

| Where | What |
|---|---|
| Core (`wt-app/tests/address_mark.rs`, 4, through `FakeShell`, with the shell held back so that a load on its way can be seen) | An address a person typed has no mark while it is on its way, over `https` and over `http`, and the page's mark once it has arrived; the field says where the tab was sent throughout. A page that is reloaded keeps its mark; one whose link leads elsewhere keeps its address and its mark until the new page arrives, and then has the new one's. A load that is stopped before anything arrives gives the field and the mark back to the page that is there. A certificate that is refused has no mark on the way and none when it has failed; gone on past, the site is "not secure" from the moment the tab is sent there, and so is another page of it typed afterwards. Four parts of the rule checked by breaking them: all caught |
| The shells | No code changed: each draws the mark the core says, and none for "unspecified" (tested since 001). Their catalogues' state "loading" now says what a core says of it, and its pictures were made again, light and dark, on the Mac, the iPhone and Android: each looked at, and none has a padlock |

## 4. What is not verified

- **In an app, by a test.** A load on its way lasts a moment: no app test holds one still to look at the mark. The rule is the core's, and is tested there; the shells' part is the picture.
- **What a person makes of it.** Other browsers differ in what they show before a page arrives; nobody was asked which reads best.

## 5. Done checklist

- [x] The core's rule, and its tests (4).
- [x] Both shells' catalogues say what a core now says of a load on its way; pictures recorded again and looked at.
- [x] `Tools/test.sh all` green, stage by stage, and `android-device` (144 on the emulator), on 2026-11-21. The Mac's UI suite, which had not run when this was built (a dialog of the system's was on the Mac's screen), ran with it.

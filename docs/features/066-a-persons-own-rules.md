# 066 — A person's own rules for Guard (W4)

| | |
|---|---|
| **Status** | **Built and tested, in the core, on the recommendations of §6.** The owner's word of 2026-11-24 was to use what is suggested where something waits on a choice, and to go on: they are built on, and stay the owner's to change. The core keeps a person's own rules as the text they wrote, says them when asked, and puts them in force the moment they are set: a new blocker in the old one's place, on every thread that asks, with nothing reloaded; and after a start. **No shell has a place to write rules yet,** so nobody can set any outside a test: where a person writes them is a shell's design, and is not designed here |
| **Phase** | W4 of [000](000-master-plan.md) §9. 000 §5.8: "The user can add lists and write rules." |
| **Contract** | 1.54: `SetGuardRules`, `QueryGuardRules`, answered with `GuardRules` |
| **Builds on** | [064](064-guards-request-engine.md): the blocker, made from rules and never changed after; its §3, "rules change by a new blocker", which nothing does yet. 000 §5.6: `custom_rule`, the user's own rules |
| **Last updated** | 2026-11-25 |

## 1. User story

A site I use shows me the same box every day, fetched from one address. I know the address. I want to tell my browser never to fetch it, in a line, and have that hold from then on.

## 2. What is in it, and what is not

**In:** the core keeps a person's own rules, as text, in the syntax the public lists are written in. They are in force from the moment they are set, on every thread that asks, and after a start. The core says what they are when asked.

**Not in, each with where it goes:**

- **Where a person writes them:** a page of settings, which is a shell's design and waits for the Guard panel.
- **The public lists.** [064](064-guards-request-engine.md) D6: the owner's word first. A person's rules are the only rules there are until then.
- **The switches** (this site, this Space, everywhere): their own feature.
- **Rules that hide or mend a page's content:** not taken in (064 §2); a line that is one is kept as the person wrote it, and does nothing yet.
- **Rules for one Space alone.** A person's rules are for everywhere.

## 3. The rule

**What is kept.** The text as the person wrote it, whole: their comments, their blank lines, their order. It is theirs to read again and change.

**What is in force.** The blocker made of that text ([064](064-guards-request-engine.md)): a rule to a line, and what is no rule ignored.

**From the moment it is set.** A new blocker is made of the new text, off the fast path, and put in the old one's place at a stroke. A decision that is being made is made on the one it began with; the next is made on the new. Nothing is reloaded: what a page has fetched it has, and what it asks for next is decided by the new rules.

**After a start.** The rules are kept in the store, and the blocker a core starts with is made of them.

**How much.** A limit on the text's length, so that a person's rules cannot be made a list by another way: 256 KB, which is some thousands of rules. Over it, the request is refused and the rules there were stay.

**Fake mode.** A suite's rules ([064](064-guards-request-engine.md), 1.53) are the rules a fake core starts with, as a person's are a real one's. Set by the request, they are replaced like a person's; fake mode's store is in memory, so nothing outlives the core.

**Nothing of it is in a log or an error:** a rule names addresses a person cares about.

## 4. The contract

1.54. Two requests:

- **`SetGuardRules { text }`:** the whole text, which replaces what there was. Answered with `Ack` when the rules are kept and the new blocker is in force. Refused with `guard.rules_too_long`, naming `text`, when the text is over 262,144 bytes; what there was then stays.
- **`QueryGuardRules {}`:** answered with `GuardRules { text }`, the text as it was set; empty when there are none.

Both are for an ordinary window's settings; neither names a window, a Space or a site. The text is marked as what a person typed is: never in a log or an error.

**As built, where the design did not say:**

- **Who holds the blocker is told by the core, and the core does not hold it.** The blocker is the fast path's (064), on the side of the core that engines' threads reach; the models and the store are on the core's own thread. The core's own thread tells that side the text each time it changes, and at once when it begins to listen, which is as the core starts: the blocker of the start's rules is in place before a shell is told the core has started. The telling is done inside the request, before it is answered.
- **No rules is no blocker.** A text that is empty, or only blank lines, makes a blocker that looks nothing up (064 §4), however it is said.
- **A store that refuses** keeps nothing: the rules hold for the run all the same, and the ones before come back at the next start. As where searches go (020).
- **The refusal has no words yet:** no screen shows it.

## 5. In what order

1. **The core:** the rules kept in the store; the two requests; the blocker a core starts with. **Built.**
2. **The fast path:** the blocker put in the old one's place at a stroke when the rules are set; many threads asking while they change. **Built.**
3. **The shells:** nothing, until there is a place to write rules. Android's client knows the new answer by its name (003).

## 6. Decisions

**All of these are recommendations** (§ Status). The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | A person's rules are one text, kept whole | It is what a person writes and reads; comments and order are theirs. One request replaces it | A rule to a row, each with its own switch (000 §5.6 has a table for them): for a list of rules with a tick beside each, which is a shell's design that is not made |
| D2 | For everywhere: every Space, every site | A rule already says which sites it is for, in its own syntax | Rules for a Space: with the switches |
| D3 | In force at once, without a reload | A rule set should hold from then on. Reloading a page a person is reading is theirs to do | Every page reloaded, so that what was fetched under the old rules goes |
| D4 | The new blocker is made on the core's own thread | A person's rules are few: thousands make a blocker in a millisecond or two (064 §5.1: 100,000 in 40 ms). The core's thread is not the fast path's | A thread of its own for making blockers: needed for the public lists, whose hundred thousand rules would hold the core's thread for tens of milliseconds, and built with them |
| D5 | A limit of 256 KB of text | A person's rules are not a list: a list has a version, a source and an update, which are the lists' feature | No limit |
| D6 | Private windows are under the same rules | A rule is a person's choice about the web, not something they did in a window | No rules of a person's in a private window: fewer traces of what a person blocks, and more of everything else |

## 7. Tests

| Where | What |
|---|---|
| The core, through the contract (`guard_rules`, 5) | **As they were written:** none at first; a text with comments, a blank line, a comment that is not ASCII and a line that is no rule is asked for again whole; another text is in its place; none is none. **Kept across a start,** and whoever holds the blocker is told them before anything is asked; taken away, they are gone at the next start too. **Over the limit:** a text of just 262,144 bytes is taken, one byte more is refused by its code, naming the text; what there was stays, and the holder of the blocker was not told of the text that was refused. **The holder is told at once and before the answer:** as it begins to listen, the start's rules; when rules are sent and not yet answered, the new ones already; once for each setting; and asking for the rules tells nobody anything. **In fake mode a suite's rules stand where a person's would,** are what is asked for, and are replaced like them; outside fake mode a suite's rules are nothing |
| The fast path, through the C functions (`wt-ffi`, `core`, +2) | A core on a store on disk: a request allowed; rules set, and the very next thing asked is refused, with nothing started again, and the exception among them allowed; other rules in their place, and what the first blocked is allowed and the new holds; taken away, everything allowed; set, and the core stopped and started: in force with nothing asked. **Eight threads asking while the rules are set and taken away three hundred times:** every answer is "ok" with one verdict or the other, what no rule names is always allowed, both verdicts were seen, and the last rules set are the ones in force |
| What a blocker of no rules holds (`wt-ffi`, `held`, in the test there was) | Nothing, in each of the three ways there are to have none |
| The core broken, a part at a time | 11 parts, 11 caught, at the first run |
| Android's client (`NativeCoreTest`, as it was) | It knows every kind of answer the contract has by its name, the new one among them |
| Every other test of the core | 802 before this, and all pass with it |

## 8. What is not verified

- **The choices of §6** are the recommendations, used on the owner's word; the owner has not looked at them.
- **In an app.** No shell sets or shows rules: nothing here was seen in an app, on any system. On Apple's systems nothing asks the fast path at all (064).
- **A store that refuses** (§4): written, and not tried; no test makes a store refuse.
- **How long a new blocker takes to make** for a text at the limit, on the core's own thread, during which the core answers nothing else: 064 §5.1 measured 40 ms for 100,000 made-up rules, which is some four megabytes of text, so a quarter megabyte should be a few milliseconds. Not measured.
- **Which of a person's lines are rules.** A line that is no rule is ignored, silently: a person is not told that what they wrote does nothing. For the page where rules are written.
- **Rules that hide or mend a page's content** are kept as written and do nothing (§2).

## 9. Done checklist

- [ ] The owner's word on §6.
- [x] The core: kept, asked for, the start.
- [x] The fast path: swapped at a stroke.
- [x] The gate: every stage of `Tools/test.sh all` green, and `android-device`, on 2026-11-25: `lint`, `core` (809, and the strings tool's 24), `unit` (407 on the Mac), `build`, the Mac's UI suite (67), `ios` (391 on the iPhone simulator), the iPhone's UI suite (56), `android` (542 and 33 on the JVM) and `android-device` (201 on the emulator).

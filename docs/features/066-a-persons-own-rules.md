# 066 — A person's own rules for Guard (W4)

| | |
|---|---|
| **Status** | **Designed, on the recommendations of §6, for the core alone.** The owner's word of 2026-11-24 was to use what is suggested where something waits on a choice, and to go on: they are to be built on, and stay the owner's to change. Not built. Where a person writes their rules is a shell's design, and is not designed here |
| **Phase** | W4 of [000](000-master-plan.md) §9. 000 §5.8: "The user can add lists and write rules." |
| **Contract** | To come: a request that sets a person's rules, and one that asks for them |
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

The next minor. Two requests:

- **Set the rules:** the whole text, which replaces what there was. Answered when the rules are kept and the new blocker is in force. Refused, with the code that says so, when the text is over the limit.
- **Ask for the rules:** answered with the text as it was set.

Both are for an ordinary window's settings; neither names a window, a Space or a site.

## 5. In what order

1. **The core:** the rules kept in the store; the two requests; the blocker a core starts with.
2. **The fast path:** the blocker put in the old one's place at a stroke when the rules are set; many threads asking while they change.
3. **The shells:** nothing, until there is a place to write rules.

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

To be filled in as built. Planned:

| Where | What |
|---|---|
| The core, through the contract | Set and asked for again, as written; kept across a start; over the limit refused, with what there was unchanged; fake mode's suite rules as the start, and replaced by the request |
| The fast path, through the C functions | A request allowed before a rule is set and refused after, with no start between; a rule taken away, and the request allowed again; eight threads asking while the rules are set and unset: every answer is the old rules' or the new, and none is an error |
| The core broken, a part at a time | Each part of keeping, limiting and swapping, left out |

## 8. What is not verified

To be filled in as built.

- **The choices of §6** are the recommendations; the owner has not looked at them.

## 9. Done checklist

- [ ] The owner's word on §6.
- [ ] The core: kept, asked for, the start.
- [ ] The fast path: swapped at a stroke.
- [ ] The gate.

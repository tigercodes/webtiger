# 062 — A row's line of things to undo keeps what changed (W3)

| | |
|---|---|
| **Status** | **Built and tested, in the core, on the recommendations of §6.** The owner's word of 2026-11-24 was to use what is suggested where something waits on a choice, and to go on: they are built on, and stay the owner's to change. A step of a row's line keeps what the thing done changed: fifty things done to a row of two thousand Sites hold 0.8 MB where they held 28.7, and to a row of forty 25 to 39 KB where they held 767. Undo is what it was, but that a thing which moved nothing moves nothing when it is undone. No shell changed |
| **Phase** | W3 of [000](000-master-plan.md) §9. 000 §7: "The core's share: ≤ 30 MB." The twelfth of W3: the other thing [060](060-the-budgets-as-tests.md) §8 found |
| **Contract** | None |
| **Builds on** | [044](044-undo-for-the-tree.md): a row's line of things to undo, which this keeps as it is to a person and makes smaller. [060](060-the-budgets-as-tests.md) §5.1: what the line was measured to hold. [061](061-a-space-put-away.md): a Space that is not shown no longer holds its line; this is for the one that is |
| **Last updated** | 2026-11-25 |

## 1. User story

I have a long row, and I arrange it all day: a tab moved, a Site kept, a Group made. Undo should go on working, fifty steps back. It should not cost me a browser's worth of memory to have it.

## 2. What is in it, and what is not

**In:** each step of a row's line (044) keeps what the thing done changed, and not the whole row. Undo does what it did.

**Not in, each with where it goes:**

- **Anything a person sees.** No command, word or picture changes. The contract does not change.
- **The line kept across a quit.** 044 D3: for the run only.
- **A longer line.** Fifty deep, as it is.
- **A Space that is not shown.** 061: it holds no line at all.

## 3. The rule

**As it was (044).** Before each thing that arranges a row, the core kept the row's whole arrangement: every tab's number, whether it is a Site and of which home and name, its Group, whether it is a Favourite; and every Group. Fifty of them. Measured (060 §5.1): for a row of forty-eight tabs, 450 KB; worked out for a row of two thousand, near 19 MB. **Measured here, before anything was changed** (§5.1): 28.7 MB for a row of two thousand Sites, which is nearly all the whole core may hold.

**As it is now.** A step keeps only what the thing done changed:

- **the tabs it changed,** each as it was: whether it was a Site (its home and its name), its Group, whether it was a Favourite. A tab the thing did not change is not kept;
- **the order,** as the tabs' numbers, and only when the thing changed the order;
- **the Groups,** as they were, and only when the thing changed a Group (made, removed, renamed, coloured or moved one).

**Undo** puts back what the step kept, by 044's rules, which do not change:

- every tab the step changed that is still in the row is what it was;
- where the step changed the order, the tabs that are still there are in the order they were, and a tab opened since is after them;
- the row is then in a row's order (041), whatever has come since: a tab opened since is where a row's order has a tab of its kind, and the others stay as they are;
- where the step changed the Groups, they are the Groups there were, each shut or open as it is now;
- a tab closed since stays closed; no page is loaded, closed, put to sleep or woken.

**Why it is the same thing.** Nothing changes whether a tab of a row is a Site, its Group or whether it is a Favourite but a thing that arranges the row, and each of those is a step. Undo takes the newest step first. So when a step is undone, every tab it did not change is already what it was before it: putting the whole row back and putting back what the step changed come to the same row.

**Where it is not the same,** and is better: a thing that did not change the order (a Group made, named or coloured; a Site let go where it was the last of the Sites) no longer moves anything when it is undone. As it was, a tab opened since was put after every other tab by any undo.

**As built, where the design did not say:** a thing that changes what a tab is and not the order can leave a tab that came since on the wrong side of it once it is undone. A Site that was the last of the Sites is let go, and is the first of the tabs, where it was; a link is opened in a tab, which goes before it; the letting go is undone, and the Site is a Site again with a tab before it. So an undo ends by putting the row in a row's order, part by part, each part's tabs staying in the order they are in: the tab that came since is after the Site, and before the tabs it was before.

**The window is told** what its newest is, by its kind, as now.

## 4. The contract

None.

## 5. In what order

1. **Measure:** what a row's line holds now, for a short row and for a row of two thousand, each arranged fifty times, by an allocator that counts (060's). Before anything is changed, so that the test is not written to a number that was never true. **Done** (§5.1).
2. **The core:** the step that keeps what changed; undo from it. **Built.** Every test 044 and 045 have of undo passes as it was written.
3. **The measure again, as a test** held at a ceiling (D4). **Built.**

### 5.1 The line, measured

By the allocator that counts (060 §5.1), on a row a start finds asleep, with nothing to undo: forty Sites, or two thousand, eight of them each in a Group of its own, and one tab. Fifty things are done that leave the row as it was, so that what the core holds more afterwards is the line and nothing else. Bytes:

| The row | Fifty times | As it was | As it is now | The test's ceiling |
|---|---|---|---|---|
| Forty Sites | A Site moved a place on and a place back (the order changes) | 767,517 | 25,153 | 32 KB |
| | A Group named one thing and another (a Group changes) | 764,670 | 34,306 | 44 KB |
| | A Site made a Favourite and not (a tab changes, and the order) | 767,525 | 38,733 | 48 KB |
| Two thousand Sites | A Site moved | 28,673,397 | 812,033 | 1 MB |
| | A Group named | 28,667,670 | 34,306 | 44 KB |
| | A Site made a Favourite and not | 28,673,405 | 809,933 | 1 MB |

What is left for a long row is the order, eight bytes a tab, fifty times (D2). A Group named again holds the same whatever the row's length.

**A thing found on the way.** The first build of the new step held 15.6 MB for the long row, and 470 KB for a Group named fifty times: over half of what it had held, with nothing in it. The changed tabs were sifted out of the list of the whole row, and a list made so keeps the room the whole row took, however few are left in it. They are gathered into a list of their own now. The measure found it; no test of undo could have.

## 6. Decisions

**All of these are recommendations** (§ Status). The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | A step keeps what the thing done changed | It is what makes the line small whatever the row's length: most things change one tab. And it is 044's undo to a person | The whole row kept, with what did not change shared between the copies and not copied: about a fifth of what it is, and still a copy of two thousand numbers and marks for a Group renamed. Or a shorter line, ten deep: a fifth, and less undo |
| D2 | The order is kept whole, as the tabs' numbers, when it changed | Eight bytes a tab. Undo's rule for tabs that have come and gone is by the tabs' numbers, and stays as it is | A move kept as "this tab, from here to there": a few bytes, and wrong when tabs have come and gone since, which is the case 044 was careful of |
| D3 | A thing undone moves nothing it did not move | It is what a person expects of "Undo Rename Group". As it is, a tab opened since goes to the row's end at any undo | As it is, kept as it is |
| D4 | The test's ceiling is 1 MB for the line of a row of two thousand tabs, moved fifty times; and 32 KB for a short row's. **As built:** a ceiling for each of the six measures of §5.1, about a quarter over what was measured | The plan has no budget for the line by itself: these are ceilings of this document's, set once the new measure is known, a little over it, so that a change that makes the line large again fails. *As built:* one ceiling for a row let a step keep the Groups for nothing, or room it did not use, and pass | No test: the measure written down once |

## 7. Tests

| Where | What |
|---|---|
| The core's memory (`wt-ffi`, `held`, +1) | §5.1: the six measures, each held at its ceiling, and none so small that something else was counted |
| The core, through the contract (`tree_undo`, `moving`, `groups`, `favourites`, `sites`, as they were) | Every test of undo there was, unchanged: each thing that arranges a row undone, the newest first; a row put back though tabs have come and gone; a Group shut or open as it is now; fifty deep, a line to each row; nothing kept of a thing that changed nothing |
| The core, through the contract (`tree_undo`, +3) | **A thing that moved nothing moves nothing when it is undone:** a Group made, named again and coloured; then a link opened in a tab beside the tab in front; each of the three undone in turn leaves every tab where it is, the tab that came since beside its opener, and the Group what it was. **A Site let go is a Site again before a tab that came since:** the case of §3, as built. **A tab moved is back where it was:** with a tab opened since, which is after the others, and a tab closed since, which is not in the row in any way: the row is as long as the tabs there are |
| The core broken, a part at a time | 16 parts, 16 caught. The first run caught 15: with a tab that had gone put back into the row's order, every test passed. The test there was of tabs that have come and gone undoes a Group's removal, which does not change the order, and so no longer puts an order back at all. The new test of a tab moved closes a tab as well now |
| Every other test of the core | 776 before this, and all pass with it |

## 8. What is not verified

- **The choices of §6** are the recommendations, used on the owner's word; the owner has not looked at them.
- **In an app.** No shell changed, and no test of an app is of this. The apps' own tests of undo (`TreeUndoTests`, and Android's) run with it (the gate).
- **The time a step takes.** Before a thing is done, the core still copies the whole row for a moment, to see afterwards what changed; it did so twice before, and does so once now. For a row of two thousand that is two thousand addresses and names copied and thrown away at each thing done. Not measured: 043's and 060's times are of other requests.
- **A row longer than two thousand.**
- **Where a tab that came since ends up** after an undo that puts an order back (the row's end, 044's rule) is as it was, and may not be what a person expects: it is not beside its opener any more. Not changed here; not tried with anyone.

## 9. Done checklist

- [ ] The owner's word on §6 (to be used as suggested, on the word of 2026-11-24).
- [x] The measure before.
- [x] The core.
- [x] The measure after, as a test.
- [ ] The gate.

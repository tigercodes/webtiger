# 062 — A row's line of things to undo keeps what changed (W3)

| | |
|---|---|
| **Status** | **Designed, on the recommendations of §6.** The owner's word of 2026-11-24 was to use what is suggested where something waits on a choice, and to go on: they are to be built on, and stay the owner's to change. Not built |
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

**As it is (044).** Before each thing that arranges a row, the core keeps the row's whole arrangement: every tab's number, whether it is a Site and of which home and name, its Group, whether it is a Favourite; and every Group. Fifty of them. Measured (060 §5.1): for a row of forty-eight tabs, 450 KB; worked out for a row of two thousand, near 19 MB, which is most of what the whole core may hold.

**As it is to be.** A step keeps only what the thing done changed:

- **the tabs it changed,** each as it was: whether it was a Site (its home and its name), its Group, whether it was a Favourite. A tab the thing did not change is not kept;
- **the order,** as the tabs' numbers, and only when the thing changed the order;
- **the Groups,** as they were, and only when the thing changed a Group (made, removed, renamed, coloured or moved one).

**Undo** puts back what the step kept, by 044's rules, which do not change:

- every tab the step changed that is still in the row is what it was;
- where the step changed the order, the tabs that are still there are in the order they were, and a tab opened since stays, where the row's order puts a tab of its kind;
- where the step changed the Groups, they are the Groups there were, each shut or open as it is now;
- a tab closed since stays closed; no page is loaded, closed, put to sleep or woken.

**Why it is the same thing.** Nothing changes whether a tab of a row is a Site, its Group or whether it is a Favourite but a thing that arranges the row, and each of those is a step. Undo takes the newest step first. So when a step is undone, every tab it did not change is already what it was before it: putting the whole row back and putting back what the step changed come to the same row.

**Where it is not the same,** and is better: a thing that did not change the order (a Group renamed, or coloured) no longer moves anything when it is undone. As it is, a tab opened since is put after every other tab by any undo.

**The window is told** what its newest is, by its kind, as now.

## 4. The contract

None.

## 5. In what order

1. **Measure:** what a row's line holds now, for the standard row and for a row of two thousand, each arranged fifty times, by an allocator that counts (060's). Before anything is changed, so that the test is not written to a number that was never true.
2. **The core:** the step that keeps what changed; undo from it. Every test 044 and 045 have of undo passes as it is written.
3. **The measure again, as a test** held at a ceiling (D4).

## 6. Decisions

**All of these are recommendations** (§ Status). The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | A step keeps what the thing done changed | It is what makes the line small whatever the row's length: most things change one tab. And it is 044's undo to a person | The whole row kept, with what did not change shared between the copies and not copied: about a fifth of what it is, and still a copy of two thousand numbers and marks for a Group renamed. Or a shorter line, ten deep: a fifth, and less undo |
| D2 | The order is kept whole, as the tabs' numbers, when it changed | Eight bytes a tab. Undo's rule for tabs that have come and gone is by the tabs' numbers, and stays as it is | A move kept as "this tab, from here to there": a few bytes, and wrong when tabs have come and gone since, which is the case 044 was careful of |
| D3 | A thing undone moves nothing it did not move | It is what a person expects of "Undo Rename Group". As it is, a tab opened since goes to the row's end at any undo | As it is, kept as it is |
| D4 | The test's ceiling is 1 MB for the line of a row of two thousand tabs, moved fifty times; and 32 KB for the standard row's | The plan has no budget for the line by itself: these are ceilings of this document's, set once the new measure is known, a little over it, so that a change that makes the line large again fails | No test: the measure written down once |

## 7. Tests

To be filled in as built. Planned:

| Where | What |
|---|---|
| The core's memory (`wt-ffi`, `held`) | A row of two thousand tabs, one moved fifty times; the standard row, arranged fifty times: what the line holds, at D4's ceilings |
| The core, through the contract (`tree_undo`, `moving`, as they are) | Every test of undo there is, unchanged |
| The core, through the contract (new) | Each kind of thing undone after tabs have come and gone since, where the step kept the order and where it did not. A thing that did not change the order moves nothing when it is undone |
| The core broken, a part at a time | Each part of what a step keeps, left out |

## 8. What is not verified

To be filled in as built.

- **The choices of §6** are the recommendations; the owner has not looked at them.

## 9. Done checklist

- [ ] The owner's word on §6 (to be used as suggested, on the word of 2026-11-24).
- [ ] The measure before.
- [ ] The core.
- [ ] The measure after, as a test.
- [ ] The gate.

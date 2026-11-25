# 061 — A Space that is not shown is put away (W3)

| | |
|---|---|
| **Status** | **Designed, not built.** The choices of §6 are recommendations, to be used on the owner's word of 2026-11-24 (to use what is suggested where something waits on a choice, and to go on); they stay the owner's to change |
| **Phase** | W3 of [000](000-master-plan.md) §9. 000 §7: "A Space with nothing live: no process; ≤ 100 KB." The eleventh of W3: what [060](060-the-budgets-as-tests.md) §8 found not met |
| **Contract** | None |
| **Builds on** | [060](060-the-budgets-as-tests.md): the measurement, and the test that fails. [009](009-session-restore.md), [024](024-several-spaces.md): the session, which keeps each Space's tabs. [044](044-undo-for-the-tree.md): a row's line of things to undo. [051](051-a-tabs-kept-state.md), [059](059-a-sleeping-tabs-picture.md): what a sleeping tab keeps in the store |
| **Last updated** | 2026-11-25 |

## 1. User story

I keep five Spaces and work in one. The other four should cost me next to nothing while I am not in them, whatever I did in them this morning.

## 2. What is in it, and what is not

**In:** a Space that no window shows, and of which nothing is alive, is put away: the core keeps of it only what the session keeps, as it does from a start of a Space that has not been shown yet. When the Space is shown again its row is made from that, as at a start.

**Not in, each with where it goes:**

- **A smaller line of things to undo** for the Space that *is* shown (060 §8: a long row's line is large). Its own change: each step keeping only what it changed.
- **A second window's Spaces, and a private window's.** The session is one window's (009): nothing keeps another window's rows to put them away to.
- **A sleeping tab made smaller.** 060 §5.1 has what one is.

## 3. The rule

**When.** At the moment the core has told its windows of a change, and written the session: each Space of the session's window that the window does not show, none of whose tabs has a web view, and none of whose tabs is still waiting for its state or its picture (053, 059: they are asked for as a tab is put to sleep, and arrive a moment later).

**What goes.** The Space's row and its tabs, as the core's model has them; and with the row, its line of things to undo (044).

**What stays.** What the session keeps of the Space, which the core has beside the model already, as it was just written: each tab's address and name, whether it is a Site and of which home, its Group and whether it is a Favourite, when it was last in front (049), its state's number (051) and so its picture (059), which tab was in front; and the Groups, with their names, colours and whether they are shut.

**Coming back.** The Space shown again, its row is made from what stayed, as at a start: the same tabs in the same order, asleep, the same one in front, which wakes from its state with its picture. Its tabs and Groups have other numbers than they had: a number is a run's, and no window was holding these.

**Tidying (049).** A Space's tabs are tidied by a look at the rows the core has, and a Space that is put away has none: it is not tidied while it is away, as a Space that has not been shown since a start is not. It is tidied as it is shown again, before the window is told of its row, so that no tab shows and then goes; that is done for every Space that is shown for the first time, which until now waited for the next minute's look.

**The store must have it.** A Space is put away only when what the session keeps of it is what the store was last given: a store that refused the last writing keeps the Space in the model, where the next writing has it still.

**What a person can tell.** After coming back to a Space that was put away, there is nothing of its arrangement to undo; and a tab that was past its Space's time while the Space was away is in the Archive from the moment the Space is shown, and not from the minute its time ran out.

## 4. The contract

None.

## 5. In what order

1. **The core:** putting away, and coming back. Against the scripted engine.
2. **The budget's test** (060) put back among the others.

## 6. Decisions

**All of these are recommendations** (§ Status). The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | A Space is put away as soon as no window shows it and nothing of it is alive | It is then exactly what a Space is at a start. A tab of it that is alive keeps it: its page goes on, and its tab sleeps by the usual rules (053), after which the Space is put away | After a time out of sight: one more timer, for a difference nobody would see but in what can be undone |
| D2 | Its line of things to undo goes with it | The line is what takes a Space over its budget (060 §5.1: four fifths of 549 KB). It names tabs by their numbers, which are gone. And 044 D3 made the line the run's already: this makes it the visit's | The line kept and made small, and its tabs' numbers kept alive for it: undo would work after coming back, and a Space that is not shown would hold a line it may never use |
| D3 | Only the Spaces of the window that keeps the session | A second window's rows are kept nowhere (009): put away, they would be lost | A second window's rows kept in memory in the session's form: for the few who have two windows on many Spaces |
| D4 | Nothing is said to a shell | No window shows the Space: there is nobody to tell, and nothing it shows changes | An event, for a shell that wants to know what is in memory |
| D5 | A Space that is away is not tidied until it is shown, and is tidied then at once | It is how a Space that has not been shown since a start is treated already, but for the "at once", which is better for both: no tab shows and then goes | Tidied while it is away, from what the session keeps of it: the Archive would have a tab the minute its time ran out, and tidying would have a second way of working to keep right. Or a Space with a time is not put away while it has a tab that could be tidied: it then holds its row for as long as its time is, a week or a month |

## 7. Tests

To be filled in as built.

## 8. What is not verified

To be filled in as built.

## 9. Done checklist

- [ ] The owner's word on §6 (to be used as suggested, on the word of 2026-11-24).
- [ ] The core.
- [ ] The budget's test put back (060).
- [ ] The gate.

# 061 — A Space that is not shown is put away (W3)

| | |
|---|---|
| **Status** | **Built and tested, in the core, on the recommendations of §6.** The owner's word of 2026-11-24 was to use what is suggested where something waits on a choice, and to go on: they are built on, and stay the owner's to change. A Space that no window shows, with nothing of it alive, is put away, and holds 41 KB in the core whatever was done to its row; the budget that 060 found not met is met, and its test is back among the others. No shell changed |
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

**Tidying (049).** A Space's tabs are tidied by a look at the rows the core has, and a Space that is put away has none: it is not tidied while it is away, as a Space that has not been shown since a start is not. It is tidied as it is shown again, before the window is told of its row, so that no tab shows and then goes. **As built:** there is a look whenever a window is shown another Space, whether or not that Space was away: it is one rule and not two, and a look that finds nothing to tidy does nothing. (A Space shown for the first time since a start waited, until now, for the next minute's look.)

**As built: two moments.** A Space is looked at for putting away when the windows have been told of a change and the session written; and when the state a sleeping tab was asked for arrives, since that is when a Space whose last tab has just been put to sleep stops waiting, and nothing else may happen for a long time.

**The store must have it.** A Space is put away only when what the session keeps of it is what the store was last given: a store that refused the last writing keeps the Space in the model, where the next writing has it still.

**What a person can tell.** After coming back to a Space that was put away, there is nothing of its arrangement to undo; and a tab that was past its Space's time while the Space was away is in the Archive from the moment the Space is shown, and not from the minute its time ran out.

## 4. The contract

None.

## 5. In what order

1. **The core:** putting away, and coming back. Against the scripted engine. **Built.**
2. **The budget's test** (060) put back among the others. **Done:** it passes, at 41 KB.

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

Nothing is said to a shell of a Space put away (D4): the tests tell by what follows from it. A row made again has other numbers for its tabs and its Groups, and nothing of its arrangement to undo.

| Where | What |
|---|---|
| The core, through the contract, on a store that keeps the session (`put_away`, 6) | **Put away, and back as it was.** A Space with a Favourite, a Group that is shut with two Sites in it, a Site in none and three tabs, the second of them in front: left with nothing alive, and shown again, it is the same tabs in the same order, each what it was (its name, its site, whether it is a Site and of what name, a Favourite, asleep, and in which Group), the same Groups, the same tab in front, which wakes from its state; its tabs and Groups have other numbers, and its arrangement has nothing to undo; and after a start it is the same again. **Not while something of it is alive:** left with its front tab's page alive, it is the same row when the window comes back, by the same numbers, with its arrangement to undo; when that tab has slept by its time, it is put away. **Not while a state is awaited:** with a shell that is slow to say the state of the tab just put to sleep, the Space stays; when the state has come it is put away, and the tab wakes from that state. **Not the Space a window shows,** though no page of it is alive; **nor another window's:** a second window's Space, left with nothing alive, is as it left it, and the first window's row is untouched by it. **What it kept is there:** a tab closed in it before it was put away is brought back by undo after; how many tabs are asleep is counted the same from the session's rows as from the model's; a tab that slept before wakes from its state with its picture named. **A Space that was never used** comes back as one empty tab, and asks for no web view. **Tidied as it is shown:** on a clock the test moves, a Space with a time of half a day is put away and a day passes; shown again, its two tabs that are past the time are not in the row the window is told, and are in its Archive; its Sites and the tab in front stay |
| The budget (`wt-ffi`, `held`; 060) | A Space that was shown and left with nothing live: 41,433 bytes; the same, its row arranged in this run: 41,001; where 102,400 are allowed. Both are tests among the others now |
| The core broken, a part at a time | 11 parts, 11 caught. The first run caught 10: with another window's Spaces looked at for putting away, every test passed, because the test of a second window looked only at the second window afterwards, and what the break put away was the first window's row. It looks at both now |
| Every other test of the core | 768 before this, and all pass with it: nothing that was tested of Spaces, sleeping, tidying, undo or the session changed |

## 8. What is not verified

- **The choices of §6** are the recommendations, used on the owner's word; the owner has not looked at them.
- **A store that refuses.** A Space is put away only when the store has what the session keeps of it. No test makes a store refuse a writing: that the Space then stays in the model is written, and not tried.
- **In an app.** No shell changed, and no test of an app is of this: the apps' own tests of Spaces run with it (the gate), and none leaves a Space with nothing alive and comes back. By hand: not done.
- **Undo after coming back,** as a thing people miss or do not (D2): not tried with anyone.
- **A second window** keeps its rows as before, at what they cost (060 §5.1).
- **What the store's own memory is** for the Spaces that are away: SQLite's, and not counted (060 §5.1).

## 9. Done checklist

- [ ] The owner's word on §6 (to be used as suggested, on the word of 2026-11-24).
- [x] The core.
- [x] The budget's test put back (060).
- [x] The gate: every stage of `Tools/test.sh all` green, and `android-device`, on 2026-11-25: `lint`, `core` (776, and the strings tool's 24), `unit` (407 on the Mac), `build`, the Mac's UI suite (66), `ios` (391 on the iPhone simulator), the iPhone's UI suite (55), `android` (537 and 32 on the JVM) and `android-device` (200 on the emulator). (The Mac's UI suite was run last, after the others, once a dialog of the system's had gone from the screen).

# 057 — The Library (W3)

| | |
|---|---|
| **Status** | **Built and tested on Apple's systems and Android, on the recommendations of §6.** The owner's word of 2026-11-24 was to use what is suggested where something waits on a choice, and to go on: they are built on, and stay the owner's to change. The sheet that four lists shared is the Library: its switch has a line of its own on every system, and its fifth list is what is awake ([056](056-what-is-awake.md)) |
| **Phase** | W3 of [000](000-master-plan.md) §9. 000 §3.3: "Library: downloads, archive". The seventh of W3: the place the lists about the browser are in, which the list of what is awake ([056](056-what-is-awake.md)) waits for |
| **Contract** | None of its own: every list in it has its own |
| **Builds on** | [008](008-history.md), [013](013-downloads.md), [014](014-permissions.md), [049](049-tidying-temporary-tabs.md): its four lists, and the sheet they share. [056](056-what-is-awake.md): its fifth |
| **Last updated** | 2026-11-24 |

## 1. User story

There are things I want from the browser now and then, and not all day: what I visited, what I downloaded, what I let a site use, the tabs that were tidied away, what is using memory. I want one place for them, that I open with one button, and that has room for the next such list without becoming harder to read.

## 2. What is in it, and what is not

**In:** the sheet that History, Downloads, Permissions and the Archive share is the Library, and is made to hold as many lists as there are. Its switch, the row of the lists' names, has a line of its own on every system, above the shown list's own buttons; where the names are more than the line holds, the row is moved sideways, and it opens with the shown list's name in sight. It gains its fifth list: **Awake** (056).

**What changed, and where.** On a Mac the switch had shared one line with the list's buttons, and now has the line above them: the one change a person sees in the four lists there. On an iPhone and on Android the switch had its own line already, and is as it was, with one name more at its end.

**Not in, each with where it goes:**

- **A Library that is a window or a page of its own,** with its lists down one side: §6 D1's other way. A sheet is what there is; the lists do not change if their place does.
- **Remembering which list was last shown:** it opens on History, as it does. A view setting, with settings.
- **Saved pages, media, or anything a list is not yet kept of:** each comes with its own feature, as a part.
- **Another name on its button.** It says "History and Downloads", which is what a person looks for (D4).

## 3. The rule

Nothing here is a rule about browsing, and nothing is the core's: it is where five lists are drawn.

**The switch has its own line.** Until now it stood where a list's title would, on one line with the list's buttons, except on an iPhone, where that line was already too short and the buttons had the line below. On a Mac four names and two buttons filled the line; a fifth name does not fit it. Now the switch is on the first line and the list's buttons on the second, on every system: one layout, and room for more names than five.

**The names are never cut short.** Where they are more than the line is wide (an iPhone, a narrow Android window, large text), the row is moved sideways, and it opens with the name of the list that is shown in sight. Each name is a button that says, to a screen reader, whether its list is the one shown.

**The order:** History, Downloads, Permissions, Archive, Awake. The lists a person opens most are first; a private window has no Archive, and its switch has no name for one.

**As built, and what was tried on the way** (2026-11-24):

- **The names on as many lines as they need,** where one line is not enough, so that every name is always in sight: tried on an iPhone, and taken out again. At the usual text size it was two lines and read well. At the largest sizes a person can ask for, it was a line for each name: the top of the list took most of the sheet, and the list under it had almost none. Apple's own audit said so twice over: first that the words of the list's buttons would be cut short (the top had given way to the list), and, once the top was made to keep its height, that the text of the list's last lines could not be made larger (there was no room left to show them). A row that is moved sideways is one line at every size.
- **The shown name in the middle of the row,** so that its neighbours show too: tried, and taken out with the above. The row opens with the shown name at its end, as it did.
- **So, on a narrow window, a name can be out of sight.** With the Archive shown on an iPhone or a narrow Android window, the names before it fill the line and Awake's is past its end, until the row is moved. With History shown, the last name in sight is cut by the line's end, which says there are more. It is the cost of one line (D2's other way, a menu, has none of it).
- **Apple's audit changes the size of the text while it looks.** A row that has been moved to its end is moved again under it as the names grow and shrink, and it once faulted the contrast of a name it had caught moving (on a simulator just started; not in five runs after). So the audit of the list of what is awake, whose name is the row's last, is made on a Mac, where nothing moves, and not on an iPhone (§8).

## 4. The contract

None. `QueryAwake` and `SleepOthers` are 056's (1.49).

## 5. In what order

1. **Apple's shell:** the switch on its own line on the Mac as on the iPhone; the fifth list, with its model, its words and its pictures. **Built.**
2. **Android's shell:** the fifth list; its switch has had a line of its own, and moved sideways, from the start. **Built.**

## 6. Decisions

**All of these are recommendations, used on the owner's word of 2026-11-24** (§ Status).

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | The Library is the sheet the four lists share, kept as a sheet | It is there, people know its button, and a list's place can change later without the list changing | A window of its own on the Mac and a page on a phone, with the lists' names down one side: more room, and a second way of showing every list to build and to test |
| D2 | The switch is on a line of its own, above the shown list's buttons, on every system; one line, moved sideways where the names are more than it holds | One layout for all three. It has room for a fifth name now and for more, where one line with the buttons had room for four. One line is one line at every text size | On a narrow window, a menu that names the shown list and opens the others: any number fits at any size, no name is past a row's end, and nothing moves under Apple's audit; the others are out of sight until asked for, and the tests of every list must open it first. Or the names on as many lines as they need: tried, and too tall at the largest text sizes (§3). Or the Mac's sheet made wide enough for five names and two buttons on one line, which the sixth name undoes |
| D3 | Awake is the last of the five | It is the one opened least, and the newest | First, since it is about now |
| D4 | The button keeps its words, "History and Downloads", and its key | They say what most people open it for. "Library" is the plan's word for the place, and need not be the button's | "Library" on the button, in the menu and in the command bar |

## 7. Tests

| Where | What |
|---|---|
| Apple's pictures (`CatalogueSnapshotTests`, +1; the Archive's four re-recorded on the Mac, four new of what is awake on each system) | On the Mac the switch's five names are on one line, with the list's buttons on the line below, in both themes, for the Archive with tabs and with none and for what is awake with tabs and with none. On the iPhone the Archive's pictures are the ones they were, to the last dot (the row at the Archive's name; Awake's past its end), and the new list's show the row moved to its end. Every one was looked at |
| The apps (`AwakeListTests`, 1, on the Mac and the iPhone simulator) | The Library's button; the fifth name pressed, on an iPhone after the row has been moved to it; the list (056 §7). And every older test of the Library's lists on the iPhone simulator, run with the fifth name there: History's, Downloads', Permissions', the Archive's, a private window's, which has no Archive and has this list |
| Android (`ArchiveListTest`, +2, on the JVM) | Where the names are more than the row is wide, the name of what is awake is brought in sight when it is the one shown, the names are in their order, and the first is out of sight; with History shown the last is past the row's end. A window with no Archive has no name for one, and has this one, which chooses this list |
| Android's pictures (`CatalogueSnapshotTest`, +2 tests, four new pictures) | What is awake, with tabs and with none, in both themes, under the switch moved to its end. Every older picture of a list was recorded again and is the one it was, to the last dot |
| Android, on an emulator (`AppAwakeTest`, 1) | The Library's button, the fifth name brought in sight and pressed, the list (056 §7) |

## 8. What is not verified

- **The choices of §6** are the recommendations, used on the owner's word; the owner has not looked at the Library as it now is.
- **Apple's audit of the list of what is awake, on an iPhone.** Not made there: the row of names is at its end for that list, the audit moves it by changing the text's size, and it once faulted a name's contrast as it moved (§3). The audit is made on a Mac. The lists audited on an iPhone are the ones whose names are at the row's start, and every name is drawn the same way.
- **A name that is out of sight** on a narrow window (§3): a person who never moves the row never learns of the fifth list. Not tried with anyone.
- **An iPad,** and a wide Android window: not run. The switch there is the Mac's one line, by the same code.
- **The largest text sizes, by eye.** The row is one line at every size by its making, and Apple's audit passes the four older lists on an iPhone as it did. No picture is recorded at a large size.

## 9. Done checklist

- [ ] The owner's word on §6 (used as suggested, on the word of 2026-11-24).
- [x] Apple's shell.
- [x] Android's shell.
- [ ] The gate.

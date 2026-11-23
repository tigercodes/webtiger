# 047 — The phone's switcher (W2)

| | |
|---|---|
| **Status** | **Built and tested in the core, the Apple shell and the Android shell, on the recommendations of §6.** D2 and D3, where its button is and that the strip stays, are the owner's (WT-D34, 2026-11-23: "use suggested choices for the blocked items"); D1, D4 and D5 are still the recommendations. A phone's toolbar has the button, and the switcher has the whole row |
| **Phase** | W2 of [000](000-master-plan.md) §9: "the phone switcher". 000 §6.4: "The tab switcher is a full-height sheet. The layout setting chooses its form: the same tree as the sidebar, or cards" |
| **Contract** | 1.38 (§5): one number. The switcher itself draws the row a window is told already |
| **Builds on** | [043](043-the-sidebar.md): the sidebar's parts, which are the switcher's; and its D3, "a phone has no sidebar". [041](041-groups.md) §9: "A phone's strip is narrow. With Groups the row is longer, and a phone shows the part round the tab in front" |
| **Last updated** | 2026-11-23 |

## 1. User story

On my phone the strip has room for the tab in front and one beside it. I have nine tabs, four of them Sites in two Groups, and I cannot see them: I slide the strip and hope. I want to see the whole row at once, as I do on a wide screen, and pick one.

## 2. What is in it, and what is not

**In:** in a window with no room for a sidebar (a phone; a narrow part of a tablet's screen), a button opens the switcher: a sheet as tall as the window, with the row as the sidebar has it. The Favourites' tiles; the tree of Sites and Groups, a Group shut or opened by its chip; the tabs that are no Sites; "New Tab". A press on a tab brings it in front and puts the sheet away. Everything a row does in the sidebar it does here: its button closes it, its menu keeps it, groups it, moves it.

**Not in, each with where it goes:**

- **Cards:** the plan's other form, a picture of each page. It needs the pictures (000 §9, W3: "snapshots").
- **The address field at the bottom,** and **sliding it to change tabs** (000 §6.4): the phone's toolbar is its own piece of work.
- **Sliding the switcher's header to change Spaces:** with the Spaces' own gestures.
- **Dragging** in the switcher: with dragging ([045](045-moving-in-the-row.md)).

## 3. The rule

**The switcher is the sidebar, shown another way.** It has no model of its own and asks the core nothing new: it draws the tabs and the Groups the window is told, in the core's order, with the sidebar's own parts (043). What a press, a button or a menu asks of the core is what it asks in the sidebar.

**It is there where the sidebar is not.** A window with room for a sidebar has the layout the person chose, strip or sidebar, and no switcher. A window without has the strip, and the switcher.

**It puts itself away** when a tab is brought in front from it, when "New Tab" is pressed in it, and when it is dismissed. It stays while tabs are closed, kept, grouped and moved in it: those are done several at a time.

**The tab in front is in view** when it opens, as in the sidebar.

## 4. Where it is kept

Nowhere: that the switcher is open is the window's, for as long as it is.

## 5. The contract (1.38)

Added; nothing is changed or removed. The switcher draws what a window is told already. Its button says how many tabs the row has (D4), which is the core's to count: a shell does not work out which of a shut Group's Sites are among the tabs it was told of.

```proto
message WindowSnapshot { … uint32 row_tabs = 14; }   // every tab of the row, those out of the way among them
```

**As built, where the design did not say:**

- **The square has the number up to 99,** and "99+" after; a screen reader is told "Tabs:" and the number, whole.
- **While the switcher covers the window, the window is nothing to a screen reader.** On Apple's systems a sheet does that by itself. On Android the switcher is drawn over the window, as the list of Spaces is, and the window under it was still there to be found: Android's own accessibility checks found a tab of the strip, cut to nothing by the edge of what was drawn, under the switcher. The window is now taken out of what a screen reader finds for as long as the switcher covers it.
- **The pages opened from the switcher** (the page of a Group, from a row's menu; the list of Spaces, from the Space at its head): on Apple's systems they are the window's own sheets, and the switcher makes way for them: it puts itself away. On Android they are drawn over the switcher, which is there again when they are put away.
- **The way out:** its own button; on an iPhone the sheet is also pulled down, as any sheet; on Android the system's Back.
- **A window that comes to have room for a sidebar** while its switcher is open has the switcher no longer: Apple's puts it away; Android's does not draw it, and it is there again if the window is made narrow before it is put away.
- **On Android "New Tab" in the switcher is the button with the plus,** as in Android's sidebar; on Apple's systems it is a row with its words, as in theirs.

## 6. Decisions

**All of these are put to the owner.** They are built as recommended. The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | The tree first; cards when there are pictures of pages | The tree is built, and is the only form that shows Groups. A card needs a picture of each page, which nothing makes yet | Cards first: what other phones' browsers show |
| D2 | **Its button is at the end of the toolbar, after the address field:** a square with the number of tabs in it, as phones' browsers have | The strip on a phone is full: the Space, the tabs, "New Tab" and three buttons. Another button there leaves room for one tab. The address field can give up the width of a button and still show a site's name | In the strip, in the place of Find, Zoom and History, which go into one button's menu: more room for tabs, and three things a press further away |
| D3 | **The strip stays on a phone** | A phone has had the strip since tabs were built (005), and the two tabs beside the one in front are a press away, not two. Taking it away changes more than a button does, and is the owner's to ask for | No strip on a phone: the page has the room, and every tab is two presses away, as in other phones' browsers |
| D4 | The number is of all the row's tabs, those of shut Groups among them | It is how many there are, which is what the number in such a button says everywhere. A row whose Groups are shut would otherwise say it has two tabs | The tabs the strip shows |
| D5 | The sheet covers the page, and has the Space's name at its head, which opens the list of Spaces | A sheet over half the page shows five rows. The Space is what the row belongs to | Half height, to be pulled up |

## 7. In what order

1. **The Apple shell** on the iPhone.
2. **The Android shell** on a phone.

## 8. Tests

| Where | What |
|---|---|
| The core (`groups.rs`, 1 more) | A window is told how many tabs its row has: four tabs, four; two of them in a shut Group, two shown and four in the row; a Site that is closed sleeps and is one of them still; a tab closed is one fewer; a window with one tab has one. Broken in 2 parts, 2 caught |
| The Apple shell's model (`LayoutModelTests`, 1 more) | The number is the core's count, and not how many tabs the window was told to show; the square's "99" and "99+"; what a screen reader is told. Broken in 2 parts, 2 caught |
| The apps (`SwitcherTests`, 1, on the Mac and the iPhone simulator, with the real core and the real engine) | **A Mac's window has no switcher.** On the iPhone: the button says one tab, then two; with the first in a Group and the Group shut, the strip shows one tab and the button says two. The switcher opens and passes Apple's accessibility audit; its Group is shut there too, and opened it has its Site by name; the Site chosen, it is in front and the switcher has gone; nothing was loaded. A Group made from a row's own menu in the switcher: the switcher makes way for the page of a Group, and the Group is made. Opened again and done with: nothing changed |
| Apple's pictures | The switcher, new: on the Mac's drawing and the iPhone's, light and dark, each looked at. The 48 pictures of an iPhone's window recorded again, since each has the button now: every one compared with what it was, pixel by pixel, and they differ at the end of the toolbar and nowhere else. (The two of the iPhone's wide window were written again too, with no button: they differ from what they were by a pixel's rendering, were looked at side by side, and are kept as they were.) |
| The Android shell on the JVM (`SwitcherTest`, 5) | The button says the core's count to a screen reader, and is shut until asked. A window with room for a sidebar has no button, and cannot open the switcher. The switcher has every tab the window was told of, both Groups' chips and the Space at its head; **the window it covers is nothing to a screen reader while it does**, and is there again after; a row pressed asks the core to bring its tab in front, and the switcher goes; so does the row of the tab in front already, asking nothing; and a Favourite's tile. A tab closed and a Group shut in it are asked of the core, and it stays; "New Tab" opens one, and it goes; Done puts it away and asks nothing. A row's own menu works in it, and the page of a Group is over it, with the switcher there again after |
| Android's pictures | The switcher at a phone's width, new, light and dark, looked at. The 56 pictures of a phone's window recorded again: every one compared with what it was, and they differ at the end of the toolbar and nowhere else |
| The Android app on the emulator (`AppSwitcherTest`, 1, the real core and the real WebView) | The button says one tab, then two; both are rows of the switcher; the first pressed comes in front and the switcher goes; the system's Back puts it away, and its own button, with the page and its tab as they were; a tab closed from its row leaves it open, and the button then says one |
| The Android shell broken, a part at a time | 14 parts: 13 caught at once; the fourteenth showed that the switcher was shut in two places for a window with room for a sidebar, and one of the two was taken out |
| Found by the whole device suite | Android's accessibility checks: the window under the switcher (above). And the app's own test took a page's title to say nobody was signed in, which is so only when no test before it has signed in: it goes to a page that says nothing of that |

## 9. What is not verified

- **The choices of §6** are the recommendations, not the owner's word.
- **An iPad made narrow** (beside another app): it is the iPhone's code, at a width the iPad then has, and nothing ran it. **A real phone.**
- **From the Space at the switcher's head to the list of Spaces:** on Apple's systems the switcher makes way for it as it does for the page of a Group, by the same lines; no test pressed it. On Android no test pressed it either.
- **A screen reader.** Apple's audit and Android's checks ran over the open switcher; nobody listened.
- **Apple's views were not broken on purpose:** that a row chosen puts the switcher away is seen by the app's test only, which was not run against a broken shell.
- **A long row:** the switcher's rows are made as they come into view, as the sidebar's are (043 §8a); not measured.
- **Windows:** its three strings are in resources that were never built on a PC.

## 10. Done checklist

- [ ] The owner's word on §6.
- [x] The core's number.
- [x] The Apple shell.
- [x] The Android shell.
- [x] `Tools/test.sh all` green, and `android-device`, on 2026-11-23: `lint`, `core` (629, and the strings tool's 24), `unit` (299 on the Mac), `build`, the Mac's UI suite (47), `ios` (281 on the iPhone simulator), the iPhone's UI suite (37), `android` (432 and 32 on the JVM) and `android-device` (166 on the emulator).

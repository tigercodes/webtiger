# 047 — The phone's switcher (W2)

| | |
|---|---|
| **Status** | **Designed; nothing built.** The choices of §6 are put to the owner |
| **Phase** | W2 of [000](000-master-plan.md) §9: "the phone switcher". 000 §6.4: "The tab switcher is a full-height sheet. The layout setting chooses its form: the same tree as the sidebar, or cards" |
| **Contract** | To be 1.38 (§5): one number. The switcher itself draws the row a window is told already |
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

## 5. The contract (to be 1.38)

Added; nothing is changed or removed. The switcher draws what a window is told already. Its button says how many tabs the row has (D4), which is the core's to count: a shell does not work out which of a shut Group's Sites are among the tabs it was told of.

```proto
message WindowSnapshot { … uint32 row_tabs = 14; }   // every tab of the row, those out of the way among them
```

## 6. Decisions

**All of these are put to the owner; none is built.** The first column is the recommendation.

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

To be filled in as built. The bar is the sidebar's: against a scripted core, that every row does what it does in the sidebar; pictures at a phone's width, looked at; in the apps, that a tab is brought in front from the switcher and nothing is loaded for opening it.

## 9. What is not verified

To be filled in as built.

## 10. Done checklist

- [ ] The owner's word on §6.
- [ ] The Apple shell.
- [ ] The Android shell.
- [ ] `Tools/test.sh all` green, and `android-device`.

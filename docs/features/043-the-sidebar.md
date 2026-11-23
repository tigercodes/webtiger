# 043 — The sidebar: tabs down the side (W2)

| | |
|---|---|
| **Status** | **Parts 1 and 2 are built and tested, on the recommendations of §6, which are provisional:** they are put to the owner and not yet answered. The core has a window's layout; the Mac draws the sidebar, and so do a touch screen's wide windows (an iPad's; Android's), with a button to switch. **Run at a tablet's width on the iPad simulator and in Android's JVM tests; on no tablet** (§9). The other side, a width of one's own, collapsing and dragging are not built (§7) |
| **Phase** | W2 of [000](000-master-plan.md) §9: "both layouts". 000 §6.1: "Two layouts, one model" |
| **Contract** | 1.33 (§5). No change to the store's schema: the choice is one of the app's settings (020) |
| **Builds on** | [039](039-sites.md), [041](041-groups.md), [042](042-favourites.md): the row as a tree, which a strip draws along the top and a sidebar draws down the side. [034](034-the-macs-menus.md): the Mac's menus |
| **Last updated** | 2026-11-22 |

## 1. User story

I want my tabs down the side of the window, where there is room for their names: my Favourites as tiles at the top, my Groups as folders with their Sites under them, and under a line the tabs I am only passing through. When I change my mind and put them back along the top, nothing has moved.

## 2. What is in it, and what is not

**In (part 1):** a window has a layout, the strip or the sidebar; switching it changes nothing else; a new window has the layout that was last chosen, also after a quit; on the Mac the sidebar is drawn, with everything the strip can do.

**Not in, each with where it goes:**

- **The iPad and Android's wide windows:** part 2, the same sidebar on each.
- **Phones.** A phone's window is too narrow for a sidebar beside a page. 000 §6.4 gives a phone a switcher of its own, a sheet whose form the layout chooses (the sidebar's tree, or cards): it is not built, and until it is a phone has the strip.
- **The other side, a width of the person's choosing, and the sidebar collapsed** to a rail or hidden until the pointer reaches the edge (000 §6.1): part 3.
- **Dragging** in the tree, which is what the sidebar has room for: with part 3.
- **Spaces as icons at the foot.** Spaces have no icons yet: the chip that names the Space stands at the foot.
- **The first run's two styles** (000 §6.3), of which the layout is half.

## 3. The rule

**A window has a layout:** the strip, or the sidebar. It is a view setting: the row, its order, the tab in front, what is asleep and what a shut Group hides are the same in both, and switching asks the core for nothing but the switch.

**Switching** (`SetLayout`) changes the window's layout and is remembered as the app's choice: a window opened afterwards has it, and so has the first window after a quit. Each window keeps its own while it is open: switching one does not switch another.

**Where no sidebar can be drawn** (a phone; a shell that has none yet), the shell draws the strip whatever the layout says, and offers no switch.

**Why the core.** The choice outlives the window and the run, and settings are the core's (020); and a layout that every shell kept for itself would be a model in each shell.

## 4. What the sidebar shows (the Mac)

Down the leading side of the window, as wide as a name needs, the page beside it at the window's full height:

| From the top | What |
|---|---|
| The toolbar | Back, forward and reload, and under them the address field, at the sidebar's width; what is offered while typing opens under it, over the sidebar and the page |
| Favourites | A row of tiles, wrapping to a second row when there are many |
| The tree | The Sites that are in no Group, each a row with its mark and its name; each Group, a row with its folder, its name and, shut, how many it holds, pressed to shut or open it; under an open Group its Sites, set in |
| A line | |
| The other tabs | Each a row with its name and the button that closes it; then "New Tab" |
| The foot | The chip that names the Space, and the buttons for Find, Zoom and History |

Every row does what its chip does in the strip: a press brings a tab in front; the same menus; the same words to a screen reader. The order is the core's, and the shell places the Groups' rows as the strip places their chips (041 §5).

**As built on the Mac:**

- **The page keeps its place.** The sidebar is drawn beside the page, and the strip and its toolbar above it, without the page being drawn again: a switch reloads nothing.
- **What is offered while typing** opens at the top of the page, beside the address field and not under it: the field is in the sidebar now, and the list is as wide as the page.
- **A Group is its chip**, as in the strip, with its Sites set in under it; a right-click on it has its menu. A tab's own commands are in the menu bar, as they are for the strip (033: a button on the Mac takes a right-click for itself).
- **A right-click beside the rows** offers the tab that was last closed back, as beside the tabs of the strip.
- **The sidebar is 260 points wide**, at the leading side.

## 5. The contract (1.33)

Added; nothing is changed or removed.

```proto
enum Layout {
  LAYOUT_UNSPECIFIED = 0;   // a core from before 1.33: the strip
  LAYOUT_STRIP = 1;
  LAYOUT_SIDEBAR = 2;
}

// Answered with Ack; "window.unknown"; or "layout.unknown".
message SetLayout { uint64 window_id = 1; Layout layout = 2; }

message WindowSnapshot { … Layout layout = 12; }
```

**As built, where the design did not say:**

- **A private window** opens with the layout last chosen, as any window. What is chosen in one is its own: it is not the next window's, and it is not written (037).
- **A switch to the layout a window has** is not refused and tells nobody anything.
- **What is kept that this core does not know** (a later build's layout, in a store brought back) is the strip.
- **In fake mode** the choice lasts the run: the store is in memory.

## 6. Decisions

**All of these are put to the owner; none is built.** The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | The layout is each window's, and a new window has the one last chosen | 000 §6.1: "a view setting, per window". The last choice is what a person expects of the next window | One layout for the whole app, switched everywhere at once |
| D2 | The strip is what a new installation has | It is what every browser has, and the first run's choice of style (000 §6.3) is not built: until it is, nobody is surprised | The sidebar first, as Arc |
| D3 | The Mac first, then the iPad and Android's wide windows; a phone has no sidebar | One shell proves the model is one; a phone has its own switcher in the plan | All shells at once |
| D4 | The sidebar on the leading side, at one width | The simplest sidebar that is useful; the other side, resizing and collapsing are part 3 | Resizable from the start |
| D5 | The toolbar and the address field at the top of the sidebar | 000 §6.1's table; it gives the page the window's whole height | Keep the toolbar across the top of the window |
| D6 | The command: "Tabs at the Side", and "Tabs at the Top" when they are at the side; View menu, ⌃⌘S | ⌃⌘S is what shows and hides a sidebar in the Mac's own apps; the name says what happens | "Show Sidebar" / "Hide Sidebar", which would suggest that the tabs go away |
| D7 | (Part 2) On a touch screen the switch is a button among the window's buttons, beside Find, Zoom and History, on a window wide enough for a sidebar; a phone has none | An iPad and a tablet have no menu bar to look in; the button is where the other things about the window are. The Mac keeps to its menu, whose bar is always there | The switch in the list of Spaces; or in a settings page, when there is one |

## 7. In what order

1. **The core, the store and the contract:** the layout, the switch, and that nothing else changes.
2. **The Mac's sidebar.**
3. **The iPad's, and Android's wide windows'.**
4. **The other side, width, collapsing, dragging.**

### Part 2: a touch screen's wide windows

- **Which windows.** A sidebar is drawn where the window is wide enough for one beside a page: by the window's width, never by the device (000 §6.5). On Apple's systems that is a window the system classes as regular in width; on Android one of 600 dp or more. An iPad's window in a narrow split, a phone's, and a folded phone's, have the strip, and no switch; made wide again, a window has what the core says, without asking.
- **The same sidebar.** The iPad draws the one the Mac draws (it is the same view), with a tab's own menu by a long press, as in the strip. Android draws its own in Compose, from the same two lists, with the same rows, menus and words, 280 dp wide.
- **The switch** (D7): a button before Find, Zoom and History, which names what it does ("Tabs at the Side", "Tabs at the Top"). With a keyboard, ⌃⌘S on an iPad and Ctrl+Shift+S on Android (not Ctrl+Alt, which types a third letter on many keyboards).
- **The foot** of the sidebar is one row where the Space and the buttons fit side by side, and two where they do not: a touch screen's buttons are a finger wide. (As first drawn it was one row everywhere, wider than the sidebar on a touch screen, and pushed everything in the sidebar over its leading edge: seen in the first picture of it.)

## 8. Tests

The plan's own bar for W2: "switching layout changes nothing in the model".

| Where | What |
|---|---|
| The core, through the contract (`layout.rs`, 5) | **Nothing else changes:** a window with a row of every kind of thing (a Favourite, a Site in no Group, an open Group with a Site asleep, a shut Group with its Site left out, a tab in front) is switched to the side and back: what the window is told is, but for its layout, what it was told, and the shell is asked for nothing; asked for the layout it has, it is as it is; a tab closed at the side is closed. **Refused:** a window there is not; no layout named; a layout this core does not have; none of it is taken for a choice. **Each window its own:** switching one does not switch another; a window opened afterwards has the one last chosen, and the open ones keep theirs; a private window opens with it, and what is chosen there is not the next window's. **Across a quit** (a store on disk): a new installation has the strip; the choice is the first window's after a quit, with its tabs; a private window's is not written; back along the top and quit. **A store that names a layout nobody has** is the strip |
| The rule broken, a part at a time | 16 parts, in the model, in what is written and read, and where the app starts: all caught |
| The Apple shell's model (`LayoutModelTests`, 2, on the Mac and the iPhone simulator) | The switch asks the core for the other layout, and nothing before the window is open; the window draws the sidebar when the core says it has it, not when it is asked for; a core that names no layout is the strip. On the iPhone: there is no sidebar, none is asked for, and the strip is drawn whatever the core says. The sidebar's three parts (the Favourites, the tree, the other tabs) are the row taken apart, in the core's order within each, every tab once |
| The strip, after its parts were moved out to be shared | Every picture of every state of the window is unchanged, on the Mac |
| The Apple shell's pictures | The window with its tabs at the side (two Favourites, a Site, an open Group with the tab in front and a Site asleep, a shut Group, a line, two tabs, "New Tab", the Space at the foot), light and dark, on the Mac; and the same state on the iPhone simulator, where it is the strip: 4 pictures, each looked at |
| The Mac app (`SidebarTests`, 1, with the real core and the real engine) | Of two tabs, "Tabs at the Side" from the View menu: the sidebar is there and the strip is not, the same tabs by the same names, the same one in front, and nothing loaded again; Apple's accessibility audit; a row pressed brings its tab in front; the address field at the side sends the tab somewhere; "New Tab" opens one and the button at its row's end closes it; the command names the way back, and ⌃⌘S takes the tabs to the top, as they are, and to the side again |
| Android (`GroupsTest`, 1 more) | A window whose core names the sidebar has its strip, its tabs and its Groups all the same, and asks the core nothing about the layout |
| Part 2, the Apple shell's model (`LayoutModelTests`, now 3) | Whether a window has room for a sidebar is the window's to say: a Mac's has from the start, a touch screen's when it says so. With room, the switch asks and the sidebar is drawn when the core says so; with none, the strip is drawn whatever the core says and nothing is asked; made wide, the window has what the core says without asking |
| Part 2, the Apple shell's pictures | The sidebar on a wide touch-screen window (the iPhone simulator, told that the window is wide), light and dark: 2 pictures, each looked at, in place of the two of part 1 that showed the strip there. Every other picture is unchanged. (A first attempt cleared the size class of every state, and 42 pictures changed: the Space's chip had its whole name where a phone has its initial.) |
| Part 2, the apps (`SidebarTests`, on the Mac, the iPhone simulator and the iPad simulator) | On the iPhone: the window has no switch, and the strip. On the iPad simulator (an iPad Pro 11-inch's, run once by hand: it is not the gate's simulator): the window's button says "Tabs at the Side" and takes them there, with the same tabs, the same one in front and nothing loaded again; Apple's accessibility audit; a row, the address field, "New Tab" and the button at a row's end as on the Mac; the button says "Tabs at the Top" and takes them back. (At its first run there the test counted the keyboard's Tab key among the tabs: the test's mistake, and it now counts the tabs of the app's own window.) |
| The apps across a quit (`QuitTests`, 1, on the Mac and the iPhone simulator: the app itself on a profile of the suite's own, a store on disk) | A row arranged from nothing: a Favourite, a Group "Work" that holds a Site and is shut, and a tab in front; on the Mac, the tabs at the side. The app is ended without a word and started again: the tab that was in front is in front and loaded; the Favourite is first, a tile with its name, asleep; the Group is there by its name, shut, saying it holds one Site; on the Mac the tabs are at the side, on the iPhone along the top. Opened, the Group shows its Site, asleep, and nothing is loaded for it; pressed, the Favourite wakes with its page, and the Site at its home |
| Part 2, Android on the JVM (`SidebarTest`, 6; `KeysTest`) | The switch against a scripted core, and that a window with no room asks for none; the key (Ctrl+Shift+S, and not with Alt) and what its command does; the sidebar's three parts. Drawn by Compose at a tablet's width: the sidebar when the core says so, with every tab, the Groups' chips, the tiles and the address field, the Favourites above the tree above the other tabs and a Group's Site set in; a row's press and its button; the switch by its name; and the strip when the core says that, with the switch. At a phone's width: the strip whatever the core says, and no switch |
| Part 2, Android's pictures | The sidebar at a tablet's width, light and dark; the same state at a phone's width, where it is the strip; and the two older pictures of a wide window, which now have the switch among their buttons: 8 pictures, each looked at |
| Part 2 broken, a part at a time | Android, 16 parts (where a sidebar is drawn, the width a window says, the switch and its name, the parts, the setting in, the key): all caught, one once the key's command had a test of its own |
| The Mac's part broken, a part at a time | 9 parts of the switch, of where a sidebar is drawn and of the sidebar's three parts: all caught by `LayoutModelTests` (one once it was written so that it compiled). That a phone draws no sidebar is the same test's other half, run on the iPhone simulator |

## 8a. Measured: a long row

The plan's budgets (000 §7) name a sidebar of 5,000 rows, "no dropped frame at 120 Hz", and "a contract round trip for a 200-row sidebar page" of 2 ms at p95. Frames cannot be counted on this machine by a test. What can be measured was, on this Mac (Apple M6), on 2026-11-22.

**The core** (`row_cost.rs`, a release build; a store on disk; of every six tabs five are Sites, in ten Groups and none). "A change" is a Group shut or opened: the core changes its model, works out what the window is told and what the session keeps, and writes the Groups.

| Rows | The window opens, with its row | What a window is told, as the contract carries it | A change, and the window told of it (median; p95) | Written; read again (median) |
|---|---|---|---|---|
| 240 (the standard session) | 1.5 ms | 22 KB | 0.19 ms; 0.26 ms | 4 µs; 17 µs |
| 1,000 | 2.7 ms | 93 KB | 0.65 ms; 0.70 ms | 18 µs; 69 µs |
| 5,000 | 14 ms | 474 KB | 3.7 ms; 4.0 ms | 88 µs; 351 µs |

- **The standard session is well inside the 2 ms.**
- **At 5,000 rows it is not:** each change costs the core about 4 ms and tells the window all 474 KB again, which is half of a frame at 120 Hz before a shell has drawn anything. The cost is by the row: the window is told the whole row whatever changed, and the session is worked out whole to see whether it changed. Telling a window what changed, and no more, is the way to the budget; it is W3's ("the budgets"), and is not built.
- **One thing was mended on the way:** each Group's count of its Sites went over the whole row, once for every Group, each time a window was told anything. They are counted in one pass now (4.3 ms to 3.7 ms at 5,000 rows and ten Groups).

**The Mac's sidebar** (`SidebarCostTests`, the package's tests as they are built for testing, not a release build): the time for a window with its tabs at the side to be drawn and to settle. About 2.1 s of each figure is the test's own settling, as the 240 rows show.

| Rows | Every row made | Rows made as they come into view |
|---|---|---|
| 240 | 2.3 s | 2.1 s |
| 1,000 | 2.9 s | 2.3 s |
| 5,000 | 8.8 s | 4.2 s |

So the rows are now made as they come into view, on the Mac and the iPad (one view) and on Android. On Android that is a test and not a measurement: a row of 5,000 makes fewer than a hundred rows, and the tab that comes in front, wherever it is in the row, is brought into view (`SidebarTest`, with three parts of that broken in turn and caught, two once the test had a plain tab far down the row come in front).

**Not measured:** frames while scrolling, on any system; a release build of an app; a phone or a tablet; the strip, whose tabs are all made still.

## 9. What is not verified

- **A tablet.** The iPad's sidebar ran on the iPad simulator; Android's in JVM tests at a tablet's width, and never on a device or an emulator at that width (the emulator here is a phone's). No app was run on a tablet.
- **A window made narrow and wide again** (an iPad's in a split; a phone unfolded): the model's test; no app test changes a window's width.
- **A phone's switcher** (000 §6.4): not built. A phone has the strip.
- **The choice across a quit, in an app:** tested on the Mac (`QuitTests`, §8), where the app itself is ended and started again on a store on disk. On a touch screen it is not: the iPhone has no switch, and the iPad simulator is not the gate's.
- **A second window** with the layout last chosen is the core's test too: no app test opens one at the side.
- **By the keyboard alone, in the sidebar:** as in the strip. Rows are buttons; a Group's chip is not a stop for the Tab key (041 §9).
- **Many rows.** The plan's bar for W2, "5,000 rows scroll within budget", is measured in part (§8a): the core's cost, which at 5,000 rows is over the budget, and the Mac's time to draw. No frame was counted.
- **No screen reader was run.**
- **The choices of §6** are the recommendations, not the owner's word.

## 10. Done checklist

- [ ] The owner's word on §6.
- [x] The core and the contract.
- [x] The Mac's sidebar.
- [x] Part 1: `Tools/test.sh all` green, and `android-device`, on 2026-11-22, over the core and the Mac's sidebar together: `lint`, `core` (596, and the strings tool's 24), `unit` (291 on the Mac), `build`, the Mac's UI suite (44), `ios` (273 on the iPhone simulator, one skipped as before), the iPhone's UI suite (33), `android` (402 and 32 on the JVM) and `android-device` (159 on the emulator).
- [x] Part 2 (a touch screen's wide windows): the same again, on 2026-11-22: `lint`, `core` (596, and the strings tool's 24), `unit` (292 on the Mac), `build`, the Mac's UI suite (44), `ios` (274 on the iPhone simulator, one skipped as before), the iPhone's UI suite (34), `android` (410 and 32 on the JVM) and `android-device` (159 on the emulator); and the sidebar's app test once on the iPad simulator, which is not part of the gate.
- [x] A long row measured, and the sidebars' rows made as they come into view (§8a): the same again, on 2026-11-22: `lint`, `core` (596, and the strings tool's 24), `unit` (293 on the Mac, one of them the measurement, which runs only when asked), `build`, the Mac's UI suite (44), `ios` (275 on the iPhone simulator, two skipped: one as before, and the measurement), the iPhone's UI suite (34), `android` (411 and 32 on the JVM) and `android-device` (159 on the emulator).
- [x] The tree and the layout across a quit, in the apps (`QuitTests`, §8): the same again, on 2026-11-22: `lint`, `core` (602, and the strings tool's 24), `unit` (293 on the Mac), `build`, the Mac's UI suite (45), `ios` (275 on the iPhone simulator), the iPhone's UI suite (35), `android` (411 and 32 on the JVM) and `android-device` (159 on the emulator).

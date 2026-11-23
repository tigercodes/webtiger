# 043 — The sidebar: tabs down the side (W2)

| | |
|---|---|
| **Status** | **Part 1 is built and tested, on the recommendations of §6, which are provisional:** they are put to the owner and not yet answered. The core has a window's layout; the Mac draws the sidebar. The iPad's and Android's, the other side, a width of one's own, collapsing and dragging are not built (§7) |
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

### Part 2, as planned

- **Which windows.** A sidebar is drawn where the window is wide enough for one beside a page: by the window's size class, never by the device (000 §6.5). An iPad's window in a narrow split, and a folded phone, have the strip; made wide again, they have what the core says.
- **The same sidebar.** The iPad draws the one the Mac draws (it is the same view). Android draws its own in Compose, from the same two lists, with the same rows, menus and words.
- **The switch** (D7), and with a keyboard the Mac's key on an iPad and Ctrl+Alt+S on Android.
- **Tests:** the models' (a narrow window draws the strip and offers no switch; a wide one both); pictures at a tablet's width on each; the adapter of each app at that width, where the simulator and the emulator have one.

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
| The Mac's part broken, a part at a time | 9 parts of the switch, of where a sidebar is drawn and of the sidebar's three parts: all caught by `LayoutModelTests` (one once it was written so that it compiled). That a phone draws no sidebar is the same test's other half, run on the iPhone simulator |

## 9. What is not verified

- **The iPad, Android's wide windows, a phone's switcher:** not built. They are told the layout (1.33) and draw the strip.
- **The choice across a quit, in an app.** The app's tests run in fake mode, where the store is in memory: that the choice is the first window's after a quit is the core's test, over a store on disk.
- **A second window** with the layout last chosen is the core's test too: no app test opens one at the side.
- **By the keyboard alone, in the sidebar:** as in the strip. Rows are buttons; a Group's chip is not a stop for the Tab key (041 §9).
- **Many rows.** The plan's bar for W2, "5,000 rows scroll within budget", is not measured: the rows are in a plain column, drawn all at once.
- **No screen reader was run.**
- **The choices of §6** are the recommendations, not the owner's word.

## 10. Done checklist

- [ ] The owner's word on §6.
- [x] The core and the contract.
- [x] The Mac's sidebar.
- [x] Part 1: `Tools/test.sh all` green, and `android-device`, on 2026-11-22, over the core and the Mac's sidebar together: `lint`, `core` (596, and the strings tool's 24), `unit` (291 on the Mac), `build`, the Mac's UI suite (44), `ios` (273 on the iPhone simulator, one skipped as before), the iPhone's UI suite (33), `android` (402 and 32 on the JVM) and `android-device` (159 on the emulator).

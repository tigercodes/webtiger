# 043 — The sidebar: tabs down the side (W2)

| | |
|---|---|
| **Status** | **Designed; nothing built.** The choices of §6 are put to the owner |
| **Phase** | W2 of [000](000-master-plan.md) §9: "both layouts". 000 §6.1: "Two layouts, one model" |
| **Contract** | To be 1.33 (§5) |
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

## 5. The contract (to be 1.33)

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

## 7. In what order

1. **The core, the store and the contract:** the layout, the switch, and that nothing else changes.
2. **The Mac's sidebar.**
3. **The iPad's, and Android's wide windows'.**
4. **The other side, width, collapsing, dragging.**

## 8. Tests

To be filled in as built. The plan's own bar for W2: "switching layout changes nothing in the model".

## 9. What is not verified

To be filled in as built.

## 10. Done checklist

- [ ] The owner's word on §6.
- [ ] The core, the store and the contract.
- [ ] The Mac's sidebar.
- [ ] `Tools/test.sh all` green, and `android-device`.

# 025 — A Space has a colour (W2)

| | |
|---|---|
| **Status** | In progress. The store and the core are built and tested (§3 to §5): a new Space gets a colour nobody has, a colour is chosen and changed, and the list and the window's snapshot say it. No shell shows a colour yet |
| **Phase** | W2 of [000](000-master-plan.md) §9: "Spaces with their login sets, colours and icons". The colour; the icon is its own feature |
| **Contract** | 1.19: `SpaceSummary.colour`, `SpacesPage.colours`, `WindowSnapshot.space_colour`, `CreateSpace.colour`, `ColourSpace` |
| **Builds on** | [024](024-several-spaces.md): several Spaces, the chip, the list |
| **Last updated** | 2026-11-20 |

## 1. User story

I have a Space for work and one for home, and I do not want to sign in to the wrong one. Each has a colour, and the window shows the colour of the Space it shows: before I read a name, I know where I am. On my phone, where the chip is only the Space's initial (024 §6.1), "Work" and "Web" no longer look the same.

A new Space has a colour nobody else has, without my choosing. I can give it another.

## 2. What is in it, and what is not

**In:** a Space has one of a short list of colours, or none; the core gives a new Space one that no Space has; a colour is chosen when a Space is made, and changed after; the window's snapshot and the list of Spaces say it; the shells show it, as a mark beside the name and never as the only way a Space is told apart.

**Not in:**

- **A Space's icon.** Its own feature.
- **Tinting the whole window** (000 §6.7: "A Space's colour tints its window"). That is the design system's work, with the palette that is not settled (001 F10). Here the colour is a mark in the chip and in the list.
- **Colours of a person's own choosing.** A short list that every shell can draw, in light and in dark, with enough contrast.

## 3. The rule

**The colours** are a list the core has, in an order: `red`, `orange`, `yellow`, `green`, `teal`, `blue`, `purple`, `pink`, `grey`. A colour crosses the contract by that name, which is a code like any other: the shells have the paint for it (a design token, light and dark) and the word (for a screen reader, and under a swatch). A shell that has no paint for a name a newer core says shows the Space with no colour: a name it does not know is never an error.

**A Space has one of them, or none.** The Space the app made has none until somebody gives it one, and so has every Space made before there were colours.

**A new Space** (`CreateSpace`) is given the colour asked for; or, with none asked for, **the first in the list that no Space has**. When every colour is somebody's, it is the one the fewest Spaces have, the first of those in the list. So two Spaces are told apart without anybody choosing, for as long as there are colours to go round.

**Changing it** (`ColourSpace`) gives a Space a colour of the list, or none (the empty name). A name that is not in the list is refused (`space.colour_unknown`), in a new Space too, and nothing is made or changed. A Space that is not there is refused (`space.unknown`).

**Who is told.** The list of Spaces says each one's colour, and the colours there are to choose among. A window's snapshot says the colour of the Space it shows, and is published again when that changes.

## 4. The contract (1.19)

Added; nothing is changed or removed.

```proto
message SpaceSummary {
  ...
  string colour = 4;                 // one of SpacesPage.colours, or empty: none
}

message SpacesPage {
  ...
  repeated string colours = 2;       // the colours a Space can have, in their order
}

message WindowSnapshot {
  ...
  string space_colour = 7;           // the colour of the Space the window shows, or empty
}

message CreateSpace {
  ...
  string colour = 3;                 // empty: the core chooses one no Space has
}

message ColourSpace {                // answered with Ack, "space.unknown" or "space.colour_unknown"
  uint64 space_id = 1;
  string colour = 2;                 // empty: none
}
```

A colour's name is no more private than a search engine's: it is not redacted.

## 5. The store

Schema 8: `space.colour`, a name or empty. A store from before has every Space with none.

## 6. The shells

Written with each shell's part: a mark of the colour in the chip, beside the name or the initial; the same mark in each row of the list; and, where a Space is made and where it is named, the colours to choose among, each with its word. A mark has to be seen against the chip's own fill, in light and in dark: the paints are chosen for that, and measured.

## 7. Tests

| Where | What |
|---|---|
| Store (`wt-store/src/space.rs`, 2) | A Space has the colour it was made with, or none; it is given another, the app's own Space among them, and none again; a Space that is not there has none to change; it is all there when the store is opened again, and a colour changes nothing else of a Space. A store from before colours (schema 7) gains them: its Spaces are as they were, each with none, and can be given one. The tests that put a store back to an older version take the column away with the rest |
| The rule (`wt-core/src/space.rs`, 2 more) | A colour is one of the list by its own name, each there once: none, one written another way and one nobody has heard of are no colour. A new Space gets the first colour nobody has, whatever order they were taken in; a Space with none and a colour this build does not know take none; when every colour is somebody's, the first again, and then the one the fewest have, the first of those |
| Core (`wt-app/tests/space_colours.rs`, 6, through `FakeShell`) | The list says the colours there are, in their order, and the app's own Space has none, as its window says. Nine new Spaces get the nine colours in order, and the next two the first two again. A colour that is asked for is the one a Space gets, taken or not, and the next new Space skips it. A colour that is not in the list is refused, in a new Space and in a change, and nothing is made or changed; a Space that is not there is refused whatever colour is asked; no name is said before no colour. A Space is given another colour, the app's own among them, and the window that shows it says so; a window that shows another is told nothing, nor is anybody by the same colour again; none is given by the empty name, and a Space with none takes none from the next. Over a store on disk the colours are there when the app starts again, the window shows its Space in its colour, and the next new Space gets the first that nobody has. Twelve of the rules checked by breaking them: all caught |
| The shells | Written with each |

## 8. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | A short list of named colours, the core's | Every shell draws the same Space the same colour, and a shell can be given paints that are legible in both appearances. A free choice of colour is a contrast problem in every place it is drawn |
| D2 | A new Space gets a colour by itself | The point of a colour is to tell Spaces apart; one that waits to be chosen tells nothing apart for most people |
| D3 | The app's own Space has none until one is chosen | It was there before colours, and a colour that appeared on it by itself would be a change nobody made |
| D4 | The colour is a mark beside the name, never the name's background | Words on a colour have to pass contrast for every colour in both appearances (Apple's audit faulted white on the accent's blue, 024 §6.1); a mark only has to be seen |

## 9. What is not verified

Written when built.

## 10. Done checklist

- [x] The store: a Space's colour (schema 8; 2 tests).
- [x] Contract 1.19 and its version in every place it is written.
- [x] The core: the rule of §3 (6 tests, and 2 for the rule itself).
- [ ] The paints: design tokens, light and dark, measured against the chip's fill.
- [ ] The Apple shell.
- [ ] The Android shell (and the client names any new answer: none here).
- [ ] `Tools/test.sh all` green, and `android-device`.
- [ ] Windows shell.

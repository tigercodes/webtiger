# 025 — A Space has a colour (W2)

| | |
|---|---|
| **Status** | Built and tested in the core and in the Apple and Android shells: a new Space gets a colour nobody has, a colour is chosen when a Space is made and changed where it is edited, and the chip and the list show it as a mark. Tested in the apps on the iPhone simulator and the Android emulator; the Mac's app has not been run (§9). Windows is not started |
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

The same in the Apple shell and the Android shell.

- **The paints** are design tokens, light and dark, one for each colour (`spaceRed` … `spaceGrey` in `Design/tokens.json`). The generator makes the table from the core's names to them, and refuses a paint that does not stand out from what it is drawn on: 3 to 1 at the least (a mark that is not text, WCAG 1.4.11), against each window's background and against the chip's fill on it, in light and in dark. "Yellow" in the light appearance is a dark mustard for that reason. The palette is provisional, as every colour here is (001 F10).
- **The mark** is a dot of the paint: before the name in the chip, and before each name in the list, where a Space with none has a ring in its place so that the names stay in line. Where the chip is only the Space's initial (a narrow window, 024 §6.1) the mark is a short bar under the initial: it takes none of the strip's room. A Space with no colour has no mark in the chip, and the chip is as it was.
- **The colours to choose among** are swatches, one for each colour the core lists that the shell has a paint for, in the core's order: on the page that makes a Space, where choosing none leaves it to the core ("If you choose none, it gets one that no other Space has"); and on the page that edits the Space the window shows, where "None" is a choice too and the colour it has is the one chosen. The one chosen is ringed, and **named in words beside the heading** ("Colour: Blue"): it is never told by the colour alone. A swatch says its colour's word to a screen reader.
- **The page that named a Space edits it** (024 §6.3): "Rename…" is "Edit…", and what is saved is what changed, the name first: a name the core will not take leaves the colour as it was. The Space the app made may be given a colour and keep no name; a Space that has a name is not given none.
- **A colour the shell has no paint or word for**, from a newer core, is shown as no colour and offered nowhere: never its name's own text.

## 7. Tests

| Where | What |
|---|---|
| Store (`wt-store/src/space.rs`, 2) | A Space has the colour it was made with, or none; it is given another, the app's own Space among them, and none again; a Space that is not there has none to change; it is all there when the store is opened again, and a colour changes nothing else of a Space. A store from before colours (schema 7) gains them: its Spaces are as they were, each with none, and can be given one. The tests that put a store back to an older version take the column away with the rest |
| The rule (`wt-core/src/space.rs`, 2 more) | A colour is one of the list by its own name, each there once: none, one written another way and one nobody has heard of are no colour. A new Space gets the first colour nobody has, whatever order they were taken in; a Space with none and a colour this build does not know take none; when every colour is somebody's, the first again, and then the one the fewest have, the first of those |
| Core (`wt-app/tests/space_colours.rs`, 6, through `FakeShell`) | The list says the colours there are, in their order, and the app's own Space has none, as its window says. Nine new Spaces get the nine colours in order, and the next two the first two again. A colour that is asked for is the one a Space gets, taken or not, and the next new Space skips it. A colour that is not in the list is refused, in a new Space and in a change, and nothing is made or changed; a Space that is not there is refused whatever colour is asked; no name is said before no colour. A Space is given another colour, the app's own among them, and the window that shows it says so; a window that shows another is told nothing, nor is anybody by the same colour again; none is given by the empty name, and a Space with none takes none from the next. Over a store on disk the colours are there when the app starts again, the window shows its Space in its colour, and the next new Space gets the first that nobody has. Twelve of the rules checked by breaking them: all caught |
| The paints (`Tools/design/tokens.py`) | Every colour's paint stands out 3 to 1 from each window's background and from the chip's fill on it, light and dark: the generator measures it whenever the tokens are written or checked (`Tools/test.sh lint`), and refuses one that does not. Tried with a paler yellow: refused |
| The Apple shell's model (`SpacesModelTests`, 6 more, against a scripted core and one against the real core) | The window says its Space's colour, and the list the colours there are, in the core's order. A Space is made with the colour chosen, or with none for the core to choose. Editing asks for what changed, the name first, and for nothing when nothing did; the model colours nothing by itself. A name the core will not take leaves the colour unasked; a colour it will not take says why. Every colour has a word and a paint; no name, another name, and the strings that are not colours have neither. **Against the real core:** the colours it lists are the ones the shell has a paint and a word for, by the same names in the same order |
| The Android shell's window (`SpacesTest`, 6 more, drawn on the JVM against a scripted core; `ThroughTheCoreTest`, 1 more, against the real core) | The chip and the list have the mark of a Space's colour, a Space with none has none, a screen reader is told the name as before, and a colour the shell cannot paint is shown as none. A new Space is made with the colour chosen, which the heading then says in words, or with none; a colour the shell cannot paint is not offered, and "None" is not offered for a new Space. Editing shows the colour the Space has as the one chosen, and asks for what changed: the colour alone, the name and no colour, or nothing. The Space nobody named is given a colour without a name; a Space with a name is not given none. A name the core will not take leaves the colour unasked, and a colour it will not take says why in words. Every colour has a word and a paint. **Against the real core:** the colours it lists are the ones the shell has paints for |
| Pictures (004), on the Mac, the iPhone simulator and Android, light and dark | The list of Spaces with each one's mark, and a ring for the Space with none; the pages that make and edit a Space, with the colours to choose among, and on the page that edits one the colour it has ringed and named beside the heading; one state of the window with a Space that has a name and a colour: a dot before the name on the Mac and in Android's wide window, a bar under the initial on the iPhone and at Android's phone width; and the tokens, with the nine paints. 44 pictures changed and no new one; each was looked at. The pages had grown past their pictures' foot, where their buttons are: their pictures are taller now, and were looked at again |
| The apps (fake mode: the real core, and real WebKit or the real System WebView) | In the test that names a Space, on the iPhone simulator and on the Android emulator: the Space that was made has the first colour there is, said in words on the page that edits it ("Colour: Red"); green is chosen, the heading says so, and it is saved with the name; opened again, the page says "Colour: Green". Apple's audit and Android's accessibility checks pass over the page with the swatches on it. (The page that makes a Space is longer by the swatches, and on the iPhone its buttons are under the keyboard while the name is typed: Apple's audit measured the keyboard's pixels for their contrast and faulted them. What is under the keyboard is not drawn: the audit's helper now leaves contrast there alone, as it does for what is cut by the window's edge. The return key makes the Space, and the page scrolls.) **On the Mac the test is written and has not run** (024 §9) |

## 8. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | A short list of named colours, the core's | Every shell draws the same Space the same colour, and a shell can be given paints that are legible in both appearances. A free choice of colour is a contrast problem in every place it is drawn |
| D2 | A new Space gets a colour by itself | The point of a colour is to tell Spaces apart; one that waits to be chosen tells nothing apart for most people |
| D3 | The app's own Space has none until one is chosen | It was there before colours, and a colour that appeared on it by itself would be a change nobody made |
| D4 | The colour is a mark beside the name, never the name's background | Words on a colour have to pass contrast for every colour in both appearances (Apple's audit faulted white on the accent's blue, 024 §6.1); a mark only has to be seen |
| D5 | The one chosen is ringed and named in words, not ticked | A tick drawn on the paint has to be seen on every paint in both appearances; a ring in the colour of the words, and the word itself, do not depend on the paint |
| D6 | The strings for the colours' words are the only ones under `space.colour.` | A colour's word is found by its name, and a name must never find some other sentence: "none", "chosen" and "automatic" did, until a test asked for them |

## 9. What is not verified

- **The Mac's app**, as for all of 024: its UI suite has not run, and the same test passes on the iPhone simulator.
- **Whether the nine can be told apart by everybody.** They cannot: red and green, teal and blue, purple and pink are alike to some eyes, and no test stands in for those eyes. That is why the colour is never the only thing that says which Space it is: the name, or the initial, is always there, and the colour chosen is said in words.
- **The paints on a real screen.** The generator's arithmetic says each stands out 3 to 1 from the backgrounds the pictures have; a phone in sunlight was not tried. The palette is provisional (001 F10).
- **The mark at the largest text sizes.** It is a fixed size; the pictures are at the default text size.
- **The whole window tinted** (000 §6.7) is not built.
- **A newer core's colour that a shell cannot paint** is shown as none and offered nowhere: tested with a scripted core on Android and by the lookups on both; no real core says such a colour.

## 10. Done checklist

- [x] The store: a Space's colour (schema 8; 2 tests).
- [x] Contract 1.19 and its version in every place it is written.
- [x] The core: the rule of §3 (6 tests, and 2 for the rule itself).
- [x] The paints: design tokens, light and dark, measured against the chip's fill by the generator.
- [x] The Apple shell: model tests (one against the real core), pictures, and the app's test on the iPhone simulator. **The Mac's UI suite has not run.**
- [x] The Android shell (and the client names any new answer: none here): the window's tests, one against the real core, pictures, and the app's test on the emulator.
- [ ] `Tools/test.sh all` green, and `android-device`.
- [ ] Windows shell.

# 050 — A Space has an icon (W2)

| | |
|---|---|
| **Status** | **Built and tested in the core, the store, and the shells of Apple's systems and Android, on the recommendations of §6, which are provisional:** they are put to the owner and not yet answered. A Space is given an icon where it is made and where it is edited; the chip and the list of Spaces draw it. A Space has none until a person chooses one. The Windows shell has none (it is not built) |
| **Phase** | W2 of [000](000-master-plan.md) §9: "Spaces with their login sets, colours and icons". The last of the three. 000 §5.6: "Space. A named place with a colour and an icon" |
| **Contract** | 1.43 (§4). The store's schema is 15: a Space's icon |
| **Builds on** | [024](024-several-spaces.md): several Spaces, the chip, the list. [025](025-space-colours.md): a Space's colour, which this is made like. [002](002-arc-import.md): a Space that is brought from another browser has an icon, an emoji or a name |
| **Last updated** | 2026-11-23 |

## 1. User story

On my phone the chip that says which Space I am in is one letter. "Work" and "Web" are both a W; their colours tell them apart, and I have to remember which is which. I would like the chip to be a small picture I chose: a briefcase, a house. In the list of Spaces the picture is beside the name, and I find the Space I want before I have read anything.

## 2. What is in it, and what is not

**In:** a Space has one of a list of icons, or none; an icon is chosen when a Space is made, and changed where it is edited; the window's snapshot and the list of Spaces say it; the shells draw it: in the chip, in the place of the Space's initial, and beside the name in the list.

**Not in, each with where it goes:**

- **Any emoji at all, typed by the person.** The icons are a list the core has (D2). A Space brought from another browser has whatever emoji it had there ([002](002-arc-import.md) §5: "An emoji is kept as it is"): what the core takes is widened with the first run's import (000 §6.3), which is what brings one.
- **Icons drawn for WebTiger.** They are the system's own emoji (D1): there is no artwork to make, and none to keep the same on four systems.
- **An icon given to a new Space without asking,** as a colour is. A colour chosen for nobody tells two Spaces apart; a picture chosen for nobody says something about a Space that may not be so.
- **A Group's icon, a Site's own icon** (a Site has its site's, [046](046-a-sites-icon.md)).

## 3. The rule

**The icons** are a list the core has, in an order. Each is an emoji, and crosses the contract as it is: it is what is drawn, by every system's own font, and what a screen reader says by the name the system has for it. There are no words for it in the strings, and no paint. A shell that is told an icon it cannot draw (an older system's font) draws what the font draws for it; an icon is never an error in a shell.

The list: house, briefcase, books, backpack, shopping bags, money bag, aeroplane, puzzle piece, guitar, popcorn, newspaper, speech balloon, hammer and spanner, test tube, seedling, heart, star, lock, globe, camera, pizza, running shoe, dog, light bulb. **Each is one that is seen on a light ground and on a dark one.** The first list had a graduation cap, a game controller, musical notes, a clapper board and paw prints: black pictures, which the first pictures of a dark window showed to be all but lost on it; and a shopping trolley and a fork and knife, which are grey, and faint on one ground or the other.

**A Space has one of them, or none.** A Space is made with none unless one is asked for, and so is every Space made before there were icons. With none, the chip shows what it shows today: the Space's initial where there is no room for its name.

**Giving one** (`IconSpace`) gives a Space an icon of the list, or none (the empty string). One that is not in the list is refused (`space.icon_unknown`), in a new Space too (`CreateSpace.icon`), and nothing is made or changed. A Space that is not there is refused (`space.unknown`). A private window is no Space, and has none to give.

**Who is told.** The list of Spaces says each one's icon, and the icons there are to choose among. A window's snapshot says the icon of the Space it shows, and is published again when that changes.

**In the windows.** The chip: where it is the Space's initial (a compact width, 024 §6.1) it is the icon instead; where it is the Space's name, the icon stands before the name, after the mark of its colour. The chip is still said to a screen reader as "Space:" and the name: the icon is not the Space's name, and is not read in its place. The list of Spaces has the icon beside each name. The pages that make a Space and edit one have the icons to choose among, each a button to press, with "None" the first, as the colours are chosen (025 §6).

**Why the core.** Which icons there are, and which a Space has, are the model's and the store's: a Space's icon is the same in every window and on every system the Space is on, and with sync (W9) it travels.

**Nothing leaves the device**, and nothing of it is in a log: an icon is a person's choice about their own Space, and is redacted as a Space's name is.

## 4. The contract (1.43)

Added; nothing is changed or removed.

```proto
message SpaceSummary { … string icon = 6; }             // one of SpacesPage.icons, or empty: none. Redacted in logs
message SpacesPage { … repeated string icons = 4; }      // the icons a Space can have, in their order
message WindowSnapshot { … string space_icon = 15; }     // the icon of the Space the window shows, or empty. Redacted in logs
message CreateSpace { … string icon = 4; }               // one of the list, or empty: none; "space.icon_unknown"
message IconSpace { uint64 space_id = 1; string icon = 2; }   // request 55; answered with Ack; "space.unknown", "space.icon_unknown"
```

The store (schema 15): `space.icon`, the emoji, empty for none.

**As built, where the design did not say:**

- **An icon is one of the list exactly as it is written there:** the same characters. An emoji that the list has not, one of the list's without the selector it is written with, two of them, one with a space after it, and words are all refused.
- **A new Space's name is looked at first, then its colour, then its icon:** what is said of a Space that has two things wrong is what was said before there were icons.
- **In the list of Spaces every row keeps the icon's place where any Space has one,** so that the names stay in line; where none has, there is no place kept.
- **The icon is drawn and not read out,** in the chip and in the list: a row says its Space's name, and whose logins it shares.
- **An icon to choose is known to a test by its place in the core's list,** from one; none is `space-icon-none`. An emoji is no name for a button.
- **The page that edits a Space asks for its icon after its colour and before its time** (049).
- **On an iPhone the audit's rule for what is under the keyboard now counts the row of suggested words above the keys** (45 points): the page that makes a Space has the keyboard, and the heading "Icon" was under that row, where the audit measured the row's colours for the heading's.

## 5. In what order

1. **The core and the store:** the list, a Space's icon kept, given and refused; the snapshot and the list of Spaces. **Built.**
2. **The shells:** the chip, the list of Spaces, and the icons to choose among where a Space is made and edited; Apple's systems and Android. **Built.**

## 6. Decisions

**All of these are put to the owner.** They are built as recommended. The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | A Space's icon is an emoji, drawn by the system | Every system draws them, a person knows them, and a Space brought from another browser has one already (002). There is nothing to draw, to license, or to keep alike on four systems | Symbols drawn for WebTiger, by name: one look everywhere, and artwork to make for each of them on each system. Or each system's own symbol set (SF Symbols on Apple's, Material's on Android): no artwork on two systems, and the same name is a different picture on each |
| D2 | The icons are a list the core has, of about two dozen | A page can show them all, and none has to be typed or searched for. Every one is known to draw on the systems WebTiger runs on | Any emoji, from the system's own picker: every person finds theirs, and the core must tell an emoji from other text (a table of them, or a library that has one) |
| D3 | A Space is made with no icon unless one is chosen | A picture nobody chose says something about the Space that may not be so; a colour does not | The first icon no Space has, as a colour is given |
| D4 | In the chip the icon takes the place of the initial; beside a name it stands before it | The initial is what there was no better thing than; the icon is the better thing. A name is still a name | The icon and the initial both; or the icon only, with no name, at every width |
| D5 | The chip is said to a screen reader by the Space's name, not its icon | "Space: Work" is what the chip is for. An emoji's name ("briefcase") beside it adds a word that is not the Space's | The icon read with the name |

## 7. Tests

| Where | What |
|---|---|
| The core's rules (`space.rs`, +1) | The icons are twenty-four, each once, each an emoji and nothing of ASCII, a house the first; an icon is one of the list as it is written there, and nothing else is: none, another emoji, one without its selector, two, one with a space, words |
| The store (`space.rs`, +2) | A Space has the icon it was made with, and another when given one, the app's own Space among them; none again; a Space that is not there has none to change; kept over a start, and given with the first Space; an icon changes nothing else of a Space. A store from before icons gains them and keeps its Spaces, each with none |
| The core, through the contract (`space_icons.rs`, 5) | The list says the icons there are, and a Space has none until it is given one, however many are made (it has its colour all the same). A Space is made with the icon asked for, two may have the same, and every icon of the list is one a Space can be made with. One that is not in the list is refused, in a new Space and in one there is, and nothing is made or changed; a Space that is not there; a name and a colour that are wrong are said first; a private window says none. A Space is given another or none: the windows that show it are told, both of two; one that does not is told nothing, and says it when it shows that Space; the same icon again says nothing. The icons are there when the app starts again |
| The core broken, a part at a time | 21 parts. 19 caught. Two were not: that every window is told when any Space is given an icon, and that the windows are told again when the icon is the one it has. Neither can be seen: a window is not sent again what it was last sent |
| The Apple shell's models and views (`SpaceIconModelTests`, 9) | The window says its Space's icon, and the list the icons there are; a Space is made with the icon chosen or with none; editing asks for the icon after the colour and before the time; one the core will not take says why in words, and the time is not asked; before the core has said which Space, nothing is asked. With the real core: its twenty-four, each one picture, each taken and said back, and another refused. What the pages offer: none, then the core's; nothing where it lists none. **Drawn, by counting pixels that are no grey:** the chip has the icon in the place of the initial and before the name; the list has it in the Space's row; the page that edits a Space draws the icons the core lists, and none where it lists none |
| Pictures (the Spaces' five states, in both appearances, on the Mac, the iPhone simulator and Android) | Recorded again, each looked at: the list with two Spaces' icons and the names in line; the pages with the icons. They are taller than they were, so that a page is seen to its buttons |
| The apps, in fake mode, with the real core and the real engine (`SpaceIconTests`, 1, on the Mac and the iPhone simulator) | The page that makes a Space has none and the core's twenty-four, none chosen; a Space is made with the second, and the chip says the Space's name; the page that edits it says the icon it was made with; it is given another, which it has when the page opens again; and none. Apple's audit of the page |
| The Android shell, against a scripted core (`SpaceIconsTest`, 8) | The chip is the icon in the place of the initial in a narrow window, and has it before the name in a wide one; it is said by the Space's name; another icon, and none. The list has a Space's icon beside its name, keeps the place in the other rows, reads neither out, and keeps no place where no Space has one. The page that edits offers the core's icons in its order with the Space's own chosen, and asks for another after the colour and before the time, and for nothing when it is the one it has. A Space is made with the icon chosen or with none. A refusal in words. A core that lists none has no choice |
| The Android shell over the real core, on the JVM (`ThroughTheCoreTest`, +1) | The window's model gives the Space the first icon the real core lists, and the window says it; one it did not list is refused by its code, and the Space keeps the one it has |
| Android, on the emulator (`AppSpaceIconTest`, 1) | The real app: the page that edits the Space has none and twenty-four; the second is chosen and saved; the chip draws the very one the page drew, and is said by the Space's name; given none again, the chip has no icon. With the accessibility checks |
| The shells broken, a part at a time | Apple: 15 parts, 15 caught (one once what the pages offer was a thing a test could hold). Android: 23 parts, 23 caught |

## 8. What is not verified

- **The choices of §6** are the recommendations, not the owner's word.
- **The Windows shell** has no icons: it is not built.
- **How each icon looks on a real phone.** The pictures are the Mac's, the iPhone simulator's and the JVM's drawing of Android's font: on each, every icon is seen on a light ground and on a dark one. A phone with another maker's emoji font was not looked at.
- **An older system that has not got one of them** (the test tube and the puzzle piece are of 2018): it draws what its font draws for a character it has not, which is an empty box. The systems WebTiger supports all have them; none older was run.
- **A screen reader** was not listened to. The chip and the rows are said by the Space's name; each icon to choose is an emoji, which the system names itself, and that it does was not heard.
- **A Space brought from another browser** with an emoji the list has not: the import screen is not built, and the core would refuse it today (§2).
- **The largest text sizes:** an emoji grows with the text; the audit ran at the usual size.

## 8a. What the pictures changed

The first list of icons had five black pictures and two grey ones. On the first dark picture of the page they were all but lost. They were changed for ones with colour in them before anything was committed: the list in §3 is the second.

## 9. Done checklist

- [ ] The owner's word on §6.
- [x] The core, the store and the contract.
- [x] The shells: Apple's systems and Android.
- [ ] The gate, over the core and the shells together.

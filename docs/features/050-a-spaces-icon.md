# 050 — A Space has an icon (W2)

| | |
|---|---|
| **Status** | **Designed; nothing built.** The choices of §6 are put to the owner |
| **Phase** | W2 of [000](000-master-plan.md) §9: "Spaces with their login sets, colours and icons". The last of the three. 000 §5.6: "Space. A named place with a colour and an icon" |
| **Contract** | To be 1.43 (§4). The store's schema to be 15: a Space's icon |
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

The list, to be settled with the pictures in front of us: house, briefcase, books, graduation cap, shopping trolley, money bag, aeroplane, game controller, musical notes, clapper board, newspaper, speech balloon, hammer and spanner, test tube, seedling, heart, star, lock, globe, camera, fork and knife, running shoe, paw prints, light bulb.

**A Space has one of them, or none.** A Space is made with none unless one is asked for, and so is every Space made before there were icons. With none, the chip shows what it shows today: the Space's initial where there is no room for its name.

**Giving one** (`IconSpace`) gives a Space an icon of the list, or none (the empty string). One that is not in the list is refused (`space.icon_unknown`), in a new Space too (`CreateSpace.icon`), and nothing is made or changed. A Space that is not there is refused (`space.unknown`). A private window is no Space, and has none to give.

**Who is told.** The list of Spaces says each one's icon, and the icons there are to choose among. A window's snapshot says the icon of the Space it shows, and is published again when that changes.

**In the windows.** The chip: where it is the Space's initial (a compact width, 024 §6.1) it is the icon instead; where it is the Space's name, the icon stands before the name, after the mark of its colour. The chip is still said to a screen reader as "Space:" and the name: the icon is not the Space's name, and is not read in its place. The list of Spaces has the icon beside each name. The pages that make a Space and edit one have the icons to choose among, each a button to press, with "None" the first, as the colours are chosen (025 §6).

**Why the core.** Which icons there are, and which a Space has, are the model's and the store's: a Space's icon is the same in every window and on every system the Space is on, and with sync (W9) it travels.

**Nothing leaves the device**, and nothing of it is in a log: an icon is a person's choice about their own Space, and is redacted as a Space's name is.

## 4. The contract (to be 1.43)

Added; nothing is changed or removed.

```proto
message SpaceSummary { … string icon = N; }             // one of SpacesPage.icons, or empty: none. Redacted in logs
message SpacesPage { … repeated string icons = N; }      // the icons a Space can have, in their order
message WindowSnapshot { … string space_icon = N; }      // the icon of the Space the window shows, or empty. Redacted in logs
message CreateSpace { … string icon = N; }               // one of the list, or empty: none; "space.icon_unknown"
message IconSpace { uint64 space_id = 1; string icon = 2; }   // answered with Ack; "space.unknown", "space.icon_unknown"
```

The store (schema 15): `space.icon`, the emoji, empty for none.

## 5. In what order

1. **The core and the store:** the list, a Space's icon kept, given and refused; the snapshot and the list of Spaces.
2. **The shells:** the chip, the list of Spaces, and the icons to choose among where a Space is made and edited; Apple's systems and Android.

## 6. Decisions

**All of these are put to the owner; none is built.** The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | A Space's icon is an emoji, drawn by the system | Every system draws them, a person knows them, and a Space brought from another browser has one already (002). There is nothing to draw, to license, or to keep alike on four systems | Symbols drawn for WebTiger, by name: one look everywhere, and artwork to make for each of them on each system. Or each system's own symbol set (SF Symbols on Apple's, Material's on Android): no artwork on two systems, and the same name is a different picture on each |
| D2 | The icons are a list the core has, of about two dozen | A page can show them all, and none has to be typed or searched for. Every one is known to draw on the systems WebTiger runs on | Any emoji, from the system's own picker: every person finds theirs, and the core must tell an emoji from other text (a table of them, or a library that has one) |
| D3 | A Space is made with no icon unless one is chosen | A picture nobody chose says something about the Space that may not be so; a colour does not | The first icon no Space has, as a colour is given |
| D4 | In the chip the icon takes the place of the initial; beside a name it stands before it | The initial is what there was no better thing than; the icon is the better thing. A name is still a name | The icon and the initial both; or the icon only, with no name, at every width |
| D5 | The chip is said to a screen reader by the Space's name, not its icon | "Space: Work" is what the chip is for. An emoji's name ("briefcase") beside it adds a word that is not the Space's | The icon read with the name |

## 7. Tests

To be filled in as built.

## 8. What is not verified

To be filled in as built.

## 9. Done checklist

- [ ] The owner's word on §6.
- [ ] The core, the store and the contract.
- [ ] The shells.
- [ ] The core: the gate.
- [ ] The shells: the gate.

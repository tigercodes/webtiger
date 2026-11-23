# 046 — A site's icon (W2)

| | |
|---|---|
| **Status** | **The core's first part is built and tested, on the recommendations of §6, which are provisional:** an icon that an engine hands over is kept for its host in its login set, and a window is told of it by number. **No adapter hands one over yet, and no shell draws one** (§7). Apple's part, where the app would fetch the picture itself (D1), is not built, and is the owner's to say first |
| **Phase** | W2 of [000](000-master-plan.md) §9. 000 §5.6: "Favourites. A row of icon tiles at the top of a Space" |
| **Contract** | 1.37 for what is built (§5); the rest to come. The store's schema is 13: a table of icons (§4) |
| **Builds on** | [042](042-favourites.md) D4: "Until sites have icons, a tile shows the first letter of the Site's name". [039](039-sites.md) D5: a Site's mark before its name, "icons alone need favicons, which are not fetched yet". [024](024-several-spaces.md): a login set is one data store, and nothing of one is seen in another |
| **Last updated** | 2026-11-23 |

## 1. User story

I tell my tabs apart by their icons before I read their names, and my Favourites by their icons alone: two tiles whose names begin with the same letter are no longer the same tile.

## 2. What is in it, and what is not

**In:** a tab shows the icon of the site it is at; a Site, asleep or awake, the icon of its home; a Favourite's tile is its icon. The icon is kept, so that a sleeping Site has one after a quit.

**Not in, each with where it goes:**

- **Icons in the History list, the suggestions and the downloads:** the same icons by the same numbers (§5), when those lists are next worked on.
- **Icons carried over from another browser** ([002](002-arc-import.md) §9): with the import screen.
- **A Space's own icon** (000 §5.6): not a site's; a choice from a set, a feature of its own.
- **Icons that are drawings in SVG:** no system decodes them the same way; later (D4).

## 3. The rule

**An icon comes from a page that has arrived in a tab, and from nowhere else.** Nothing is fetched for an address that was only typed, suggested, listed or imported.

**Where each engine has it from:**

- **The System WebView (Android) fetches a page's icon itself,** as part of loading the page, in the tab's own login set, and hands the picture over. The adapter passes it on.
- **WebKit (Apple) hands over nothing.** The adapter reads which icons the page names (`<link rel="icon">`, in the adapter's own script world, where it already reads what was pressed, 022), tells the core, and the core says which one to fetch, or none. **The fetch is the app's own** (D1): one request, for the picture the core named, that carries no cookie and no login, keeps no cookie that comes back, follows redirects only to `https`, stops at 200 KB, and is not made again for that page.

**What the core decides** (it is the same on every system, and so is not the shell's):

- **Which icon:** of those the page names, one that is a picture every system decodes (PNG, ICO, JPEG, GIF; not SVG), the one nearest 64 points across; a page's "touch icon" counts as one of them. A page that names none is asked for `/favicon.ico` at its own origin.
- **Which addresses:** `http` and `https` only; a page that came by `https` gets no icon by `http`. Nothing is fetched for a page that failed, for a page behind a certificate warning that was passed (026), or in a window where the person is being asked something by that page.
- **Whose it is:** an icon is kept for the page's host **in the login set it was fetched in** (D2). Another Space with other logins fetches its own.
- **What a tab shows:** the icon of the host of the page it shows, in its login set; a Site, the icon of its home's host, though it has wandered (D3): its name and its tile do not change as it wanders, and nor does its icon. A tab with no icon is drawn as now.
- **How often:** when a page arrives; and when a page changes its own icon while it is shown (a count of unread messages), at most once a minute for a tab.

**What a shell does:** it turns what was fetched into a small PNG (at most 128 pixels across) with the system's decoder, told which kinds of picture to accept and no others, and hands the PNG to the core; and it draws the PNGs the core gives it. It decides nothing.

**A private window** has icons while it is open, and nothing of them is written (037).

## 4. Where it is kept

A table `icon` (schema 13): the login set, the host, the PNG, and when it was last used. The thousand used most lately are kept (D5). An icon is used when it is kept, when the engine hands the same picture over again, and when a window is first told of a tab that shows it: so the icon of a Site, which comes back with every start, is used at every start. A private window's are in the store that is in memory, and go with its login set (037).

## 5. The contract (1.37, and what is to come)

Added; nothing is changed or removed. **Built (1.37):** `IconReceived`, `TabSummary.icon_id`, `QueryIcons` and `IconsPage`. **To come, with Apple's part:** `IconNamed` and `WebViewFetchIcon`.

**As built, where the design did not say:**

- **`IconReceived` names its page** (`page_url`): the page the engine says the picture is the icon of. An icon of a page of another host than the tab's is ignored: an engine's word about one page is not taken for another.
- **Only a page that came over `https`, or never left this device, has an icon.** (The design let a page that came by `http` have one by `http`. A picture that anyone on the way could have put there is not kept.) A page whose certificate was gone on past has none (026); a page that failed never arrived, and has none.
- **What is kept is a PNG of at most 256 pixels each way and 64 KB,** by what its first bytes say of it: the core reads a picture's header and decodes nothing.
- **A number is one picture of one host in one login set, for as long as the core runs.** A host whose picture changes has another number, and the number it had is nobody's. Numbers are not kept across a quit: a shell's pictures are not either.
- **An answer has at most 64 pictures,** the first asked for; a shell asks for more in turn.
- **A host is a host:** `mail.example.com` and `docs.example.com` each have their own; a port does not tell two servers of one host apart.
- **A window is never told of a tab without the icon the store has for it:** the store is asked before the window is told, the first time a host is met in a run.

```proto
// Engine events, of a web view.
message IconNamed {            // WebKit: the page names these icons
  repeated NamedIcon icons = 1;   // address, kind ("icon", "apple-touch-icon"), sizes, type: as the page wrote them
}
message IconReceived {         // the System WebView: the engine fetched this
  bytes png = 1;
}

// A host request, of a web view (WebKit): fetch this one, as §3 says, and answer with it as a PNG.
message WebViewFetchIcon { uint64 web_view_id = 1; string url = 2 [(wt.redact) = true]; }
message IconFetched { bytes png = 1; }      // or an error: "icon.failed", "icon.too_big", "icon.not_a_picture"

// To a window: the number of a tab's icon, 0 for none. A number is one picture, for ever.
message TabSummary { … uint64 icon_id = 13; }
// A shell asks for the pictures it has not got.
message QueryIcons { repeated uint64 icon_ids = 1; }
message IconsPage { repeated Icon icons = 1; }   // icon_id, png
```

A window's snapshot carries numbers, not pictures: a row of 5,000 tabs is no heavier for having icons ([043](043-the-sidebar.md) §8a).

## 6. Decisions

**All of these are put to the owner.** D2, D3 and D5 are built in the core as recommended; D1 and D4 are Apple's part, and D6 the shells': not built. The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | **On Apple's systems the app fetches a page's icon itself:** one request for the picture the page named, with no cookie and no login, nothing kept of what comes back but the picture | WebKit gives no icon by any public way: a browser built on it that shows icons has to fetch them. With no cookie the request says nothing of who asks beyond what the page's own load said, it cannot be answered with a picture that is one account's, and nothing crosses from one login set to another. **What it costs, said plainly:** it is a request the web engine did not make, so nothing that watches the engine's requests sees it (Guard, when it is built in W4, has to be asked about it by the core, and will be); and the picture is decoded by the system's decoder in the app, not in the engine's sandbox: the adapter accepts four kinds of picture only, by what the bytes are and not by what the server calls them, and nothing over 200 KB | **In the page:** a script draws the icon on a canvas and hands the pixels over. The engine fetches and decodes, with the page's cookies; but a page may not read a picture from another site, where many icons are (a site's own file server), so many sites would have none. **No icons on Apple's systems:** letters, as now |
| D2 | An icon is kept for a host in the login set it came from, and is not shown in another | On Android the engine fetches with the tab's cookies: the picture may be one account's (a face, a count). What one login set fetched is not shown where another is signed in | One icon for a host everywhere: fewer fetches, and a Space shows a picture another Space's login fetched |
| D3 | A Site shows its home's icon, wherever it has wandered | The Site is its home: its name and its place do not change as it wanders (039), and a tile that changed its picture would be another tile | The icon of the page it shows now |
| D4 | The page's own icon nearest 64 points, its touch icon among them; PNG, ICO, JPEG and GIF; no SVG yet | A tile is as large as a button and a tab's icon is small: one picture serves both on every screen. SVG is a language, not a picture: no two systems draw it alike, and its decoders are the largest | The smallest icon (16 points), as a tab alone needs: tiles would be blurred |
| D5 | Kept on disk: the thousand used most lately, where an icon is used whenever a tab that shows it comes back at a start (**as built:** the design had every Site's kept apart from the thousand; one rule does it, since a Site comes back at every start) | A sleeping tab has its icon after a quit; the store does not grow with every site ever visited | Sites' only; or all, for ever |
| D6 | Where it is drawn: before a tab's name, in the strip and at the side, in the place of the dot that says a tab is loading (which then goes round the icon); on a tile, the icon alone. A Site keeps its mark (039 D5) | Nothing moves in the row. Whether a Site needs a mark once it has an icon is 039's question, and stays there | The icon in the place of a Site's mark |

## 7. In what order

1. **The core, for an icon an engine hands over:** the rule of §3 against the scripted engine, the table, the contract. **Built.**
2. **Android,** where the engine has the icon: the adapter, the adapter suite, the window.
3. **Apple,** after the owner's word on D1: in the core, which icon of those a page names; the adapter's reading of the page, the fetch, the adapter suite, the window.

## 8. Tests

No test fetches from anywhere but the test's own site, in the test's own process. (So far no test fetches at all: the scripted engine hands pictures over.)

| Where | What |
|---|---|
| The store (`icon.rs`, 6) | A host's icon is kept, read and replaced; another host's and another login set's are their own (a host is not its site); no bytes, more than 64 KB, no host and a login set there is not are refused, and what was kept stays; the thousand used most lately are kept, an icon used again is not the one to go, and a picture put in another's place is used now; a store from before icons gains them and keeps what it had; a login set that is taken out takes its icons |
| The core, what may be an icon (`icons.rs` in the crate, 1) | A PNG by its own bytes, from 1 to 256 pixels each way, of at most 64 KB; not another kind of picture, a PNG's first bytes alone, a first part that is not the header, or a header of another length |
| The core, through the contract (`icons.rs`, 8) | **Kept and told:** a page's icon handed over gives its tab a number, and the number is that picture; the shell is asked for nothing; another page of the host has it from the moment it is there, another host has none until its own; pictures are answered in the order asked, a number nobody has left out; the same picture again is the number it has; another picture when the host's page next arrives is another number for every tab of the host, and the number it had is nobody's. **Ignored:** no bytes, another kind of picture, one too wide, one of no height; the icon of another host's page, of no page, of no address; of a web view that is no tab's (none of which uses up the page's turn); of a page that came by `http` (one that never left the device has its icon); of a page that failed; of a page whose certificate was gone on past. **A Site** has its home's icon wherever it wanders, the host it wanders to keeps its own, and let go the tab has the icon of the page it shows. **Each login set its own:** a Space with other logins has none for the same host until its own; a private window the same, and when it closes its number is nobody's and the others are as they were. **Once a minute:** a page that goes on changing its icon is not heard for the rest of the minute, and is then; a page that arrives is heard at once. **An answer** has at most 64 pictures, the first asked for. **Across a quit** (a store on disk): the tabs come back with the icons their hosts had, the sleeping ones too, and the window is never told of them without; what a private window had was never written. **A full store:** the icon a sleeping tab shows, the oldest of a thousand, is not the one to go when one more is kept |
| The rule broken, a part at a time | 40 parts, in the core and the store. 38 caught: 35 at once, one once its text was found, two by tests written for them (that a window is not first told without its icons; a full store). Two were not: that a closed private window's numbers are forgotten (nothing can be asked of them either way: their pictures went with the login set), and that a picture handed over again is said to be used (it matters only in a full store while the app runs; no test fills one then) |

## 9. What is not verified

- **No adapter hands an icon over, and no shell draws one.** The contract has it (1.37); every picture in the tests is the scripted engine's.
- **The choices of §6** are the recommendations, not the owner's word, and D1 is not built at all.
- **A picture is not decoded by the core:** that the bytes after the header are a picture is the word of the adapter that made them. A shell decodes what it draws.
- **A long row.** Each tab told to a window now looks up its icon's number: not measured at thousands of rows ([043](043-the-sidebar.md) §8a).
- **An icon a tab shows for longer than a thousand others are met** in one run may be forgotten by the store while the tab still has its number: the picture is then not answered, and comes back when the page next hands it over.
- **Two servers at one host** (two ports) share an icon.

## 10. Done checklist

- [ ] The owner's word on §6, D1 before any of Apple's part.
- [x] The core and the contract, for an icon an engine hands over.
- [ ] Android.
- [ ] Apple.
- [ ] `Tools/test.sh all` green, and `android-device`.

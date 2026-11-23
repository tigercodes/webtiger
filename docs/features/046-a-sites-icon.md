# 046 — A site's icon (W2)

| | |
|---|---|
| **Status** | **Designed; nothing built.** The choices of §6 are put to the owner. The first of them is what the app may fetch by itself, which is the owner's to say before any of it is built on Apple's systems |
| **Phase** | W2 of [000](000-master-plan.md) §9. 000 §5.6: "Favourites. A row of icon tiles at the top of a Space" |
| **Contract** | To be 1.37 (§5). A table in the store (§4) |
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

A table `icon`: the login set, the host, the PNG, and when it was last used. An icon whose host is the home of a Site is kept as long as the Site is. Of the others, the thousand used most lately (D5).

## 5. The contract (to be 1.37)

Added; nothing is changed or removed.

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

**All of these are put to the owner; none is built.** The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | **On Apple's systems the app fetches a page's icon itself:** one request for the picture the page named, with no cookie and no login, nothing kept of what comes back but the picture | WebKit gives no icon by any public way: a browser built on it that shows icons has to fetch them. With no cookie the request says nothing of who asks beyond what the page's own load said, it cannot be answered with a picture that is one account's, and nothing crosses from one login set to another. **What it costs, said plainly:** it is a request the web engine did not make, so nothing that watches the engine's requests sees it (Guard, when it is built in W4, has to be asked about it by the core, and will be); and the picture is decoded by the system's decoder in the app, not in the engine's sandbox: the adapter accepts four kinds of picture only, by what the bytes are and not by what the server calls them, and nothing over 200 KB | **In the page:** a script draws the icon on a canvas and hands the pixels over. The engine fetches and decodes, with the page's cookies; but a page may not read a picture from another site, where many icons are (a site's own file server), so many sites would have none. **No icons on Apple's systems:** letters, as now |
| D2 | An icon is kept for a host in the login set it came from, and is not shown in another | On Android the engine fetches with the tab's cookies: the picture may be one account's (a face, a count). What one login set fetched is not shown where another is signed in | One icon for a host everywhere: fewer fetches, and a Space shows a picture another Space's login fetched |
| D3 | A Site shows its home's icon, wherever it has wandered | The Site is its home: its name and its place do not change as it wanders (039), and a tile that changed its picture would be another tile | The icon of the page it shows now |
| D4 | The page's own icon nearest 64 points, its touch icon among them; PNG, ICO, JPEG and GIF; no SVG yet | A tile is as large as a button and a tab's icon is small: one picture serves both on every screen. SVG is a language, not a picture: no two systems draw it alike, and its decoders are the largest | The smallest icon (16 points), as a tab alone needs: tiles would be blurred |
| D5 | Kept on disk: every Site's, and the thousand used most lately of the rest | A sleeping tab has its icon after a quit; the store does not grow with every site ever visited | Sites' only; or all, for ever |
| D6 | Where it is drawn: before a tab's name, in the strip and at the side, in the place of the dot that says a tab is loading (which then goes round the icon); on a tile, the icon alone. A Site keeps its mark (039 D5) | Nothing moves in the row. Whether a Site needs a mark once it has an icon is 039's question, and stays there | The icon in the place of a Site's mark |

## 7. In what order

1. **The core:** the rule of §3 against the scripted engine, the table, the contract.
2. **Android,** where the engine has the icon: the adapter, the adapter suite, the window.
3. **Apple,** after the owner's word on D1: the adapter's reading of the page, the fetch, the adapter suite, the window.

## 8. Tests

To be filled in as built. No test fetches from anywhere but the test's own site, in the test's own process.

## 9. What is not verified

To be filled in as built.

## 10. Done checklist

- [ ] The owner's word on §6, D1 before any of Apple's part.
- [ ] The core and the contract.
- [ ] Android.
- [ ] Apple.
- [ ] `Tools/test.sh all` green, and `android-device`.

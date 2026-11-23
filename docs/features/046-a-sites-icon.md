# 046 — A site's icon (W2)

| | |
|---|---|
| **Status** | **Built and tested in the core, on Android and on Apple's systems.** The System WebView hands a page's icon over; where WebKit hands none, the page says which it names, the core chooses one, and the adapter fetches it as nobody (§5a; D1 is the owner's, WT-D33, 2026-11-23: "use suggested choices for the blocked items"). The core keeps an icon for its host in its login set, and every window draws it before a tab's name and as a Favourite's tile. D2 to D6 are the recommendations of §6, which are provisional |
| **Phase** | W2 of [000](000-master-plan.md) §9. 000 §5.6: "Favourites. A row of icon tiles at the top of a Space" |
| **Contract** | 1.37 (§5) and, for an engine that hands no icon over, 1.39 (§5a). The store's schema is 13: a table of icons (§4) |
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
- **An icon that comes a moment before its page waits for it.** The System WebView can hand over the icon of a page it is loading before it says the page has arrived (an icon the page carries in itself is ready at once, and the two words come by different ways). The core holds such an icon while a load is on its way, takes it when the page that arrives is the one it named, and drops it when another arrives, or the load fails. **Found on the emulator:** the app's test passed alone and failed in the whole suite, where the two words came the other way round; the first form of the rule dropped the icon.
- **Android's adapter** hands over what the WebView gives it, made a PNG, and made smaller in its own shape when it is more than 128 pixels a side; nothing over 64 KB. It decides nothing else.
- **Android's window** asks for the pictures of the numbers its tabs have, once each, 64 to a question, and keeps them while it is open (more than 512, and it keeps those its tabs show). **Drawn** (D6): before a tab's name, after a Site's mark, 16 points; a Favourite's tile is its icon, 24 points, and its letter until it has one; asleep, fainter, as a sleeping tab's name is. The dot of a tab that is loading is where it was: the design had it go round the icon, which is not built.

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

## 5a. Apple's part (1.39)

WebKit hands an app no icon, and the app fetches the one the page names itself (WT-D33). Added to the contract; nothing is changed or removed.

```proto
// An engine event (WebKit): the page names these icons. As the page wrote them, each
// address made whole against the page's own.
message IconNamed { repeated NamedIcon icons = 1; string page_url = 2; }
message NamedIcon { string url = 1; string rel = 2; string sizes = 3; string type = 4; }

// A host request (WebKit): fetch this one for that page. Answered with Ack, at once.
message WebViewFetchIcon { uint64 web_view_id = 1; string url = 2; string page_url = 3; }
```

**The picture comes back as `IconReceived`,** as Android's does: there is one way in for a picture, and one set of rules about whose it is (§5). A fetch that fails says nothing.

**The core chooses** (D4), the same on every system that has to fetch: of the icons named, one whose address came over `https` (or never leaves this device), that does not say it is SVG; the one nearest 64 pixels across by what the page says of its size, a touch icon taken as 180 and an icon that says nothing as 32; with none that will do, `/favicon.ico` at the page's own origin. Only for a page whose icon would be kept (§5). **An address is asked for once in a run, for a host in a login set,** however often the host's pages name it: a page that is loaded twenty times is asked for its icon once, and a site whose pages take turns naming two icons is asked for each once. **No more than eight addresses for a host:** a site that names another address on every page is not fetched from for ever.

**The adapter fetches** as WT-D33 says: a session of its own that keeps nothing (no cookie sent or taken, no login, no cache); no referrer; the page's own user agent, so that the request tells the site nothing the page's requests did not; redirects followed only to `https` (or within this device); nothing over 200 KB. **It decodes with the system's decoder** only what is, by its own bytes, a PNG, an ICO, a JPEG or a GIF; makes it at most 128 pixels a side; and hands over a PNG.

**As built, where the design did not say:**

- **An engine fetches nothing unless it is made to.** As with the system's file picker and the camera, the adapter's engine is made able by the app, started for real: then it asks the web. In fake mode it reads no page's icons and fetches none, so that a suite's site is asked for what its pages ask for and no more; a suite that wants icons says so (`WEBTIGER_FAKE_ICONS=1`), and its engine then asks this device only, where its site is. A suite cannot fetch from the web by mistake.
- **A page is asked once it has loaded,** in a world of the adapter's own where no script of the page's runs, and says which page it is: the web view may have gone on by the time it answers. At most 16 icons are taken from a page, and an address longer than 2,048 characters is left out (a picture written out in the page itself, when it is long). **A page that names its icon later,** by script, is not asked again: it has the one it named when it loaded.
- **Each fetch has a session of its own,** which ends with it: there is nothing for one fetch to leave the next. The request says it handles no cookie, and the session has nowhere to keep one and no leave to take one. It waits 20 seconds.
- **An address that names somebody** (`https://name:password@…`) is never asked, at first or when a server sends the fetch there: a fetch has no login. **A server that asks who is there** is told nothing, and the fetch ends. A server's certificate is the system's to judge, as for any request; one a person went on past in a web view (026) is not let through here, and the core asks nothing for such a page.
- **A picture is known by its own first bytes,** and the system's decoder must take it for the same kind. Of an `.ico`'s several pictures the largest is taken. A picture of more than 2,048 pixels a side is not decoded at all. What is handed over is at most 128 pixels a side, in its own shape; and if its PNG is more than the 64 KB the core keeps (noise, as large as is handed over), it is made smaller, to 96 and then 64.
- **Once in a run is once whether or not there was a picture:** a fetch that fails is not tried again until the app is started again. (As first built the core remembered only the last address it had asked a host for, so that a site whose pages named different icons was asked again at every move between them. Nothing leaked by it; it was put right before the windows drew anything.)
- **Apple's windows** ask the core for the pictures of the numbers their tabs have, as Android's do: once for each number, 64 to a question, and a window that has kept more than 512 lets go of those its tabs do not show. A picture is drawn 16 points a side before a tab's name, after a Site's mark, and 24 as a Favourite's tile, in the place of its letter; a sleeping tab's is half as strong. It says nothing to a screen reader: a tab's name is said, and the icon adds no word.
- **The picture comes for the page it was asked for.** A web view that has gone since is told of none; whether the page is still the tab's is the core's (§3), as for an icon Android's engine hands over.

## 6. Decisions

**D1 is the owner's (WT-D33). The rest are put to the owner,** and built as recommended: D2, D3 and D5 in the core, D4 in the core and the WebKit adapter, D6 in every window. The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | **On Apple's systems the app fetches a page's icon itself:** one request for the picture the page named, with no cookie and no login, nothing kept of what comes back but the picture | WebKit gives no icon by any public way: a browser built on it that shows icons has to fetch them. (A request of the app's own is not new here: Android's adapter fetches a download's bytes itself, so that one site's cookies are not handed to another, [027](027-cookies-stay-with-their-site.md).) With no cookie the request says nothing of who asks beyond what the page's own load said, it cannot be answered with a picture that is one account's, and nothing crosses from one login set to another. **What it costs, said plainly:** it is a request the web engine did not make, so nothing that watches the engine's requests sees it (Guard, when it is built in W4, has to be asked about it by the core, and will be); and the picture is decoded by the system's decoder in the app, not in the engine's sandbox: the adapter accepts four kinds of picture only, by what the bytes are and not by what the server calls them, and nothing over 200 KB | **In the page:** a script draws the icon on a canvas and hands the pixels over. The engine fetches and decodes, with the page's cookies; but a page may not read a picture from another site, where many icons are (a site's own file server), so many sites would have none. **No icons on Apple's systems:** letters, as now |
| D2 | An icon is kept for a host in the login set it came from, and is not shown in another | On Android the engine fetches with the tab's cookies: the picture may be one account's (a face, a count). What one login set fetched is not shown where another is signed in | One icon for a host everywhere: fewer fetches, and a Space shows a picture another Space's login fetched |
| D3 | A Site shows its home's icon, wherever it has wandered | The Site is its home: its name and its place do not change as it wanders (039), and a tile that changed its picture would be another tile | The icon of the page it shows now |
| D4 | The page's own icon nearest 64 points, its touch icon among them; PNG, ICO, JPEG and GIF; no SVG yet | A tile is as large as a button and a tab's icon is small: one picture serves both on every screen. SVG is a language, not a picture: no two systems draw it alike, and its decoders are the largest | The smallest icon (16 points), as a tab alone needs: tiles would be blurred |
| D5 | Kept on disk: the thousand used most lately, where an icon is used whenever a tab that shows it comes back at a start (**as built:** the design had every Site's kept apart from the thousand; one rule does it, since a Site comes back at every start) | A sleeping tab has its icon after a quit; the store does not grow with every site ever visited | Sites' only; or all, for ever |
| D6 | Where it is drawn: before a tab's name, in the strip and at the side, in the place of the dot that says a tab is loading (which then goes round the icon); on a tile, the icon alone. A Site keeps its mark (039 D5) | Nothing moves in the row. Whether a Site needs a mark once it has an icon is 039's question, and stays there | The icon in the place of a Site's mark |

## 7. In what order

1. **The core, for an icon an engine hands over:** the rule of §3 against the scripted engine, the table, the contract. **Built.**
2. **Android,** where the engine has the icon: the adapter, the adapter suite, the window. **Built.**
3. **Apple:** in the core, which icon of those a page names; the adapter's reading of the page, the fetch, the adapter suite. **Built.**
4. **Apple's windows:** the icon drawn, as Android's draw it. **Built.**

## 8. Tests

No test fetches from anywhere but the test's own site, in the test's own process. The test site sends pictures since this feature: a page of it may be bytes.

| Where | What |
|---|---|
| The store (`icon.rs`, 6) | A host's icon is kept, read and replaced; another host's and another login set's are their own (a host is not its site); no bytes, more than 64 KB, no host and a login set there is not are refused, and what was kept stays; the thousand used most lately are kept, an icon used again is not the one to go, and a picture put in another's place is used now; a store from before icons gains them and keeps what it had; a login set that is taken out takes its icons |
| The core, what may be an icon (`icons.rs` in the crate, 1) | A PNG by its own bytes, from 1 to 256 pixels each way, of at most 64 KB; not another kind of picture, a PNG's first bytes alone, a first part that is not the header, or a header of another length |
| The core, through the contract (`icons.rs`, 8) | **Kept and told:** a page's icon handed over gives its tab a number, and the number is that picture; the shell is asked for nothing; another page of the host has it from the moment it is there, another host has none until its own; pictures are answered in the order asked, a number nobody has left out; the same picture again is the number it has; another picture when the host's page next arrives is another number for every tab of the host, and the number it had is nobody's. **Ignored:** no bytes, another kind of picture, one too wide, one of no height; the icon of another host's page, of no page, of no address; of a web view that is no tab's (none of which uses up the page's turn); of a page that came by `http` (one that never left the device has its icon); of a page that failed; of a page whose certificate was gone on past. **A Site** has its home's icon wherever it wanders, the host it wanders to keeps its own, and let go the tab has the icon of the page it shows. **Each login set its own:** a Space with other logins has none for the same host until its own; a private window the same, and when it closes its number is nobody's and the others are as they were. **Once a minute:** a page that goes on changing its icon is not heard for the rest of the minute, and is then; a page that arrives is heard at once. **An answer** has at most 64 pictures, the first asked for. **Across a quit** (a store on disk): the tabs come back with the icons their hosts had, the sleeping ones too, and the window is never told of them without; what a private window had was never written. **A full store:** the icon a sleeping tab shows, the oldest of a thousand, is not the one to go when one more is kept |
| The core, an icon before its page (`icons.rs`, 1 more: 9 through the contract) | The engine has a page's icon before it says the page has arrived: the tab has no icon while it shows the page it had, and has it when the page arrives. An icon for a page on its way, when another arrives: nobody's, and not there when the same tab later goes to the site it named. One whose load fails: nobody's, though the tab then goes to that page. A tab at rest waits for nothing. Broken in 9 parts: 8 caught, two of them once the test sent the same tab on to the site the icon had named; the ninth was a check made twice, and one of the two was taken out |
| Android's adapter on the JVM (`IconsTest`, 3, with real pictures) | A small picture is handed over as it is, the same size and colour; a large one is made smaller in its own shape, its longer side 128 and no side nothing, and the WebView's own picture is not given back to the system; a picture that cannot be read, and one whose PNG is one byte more than is kept, are not handed over. Broken in 6 parts, 6 caught |
| The adapter suite on the emulator (`SystemWebViewIconTest`, 4, the real System WebView) | A page that carries its icon in itself: the WebView hands it over, a PNG of a size the core keeps, of the page's own colour, naming the page. **A page whose icon is at an address of the site: the WebView asks the site for it itself, and hands over the site's picture.** **With whose cookies:** a page whose icon's address sends whoever asks on to an address named after their cookie; signed in, in one login set, the site is asked as that login set; another login set, at the same page while the first is open, is asked as nobody. A page that names none hands none over, whether or not the WebView asks the site for one |
| The test site | It sends a picture as its bytes, of the kind it says, with no alphabet named: its own check (`site.py self-test`), and through WebKit, where a page of the site asks for the picture and has one of 32 pixels each way (`TestSitePictureTests`, 1, on the Mac and the iPhone simulator) |
| Android's window on the JVM (`IconsWindowTest`, 6, against a scripted core, with real pictures) | The model asks for the pictures it has not got, once for each number, in one question; told the same again it asks nothing; a number the core has no picture of, bytes that are no picture and a picture larger than the core keeps are not drawn, and not asked for again; no more than 64 to a question; a window that has more than it keeps lets go of those its tabs do not show, and asks again for one it then shows. Drawn: a tile with an icon is its icon and has no letter, the other has its letter; a Site has its icon and keeps its mark; each icon is in its own tab, and a tile's is the larger; the same down the side. Broken in 13 parts, 13 caught |
| Android's pictures | The Favourites' and the sidebar's states have icons now: a tile with an icon beside one with a letter, a Site and a tab with theirs, a sleeping Site's fainter. Four pictures recorded again and looked at, light and dark. (The picture test had passed with the old ones: what it lets by is more than an icon. They were taken away to be made again) |
| The Android app on the emulator (`AppIconsTest`, 1, the real core and the real WebView) | A tab at a page that names its icon shows it, and the other tab's site has none; made a Favourite, its tile is its icon and has no letter; put to sleep it keeps it; a Site again, it has it before its name |
| The core, which icon is fetched (`icons.rs` in the crate, 3 more: 4) | Of the icons a page names: the one nearest 64 pixels by what the page says, whatever order they come in; with none that size, a larger before a smaller, and of the larger the nearest; of smaller ones the largest; an icon that lists several sizes is as near as its nearest, one that is not square as large as its longer side; a touch icon that says nothing is taken to be 180 and any other 32; of two alike the first named. Passed over: what the page does not call an icon, a drawing in SVG by its kind or by its name, an address that did not come over `https`, what is no address. With nothing left, and for a page that names none: `/favicon.ico` at the page's own origin; an icon at another site is an icon; a page on this device may name one on this device; a page that did not come over `https` is asked for nothing. And the rule of once: an address is asked for once for a host in a login set, another host, login set or address each once more, and a login set that is gone takes what was asked for it |
| The core, the fetch through the contract (`icons.rs`, 2 more: 11) | A page names its icons: the shell is asked for the one chosen, and the picture it hands over is the tab's icon. The page again, another page of the host and another tab at it ask nothing more. The host's page names another address: that is asked for, and is the host's icon from then. A private window asks for its own. **Nothing is asked** for a page that names none but `/favicon.ico` once, and not again when nothing was there; for icons named for another host's page, for no address, or by a web view that is nobody's; for a page that came by `http`; for one whose certificate was gone on past; nor of an engine that says nothing of icons. Broken in 27 parts, 27 caught |
| The WebKit adapter's pictures (`WebKitIconPictureTests`, 5, with pictures the test makes; no web view, no site) | A picture is known by its own first bytes: the four kinds, and not words, SVG, WebP, TIFF or nothing. Each kind is handed over as a PNG: a small one at its own size, a large one 128 a side at most in its own shape, of an `.ico`'s several the largest. What begins as a picture and is none, and a picture wider or taller than is decoded, are nothing. Noise whose PNG is more than the core keeps is handed over smaller, at the largest size that is few enough bytes; exactly as many as are kept is few enough; with no size few enough, nothing. **Which addresses may be asked,** one by one and asking none: this device always, anywhere else only over `https` and only by an engine let reach the web, and never an address that names somebody |
| The adapter suite (`WebKitIconTests`, 6, real WebKit and the test site, on the Mac and the iPhone simulator) | **A page says which icons it names:** whole addresses, with what the page calls each, its size and its kind; a link that is no icon is not named; a page that names none says so; and nothing is fetched unasked, by WebKit or by the adapter. **The icon asked for comes as the page's,** a PNG of the size it was served. **As nobody, tested as a security property:** with somebody signed in, in the page's login set, the fetch carried no cookie, no word of where it came from and no login, and said it is what the page says it is; the site sends whoever asks on to an address named after their cookie, and sent the fetch on as nobody, and the page itself as who was signed in. **What a fetch is given to keep is not kept:** a cookie set on the way to somewhere else is not sent along; a fetch that a server signs in leaves the next fetch nobody, and nothing where the process keeps cookies; a web view of another login set is nobody still. **A server is followed only where the fetch may ask:** to a picture that moved within the site, and not to an address that names somebody; words and a page that is not there are no picture; no more bytes are taken than an icon may have, to the byte; an address that may not be asked is not asked at all. **An engine not let fetch** asks no page about its icons, and asked for one, answers and fetches nothing; a web view there is none of is said to be unknown. **A web view that has gone** while its icon was on its way is told of none. Broken in 42 parts, 42 caught (two of them once they were written so that they built) |
| The test site, for the fetch | Two pages more: a picture that has moved, and one that sends whoever asks on to an address that names somebody. The page whose icon is served now says the icon's size, and links to something that is no icon. Apple's server keeps what each request said of itself (its headers), so that a test can see a cookie that a path does not show |
| The core, once for an address (the same tests, changed) | A site whose pages take turns naming two icons is asked for each once, as a tab goes back and forth between them; a host is asked for eight addresses in all, and then none, while another host is asked still. Broken in 8 parts, 8 caught, one of them once the test said eight and not the rule's own number |
| Apple's window model (`IconModelTests`, 4, against a scripted core, with pictures the test makes) | The window asks for the pictures it has not got: once for each number, in one question, a tab with no icon asked about not at all; told the same again it asks nothing, and for a new number it asks for that one alone. A number the core has no picture of, words, a picture larger than the core keeps and a PNG's first bytes alone are not drawn, and not asked for again. No more than 64 to a question, each number once, in order. A window with fewer than it keeps lets go of none; with more, it keeps those its tabs show, and asks again for one it then shows |
| Apple's windows, drawn (`IconModelTests`, 2 more: 6; on the Mac and the iPhone simulator) | **By the icons' own colours, counted:** the pictures' check lets by a change as small as an icon (a hundredth of a picture may differ), and passed with the icons added before one picture was recorded again. A window whose tabs have no icons has none of the colours. Along the top, a Favourite's tile is its icon, the larger, and a Site has its icon before its name, the smaller. Down the side the same, and a tab that is no Site has its icon; the sleeping Site's is fainter, and is not its colour. A tab's label alone: no icon, none of the colour; an icon, the colour; asleep, none of the full colour and as much that is blue. Broken in 20 parts, 20 caught: 16 at once, two once the tests were made to say more (a sleeping tab's icon that is not drawn at all; a window that never lets go), one once it was written so that it built, and one added |
| Apple's pictures | The Favourites', the sidebar's and the switcher's states have icons now: twelve pictures recorded again, six on each system, each compared with what it was (they differ in the rows of tabs and tiles and nowhere else, by half a hundredth to a hundredth of the picture) and looked at |
| The apps (`IconTests`, 2, on the Mac and the iPhone simulator, with the real core and the real engine) | **The app asks for a page's icon once, and as nobody.** Signed in at the site, at a page that names no icon: the site is asked for `/favicon.ico`, once. At a page that names its icon: the site is asked for it, once; the page itself was asked for with its login set's cookie, and its icon with none, saying nothing of where it came from, and to be what the page says it is. The same page in a second tab, and a page that names none again: neither address is asked for again. **In fake mode no icon is asked for unless a suite says so** |
| Found by the gate | The app's test first signed in with a cookie that is kept for an hour. In fake mode the app's web data outlives a test (its login sets are the same ones at every start), so the next seven tests of the Mac's suite that expect nobody to be signed in found somebody, and failed. The test signs in for the session only now, as the Spaces' test does, with a cookie the engine keeps in memory; and the cookie it had left was taken out of the app's test data by having the app itself load a page that forgets it. (Since then what pages keep is in memory in fake mode, and no test can leave such a thing: [001](001-foundations.md) §7, WT-D35) |
| The rule broken, a part at a time | 40 parts, in the core and the store. 38 caught: 35 at once, one once its text was found, two by tests written for them (that a window is not first told without its icons; a full store). Two were not: that a closed private window's numbers are forgotten (nothing can be asked of them either way: their pictures went with the login set), and that a picture handed over again is said to be used (it matters only in a full store while the app runs; no test fills one then) |

## 9. What is not verified

- **That the picture is on the screen of the running app** on Apple's systems: no test sees it. An icon says nothing to a screen reader, which is how a test from outside sees an app; the app's tests see what the site was asked for, the model's tests that the window then has the picture, and the drawn tests that a window with the picture draws it.
- **An iPad:** nothing of this ran there.
- **D2 to D6** are the recommendations, not the owner's word.
- **A site whose pages name different icons** has, for its host, the one that was fetched last in the run: one icon for a host is the rule (§3), and which of a site's several it is was not chosen with any care.
- **Nothing was fetched from the web.** Every fetch that ran was of the test's own site, on this device, over plain `http`: so a fetch over `https`, a certificate judged, and a server that sends a fetch from `https` on to plain `http` were not run. Which addresses may be asked at all is tested as a rule, address by address, without asking any.
- **Whether two of the app's fetches share anything the system keeps for itself** (a connection, a session of TLS that is taken up again): each fetch has a session of its own, which is as far as the app can say; what the system does beneath was not looked at.
- **A server that asks who is there,** or for a certificate of the person's: by its lines the fetch ends; the test site has no such picture, and it was not run. **A picture sent with a status that is no success** is not taken, by its lines; not run either.
- **A proxy:** the fetch uses the system's, as any request of the app's does. A web view is given none of its own today.
- **The picture is decoded in the app's own process,** by the system's decoder: only four kinds, each checked by its first bytes, at most 200 KB and 2,048 pixels a side. That the decoder must take a file for the kind it begins as was not broken on purpose: no file was found that begins as one kind and is decoded as another.
- **A page with more than 16 icons,** and an icon whose address is longer than 2,048 characters: by the page's own script, which no test gave so many.
- **An icon whose address is at another site** than its page's: the tests' icons are at their page's own site. With whose cookies the WebView asks another site for one was not looked at.
- **The icon every site may have at `/favicon.ico`,** for a page that names none: the test site has none there (every page of every test would have it), so that the WebView asks for it and hands it over was not seen.
- **A real phone or tablet;** another version of the System WebView than the emulator's. Down the side, Android's icons are drawn on the JVM only (the emulator is a phone).
- **A tab that is loading and has an icon:** the dot and the icon side by side; in no picture.
- **A picture is not decoded by the core:** that the bytes after the header are a picture is the word of the adapter that made them. A shell decodes what it draws.
- **A long row.** Each tab told to a window now looks up its icon's number: not measured at thousands of rows ([043](043-the-sidebar.md) §8a).
- **An icon a tab shows for longer than a thousand others are met** in one run may be forgotten by the store while the tab still has its number: the picture is then not answered, and comes back when the page next hands it over.
- **Two servers at one host** (two ports) share an icon.

## 10. Done checklist

- [ ] The owner's word on §6: D1 is given (WT-D33); D2 to D6 are not.
- [x] The core and the contract, for an icon an engine hands over.
- [x] Android.
- [x] Apple: the core's choice, and the adapter's fetch.
- [x] Apple's windows.
- [x] The core: `Tools/test.sh all` green, and `android-device`, on 2026-11-23: `lint`, `core` (627, and the strings tool's 24), `unit` (296 on the Mac), `build`, the Mac's UI suite (46), `ios` (278 on the iPhone simulator), the iPhone's UI suite (36), `android` (416 and 32 on the JVM) and `android-device` (160 on the emulator). No shell changed but for the contract's number.
- [x] Android: the same again, on 2026-11-23: `lint`, `core` (628, and the strings tool's 24), `unit` (296 on the Mac), `build`, the Mac's UI suite (46), `ios` (278 on the iPhone simulator), the iPhone's UI suite (36), `android` (425 and 32 on the JVM) and `android-device` (163 on the emulator).
- [x] The test site's pictures, and an icon asked for as its login set: the same again, on 2026-11-23: `lint`, `core` (628, and the strings tool's 24), `unit` (297 on the Mac), `build`, the Mac's UI suite (46), `ios` (279 on the iPhone simulator), the iPhone's UI suite (36), `android` (425 and 32 on the JVM) and `android-device` (165 on the emulator).
- [x] Apple's fetch: `Tools/test.sh all` green, and `android-device`, on 2026-11-23: `lint`, `core` (634, and the strings tool's 24), `unit` (312 on the Mac), `build`, the Mac's UI suite (48), `ios` (294 on the iPhone simulator), the iPhone's UI suite (38), `android` (437 and 32 on the JVM) and `android-device` (167 on the emulator).
- [x] Apple's windows: `Tools/test.sh all` green, and `android-device`, on 2026-11-23: `lint`, `core` (634, and the strings tool's 24), `unit` (318 on the Mac), `build`, the Mac's UI suite (50), `ios` (300 on the iPhone simulator), the iPhone's UI suite (40), `android` (437 and 32 on the JVM) and `android-device` (167 on the emulator).

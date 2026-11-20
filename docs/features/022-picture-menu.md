# 022 — A picture's menu, and what a menu saves (W1)

| | |
|---|---|
| **Status** | Built and tested in the core, the Apple shell and the Android shell (on an emulator). On the Mac, WebKit's own "Download Image" and "Download Linked File" now save, where they fetched the file and threw it away (§5); on the iPhone the link's menu saves what the link leads to; on Android a picture has a menu, and a link's menu saves. Windows is not started |
| **Phase** | W1 of [000](000-master-plan.md) §9: "context menus". After a link's ([018](018-link-in-new-tab.md)): a picture's, and the items of either that save a file |
| **Contract** | 1.16: `DownloadRequested.user_asked` |
| **Builds on** | 018: a link's menu, and an address the person asks for in a new tab. [013](013-downloads.md) and [015](015-first-download.md): a download waits for the core's word, and a site's first one asks |
| **Last updated** | 2026-11-20 |

## 1. User story

There is a picture on a page and I want to keep it, or to see it by itself. I press and hold it (on a phone) or right-click it (on a Mac). "Download Image" puts it with my downloads, where I can see it in the list; "Open Image in New Tab" shows it alone. A link's menu can save what the link leads to, the same way. I chose the item myself, so nobody asks me whether the site may download.

## 2. What is in it, and what is not

**In:**

- **A file a menu saves is a download** (013), and one the person asked for: it starts without the site's first-download question (015).
- **The Mac:** WebKit's own menu. Its items that save, "Download Image" and "Download Linked File", save. Its "Open Image in New Window" opens a tab, as its item for a link does (018), and says so.
- **Android:** the shell's menu for a picture pressed and held: "Open Image in New Tab", "Download Image". A link's menu gains "Download Linked File". A picture that is a link has both.
- **iPhone and iPad:** the link's menu, which is the shell's, gains "Download Linked File".

**Not in:**

- **The iPhone's menu of a picture that is no link.** It stays WebKit's own (§7 D3).
- **Copying a picture itself** on Android, and **sharing**. On Apple's systems those are WebKit's items.
- **What a menu offers for words that are selected**, among them a search for them. That item is the system's on every platform, and goes to the system's choice of search engine, not WebTiger's ([020](020-search-engine.md)): it is the next feature.
- **Pictures a page draws by its style** (a background), and **menus of video and sound**.
- **A picture with no address on the web**, one a page made itself (`data:`, `blob:`): only an address of the web is fetched.

## 3. The rule

**A download the person asked for themselves** is reported like any other (`DownloadRequested`), with `user_asked`. The shell says so only of a choice the person made in a menu: one the shell drew, or the engine's own. For such a download the core:

- drops it when its web view is no tab's, as any other;
- **starts it without asking**, whoever the site is and whatever was said to it before: the question of 015 is whether a *page* may save a file, and no page asked. Nothing is written down about the site: allowing one file is not allowing the site;
- does not hold it to one download a second (013 §3), and does not count it against the page's: that rule is against a page that sends many;
- leaves what the person typed as it was: an address they sent the tab to is still good for its own download (015 §3);
- names it, lists it and reports it as any other (013).

**A picture opened in a new tab** is `LinkOpenRequested` (018) with the picture's address, and 018's rule as it stands: only an address of the web opens a tab, behind the one that has the picture. (On the Mac the item is WebKit's, and asks for a window: 006.)

## 4. The contract (1.16)

Added; nothing is changed or removed.

```proto
// engine.proto
message DownloadRequested {
  ...
  bool user_asked = 6;                      // the person chose it in a menu: no page asked
}
```

## 5. The engines and the shells

| | Mac | iPhone and iPad | Android |
|---|---|---|---|
| The menu of a picture | WebKit's own. "Open Image in New Window" asks for a window (006), which the core opens as a tab in front: the item is renamed "Open Image in New Tab". "Download Image" saves the picture. "Copy Image" is WebKit's | WebKit's own (D3) | The shell's, as a link's: "Open Image in New Tab", "Download Image" |
| The menu of a link | As in 018; "Download Linked File" saves what the link leads to | The shell's (018), with "Download Linked File" | The shell's (018), with "Download Linked File" |
| How the adapter knows what was pressed | WebKit does not say what its menu is for. A script of the adapter's own, in a world no page can reach, tells the adapter what picture and what link a right-click was on, as the click happens | WebKit says what link was pressed | The WebView says what was pressed: a picture, a link, or both |
| What saves the file | WebKit, asked to download the address in the page's own session | The same | The system's downloader, as for every download (013) |

**As found on the Mac, before this** (macOS 27, in the app in fake mode): WebKit's menu for a picture has "Open Image in New Window", "Download Image", "Copy Image", "Copy Subject" and "Look Up"; for a picture that is a link also "Open Link", "Open Link in New Tab" (018's words), "Download Linked File", "Copy Link" and "Share…". "Open Image in New Window" opened a tab in front. **"Download Image" and "Download Linked File" each asked the site for the file, and nothing was saved, listed, or said**: WebKit hands such a download to an app in a way that is not open to one that keeps to the system's declared interfaces, and drops it when nobody takes it.

**As built on Apple** (`WebKitMenus.swift`: `saveFile`, `PressedReports`, `MenuWebView`; `Downloads`), and as seen against real WebKit:

- **A file a menu saves** is asked of WebKit as a download of that address, in the web view's own session: it was fetched as the web view's login set (its cookies), and another login set's web view was nobody there. It is the adapter's download from the moment WebKit has made it, and is reported like any other when WebKit asks where it goes, with `user_asked`; a download a page's own link starts is reported without. Only an address of the web (`http`, `https`) is fetched: for anything else nothing happens.
- **A file that could not be fetched at all** (the site is not there): WebKit never asks where it goes. It is reported all the same, as the person's own and with no name suggested; when the core has listed it, it is said to have failed. The person asked, and can see what became of it.
- **The Mac's script** is put in every page and every frame, in a world of the adapter's own: a page's scripts cannot see it, and cannot call what it calls (seen: the page has no such thing). It listens for the menu's event on the window, before any listener of the page's, since it is there first: a page that takes the event for itself and lets nothing else hear it was heard all the same. It takes only an event the engine made: a right-click a page's own script made up told the adapter nothing. It says the address of the picture (the innermost `img` the click was in, as shown) and of the link (the innermost `a` or `area` with an address), where they are addresses of the web.
- **What was said is of one click**: it is used by the menu that opens next in that web view, once, and only within five seconds. A menu that opens with nothing said of it is given nothing to save.
- **The Mac's menu**, as WebKit hands it over before it is shown: the items for a link and for a picture "in a new window" are given the shell's words ("Open Link in New Tab", "Open Image in New Tab"). In place of each of WebKit's two items that save stands one of the adapter's own, under WebKit's words and name, that saves the address the script named. **Where the script named none, that item is taken out**, and no line of the menu is left at an end or beside another: an item that does nothing is not shown. So a picture a page made itself (`data:`, `blob:`) is not offered for saving on a Mac.
- **In the Mac app**: a right-click on the test site's picture, and WebKit's menu has "Open Image in New Tab" and "Download Image" and, there being no link, no "Download Linked File"; "Download Image", and the picture is in the list of downloads, whole, with nobody asked whether the site may download.
- **The iPhone's menu of a link** has "Download Linked File" between "Open in New Tab" and "Copy Link". In the app on the simulator: a finger held on the link, "Download Linked File", and what the link leads to is in the list, whole, with nobody asked.

**As built on Android** (`WebViewPage`, `SystemWebViewEngine.onPressed` and `saveFile`; `LinkMenu`, `BrowserModel`), and as seen on the emulator (System WebView 133):

- **A long press on a picture** is the adapter's, as one on a link is (018). The WebView names a picture's address at once; of a picture that is a link it names the picture at once and, asked, where the link leads. Seen with a finger held on the test site's pictures: a picture was told with its own address and no link; a picture that is a link with both, the link's being the link's and not the picture's. A picture a page made itself (`data:`, `blob:`) is not told of: alone, its press stays the WebView's; in a link, the menu is the link's.
- **The menu** is the one a link has, with what was pressed deciding its choices. For a link: where it leads, "Open in New Tab", "Download Linked File", "Copy Link". For a picture: "Open Image in New Tab", "Download Image"; with no link, the address the menu shows is the picture's, and it is named "Image" to a screen reader. A picture that is a link has all five. Then "Cancel".
- **The menu scrolls.** With five choices it is taller than a short window has room for under its toolbar, a phone held sideways among them: drawn without scrolling, its last choices and "Cancel" were cut off, which the first pictures of it showed. Each choice can now be brought into view.
- **"Open Image in New Tab"** tells the core of the picture's address as it does of a link's (`LinkOpenRequested`), behind.
- **"Download Image" and "Download Linked File"** have the page offer the address as a download the person chose, reported with `user_asked`. The system's downloader fetches it, with the login set's cookies, as every download (013).
- **Before it is offered, the adapter asks where the file is and what its site says of it** (`FileAsker.kt`), fetching nothing of it. A download a page starts comes with that: the WebView has followed its address and says its kind, its name and its size. A file a menu saves is only an address, and two things came of that:
  - **The system's downloader sends the cookies it is handed on to every site the address sends it to.** Seen on the emulator: a file of one site, at an address that sent the downloader on to another site, was fetched there with the first site's cookies. So the adapter follows the address by hand: each site on the way is asked for the address's head alone (or, where a site will not answer that, for the file, of which nothing is read), with that site's own cookies and no other's, up to ten times. The file is offered as being where that ends, and the downloader is handed that address with that site's cookies. Seen after: the same file was fetched at the second site as nobody.
  - **The core makes the file's name, and did not know its kind.** A picture whose address has no ending was saved without one. The site's answer says the kind, and any name it suggests: the test site's picture at such an address was saved with the ending of its kind.
- **Where nobody answers**, or the answer is no file's, the file is offered as being at the last address there was, with nothing known of it; the downloader then finds what there is to find.
- **In the app on the emulator**: a finger held on the test site's picture, and the menu says what picture it is and offers nothing for a link; "Download Image", and the system fetched the picture into the app's folder, the file being the picture itself, with nobody asked; a picture that is a link showed all five choices.

## 6. Tests

| Where | What |
|---|---|
| Core (`wt-app/tests/first_download.rs`, 4 more, through `FakeShell`) | A file the person chose starts with nobody asked, on a site that was never allowed; it is named and listed as any other; nothing is written about the site, and the page's own next download still asks; it starts while that question is there, and after the person said no to it. It is not held to a page's one download a second, nor counted against it: the page's own, in the same moment, is asked about and not dropped for coming too soon. It does not use up what the person typed. One for a web view that is no tab's is dropped. Three parts of the rule checked by breaking them |
| The test site | A page with a picture, a picture that is a link, and words; and the same page taking the menu's event for itself before anything else of the page can hear it, which keeps the engine's menu from opening |
| WebKit adapter suite (`WebKitMenuTests`, 10 on the Mac, 5 on the iPhone simulator) | Against real WebKit. A file chosen is reported as the person's own, with what the site says of it, and saved under the name the core gives; the page stays where it is. A download a page starts is not said to be the person's. A file chosen is fetched as its login set. Only an address of the web is fetched, and only for a web view there is. A file that cannot be fetched is reported, and said to have failed once the core has listed it. On the Mac, with the mouse's own events given to a web view in a window that is not on the screen: a right-click tells the adapter the picture's address, on a picture that is a link the link's as well, and on words neither; said once, and only for a moment. A right-click a page's script makes up tells the adapter nothing, and the page cannot reach what the script calls. The menu that opens after a right-click on the picture saves that picture, and one that opens with nothing said saves nothing. The menu's items are given the shell's words and their work, those items and no other; an item with nothing to save is taken out and leaves no line behind. Fourteen of the rules checked by breaking them |
| The Apple shell (`LinkMenuTests`) | A session gives its engine the words for a picture's item and for what a link's menu saves |
| System WebView adapter, on the JVM (`LinksTest`, 3 more; `DownloadsTest`) | A long press on a picture is told with the picture's address, and the link's when it is one; a picture that is not of the web is not told of, and one that is a link is told as the link. A file a menu saves is offered to the core as the person's own, for the web view whose page has it, with no name suggested; only an address of the web is, and only for a web view there is. What a page offers is not said to be the person's |
| System WebView adapter suite (`SystemWebViewLinkTest`, 2 more; `SystemWebViewDownloadTest`, 1 more; on the emulator) | With a finger pressed and held on the test site's pages, against the real WebView: a picture is told with its address, the page stays, and nothing is said to the core; a picture that is a link is told with the link's address and the picture's; a picture chosen to be saved is offered to the core as the person's own, with the kind of file its site says it is. A file the person chose is fetched as its login set; the same file at an address that sends on to another site is offered as being there, with what that site says of it, and is fetched there as nobody |
| Where a chosen file is (`FileAskerTest`, 7; `DownloadsTest`, 1 more; on the JVM) | A file where its address says is told with what its site says; one that is somewhere else is asked after there with that site's cookies and no other's; a site that sends on with only a path sends on within itself; the asking stops where nobody answers, where the answer is no file's, where the next address is not the web's, and after ten. A chosen file is asked after first, as the page's requests say who they are and in its login set, and offered as where it is; a page's own offer is not asked after. Eight of the rules checked by breaking them, one of them on the emulator |
| The Android menu (`LinkMenuTest`, 6 more, drawn on the JVM against `ScriptedCore`) | A link in words is offered nothing for a picture; a picture shows its own address and its two choices, and nothing for a link; a picture that is a link has every choice of either. "Download Linked File" offers what the link leads to, not the picture, as the person's own. "Open Image in New Tab" asks the core for a tab behind, with the picture and not where the link leads. "Download Image" offers the picture. Each puts the menu away. The last of the menu's choices can be reached in a window too short for them all. Pictures of the three menus, light and dark, drawn tall so that the whole menu is seen; each looked at. Fourteen of the rules checked by breaking them |
| The Android app, on the emulator | With the real core and WebView and the system's downloader, in fake mode: a finger held on a picture; the menu, over which Android's accessibility checks pass; "Download Image", and the picture is in the app's folder, whole, with nobody asked; a picture that is a link has both menus' choices; a picture whose address does not say its kind is saved under a name with the ending of its kind |
| The Apple apps (`MenuTests`) | With the real core and WebKit, in fake mode. On the Mac: the picture's menu, opened with the mouse, has the shell's words for a new tab and no item for a link there is not; "Download Image" saves the picture, which is listed, whole, with nobody asked; the page is where it was. On the iPhone simulator: the link's menu saves what the link leads to, which is listed, whole, with nobody asked |

## 7. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | A file a menu saves is not asked about | The person chose it. Asking "may this site download?" of a choice they just made teaches them to say yes without reading |
| D2 | One file allowed is not the site allowed | The person said yes to a picture, not to whatever the page sends next |
| D3 | The iPhone's menu of a picture stays WebKit's | It keeps a picture in Photos, copies it and shares it, which is what a picture is wanted for on a phone. A menu of the shell's own would replace all of that, and WebKit does not say which picture was pressed |
| D4 | On the Mac, WebKit's items are made to work, not replaced | As 018 D5: WebKit builds that menu. What the adapter adds is the address, from a script no page can reach |
| D5 | An item that cannot do its work is taken out | The alternative is what there was: an item that looks as if it saved |
| D6 | The file is fetched with no word of the page it was on | Which page a person was reading is theirs to tell. A site that gives its pictures only to its own pages may refuse (§8) |
| D7 | On Android the site is asked about a chosen file before it is fetched | One more question to the site, of a file the person asked for. Without it, what one site knows the person by is sent to another, and a picture is saved with no ending |

## 8. What is not verified

- **That the script's word always comes before the menu.** It did, each time the app's test opened the menu on the Mac. Were it ever late, the items that save would be missing from that menu, not wrong.
- **"Download Linked File" in the Mac's real menu.** The app's test chooses "Download Image"; the item for a link is the same code, tested with a menu the test made.
- **A picture in a frame**, and one inside a part of the page that is closed to scripts: the script is put in every frame, and no test has a frame. A picture it cannot see is not offered for saving.
- **Pictures that are not `img`** (a form's picture button, a drawing's own picture): WebKit's menu may offer to save them; the script names none, and the item is taken out.
- **A site that gives its pictures only to its own pages.** The file is asked for without saying which page it was on (D6); such a site may refuse, and what it sends instead is what is saved. Not tried.
- **Whether the picture is fetched again** or taken from what WebKit already has: not looked at.
- **The iPhone's own menu of a picture** (WebKit's): not opened by any test. What it offers was not looked at.
- **Android: a site that sends the downloader on differently from how it sent the adapter on.** The adapter follows a chosen file's address first and hands the downloader where that ended. A site that sends on again from there, or elsewhere the second time, would have the downloader carry that address's cookies on, as before (013 §9). Not tried.
- **Android: a site that will not answer for a head alone**, and one that takes long to answer (the adapter waits five seconds for each): tested as the walk's own rules, not against a site.
- **Android: a cookie a site sets while it sends on** is not kept by the asking, as a browser's own load would keep it.
- **Android: "Open Image in New Tab" and "Download Linked File" in the app.** The app's test on the emulator chooses "Download Image"; the other two are the menu's tests on the JVM.
- **Android: that nobody is asked, in the app.** The app's tests on a device share one core, in which another test may have allowed the site already. That a file the person chose is never asked about is the core's own test.
- **Android: a phone**, and one held sideways. It ran on an emulator, upright; the menu's scrolling is tested on the JVM.

## 9. Done checklist

- [x] Contract 1.16 and its version in every place it is written.
- [x] The core: the rule of §3 (4 tests).
- [x] The Mac: the script; the menu's items; the adapter suite (10); a UI test.
- [x] Android: the picture's menu and "Download Linked File"; the adapter suite on the emulator (2 more); pictures; a device test of the app.
- [x] iPhone: "Download Linked File" in the link's menu; the adapter suite on the simulator (5); a UI test.
- [ ] `Tools/test.sh all` green, and `android-device`.
- [ ] Windows shell.

# 022 — A picture's menu, and what a menu saves (W1)

| | |
|---|---|
| **Status** | Designed. Nothing is built yet. On the Mac today, WebKit's own menu offers "Download Image" and "Download Linked File", and in WebTiger both fetch the file and throw it away: nothing is saved and nothing is said (§5, as found) |
| **Phase** | W1 of [000](000-master-plan.md) §9: "context menus". After a link's ([018](018-link-in-new-tab.md)): a picture's, and the items of either that save a file |
| **Contract** | 1.16: `DownloadRequested.user_asked` |
| **Builds on** | 018: a link's menu, and an address the person asks for in a new tab. [013](013-downloads.md) and [015](015-first-download.md): a download waits for the core's word, and a site's first one asks |
| **Last updated** | 2026-11-20 |

## 1. User story

There is a picture on a page and I want to keep it, or to see it by itself. I press and hold it (on a phone) or right-click it (on a Mac). "Save Picture" puts it with my downloads, where I can see it in the list; "Open Picture in New Tab" shows it alone, in a tab behind the one I am reading. A link's menu can save what the link leads to, the same way. I chose the item myself, so nobody asks me whether the site may download.

## 2. What is in it, and what is not

**In:**

- **A file a menu saves is a download** (013), and one the person asked for: it starts without the site's first-download question (015).
- **The Mac:** WebKit's own menu. Its items that save, "Download Image" and "Download Linked File", save. Its "Open Image in New Window" opens a tab, as its item for a link does (018), and says so.
- **Android:** the shell's menu for a picture pressed and held: "Open Picture in New Tab", "Save Picture". A link's menu gains "Download Linked File". A picture that is a link has both.
- **iPhone and iPad:** the link's menu, which is the shell's, gains "Download Linked File".

**Not in:**

- **The iPhone's menu of a picture that is no link.** It stays WebKit's own (§7 D3).
- **Copying a picture itself** on Android, and **sharing**. On Apple's systems those are WebKit's items.
- **What a menu offers for words that are selected**, among them a search for them. That item is the system's on every platform, and goes to the system's choice of search engine, not WebTiger's ([020](020-search-engine.md)): it is the next feature.
- **Pictures a page draws by its style** (a background), and **menus of video and sound**.
- **A picture with no address on the web**, one a page made itself (`data:`, `blob:`), on Android: the system's downloader fetches from the web only.

## 3. The rule

**A download the person asked for themselves** is reported like any other (`DownloadRequested`), with `user_asked`. The shell says so only of a choice the person made in a menu: one the shell drew, or the engine's own. For such a download the core:

- drops it when its web view is no tab's, as any other;
- **starts it without asking**, whoever the site is and whatever was said to it before: the question of 015 is whether a *page* may save a file, and no page asked. Nothing is written down about the site: allowing one file is not allowing the site;
- does not hold it to one download a second (013 §3), and does not count it against the page's: that rule is against a page that sends many;
- names it, lists it and reports it as any other (013).

**A picture opened in a new tab** is `LinkOpenRequested` (018) with the picture's address, and 018's rule as it stands: only an address of the web opens a tab, behind the one that has the picture.

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
| The menu of a picture | WebKit's own. "Open Image in New Window" asks for a window (006), which the core opens as a tab in front: the item is renamed "Open Image in New Tab". "Download Image" saves the picture. "Copy Image" is WebKit's | WebKit's own (D3) | The shell's, as a link's: "Open Picture in New Tab", "Save Picture" |
| The menu of a link | As in 018; "Download Linked File" saves what the link leads to | The shell's (018), with "Download Linked File" | The shell's (018), with "Download Linked File" |
| How the adapter knows what was pressed | WebKit does not say what its menu is for. A script of the adapter's own, in a world no page can reach, tells the adapter what picture and what link a right-click was on, as the click happens | WebKit says what link was pressed | The WebView says what was pressed: a picture, a link, or both |
| What saves the file | WebKit, asked to download the address in the page's own session | The same | The system's downloader, as for every download (013) |

**As found on the Mac, before this** (macOS 27, in the app in fake mode): WebKit's menu for a picture has "Open Image in New Window", "Download Image", "Copy Image", "Copy Subject" and "Look Up"; for a picture that is a link also "Open Link", "Open Link in New Tab" (018's words), "Download Linked File", "Copy Link" and "Share…". "Open Image in New Window" opened a tab in front. **"Download Image" and "Download Linked File" each asked the site for the file, and nothing was saved, listed, or said**: WebKit hands such a download to an app in a way that is not open to one that keeps to the system's declared interfaces, and drops it when nobody takes it.

**The Mac's script.** It is put in every page and every frame, in a world of the adapter's own: a page's scripts cannot see it, call what it calls, or change what it reads from the event. It listens for the menu's event before the page's own listeners, takes only an event the engine made (never one a page's script made), and tells the adapter the address of the picture and of the link the event was on. WebKit opens its menu after that. When the menu opens, the adapter gives its items that save something to do: download that address. **When the adapter was told nothing** (a page the script is not in, an element it does not know), **those items are taken out of the menu**: an item that does nothing is not shown.

What each does in fact, as observed, is written here when built.

## 6. Tests

| Where | What |
|---|---|
| Core (through `FakeShell`) | A download the person asked for starts without asking, on a site that was never allowed, and nothing is written about the site: the page's own next download still asks. It starts though the page was just refused, and within a second of another. It is not started for a web view that is no tab's. It is named and listed as any other |
| The test site | A page with a picture, a picture that is a link, and words; and the same page keeping the engine's menu from opening |
| WebKit adapter suite, against real WebKit | A right-click on a picture, with the mouse's own events, tells the adapter the picture's address, and on a picture that is a link the link's as well; a click a page's script makes tells it nothing. The menu's items that save are given the address, and are taken out when there is none. A picture asked for is downloaded as the person's own |
| System WebView adapter suite, on the emulator | A finger held on a picture is told with the picture's address; on a picture that is a link, with both; a picture asked for is offered to the core as the person's own |
| The shells' menus | What each shows for a picture, a link, and a picture that is a link; what each choice asks, and that nothing is asked until one is made |
| The apps | On the Mac: "Download Image" in WebKit's menu saves the picture, with nobody asked, and it is in the list. On the emulator: a picture pressed and held, "Save Picture", and it is in the list |

## 7. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | A file a menu saves is not asked about | The person chose it. Asking "may this site download?" of a choice they just made teaches them to say yes without reading |
| D2 | One file allowed is not the site allowed | The person said yes to a picture, not to whatever the page sends next |
| D3 | The iPhone's menu of a picture stays WebKit's | It keeps a picture in Photos, copies it and shares it, which is what a picture is wanted for on a phone. A menu of the shell's own would replace all of that, and WebKit does not say which picture was pressed |
| D4 | On the Mac, WebKit's items are made to work, not replaced | As 018 D5: WebKit builds that menu. What the adapter adds is the address, from a script no page can reach |
| D5 | An item that cannot do its work is taken out | The alternative is what there is today: an item that looks as if it saved |

## 8. What is not verified

Written when built.

## 9. Done checklist

- [ ] Contract 1.16 and its version in every place it is written.
- [ ] The core: the rule of §3.
- [ ] The Mac: the script; the menu's items; the adapter suite; a UI test.
- [ ] Android: the picture's menu and "Download Linked File"; the adapter suite on the emulator; pictures; a device test of the app.
- [ ] iPhone: "Download Linked File" in the link's menu; the adapter suite on the simulator.
- [ ] `Tools/test.sh all` green, and `android-device`.
- [ ] Windows shell.

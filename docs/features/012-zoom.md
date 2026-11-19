# 012 — Zoom (W1)

| | |
|---|---|
| **Status** | Built and tested in the store, the core, the Apple shell and the Android shell (on an emulator). Windows is not started. What it leaves out is in §2; Android scales a page's text, not the page (§6) |
| **Phase** | W1 of [000](000-master-plan.md) §9: "zoom". The `WebViews` host service's zoom (000 §5.5), and the store's `site_setting` table (000 §5.6) in its first form |
| **Contract** | 1.8: `ZoomPage`; the host request `WebViewSetZoom`; `TabView.zoom` |
| **Last updated** | 2026-11-19 |

## 1. User story

A site's text is too small for me. I make the page bigger and it stays bigger: on every page of that site, in every tab, and when I open WebTiger again. Other sites are as they were. One press puts it back.

## 2. What is in it, and what is not

**In:** bigger, smaller and actual size, for the page of the tab in front; the level remembered for the site, in the Space's login set; the level in the tab's snapshot; a control in each shell.

**Not in, each with where it goes:**

- **A level for every site at once**, and choosing between scaling the page and scaling its text: settings.
- **Pinching** on a touch screen: the engine's own, as it is now, and not remembered.
- **Forgetting a site's zoom.** Clearing history does not clear it: it is a setting, though one that says the site was visited. It is forgotten with the other things a site is remembered by, when there is a screen for those.
- **Private windows** (W2): nothing of one is ever written.

## 3. Who does what

The shell draws the control and says "bigger", "smaller" or "actual size". The core knows the levels, which site a page is of, and what the site was last given; it tells every web view that shows the site. The engine zooms.

| Request | What the core does | Answer |
|---|---|---|
| `ZoomPage { tab_id, change }` | Gives the site of the tab's page the next level up, the next down, or 100% | `Ack`, also when there is no further level; `tab.unknown`; `tab.no_page` for a tab with no page, or a page with no site; `tab.bad_command` for a change that is none of the three |

- **The levels** are 50, 67, 75, 80, 90, 100, 110, 125, 150, 175, 200, 250 and 300 per cent. A shell does not know them: the snapshot says the level, and whether there is one above it and one below.
- **A site** is what someone registered: `example.co.uk` for `www.example.co.uk` and for `shop.example.co.uk`, whatever the scheme and the port. A host with nothing registered (an IP address, `localhost`) is its own site. A page with no host (`about:blank`) has no site and no zoom.
- **A level is the site's, in the login set.** Changing it tells every tab of that login set whose page is of the site, at once, in the order the tabs were opened; and it is written to the store before the request is answered. A site at 100% is not written: that is what a site nobody zoomed has.
- **A page that arrives is given its site's level.** When a page commits in a tab, the core asks the tab's web view for the site's level if the web view was last told another. A new web view is at 100%, and is told nothing until its page's site has another level.
- **The snapshot says what the core asked for** (`TabView.zoom`), not what an engine did: an engine does not report its zoom. Not set while the tab has no page with a site.

## 4. The contract (1.8)

Added; nothing is changed or removed.

```proto
// browser.proto
message ZoomPage {
  uint64 tab_id = 1;
  ZoomChange change = 2;
}

enum ZoomChange {
  ZOOM_CHANGE_UNSPECIFIED = 0;
  ZOOM_CHANGE_IN = 1;       // bigger
  ZOOM_CHANGE_OUT = 2;      // smaller
  ZOOM_CHANGE_RESET = 3;    // 100%
}

// What a tab's page is zoomed to: TabView.zoom. Not set while the tab has no page with a site.
message ZoomView {
  uint32 percent = 1;
  bool can_zoom_in = 2;     // there is a level above
  bool can_zoom_out = 3;    // there is a level below
}

// host.proto: WebViews. Sets a web view's zoom, which stays until it is set again.
message WebViewSetZoom {
  uint64 web_view_id = 1;
  uint32 percent = 2;       // 100 is actual size
}
```

## 5. The store

Migration 4 adds `site_setting`, the table 000 §5.6 names, with the one column this needs; permissions, Guard's switches and the engine arrive with what uses them.

```sql
CREATE TABLE site_setting (
    login_set_id BLOB NOT NULL REFERENCES login_set (id),
    site TEXT NOT NULL,
    zoom INTEGER NOT NULL CHECK (zoom BETWEEN 25 AND 500),
    PRIMARY KEY (login_set_id, site)
) STRICT, WITHOUT ROWID;
```

## 6. The shells

Bigger, smaller, and the level between them, which put back to 100% when pressed. On a Mac also ⌘+, ⌘− and ⌘0.

| | WebKit | System WebView |
|---|---|---|
| Zooms | `WKWebView.pageZoom`: the page is laid out again at the level | `WebSettings.textZoom`: the text is scaled. The WebView has no public way to zoom the whole page |

**As built on Android** (`WebViewPage.zoom`; `ZoomBar` in `BrowserWindow`; the model's part in `BrowserModel`), and what the device suite found:

- **The WebView scales the text, and the page is laid out in the room it had.** At 200% a line's words are twice as wide and the page is as wide as it was; pictures, and whatever a page sizes in its own units, stay as they were. That is all the System WebView offers an app: it has no public way to zoom a page as a whole, as WebKit has. The level is the same number in both shells, and means less here.
- The WebView keeps its text size for the page it goes on to, and a web view made anew (after its renderer died) is given the level again. One web view's is not another's. A level the core does not have is held to between half and five times.
- **A field read before it had its value.** The first form of this kept the level in a field declared below the one that makes the WebView, which is made as the page object is, and is given the level as it is made: the level was still nothing then, and every new web view drew its text at no size at all. The adapter suite's first test, which measures a page's words before any zoom is asked for, said so before anything was committed. The field is now above the one that reads it.
- The same bar as on Apple, without the keys: smaller, the level, bigger, Done, under the toolbar and in place of the find bar. A screen reader is told what pressing the level does ("Actual Size") and the level as its state. The system's Back closes the bar before it does anything to the page.
- With a third button beside the tabs, on a narrow phone with three tabs the third tab's own close button is past the strip's end, as the button for a new tab already was. The strip is W2's to lay out again.

**As built on Apple** (`WebKitEngine`'s answer to `WebViewSetZoom`; `ZoomBar`; the model's part in `BrowserModel`):

- WebKit's own page zoom, as the adapter suite found it: the page is laid out again in the web view's width divided by the level, with its text the size it was in the page's own units; the web view keeps the level for the page it goes on to, so the core asks only for a change; one web view's level is not another's. A level the core does not have is held to between half and five times.
- **A page that does not say how wide it is to be laid out** is laid out 980 wide on an iPhone and shrunk to fit, as in Safari; its own measures change with the level in a way the suite could not make sense of, and a picture of it could not be taken off screen (§8). The suite's page says it is as wide as the device, as pages made for phones do, and measures the same on the iPhone as on the Mac.
- The zoom bar is under the toolbar, where the find bar is when that is open instead: one bar at a time. Smaller, the level, bigger, Done; pressing the level puts the page back, and at 100% there is nothing to press. It is opened by a button beside the tabs, which is off without a page that can be zoomed. On a Mac ⌘+ (and ⌘=), ⌘− and ⌘0 zoom with the bar open or shut.
- The bar says what the core says: the model keeps no level, and which buttons can be pressed is the snapshot's "one above" and "one below". It closes when the page in front cannot be zoomed.
- **The catalogue's pictures did not notice the new button.** A button in the strip is a thousandth of a picture, and the pictures match within one part in a hundred ([004](004-design-catalogue.md) §4.1): every window picture went on matching without it. They were all recorded again and looked at. The catalogue's pages now say the zoom a core would say of them, so the button is drawn on where a real window's is.

## 7. Tests

| Where | What |
|---|---|
| Store (`wt-store`, 5) | The migration from version 3, which keeps what was there, and the level is there when the store is opened again. A site's level is written, read and replaced; at 100% nothing is left written of the site; another site's and another login set's are not touched; a level the store cannot mean, and a login set it does not have, are refused and change nothing |
| The levels and the site (`wt-core`, 5) | Up one level at a time to the largest, and down to the smallest; actual size has one on each side; a level that is none of them goes to its neighbours. A site is what was registered, or else the host, whatever the scheme and the port; an address with no host has none |
| Core (`wt-app/tests/zoom.rs`, 10 through `FakeShell`) | Bigger, smaller and actual size each ask the tab's web view, one level at a time, and the snapshot says the level. The ends: nothing asked, and the snapshot says there is nothing above, or below. Actual size at actual size asks nothing. No such tab; a tab with no page; a page with no site; a change that is none of the three. Every tab of the site is told, in the order they were opened, and a tab of another site is not; a tab behind has its own page's level. A page that arrives is given its site's level, in the same tab and in a new one, and one whose site is at the level the web view has is told nothing. The site: another host of it by another scheme and port; an IP address is its own. A page changing its own address asks nothing. Started again, the tab that came back is given the level its site was left at; a site put back to 100% is told nothing. Six of the rules checked by breaking them |
| WebKit adapter suite (`WebKitZoomTests`, 5, on the Mac and the iPhone simulator) | A zoomed page is laid out in half the room at 200% and twice at 50%, with its text as tall as it was in the page's units, and in all of it again at 100%; the zoom stays for the page the web view goes on to; one web view's zoom is not another's; a level no page can be read at is held to half, and to five times; a web view that is not there is refused with a code |
| The Apple model and bar | `ZoomModelTests` (5): nothing to zoom, and nothing asked, without a page; bigger, smaller and actual size ask the core for the tab in front, with the bar shut too; the bar says what the core says, and nothing of its own until then; one bar at a time; the bar follows the page in front and closes when that cannot be zoomed. Pictures of the bar at 125%, light and dark, on the Mac and the iPhone |
| System WebView adapter | On the JVM: a zoom reaches the page of the web view the core named, and a web view that is not there is refused (1). On the device (`SystemWebViewZoomTest`, 5): a page's words have a width before any zoom; at 200% they are twice as wide and the page is as wide as it was, at 50% half, at 100% as they were; the zoom stays for the page the web view goes on to; another web view is at actual size; a level no page can be read at is held to half; a web view that is not there |
| The Android model and bar | `ZoomModelTest` (5): the same five as Apple's. `ZoomBarTest` (3, drawn on the JVM): shut until asked for, and off without a page; what it sends for smaller, bigger and the level, saying only what the core says; the ends of the levels cannot be pressed. Pictures of the bar at 125%, light and dark; every window picture recorded again for the strip's new button |
| The Android app, on the emulator, with the real core and WebView | The bar is shut until asked for; bigger twice and smaller once, the bar saying the level and the WebView drawing its text at it; Android's accessibility checks over the bar; pressing the level puts the page back; left at 110%, another page of the site has the level; the system's Back closes the bar |
| The Apple apps, with the real core and engine | On the Mac and the iPhone simulator: the bar is shut until asked for; bigger twice, smaller once, with the level the core says after each; Apple's audit over the bar; pressing the level puts the page back, and at 100% it cannot be pressed; left at 110%, another page of the site has the level. On the Mac also the four keys |

## 8. What is not verified

- **Another login set, through the core.** There is one Space and one login set until W2: that a site's level in one is not another's is the store's test, not the core's.
- **A shell that refuses.** When a web view cannot be zoomed the level is still the site's and the snapshot still says it: an engine does not report its zoom, so the core cannot know. No shell asks yet.
- **What a zoomed page looks like.** The adapter suite measures how wide a page is laid out; nothing looked at a page to see it bigger. The app's test reads the bar, which says what the core asked for.
- **A page that does not say how wide it is, on an iPhone** (above): zoomed through the same call, with measures the suite does not assert.
- **Text sizes.** The bar was audited at the size the simulator starts with; the level's button at the largest text sizes was not looked at.
- **A physical Android device.** Zoom was used on an emulator.
- **That text is what a person wants bigger on Android.** A page of pictures, or one that sizes its text in a way the WebView does not scale, is not helped by this; none was tried.

## 9. Done checklist

- [x] The store: migration 4; a site's zoom. 5 tests.
- [x] Contract 1.8 and its version in every place it is written.
- [x] The core: the request, the levels, the site, the rules of §3. 10 tests through `FakeShell`, 5 of the levels and the site.
- [x] Apple: the adapter and its suite (5); the bar and the model's tests (5); pictures; a UI test on the Mac and the iPhone simulator.
- [x] Android: the adapter, on the JVM (1) and the device (5); the bar and the model's tests (8); pictures; a device test of the app.
- [x] `Tools/test.sh all` green, and `android-device` (84 on the emulator).
- [ ] Windows shell.

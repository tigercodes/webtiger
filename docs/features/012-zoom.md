# 012 — Zoom (W1)

| | |
|---|---|
| **Status** | In progress. The store, the contract (1.8) and the core are built and tested; no shell zooms yet, and each answers the core's request as a shell built before 1.8 does (§9) |
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

Written out when the shells are built.

## 7. Tests

| Where | What |
|---|---|
| Store (`wt-store`, 5) | The migration from version 3, which keeps what was there, and the level is there when the store is opened again. A site's level is written, read and replaced; at 100% nothing is left written of the site; another site's and another login set's are not touched; a level the store cannot mean, and a login set it does not have, are refused and change nothing |
| The levels and the site (`wt-core`, 5) | Up one level at a time to the largest, and down to the smallest; actual size has one on each side; a level that is none of them goes to its neighbours. A site is what was registered, or else the host, whatever the scheme and the port; an address with no host has none |
| Core (`wt-app/tests/zoom.rs`, 10 through `FakeShell`) | Bigger, smaller and actual size each ask the tab's web view, one level at a time, and the snapshot says the level. The ends: nothing asked, and the snapshot says there is nothing above, or below. Actual size at actual size asks nothing. No such tab; a tab with no page; a page with no site; a change that is none of the three. Every tab of the site is told, in the order they were opened, and a tab of another site is not; a tab behind has its own page's level. A page that arrives is given its site's level, in the same tab and in a new one, and one whose site is at the level the web view has is told nothing. The site: another host of it by another scheme and port; an IP address is its own. A page changing its own address asks nothing. Started again, the tab that came back is given the level its site was left at; a site put back to 100% is told nothing. Six of the rules checked by breaking them |
| The shells | Written when built |

## 8. What is not verified

- **Another login set, through the core.** There is one Space and one login set until W2: that a site's level in one is not another's is the store's test, not the core's.
- **A shell that refuses.** When a web view cannot be zoomed the level is still the site's and the snapshot still says it: an engine does not report its zoom, so the core cannot know. No shell asks yet.
- The shells: written when built.

## 9. Done checklist

- [x] The store: migration 4; a site's zoom. 5 tests.
- [x] Contract 1.8 and its version in every place it is written.
- [x] The core: the request, the levels, the site, the rules of §3. 10 tests through `FakeShell`, 5 of the levels and the site.
- [ ] Apple: the adapter and its suite; the control; pictures; a UI test.
- [ ] Android: the adapter, on the JVM and the device; the control; pictures; a device test.
- [ ] `Tools/test.sh all` green, and `android-device`.
- [ ] Windows shell.

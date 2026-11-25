# 059 — A sleeping tab's picture (W3)

| | |
|---|---|
| **Status** | **Designed, not built.** The choices of §6 are recommendations, put to the owner and not yet answered. Its first step is to try what each engine can do (§5): until then two of them are guesses, and are marked so |
| **Phase** | W3 of [000](000-master-plan.md) §9: "snapshots". 000 §5.7: "Asleep. The web view is gone. A snapshot, the saved state and the scroll position remain. Cost: a database row and a small image on disk." 000 §7: "Wake a sleeping tab: snapshot within 50 ms; the engine loading within 100 ms." The ninth of W3 |
| **Contract** | 1.50, to be added (§4): a web view's picture asked of a shell, and a picture asked of the core. The store's schema would be 17 |
| **Builds on** | [051](051-a-tabs-kept-state.md): a tab's kept state, which this follows in almost everything. [053](053-a-tab-sleeps-by-itself.md): putting a tab to sleep. [046](046-a-sites-icon.md): pictures that cross the contract by a number, asked for when wanted. [037](037-private-windows.md): what is never written |
| **Last updated** | 2026-11-24 |

## 1. User story

I go back to a tab I left this morning. The browser had put it to sleep, and for a moment the window is empty while its page comes again. I would rather see what was there, and have it become the page.

## 2. What is in it, and what is not

**In:** when a tab is put to sleep, a picture of its page as it was is kept with its state. When the tab wakes, the picture is shown where the page will be, at once, until the page has arrived.

**Not in, each with where it goes:**

- **The phone's switcher as cards** ([047](047-the-phones-switcher.md) D1: "cards when there are pictures of pages"): a use of these pictures, with its own design. It needs pictures of the tabs that are awake too, which this does not take.
- **A picture for a tab that came back at a start and was never awake in this run:** it has the one it slept with, if it slept; one that was in front when the app was quit has none (D3's other way).
- **A Site:** it sleeps at its home (039), and a picture of where it had wandered would be of another page. None, as it has no state (051 D4).
- **A private window's tabs:** they do not sleep (053), and nothing of theirs is written.

## 3. The rule

**A picture is taken as the tab is put to sleep,** when its state is asked for (053): the core asks the shell for the state, for a picture, and to let the web view go, the three together and in that order. The picture is the page as its web view last drew it: what was in sight when the tab was last looked at, not the whole page.

**It is kept where the state is,** in the store, by a number the tab's place in the session names. And it is dropped when the state is: when the tab goes on to another page, is closed, becomes a Site, or history is forgotten for its logins (a picture of a page is as much a record of a visit as its address is).

**It is small.** At most 800 points wide and as a JPEG, and no more than 96 KB: a state of at most 128 KB ([051](051-a-tabs-kept-state.md) D3) and a picture of at most 96 KB are inside the plan's budget of 250 KB on disk for a sleeping tab. A picture that comes out larger is taken again smaller by the shell; one that is still too large is not kept.

**It is shown while the tab wakes:** from the moment the tab is brought in front until its page has arrived (the first commit), or failed. The core says which picture, by its number, in what it tells the window of the tab in front; the shell asks for the bytes, as it asks for an icon's (046), and draws them where the page will be. When the page arrives the core says no picture, and the shell shows the page.

**Why the core.** When a picture is taken, kept, dropped and shown follows the tab's state, which is the model's. A shell takes a picture when asked and draws one when told.

## 4. The contract (1.50), to be added

```proto
// Host request. A picture of what a web view shows, no wider than so many points, as a JPEG.
// Answered with WebViewPicture; with none when the engine cannot take one.
message WebViewSnapshot {
  uint64 web_view_id = 1;
  uint32 max_width = 2;
  uint32 max_bytes = 3;
}

message WebViewPicture {
  bytes jpeg = 1;          // empty: none
  uint32 width = 2;        // in points, as it was drawn
  uint32 height = 3;
}

// TabView: the picture to show where the page will be, while the tab wakes. 0: none.
uint64 picture_id = …;

// Request. The pictures with these numbers. Answered with PicturesPage: each that is still
// kept, with its bytes; one that was dropped since is left out.
message QueryPictures { repeated uint64 picture_ids = 1; }
message PicturesPage { repeated Picture pictures = 1; }
message Picture { uint64 picture_id = 1; bytes jpeg = 2; uint32 width = 3; uint32 height = 4; }
```

## 5. In what order

1. **What each engine can do,** tried in the adapter suites against the suite's own pages: whether a web view that is **out of sight** can be pictured at all (a tab is put to sleep from behind); how long it takes; how large a JPEG of a full page of text and of a picture is at 800 points. On WebKit (a Mac, the iPhone simulator) and on Android's System WebView (the emulator). The design below this line bends to what is found: if an engine cannot picture a web view that is out of sight, its picture is taken as the tab **leaves the front** instead (as the state is, 051), and kept only if the tab then sleeps.
2. **The core and the store:** the request, the picture kept and dropped with the state, shown while the tab wakes. Against the scripted engine.
3. **The adapters:** taking the picture.
4. **The shells:** the picture drawn where the page will be, and gone when the page arrives.

## 6. Decisions

**All of these are put to the owner.** The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | A picture is taken as the tab is put to sleep, not before | One picture for each tab that sleeps, and none for the tabs that never do. *A guess until §5.1 is done:* it needs an engine that can picture a web view that is out of sight | Whenever a tab leaves the front: always possible, and a picture taken for every tab switch, most of them thrown away |
| D2 | It is kept in the store, with the state, and crosses the contract as bytes | One place for what a sleeping tab keeps, one rule for when it goes, and a forgetting of history that forgets it for good (008, 051 D6) | Files of the shell's own, named by the core: no bytes across the contract, and a second place to forget, to count against the budget, and to find after a crash |
| D3 | Only a tab that sleeps has a picture | It is what the plan asks for: something to show while a sleeping tab wakes | Also the tab in front when the app is quit, so that a start shows it at once: a picture taken at every quit |
| D4 | At most 800 points wide, a JPEG, 96 KB | Enough to be recognised and to stand in for a second; inside the budget with the state. *The numbers are guesses until §5.1 measures a page* | Larger, and the budget raised. Or as sharp as the screen, which is four times the bytes for a second's use |
| D5 | The picture shows what was in sight, at the size it was | It is replaced by the page at the place the page was scrolled to (051): the two then match | The top of the page |
| D6 | Every page that sleeps is pictured, a page of a bank's as any other | A browser's own files hold the page's address, its state and what was typed in its forms already; other browsers keep such pictures | Not for a page with a password field in it; or not for pages the person marks |
| D7 | The picture is drawn plainly, and is not a thing to press | It stands in for the page for a moment. Something that looked alive and was not would be worse than an empty window | Dimmed, or under a mark that says the page is coming |

## 7. Tests

To be filled in as built.

## 8. What is not verified

To be filled in as built.

## 9. Done checklist

- [ ] The owner's word on §6.
- [ ] What each engine can do (§5.1).
- [ ] The core and the store.
- [ ] The adapters.
- [ ] The shells.
- [ ] The gate.

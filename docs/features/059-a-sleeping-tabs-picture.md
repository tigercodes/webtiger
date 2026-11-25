# 059 — A sleeping tab's picture (W3)

| | |
|---|---|
| **Status** | **Built and tested on Apple's systems and Android, on the recommendations of §6.** The owner's word of 2026-11-24 was to use what is suggested where something waits on a choice, and to go on: they are built on, and stay the owner's to change. A tab that is put to sleep is asked for a picture of its page, which is kept with its state; when the tab wakes, the page as it was stands where the page will be, until the page arrives |
| **Phase** | W3 of [000](000-master-plan.md) §9: "snapshots". 000 §5.7: "Asleep. The web view is gone. A snapshot, the saved state and the scroll position remain. Cost: a database row and a small image on disk." 000 §7: "Wake a sleeping tab: snapshot within 50 ms; the engine loading within 100 ms." The ninth of W3 |
| **Contract** | 1.50 (§4): a web view's picture asked of a shell (`WebViewSnapshot`, `WebViewPicture`), which picture a waking tab is shown with (`TabView.picture_id`), and a picture asked of the core (`QueryPictures`, `PicturesPage`). The store's schema is 17 |
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

**A picture is taken as the tab is put to sleep,** when its state is asked for (053): the core asks the shell for the state, for a picture, and to let the web view go, the three together and in that order. The picture is the page as its web view last drew it: what was in sight when the tab was last looked at, not the whole page. Whatever puts the tab to sleep: its time (053), the limit of live tabs (054), the system short of memory (055), "Sleep All Others" (056). **But not when the system is very short of memory** (D8): drawing a picture takes memory, and time before the web view is let go, when there is neither. And of a tab that is asked for no state, no picture: a Site, a tab that is on its way somewhere, one whose history was forgotten (051).

**It is kept with the state,** in the state's own row in the store. So it is there for as long as the state is, and goes when the state goes, however that is: when the tab goes on to another page, is closed, becomes a Site, or history is forgotten for its logins (a picture of a page is as much a record of a visit as its address is: it is gone from the store's files then, as the state is). **A tab that sleeps on no state has no picture:** the engine gave no state, or one too large to keep, and the tab wakes at its address with nothing before its page. Where none is given as the tab sleeps and the tab has the state from when it left the front (it is of the same page), the picture is kept with that one.

**It is small.** At most 800 pixels wide and as a JPEG, and no more than 96 KB: a state of at most 128 KB ([051](051-a-tabs-kept-state.md) D3) and a picture of at most 96 KB are inside the plan's budget of 250 KB on disk for a sleeping tab. The adapter makes it so (below); what a shell gives all the same that is larger, wider, or says no size, the core does not keep.

**It is shown while the tab wakes:** from the moment the tab is brought in front until its page has arrived (the commit, and not the end of the load), or has failed, or its load was stopped, or the tab is sent somewhere else. The core says which picture, by its number, in what it tells the window of the tab in front; the shell asks for the bytes, as it asks for an icon's (046), and draws them where the page will be. When the page arrives the core says no picture, and the shell shows the page.

**Why the core.** When a picture is taken, kept, dropped and shown follows the tab's state, which is the model's. A shell takes a picture when asked and draws one when told.

**As built, where the design did not say:**

- **A picture's number is its state's.** A state's number is never used twice (051), so neither is a picture's.
- **A waking tab holds its picture.** A tab that is woken is given its state, which is then the store's no longer (051); the picture is read with it, and held by the tab in the core's memory for as long as the tab is shown with it. Once the page has arrived it is held by nothing. So `QueryPictures` answers only the pictures that tabs are being shown with now: at most eight in an answer, in the order asked; a number that no tab is shown with is left out.
- **Forgetting history takes it from a waking tab too:** the tab says no picture from that moment, and the picture is asked for in vain.
- **A tab that goes behind while it is on its way** keeps its picture, and is shown with it when it is in front again, if its page has still not come.
- **A picture that comes late** (a shell that is slow to draw) is kept when the tab sleeps on that state still; a tab that was woken meanwhile took its state with it, and the picture is nobody's.
- **How an adapter makes a picture small.** It draws the web view no wider than was asked (WebKit is asked for fewer points, so that it draws the picture small itself: a phone's web view at three pixels to a point is otherwise 2,400 wide on the way; Android's WebView is drawn through a smaller scale in one go). Then, where the JPEG is of more bytes than were asked for, by steps, the same on both systems: the whole width at six tenths of a JPEG's best; the whole width at four tenths; three quarters of the width; half; half at a quarter. The first that fits is the answer; where none does, there is no picture.
- **A web view that was never given a size** (a tab opened behind and never looked at) has drawn nothing: no picture.
- **WebKit is waited for three seconds.** A web view whose page's process is gone may never say; the adapter then answers with none, and the web view is let go. The host requests that follow wait meanwhile: a shell answers them one at a time, in order, and the next lets the web view go. It was one second at first, and that was too few: in the app on the iPhone simulator, a web view that had been out of its window for some seconds took 734 ms to draw the first time (11 to 198 ms the other times it was read), and in the run before that one, which was not read, the app's test saw no picture at all.
- **In a window, the picture is drawn over the page's place:** as wide as the place, from its top, in its own shape, and cut off where the place ends. A window that is as it was when the tab slept shows the page where it was. In one that has been made taller, what is under the picture (the web view, with nothing in it yet) shows below it; in one that is shorter, the picture's end is not seen. It is not made to fill a place of another shape: a page laid out for one width is not the same page at another height.
- **It says what it is.** To a screen reader it is a picture called "The page as it was". A press or a touch goes through it to what is under it. (D7.)
- **What is drawn is checked first.** Only bytes that begin as a JPEG does are handed to the system's decoder; the picture's size is read before any of it is decoded, and one wider than the core keeps (800 pixels), or taller than 8,192, is not drawn. On Android a decoder that throws, or has no memory for the picture, means no picture, and the window goes on.
- **A shell asks once for each number its tab names,** and shows the picture when it comes if the tab names that number still; a tab that names none has none at once. An answer that comes after the page has, or after another tab is in front, is nobody's.
- **A dropped picture, like a dropped state,** is overwritten in the database at once; the write-ahead log has its pages until they are next written back. Forgetting history writes them back there and then (008).

## 4. The contract (1.50)

```proto
// Host request. A picture of what a web view shows now, whether or not it is on the screen:
// a JPEG no wider than `max_width` pixels and of no more than `max_bytes` bytes. Answered
// with WebViewPicture, whose bytes are empty when the engine has none to give, or none that
// small; or "webview.unknown".
message WebViewSnapshot {
  uint64 web_view_id = 1;
  uint32 max_width = 2;
  uint32 max_bytes = 3;
}

message WebViewPicture {
  bytes jpeg = 1 [(wt.redact) = true];
  uint32 width = 2;        // in pixels
  uint32 height = 3;
}

// TabView: the number of the picture to show where the tab's page will be, while the tab
// wakes. 0 when there is none to show. A number is one picture, and is not used twice.
uint64 picture_id = 22;

// Request. The pictures with these numbers. Answered with PicturesPage: those the core has
// now, in the order asked; one it has not (its tab's page has arrived since) is left out.
message QueryPictures { repeated uint64 picture_ids = 1; }
message PicturesPage { repeated PagePicture pictures = 1; }
message PagePicture {
  uint64 picture_id = 1;
  bytes jpeg = 2 [(wt.redact) = true];
  uint32 width = 3;
  uint32 height = 4;
}
```

The store (schema 17): a state's row has its picture, a JPEG and its size; none for every state there was.

A shell that does not know `WebViewSnapshot` answers as it answers any request it does not know, and the core has no picture: the tab sleeps and wakes as it did.

## 5. In what order

1. **What each engine can do,** tried in the adapter suites against the suite's own pages: whether a web view that is **out of sight** can be pictured at all (a tab is put to sleep from behind); how long it takes; how large a JPEG is. On WebKit (a Mac, the iPhone simulator) and on Android's System WebView (the emulator). **Done, 2026-11-24:**

   | Engine | A web view in no window | As the engine gives it | No wider than 800 pixels | Taken in |
   |---|---|---|---|---|
   | WebKit, on the Mac | Pictured, by WebKit's own way of taking one (`takeSnapshot`) | 800 by 600 pixels; 14 KB as a JPEG | The same | 4 to 5 ms |
   | WebKit, on the iPhone simulator | Pictured, the same way | 2,400 by 1,800 pixels (three to a point); 89 KB | Drawn again smaller: 800 by 600; 15 KB | 115 and 262 ms, each the first of its run |
   | Android's WebView, shown and then taken out of its window | Pictured, by drawing the view into a bitmap | 1,080 by 2,400 pixels; 21 KB | The view drawn smaller, in one go: 800 by 1,777; 14 KB | 10 to 25 ms |
   | Android's WebView, never in a window, given a size | Pictured, the same way | 800 by 1,200 pixels; 11 KB | The same | 9 to 14 ms |

   The page is the suite's page of pictures and words; a JPEG is made at six tenths of its best.

   What follows from it. **A picture can be taken as the tab is put to sleep** (D1 is no guess now): neither engine needs the web view to be on the screen. **The limit is in pixels, not points** (D4): a phone draws three pixels to a point, and the same page is 15 KB at 800 pixels and 89 KB as WebKit gives it; a picture is made no wider than 800 pixels before it is a JPEG. **The sizes are of a simple page:** the suite has no page of photographs, and a photograph is many times a drawing; the limit of 96 KB is kept by the adapter, which makes a picture smaller until it is under it, and that is not tried yet.
2. **The core and the store:** the request, the picture kept and dropped with the state, shown while the tab wakes. Against the scripted engine. **Built.**
3. **The adapters:** taking the picture. **Built,** for WebKit and for Android's WebView.
4. **The shells:** the picture drawn where the page will be, and gone when the page arrives. **Built,** on Apple's systems and on Android.

## 6. Decisions

**All of these are recommendations, used on the owner's word of 2026-11-24** (§ Status). The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | A picture is taken as the tab is put to sleep, not before | One picture for each tab that sleeps, and none for the tabs that never do. Both engines can picture a web view that is out of sight (§5.1) | Whenever a tab leaves the front: a picture taken for every tab switch, most of them thrown away |
| D2 | It is kept in the store, in its state's row, and crosses the contract as bytes | One place for what a sleeping tab keeps, one rule for when it goes (the row's), and a forgetting of history that forgets it for good (008, 051 D6). A tab with no state has no picture: it is rare, and such a tab wakes at its address anyway | Files of the shell's own, named by the core: no bytes across the contract, and a second place to forget, to count against the budget, and to find after a crash. Or a row of its own in the store, so that a tab with no state can have a picture: a second thing to drop in every place a state is dropped |
| D3 | Only a tab that sleeps has a picture | It is what the plan asks for: something to show while a sleeping tab wakes | Also the tab in front when the app is quit, so that a start shows it at once: a picture taken at every quit |
| D4 | At most 800 pixels wide, a JPEG, 96 KB | Enough to be recognised and to stand in for a second; inside the budget with the state. A simple page is 11 to 15 KB so (§5.1); a page of photographs was not measured | Larger, and the budget raised. Or as sharp as the screen: the same simple page was 89 KB at a phone's three pixels to a point |
| D5 | The picture shows what was in sight, at the size it was | It is replaced by the page at the place the page was scrolled to (051): the two then match | The top of the page |
| D6 | Every page that sleeps is pictured, a page of a bank's as any other | A browser's own files hold the page's address, its state and what was typed in its forms already; other browsers keep such pictures | Not for a page with a password field in it; or not for pages the person marks |
| D7 | The picture is drawn plainly, and is not a thing to press | It stands in for the page for a moment. Something that looked alive and was not would be worse than an empty window | Dimmed, or under a mark that says the page is coming |
| D8 | When the system is very short of memory, no picture is taken; when it is short, one is | A picture is drawn before the web view is let go: it takes a little memory and a little time (5 to 260 ms, §5.1) at the moment the system has asked for both. Short of memory is a warning, and the picture is small; very short is the last word before the system ends the app | A picture always: a tab put to sleep in the worst case wakes with one too, at the risk of being the app that did not let go in time. Or never when memory is short at all: on a phone that is how most tabs sleep, and they would wake with nothing |

## 7. Tests

| Where | What |
|---|---|
| The store (`wt-store`, `state`, +5) | A picture is kept with a state and read back, with its size; a state has none until it is given one; another takes its place; with no such state nothing is kept, and that is said. It goes with its state: dropped, or at a start when no tab names the state, and it stays with one that is named. A picture whose state is forgotten is gone from the store's files, and so is the one it had before. A store from before pictures gains them and keeps its states |
| The core, through the contract, against the scripted engine (`pictures`, 12) | A tab that leaves the front is asked for no picture; as it sleeps it is asked for one, once, of its own web view, no wider than 800 pixels and of no more than 96 KB, before the web view is let go; the picture is with the state the tab sleeps on and nowhere else. A tab that wakes says its picture's number, on its way; the state and the picture are the store's no longer; the picture is answered by its number while the shell is still busy with the page; when the page arrives the tab says none and there is none to ask for. The picture goes at the commit, and not at the load's end; when the load fails, is stopped, or ends with no page; when what drew the page is gone; when the tab is sent elsewhere; when it is closed. A tab that goes behind on its way has it still. A tab that did not sleep, or slept with no picture, says none. Which pictures are kept: at 96 KB and not a byte over; 800 wide and not 801; of some width and some height; with bytes. A tab with no state keeps none, and wakes at its address; with none given as it sleeps, or one too large, the picture is with the state it had. A picture that comes late is kept for a tab asleep still, and not for one that was woken. A Site is asked for none; a private window's tab for nothing. The limit, the system short of memory and "Sleep All Others" ask for one; the system very short of memory asks for the state and for no picture. Kept over a start, and shown when the tab then wakes. Forgetting history takes a sleeping tab's picture from the files, and a waking tab's from the tab, with nothing between the two requests. Pictures are answered in the order asked, twice when asked twice, and the first eight numbers are looked at |
| The core broken, a part at a time | 34 parts, 34 caught. The first run caught 29 of 31. One was a gap: with the picture left in place at the commit, every test passed, because the scripted engine's load ends a moment after it commits and the end took the picture away; a page that arrives and is loading still is now tried. The other was a line that did nothing (telling a window twice). Three parts were added with D8 and the limit, and caught |
| WebKit, against the suite's page of pictures and words (`WebKitPictureTests`, 6, on the Mac and the iPhone simulator) | **What WebKit does:** a web view that is in no window, as a tab behind is, is pictured. Made no wider than 800 pixels, the picture is of the page (more than a two-hundredth of it is drawn on), has the web view's shape, and is under half of 96 KB as a JPEG (§5.1). **What the adapter answers:** a JPEG, of the size the answer says, 800 wide where the web view is, in its shape, of the page; asked for a narrower one, as narrow; asked for a byte fewer than that comes to, the next step, as wide and a poorer JPEG; asked for fewer bytes than any picture has, none, with no size; the web view's state is there to ask for afterwards. A web view with no size has no picture, and one that is not there an error. With no time given, none; what WebKit says afterwards is nobody's, and the next picture asked for comes. **The drawing:** a picture is made to fit, the whole of it, in its shape, and one that fits is left as it is; a picture of noise goes down the five steps a byte at a time, and is none below the last |
| The WebKit adapter broken, a part at a time, on the Mac | 14 parts, 13 caught. The one that is not changes nothing that is answered: asked of WebKit at the web view's full width whatever the core asked, the picture is made to fit afterwards and is the same size. What that part saves is the large picture on the way, which no test measures. The first run caught 10 of the 12 it was aimed at rightly (the other two were aimed at text that was not there, and were caught when run): the second miss was a picture of exactly the bytes asked for, which was not tried, and is now. (No part was broken in a way that could end a test's process: nothing that waits was touched) |
| Android's drawing and the engine's answer, on the JVM (`PicturesTest`, 4; `SystemWebViewEngineTest`, +1) | Any view is drawn the same way. At its own size where it is no wider than asked; smaller, the whole of it, in its shape, where it is wider (a phone's 1,080 by 2,400 is 800 by 1,778); on white; a JPEG of the size it says. A view with no size has no picture. Noise goes down the five steps a byte at a time, and is none below the last; the steps are WebKit's. The engine asks its page for a picture as wide and as large as the core said, answers with its bytes and size, with no bytes and no size when the page has none, and with an error for a web view that is not there |
| Android's adapter broken, a part at a time | 17 parts, 17 caught. The first run had an eighteenth, not caught: a check that a picture of no bytes was asked for, which the steps answer anyway (none fits). The check was taken out, on both systems |
| Android's WebView, on an emulator (`SystemWebViewPictureTest`, 4) | **What the WebView does:** a web view that was shown and is then in no window is pictured by drawing it into a bitmap, smaller where it is wider than 800 pixels; one that was never in a window and was given a size is pictured too (§5.1). **What the adapter answers:** as WebKit's, with the real WebView: a JPEG 800 wide in the web view's shape, of the page; as narrow as asked; the next step for a byte fewer; none for fewer than any picture has; the state there afterwards; an error for a web view that is not there. A web view that was never given a size has no picture |

| Apple's window, against a scripted core (`PagePictureTests`, 4, on the Mac and the iPhone simulator) | A tab that names no picture: nothing asked, nothing shown. One that names a picture: asked for once, by its number alone, and shown; told again while the page comes, not asked again; when the tab names none, none at once; another tab's picture, and the first's again, each asked for as it is named. What is no picture is not drawn: none from the core, words, a PNG, a JPEG wider than the core keeps or taller than a window, the first bytes of a JPEG and no more. An answer that comes after the page has, after another picture is named, or that carries another picture than was asked for, is not shown. The picture drawn: in a place of its shape it fills it, top half above bottom; in a taller place it is as wide, from the top, and what is under shows below; in a shorter one it is cut off at the place's end and goes no further; in a wider place than it has dots it is drawn larger |
| Apple's pictures (`CatalogueSnapshotTests`, a state more: two pictures on each system) | The window with a tab that slept on its way to its page, in both themes: the tab loading, and the page as it was where the page will be, cut off at the window's end. Every other picture was recorded again and is the one it was, to the last dot |
| The apps, in fake mode, with the real core and the real engine (`WakingPictureTests`, 1, on the Mac and the iPhone simulator) | A tab at the suite's slow page, which answers after three seconds, is left behind and sleeps. No picture is over the other tab's page. Pressed, it is on its way for three seconds: the picture is there, and says "The page as it was". The page comes: the picture is gone, and the tab is awake |
| Apple's shell broken, a part at a time, on the Mac | 11 parts, 11 caught. The first run caught 10: with the picture not cut off at its place's end every test passed, because the tests' place had nothing under its end to be drawn over. One now has |
| Android's window, on the JVM (`PagePictureTest`, 7) | As Apple's first three, with a core that will not answer as well: no picture, and the tab in front all the same. And the picture drawn by Compose: in a place of its shape; in a taller one, with what is under showing below; in a shorter one, as wide and cut off; and it says "The page as it was" |
| Android's pictures (`CatalogueSnapshotTest`, a state more: two pictures) | As Apple's, in both themes, at a phone's width. Every other picture was recorded again and is the one it was |
| Android through the real core, on the JVM (`ThroughTheCoreTest`, +1) | With the app's own engine over pages that have no WebView: a page is asked for no picture as its tab leaves the front, and for one, 800 wide and of 96 KB at most, as the core puts the tab to sleep; brought in front, the tab names a picture, and the core hands back the bytes the page gave, in an answer this shell names; the page comes, the tab names none, and the core has none. The bytes are no JPEG and this JVM has nothing to decode with: the window shows none and goes on (before the decoder was made safe, this ended the window's session) |
| Android's shell broken, a part at a time | 22 parts, 22 caught. (One was first written so that it would not build, and was caught when written so that it did) |
| Android, on an emulator (`AppPagePictureTest`, 1), in fake mode, with the real core and the real WebView | The suite's slow page; another tab in front; the first put to sleep from the Library ("Sleep All Others": the app's tests share one core, with the usual half hour); touched, the picture is there for the three seconds and says what it is; the page comes, the picture is gone, and the tab is named by its page again |

## 8. What is not verified

- **The choices of §6** are the recommendations, used on the owner's word; the owner has not looked at them.
- **A page of photographs, with a real engine.** The suite's pages are drawings and words. That a picture too large for 96 KB goes down the steps is tried with noise, on each system's own drawing; how a photograph's picture looks at the step it ends at is not.
- **A phone and an iPhone.** The engines were tried on a Mac, a simulator and an emulator.
- **A Mac whose screen draws two pixels to a point.** The tests' web view is in no window and is drawn at one. In the app the adapter guesses the scale from the window the web view is in, or was last drawn for, and makes whatever WebKit then gives fit: the fitting is tested, the guess is not.
- **A web view that is drawing something as it is pictured** (a film, a page still loading): not tried.
- **How long a picture takes when the machine is busy:** the times of §5.1 are of one page on a machine doing nothing else, and the iPhone simulator's is the first of its run. On Android the view is drawn and made a JPEG on the main thread, 5 to 25 ms on the emulator, for each tab put to sleep.
- **WebKit never saying.** The second it is waited for is tried with no time at all, which ends the wait before WebKit can have said; a web view whose process is gone was not made.
- **A tab on its way, put to sleep again before its page comes** (the system very short of memory): it gave its state as it woke and is asked for none as it sleeps, so it sleeps with no state and no picture, and next wakes at its address. As 053 has it; a picture changes nothing there.
- **Memory, measured.** D8 rests on what a picture is known to cost in bytes of bitmap, not on a measurement of an app under pressure. No test makes a system short of memory.
- **The 50 ms of the plan** ("snapshot within 50 ms"). Not measured as a budget. What was read once, in the app on the iPhone simulator: 16 and 21 ms from the window being told the picture's number to the picture being shown. The budgets as tests are later in W3.
- **A press that goes through the picture.** It is made so on both systems and no test presses one: the page under it is not there to be pressed.
- **Apple's audit with the picture on the screen.** The audit takes longer than the three seconds the suite's slow page gives. The picture's name is read by the app's test; its contrast is the page's own.
- **The moment the picture goes.** It goes at the page's commit, when the web view may not have drawn the page yet: a blink of an empty page between the two was looked for in no test, and by nobody.
- **A window that is not the shape it was** (a phone turned, a Mac's window made narrower): the picture is as wide as its place, and the page it stands in for will be laid out anew. Drawn in tests; not looked at in an app.
- **A dark page, and a dark window.** The picture is the page's own colours. A place taller than the picture shows the empty web view below it, which is white before a page comes.
- **A picture too tall for Android's memory.** That no memory for a picture means no picture is written, and cannot be tried.

## 9. Done checklist

- [ ] The owner's word on §6 (used as suggested, on the word of 2026-11-24).
- [x] What each engine can do (§5.1): tried on the Mac, the iPhone simulator and the Android emulator on 2026-11-24.
- [x] The core and the store.
- [x] The adapters.
- [x] The shells.
- [ ] The gate.

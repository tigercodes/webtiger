# 031 — A link from another app opens in WebTiger (W1)

| | |
|---|---|
| **Status** | Built and tested in the core and the Android shell: on the JVM, in the app on an emulator, and by hand in the app started for real on an emulator, against the test site with the emulator's network off. Not run on a phone, and not from a real app's link (§9). The Mac is not built; the iPhone cannot be yet (§2) |
| **Phase** | W1 of [000](000-master-plan.md) §9: "outside links", the way in. [007](007-outside-links.md) §2 left it for later in W1: "Links from outside into WebTiger (being the system's browser)" |
| **Contract** | 1.23: `OpenAddress` |
| **Builds on** | [005](005-tabs.md): a tab opened in a window. [028](028-search-from-other-apps.md): words another app hands over, and how Android's window is told of them. [029](029-the-mark-waits-for-the-page.md): no mark beside an address until its page arrives |
| **Last updated** | 2026-11-21 |

## 1. User story

Somebody sends me a link in a message. I press it, and the system asks which browser, or WebTiger is the one I chose: WebTiger comes forward with a new tab, and the page loads in it. My other tabs are as they were. Nothing else the other app might have liked to say gets in: only an address of the web.

## 2. What is in it, and what is not

**In:** WebTiger answers the system's "show this address" for addresses of the web (`http`, `https`). So the system lists it among the browsers, offers it where it offers a choice, and lets a person make it theirs in the system's own settings. An address handed over opens in a new tab, in front, in the window WebTiger has.

**Not in:**

- **Asking to be the default browser**, and saying whether WebTiger is. Both are the first run's fourth choice (000 §6.3), and each system asks the person in a dialog of its own. Here WebTiger only becomes a browser the system knows of.
- **The iPhone and iPad.** There the system hands links only to an app Apple has granted the browser entitlement (000 §13). Not built.
- **Files.** A page saved on the device, opened with WebTiger: WebTiger opens no local files yet (the navigation policy refuses `file:`).
- **Which Space a link opens in.** The Space the window shows. Rules about links are W7.
- **Using an empty tab that is in front** instead of opening another. Started for the first time by a link, the window has its one empty tab and the link's beside it.
- **Anything else an app can ask a browser** on Android: a tab of the app's own (Custom Tabs), headers to send, a page to come back to.

## 3. The rule

**The shell** hears that the system hands it an address, and hands it to the core as it was given, with the window it is for (`OpenAddress`). It decides nothing: not whether the address is one to open, and not where it opens.

**The core** decides:

- **Only an address of the web is opened**: `http` or `https`, whole, with a host. Everything else is refused (`address.not_web`) and nothing opens: `javascript:`, `data:`, `file:`, `about:`, `blob:`, `content:`, `intent:`, an app's own scheme, words, nothing at all.
- **It is not typed text.** It is not mended, completed or searched for: what is not an address of the web is refused, not sent to a search engine. (A link is not words; and another app's text is never sent anywhere the person did not choose.)
- **It is not a page's navigation either.** Nothing is handed on to the system from it (a `mailto:` from another app is refused here, not passed along: the other app could have asked the system itself).
- **It is opened as it was given.** An address over `http` is not made `https`.
- A window the core does not have is refused (`window.unknown`), and that is said first.
- Otherwise **a new tab opens, after the tab in front, in front**, in the Space the window shows and with its logins, and loads the address. Its field shows the address from the start, as an address typed there does, with no mark until the page arrives (029). The tab that was in front is as it was.

**An address from another app is another app's.** The way in is one any app on the device can use, with any address: that is what being a browser means. So:

- nothing is taken but the address: no headers, no referring page, no name of the app that asked;
- the address is loaded as a person's typed address would be, in the Space's login set: the site sees the person as it knows them, which is what pressing a link in any browser does;
- it is never written in a log, and is kept only as the visit it becomes when its page arrives ([008](008-history.md)).

**What a link from outside cannot do** is anything a typed address cannot: run a script, show a page under no address, open a file, or start another app.

**Every link that is handed over is opened**, in the order they came, however fast they come: an app that opens a list of links sends them all at once.

## 4. The contract (1.23)

Added; nothing is changed or removed.

```proto
// browser.proto
// An address the system handed the shell from outside WebTiger: a link
// pressed in another app (031). Opened only when it is an address of the
// web (http or https, whole, with a host), in a new tab after the tab in
// front, in front, in the Space the window shows. It is not typed text:
// never mended, and never searched for; and nothing of it is handed on to
// the system. Answered with TabOpened; "window.unknown"; or
// "address.not_web", which opens nothing.
message OpenAddress {
  uint64 window_id = 1;
  string url = 2;                           // redacted
}
```

In the core the rule about what an address of the web is (`Address::web`) is one function, used here and for a page's link opened in a new tab ([018](018-link-in-new-tab.md)), which had the same rule written out.

## 5. Android

- **The window answers** (`MainActivity`): it says in the manifest that it shows addresses over `http` and `https`, of any site, as something a page may link to (`ACTION_VIEW`, with the categories `DEFAULT` and `BROWSABLE`). That is what makes the system count WebTiger among the browsers.
- **The window is one** (`singleTask`, as since 028): asked again, the one there is comes forward and is told. Started by a link when it is not running, it starts, and opens the link once it is open.
- **Nothing but the address is read** from what the system hands over: not what else the other app put in (headers to send, the page the link was on, the app's name). An address longer than 8,000 characters is not handed on at all: a part of an address is another address.
- **The window decides nothing about the address.** The system sends it only addresses over `http` and `https` (the manifest says so); an app that starts the window by name with some other address has that address handed to the core like any other, and the core refuses it. The window comes forward all the same, with nothing new in it.
- **Addresses that come before the window is open wait for it**, each of them, and are opened in the order they came.
- **Not again.** When the system makes the window again from the list of recent apps, it brings the request the window was first started with, and says so: nothing is handed over a second time. The same holds for words to search for (028).

**As first built, and what the emulator showed.** The first form put an activity that shows nothing in front of the window, as "Search with WebTiger" has (028 D3): it read the address, started the window with it, and was gone. In the app on the emulator, six requests sent with nothing between them were not all opened: the system's log has five of that activity started within seven thousandths of a second, and two of them ever made; the other three sat behind the window, never run, until the app's process ended. A person cannot press links that fast; an app that opens a list of them does exactly that, and a link that is silently not opened is worse than a window that answers for itself. So the window answers. Every request to a window that is one reaches it, in order (seen: the same six, three runs, all there).

What the first form was for does not apply to a link: the item for selected words is started inside the other app's task and expects an answer; a browser's window asked to show a link is not. And the window being one already keeps a second from being made.

## 6. The Mac

Not built. The app would say in its `Info.plist` that it opens `http` and `https` addresses, and hear them from the system (`onOpenURL`). Two things to settle first: a Mac's system keeps a record of every app that says so, a development build included, so WebTiger will be in the list of browsers in System Settings on any Mac it is built on; and the app's window must take the address itself, and not have the system open a second window for it.

## 7. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | The core's own request, not "open a tab with this text" | A tab opened with text takes it as typed: mended, and searched for when it is no address. Another app's address must be neither. The rule is the core's to keep, once, for every shell |
| D2 | What is not of the web is refused, not handed to the system | WebTiger was asked to show a page. Passing a `tel:` or an app's scheme along would make it a way round the system's own asking |
| D3 | On Android the window answers the system itself; no activity stands in front of it | With one in front, links that came at once were lost (§5). The window is one, and is told of each |
| D4 | A new tab in front, always | The person pressed a link to see it. Which Space, and whether an empty tab should be used instead, are refinements (W7) |
| D5 | Every address that comes before the window is open is kept | Each is a link somebody pressed. (Words to search for keep only the last, 028: a person selects one thing at a time) |

## 8. Tests

| Where | What |
|---|---|
| Core (`wt-core`, 1; `wt-app/tests/open_address.rs`, 6, through `FakeShell`) | What an address of the web is: `http` and `https`, whole; never what typed text would be mended into, nor any other scheme (22 of them and of what is no address at all). Through the contract: an address opens in a new tab after the one in front, in front, with the address in its field; one over `http` is opened as it is and marked as it is; the tab that was in front is as it was. The field says where the tab was sent from the start, and the mark waits for the page. Everything that is not of the web is refused (20 of them) and opens nothing: nothing is loaded, so nothing is searched for, and nothing goes to the system. A window there is not is refused first. It opens in the Space the window shows, with that Space's logins. A page that cannot be had shows in the tab that opened for it. Seven parts of the rule checked by breaking them: all caught. (An eighth was not: that the tab is still answered for when the shell itself refuses to load. No test has a shell that refuses; the same is true of 028.) |
| The real core from Kotlin (`ThroughTheCoreTest`, 1 more) | The window's model hands the address on; the real core, through JNI, opens a new tab in front and asks its web view for that address as it was given; what is no address of the web (a name with no scheme, words, a script, a file, an `intent:`) opens nothing; an address over `http` asked after them is opened, not made `https`. The web view is the test's: nothing is fetched |
| Android (`ViewAddressTest`, 6, on the JVM) | The address is read only from the system's own request, as it was given, and only when there is one; one too long is not handed on, and not cut. The window hands it to the core with its window, and nothing else of what it was handed: not what another app put beside it. An intent that asks for no address to be shown asks the core nothing: the app's own launch, a thing that is sent, words (which are searched for, an address among them included). What was handed over is not handed over again when the system makes the window again from recent apps: links, and words. Addresses that come before the window is open wait for it, each of them, in order, once. **The system is told** the window shows an address of the web of any site, as something a page may link to, and nothing of any other kind (a file, what an app keeps, a letter, a number, a script, an app's own scheme); and the window is one. Fourteen parts of this checked by breaking them: all caught (one only after the test's addresses were made to differ: a list that read the same backwards hid a wrong order) |
| The Android app, on the emulator (`AppLinksTest`, 1) | With the real core and the real WebView, in fake mode, the window asked as the system asks it: a link opens in a new tab after the one in front, in front, its page loads and the field says where it is; the same link again opens again; six requests at once, two pages of the site among what is no address of the web, are each answered in order: two more tabs, the last in front, and the site asked for four pages in all and nothing else. Passed three runs one after another |
| By hand (2026-11-21, the emulator, the app started for real, the test site served from the Mac, **the emulator's network switched off**) | The system lists WebTiger among what answers a link of the web, and the system's "Default browser app" list offers it beside the browser that is there (nothing was chosen). Started by a link with WebTiger not running: it opened, asked where searches go as a first run does, and had the page in a new tab in front, marked not secure (the test site is over `http`). From the home screen, the very same link: the one window came forward with another tab. In front, the same again, and another: a tab each. Six addresses that are not of the web, handed to the window by name: no tab, the field as it was, and no other app started. The app's icon: the same window, unchanged. One window and one entry in recent apps throughout. With the app's process ended behind the person's back, and the icon pressed: the five tabs came back, and the link that first started the app was not opened again. The network was switched back on and the app's data cleared |

## 9. What is not verified

- **A link pressed in a real app.** By hand the window was started the way the system starts it (`am start`, with the system's action and the address), not by pressing a link in another app; and the emulator's own browser stayed its default, so the system's routing of a link to WebTiger as *the* browser was not seen. The system's lists of who answers were.
- **A phone**, and other makers' systems.
- **A link that arrives while a system picker of WebTiger's is open** (a file being picked for a page): the window coming forward ends the picker. Not tried.
- **Very many links at once.** Six were sent; nothing limits how many tabs another app can open this way, as nothing does in other browsers.
- **The first form's fault on other versions of Android.** It was seen on the emulator's (Android 16); whether older systems start every one of such activities was not looked at, and no longer matters here.
- **The Mac, the iPhone and the iPad**: not built (§2, §6).

## 10. Done checklist

- [x] Contract 1.23 and its version in every place it is written.
- [x] The core: the rule of §3 (7 tests).
- [x] Android: the way in, what it hands on, and what the window asks of the core (6 tests, 1 through the real core, 1 in the app on the emulator).
- [x] On the emulator, by hand.
- [ ] The Mac.
- [x] `Tools/test.sh all` green, and `android-device`, on 2026-11-21: `lint`, `core` (505), `unit` (261 on the Mac), `build`, the Mac's UI suite (28), `ios` (243 on the iPhone simulator, one skipped as before), the iPhone's UI suite (25), `android` (348 and 32 on the JVM) and `android-device` (145 on the emulator).

# 031 — A link from another app opens in WebTiger (W1)

| | |
|---|---|
| **Status** | Written before the code. Nothing is built yet |
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
- **Anything else an app can ask a browser** on Android: a tab of the app's own (Custom Tabs), headers to send, a page to come back to.

## 3. The rule

**The shell** hears that the system hands it an address, and hands it to the core as it was given, with the window it is for (`OpenAddress`). It decides nothing: not whether the address is one to open, and not where it opens.

**The core** decides:

- **Only an address of the web is opened**: `http` or `https`, whole, with a host. Everything else is refused (`address.not_web`) and nothing opens: `javascript:`, `data:`, `file:`, `about:`, `blob:`, `content:`, `intent:`, an app's own scheme, words, nothing at all.
- **It is not typed text.** It is not mended, completed or searched for: what is not an address of the web is refused, not sent to a search engine. (A link is not words; and another app's text is never sent anywhere the person did not choose.)
- **It is not a page's navigation either.** Nothing is handed on to the system from it (a `mailto:` from another app is refused here, not passed along: the other app could have asked the system itself).
- A window the core does not have is refused (`window.unknown`), and that is said first.
- Otherwise **a new tab opens, after the tab in front, in front**, in the Space the window shows and with its logins, and loads the address. Its field shows the address from the start, as an address typed there does, with no mark until the page arrives (029). The tab that was in front is as it was.

**An address from another app is another app's.** The way in is one any app on the device can use, with any address: that is what being a browser means. So:

- nothing is taken but the address: no headers, no referring page, no name of the app that asked;
- the address is loaded as a person's typed address would be, in the Space's login set: the site sees the person as it knows them, which is what pressing a link in any browser does;
- it is never written in a log, and is kept only as the visit it becomes when its page arrives ([008](008-history.md)).

**What a link from outside cannot do** is anything a typed address cannot: run a script, show a page under no address, open a file, or start another app.

## 4. The contract (1.23)

Added; nothing is changed or removed.

```proto
// browser.proto
// An address the system handed the shell from outside WebTiger: a link
// pressed in another app (031). Opened only when it is an address of the
// web (http, https), in a new tab after the tab in front, in front, in the
// Space the window shows. Never mended, and never searched for. Answered
// with TabOpened; "window.unknown"; or "address.not_web", which opens
// nothing.
message OpenAddress {
  uint64 window_id = 1;
  string url = 2;                           // redacted
}
```

## 5. Android

- **The way in** is an activity of the app's that other apps can start (`ViewAddressActivity`, exported), which answers the system's "view" for `http` and `https` addresses, of any host, as a browser does (`ACTION_VIEW`, with the categories `DEFAULT` and `BROWSABLE`). That is what makes the system count WebTiger among the browsers.
- **It shows nothing**, as the item for selected words does (028): it reads the address, starts the app's window with it (bringing the one there is to the front, or starting it), and is gone. It has no task of WebTiger's, is not in recent apps, and keeps no history. With no address it starts nothing.
- **Nothing but the address is read**: no extras. An address longer than 8,000 characters is not carried.
- **The window** (`MainActivity`) is told of the address when it is started with it, or when it arrives while the window is there, and hands it to the window's model, which asks the core once the window is open. An address that comes with the start of the app waits for the window. The window itself still answers nothing of another app's.

## 6. The Mac

Not in the first part. The app says in its `Info.plist` that it opens `http` and `https` addresses, and hears them from the system (`onOpenURL`). Two things to settle when it is built: a Mac's system keeps a record of every app that says so, a development build included, so WebTiger will be in the list of browsers in System Settings on a Mac it is built on; and the app's window must take the address itself, and not have the system open a second window for it.

## 7. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | The core's own request, not "open a tab with this text" | A tab opened with text takes it as typed: mended, and searched for when it is no address. Another app's address must be neither. The rule is the core's to keep, once, for every shell |
| D2 | What is not of the web is refused, not handed to the system | WebTiger was asked to show a page. Passing a `tel:` or an app's scheme along would make it a way round the system's own asking |
| D3 | An activity that shows nothing, in front of the window (Android) | The same as 028 D3, and the window keeps answering nothing but its own launch |
| D4 | A new tab in front, always | The person pressed a link to see it. Which Space, and whether an empty tab should be used instead, are refinements (W7) |

## 8. Tests

| Where | What |
|---|---|
| Core (`wt-app/tests/open_address.rs`, through `FakeShell`) | An address of the web opens in a new tab after the one in front, in front, with the address in its field and no mark until the page arrives; the tab that was in front is as it was. `http` and `https` both. Everything else is refused and opens nothing: each scheme of §3, words, an address with no host, nothing. Nothing is searched for and nothing is handed to the system. A window there is not is refused first. It opens in the Space the window shows, with that Space's logins |
| Android, on the JVM | What the activity reads of what it is handed, and what it starts; what the window asks of the core; what the system is told of the activity (the manifest); the real core through JNI |
| Android, by hand on the emulator | The system lists WebTiger for an address of the web. Started as the system starts it, with the test site served from the Mac (nothing of the internet): not running, behind, and in front |

## 9. What is not verified

To be written as it is built.

## 10. Done checklist

- [ ] Contract 1.23 and its version in every place it is written.
- [ ] The core: the rule of §3.
- [ ] Android: the way in, what it hands on, and what the window asks of the core.
- [ ] On the emulator, by hand.
- [ ] The Mac.
- [ ] `Tools/test.sh all` green, and `android-device`.

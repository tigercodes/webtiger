# 007 — A link for the system (W1)

| | |
|---|---|
| **Status** | Built and tested in the core, the Apple shell and the Android shell (on an emulator). Windows is not started. What it leaves out is in §2, and what no test could show is in §8 |
| **Phase** | W1 of [000](000-master-plan.md) §9: "outside links". The `System` host service's first request (000 §5.5) |
| **Contract** | 1.4: `OutsideLinkRequested`; the host request `OpenOutside` |
| **Builds on** | [006](006-new-windows.md): a window refused the only place it was going |
| **Last updated** | 2026-11-19 |

## 1. User story

I click "email us" and my mail app opens, ready to write to them. I click a phone number and my device offers to ring it. A page cannot send me into another app by itself: only when I click.

Until now such an address was only refused ([001](001-foundations.md) F4): the click did nothing.

## 2. What is in it, and what is not

**In:** a link to write a letter (`mailto:`), to ring (`tel:`) or to send a message (`sms:`), clicked, is handed to the system, which opens whatever the person uses for it.

**Not in, each with where it goes:**

- **Every other scheme** (another app's own: `zoommtg:`, `spotify:`, a shop's): still only refused. The three above only ever open something the person must then confirm in (a letter is not sent, a number is not rung, by opening it). Another app's scheme can make that app *do* something, so it needs asking first ("Open this in Zoom?"), and the asking arrives with W1's prompts.
- **Saying that nothing on the device opens it.** There is nowhere to say it yet; that too arrives with the prompts.
- **An address typed in the address field.** Typed text is never executed (001 §4.2): `mailto:` typed there is a search, as before.
- **Links from outside into WebTiger** (being the system's browser): the first-run flow and the default-browser state, later in W1.

## 3. Who decides

**The core decides, and the shell opens.** The navigation policy already says, on the fast path, that such an address is not a browser's (`EXTERNAL`), and the shell refuses it in the web view. From there:

1. The shell **reports** it: `OutsideLinkRequested`, with the address and whether the engine says a person asked.
2. The core **decides** whether it goes to the system (the table below), trusting nothing the shell could be wrong about.
3. The core **asks** the `System` host service: `OpenOutside`. The shell opens it with the system, or answers that nothing does.

| | The core |
|---|---|
| The address is not one its own policy calls external (a shell that is confused, or worse) | Does nothing |
| The scheme is not `mailto`, `tel` or `sms` | Does nothing (§2) |
| The report is about a web view no tab has | Does nothing |
| Nobody asked: no gesture | Does nothing. A page does not open another app by itself |
| The same web view had one handed on less than a second ago | Does nothing. One click opens one thing, not ten |
| Otherwise | Asks the system to open it |

- **A window opened by a click carries the click, once.** A link that opens a window on `mailto:` is common. On Android the window is made first and the address is asked about in it, where the engine no longer says anyone clicked (006 §5). The window exists only because somebody clicked (006 §3), so the first address of a window a page opened, before any page has been in it, counts as asked for. One address: the click is then spent, whatever becomes of the window. The window is closed as 006 says, and the report must therefore come **before** the refusal that closes it.
- **The page that asked is left as it is.** Nothing about its tab changes: no load began in it.
- **Nothing a person visited is logged**: the address is marked for redaction in both messages, as every address is.

## 4. The contract (1.4)

Added; nothing is changed or removed.

```proto
// engine.proto: one more kind of EngineEvent.
// A page tried to go to an address that is not a browser's: wt_nav_check answered
// EXTERNAL, and the shell refused it in the web view. Reported for a frame as for the page
// itself, and before the NavigationFailed the refusal may also be reported as.
message OutsideLinkRequested {
  string url = 1;            // redacted
  bool user_gesture = 2;     // a click or a key, as far as the engine says
}

// host.proto: the System service's first request.
// Opens an address with whatever the system has for it. Answered with Ack, or with
// "system.no_handler" when nothing on the device opens it.
message OpenOutside {
  string url = 1;            // redacted
}
```

A shell built before 1.4 answers a host request it does not know with `host.unsupported`, and the core carries on: nothing opens, as before.

## 5. The shells

| | Apple | Android |
|---|---|---|
| Reports | `decidePolicyFor`, when the verdict is `external` | `shouldOverrideUrlLoading`, when the verdict is `EXTERNAL` |
| A gesture | A link that was clicked or a form that was sent (`navigationType`). WebKit does not say whether a script's navigation was clicked for, so `location = 'mailto:…'` from a button's script is not known to be, and is not opened (§8) | `hasGesture`, as the WebView says it |
| Opens | `NSWorkspace.open` on a Mac; `UIApplication.open` on an iPhone | An `ACTION_VIEW` intent in a new task; `system.no_handler` when no activity takes it |
| In fake mode | Opens nothing, and answers as if it had: a UI suite must not start the mail app of the machine it runs on | The same |
| A script's navigation | Not known to be clicked for (above) | Said to be asked for when a finger is on the page |

The System service is answered by the shell itself, not by an engine adapter: opening a link is nothing to do with a web engine. Each shell gives its host a small opener, which a test replaces with one that only writes down what it was asked.

**As built on Android** (`WebViewPage`'s `shouldOverrideUrlLoading`, through `PageTracker.outsideLink`; `ShellHost`, `OutsideOpener` and `SystemOpener` in `ui`), and what the device suite found:

- The same report, before the same refusal. A link that opens a window on such an address is reported in the window, which the WebView makes first (006 §5); the core counts the touch that opened it, and closes the window.
- **The WebView says a script's navigation was asked for** when a finger is on the page: a button whose script goes to such an address works on Android, which it does not on Apple (§8).
- `SystemOpener` starts an activity that says it views the address, with the category for links from the web, in a task of its own, and names no app: the system chooses. No activity for it is `system.no_handler`.
- On the device the opener's intent is caught by the test before it leaves the app, so the mail app and the dialler of the emulator are never started. A scheme nothing is registered for was asked of the system for real.

**As built on Apple** (`WebKitEngine`'s policy callback; `ShellHost` and `OutsideOpener` in `WTUI`):

- The adapter reports the link first and then refuses the navigation as it always did, so the core hears of the link before the refusal that may close a window (006).
- A link that opens a window on such an address is reported for the page that asked, and no window is made: WebKit asks about a link's address before it asks for a window. A window script opens is made first, and the link is reported in it, with no gesture; the click that opened the window is what the core counts (§3).
- `ShellHost` answers `OpenOutside` and passes every other request to the engine; the engine itself answers `OpenOutside` with `host.unsupported`. An address the system cannot read is answered like one nothing opens.
- The Mac's own opener was asked for real about a scheme nothing is registered for: it says so. **It also shows a dialog**, the system's own ("There is no application set to open the URL"), which is what a person is told and offers to choose an app. The test said it showed nothing, and was wrong for as long as it existed (§8): it now asks without the system's prompt (`promptsUserIfNeeded`), and the app asks with it. With a letter to write the opener starts the mail app, which no test does (§8).

## 6. Tests

| Where | What |
|---|---|
| Core (`wt-core`, 2) | Which addresses may be handed on: the three schemes, in any case; nothing else, look-alikes among them |
| Core (`wt-app/tests/outside_links.rs`, 12 through `FakeShell`) | A clicked link is handed to the system, once, and its tab is as it was. Each row of §3's table: not external by the core's own policy; another scheme; an unknown web view; no gesture; ten within one click, one a millisecond short of a second later, and one after it; another tab's page is not held back. A window opened by a click on such a link: handed on, and the window closed. The click that opened a window is good for one address. A window that has had a page does not carry the click. A shell that says nothing opens it, or that does not know the request: nothing breaks, nothing waits |
| WebKit adapter suite (`WebKitOutsideLinkTests`, 7) | A clicked link to such an address reports it, with a gesture, before its refusal, and the web view stays where it was. A script's navigation, and a page that goes there by itself as it loads, report it without one. A link in a frame reports it, and nothing is said of the page. A link that opens a window on one reports it and opens no window; a window script opens reports it in the window, before the refusal. The engine itself opens nothing |
| System WebView adapter | On the JVM: the tracker's report and its order with the refusal (2); the engine itself opens nothing (1). On the device (`SystemWebViewOutsideLinkTest`, 5), with a finger on the page: a touched link is reported with a gesture before its refusal, which is said to be one; a button's script is reported as asked for; a link in a frame; a page that goes there by itself, as not asked for; a link that opens a window reports it in the window |
| Each shell's host (Apple: `ShellHostTests`, 5, and one more on the Mac) | `OpenOutside` reaches the opener with the address as it was given; the opener's "nothing opens it" is `system.no_handler`; what is not an address is not given to the system; every other request still reaches the engine; fake mode's opener opens nothing and says it did. On the Mac, the system's own opener says when nothing opens an address. Android (`ShellHostTest`, 7 on the JVM): the same, and the intent the system's opener starts; on the device (`SystemOpenerTest`, 2) that intent is caught before it leaves the app, and a scheme nothing opens is said to be so |
| The apps | With the real core, in fake mode: a click on a link to write a letter leaves the page as it was, opens no tab, and leaves the app in front; on the Mac, the mail app is not running afterwards unless it was before; on Android, no activity was started for the letter |

## 7. Decisions

| # | Decision | Why |
|---|---|---|
| O1 | The core decides and asks; the shell only reports and opens | "The core decides, the shell draws": the rules of §3 are the same on every platform, and are tested once |
| O2 | No gesture, nothing opens | A page that could start the dialler by itself would do it on load |
| O3 | Only `mailto`, `tel` and `sms`, with no asking | §2. Each opens something that does nothing until the person confirms there. The rest wait for a prompt |
| O4 | One a second from a web view | A click's permission lasts a few seconds in an engine, and a page can ask many times within it |
| O5 | The core checks the address against its own policy again | The report comes from the shell, and the core trusts no shell with what leaves the browser |
| O6 | Fake mode opens nothing | A test suite, or a screenshot run, must not start other apps on the machine |

## 8. What is not verified

- **The system's opener with a real letter, number or message.** Every test replaces the opener, catches its intent, or runs in fake mode, on purpose: none starts another app. That `NSWorkspace.open`, `UIApplication.open` and Android's intent then start the mail app or the dialler is the system's own behaviour, and was not tried from WebTiger on this machine or on a phone.
- **A physical Android device**, and which apps there offer themselves for such links: the emulator's were never started.
- **A test put a dialog of the system's on the screen, every time it ran** (found 2026-11-19, with 011). `testTheSystemsOpenerSaysWhenNothingOpensAnAddress` asked the Mac to open a scheme nothing handles, as the app asks; the system answered the test with a code and the person at the Mac with a dialog. The Mac's log has it 21 times that day. Nobody saw one, because the UI suite that ran next pressed each one's Cancel button: XCUITest does that, unasked, to a dialog of the system's that is in its way, whoever the dialog was for. (How that is known: in the run that hung, XCUITest's own record says "Handling button by pressing button with 'Cancel' attribute" of each dialog; and in the earlier runs the system's log has XCTest reading the dialog's process during the UI suite, after which the dialog was gone. The press itself was not watched.) Three runs without a UI suite between them left three dialogs, and with two in its way XCUITest goes round between them until the test's ten minutes are up. What was changed: the test asks without the prompt, and was seen to add no dialog (the system's log has the refused launch and no dialog after it); `Tools/test.sh ui` counts the system's dialogs first and does not start the Mac's suite while there is one. What was not: the app still lets the system say so itself when nothing opens an address; whether WebTiger should say it in its own words instead is not decided.
- **A second kind of dialog, not a test's doing** (found 2026-11-20, with [014](014-permissions.md)). "… quit unexpectedly", shown by the system's `UserNotificationCenter` when a process of the iPhone simulator's crashes during the gate. It lies over the top middle of the screen, and a Mac UI test whose control is under it fails as "not hittable" with nothing in the accessibility tree to say why; a picture of the sheet, kept from the failed run, showed it. `Tools/apple/system-dialogs.swift` counts these windows too now, so the gate stops and says a dialog is there. It is still a person who answers it.
- **A button whose script goes to such an address** (`location = 'mailto:…'`) opens nothing on Apple: WebKit does not tell an app whether a script's navigation was clicked for, and the adapter does not say it was. A plain link works. Telling the two apart needs the page's own clicks to be watched, which is not built.

## 9. Done checklist

- [x] Contract 1.4 and its version in every place it is written.
- [x] The core: the rules of §3; `OpenOutside` asked of the host.
- [x] Core tests through `FakeShell`: 12, and each rule checked by breaking it (the shell's word; the gesture; the limit; the carried click; its being spent).
- [x] Apple: the adapter's report (7 tests, its gesture and its frames checked by breaking them); the shell's host and opener (5 tests, 6 on the Mac); a UI test with the real core on the Mac and the iPhone simulator.
- [x] Android: the adapter's report; the shell's host and opener; 10 tests on the JVM and 8 on the emulator (5 of the adapter, 2 of the opener, 1 of the app).
- [x] `Tools/test.sh all` green, and `android-device` (65 on the emulator).
- [ ] Windows shell.

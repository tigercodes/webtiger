# 007 — A link for the system (W1)

| | |
|---|---|
| **Status** | In progress. The contract (1.4) and the core are built and tested; no shell reports such a link or opens one yet, so in the apps a click on one still does nothing (§9) |
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

The System service is answered by the shell itself, not by an engine adapter: opening a link is nothing to do with a web engine. Each shell gives its host a small opener, which a test replaces with one that only writes down what it was asked.

## 6. Tests

| Where | What |
|---|---|
| Core (`wt-core`, 2) | Which addresses may be handed on: the three schemes, in any case; nothing else, look-alikes among them |
| Core (`wt-app/tests/outside_links.rs`, 12 through `FakeShell`) | A clicked link is handed to the system, once, and its tab is as it was. Each row of §3's table: not external by the core's own policy; another scheme; an unknown web view; no gesture; ten within one click, one a millisecond short of a second later, and one after it; another tab's page is not held back. A window opened by a click on such a link: handed on, and the window closed. The click that opened a window is good for one address. A window that has had a page does not carry the click. A shell that says nothing opens it, or that does not know the request: nothing breaks, nothing waits |
| WebKit adapter suite | A clicked link to such an address reports it, with a gesture, before any failure; a script's navigation reports it without one; in a frame too; the web view stays where it was |
| System WebView adapter | The same on the JVM (the tracker and the engine) and on the device, with a finger on the link |
| Each shell's host | `OpenOutside` reaches the opener with the address; the opener's "nothing opens it" is `system.no_handler`; every other request still reaches the engine. Android: the intent that is started, on the JVM and, on the device, caught before it leaves the app |
| The apps | With the real core, in fake mode: a click on a link to write a letter leaves the page as it was and opens nothing real |

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

- Written when built.

## 9. Done checklist

- [x] Contract 1.4 and its version in every place it is written.
- [x] The core: the rules of §3; `OpenOutside` asked of the host.
- [x] Core tests through `FakeShell`: 12, and each rule checked by breaking it (the shell's word; the gesture; the limit; the carried click; its being spent).
- [ ] Apple: the adapter's report; the shell's host and opener; their tests; the UI test.
- [ ] Android: the adapter's report; the shell's host and opener; their tests, on the JVM and on a device.
- [ ] `Tools/test.sh all` green, and `android-device`.

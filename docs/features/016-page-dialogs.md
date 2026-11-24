# 016 — A page says something, or asks (W1)

| | |
|---|---|
| **Status** | Built and tested in the core, the Android shell and the Apple shell. Run end to end on Android (an emulator) and on Apple against real WebKit: in the adapter's suite, and in the app, on the Mac and the iPhone simulator. Windows is not started |
| **Phase** | W1 of [000](000-master-plan.md) §9: a browser a day's ordinary use needs no other for. Pages say things (`alert`), ask yes or no (`confirm`) and ask for a line of text (`prompt`); today WebKit drops all three, and the System WebView shows them in a dialog of its own |
| **Contract** | 1.12: `DialogRequested`; the host request `DialogAnswer`; `DialogPrompt`, a third kind of `PromptView`; two fields of `AnswerPrompt` |
| **Builds on** | [014](014-permissions.md): a tab's prompt, its queue, and the rule that it is not answered in the moment it appears |
| **Last updated** | 2026-11-20 |

## 1. User story

A page tells me something, or asks me to confirm, or asks for a word. I see it in that tab, with the name of the site that says it, and I answer it there; the page gets my answer. It cannot cover the window, cannot pass its words off as the browser's, and if it will not stop, I can stop it.

## 2. What is in it, and what is not

**In:** `alert`, `confirm` and `prompt` from a page reach the core; the tab shows what the page says, under the site's name; the answer goes back to the page; a page that says more than once can be silenced.

**Not in:**

- **"Leave site?"** (`beforeunload`). A page is never let hold a person on it: leaving is not asked about, on any engine. (Unsaved work in a form deserves better than a page's own veto; it arrives with forms, if at all.)
- **Dialogs of a frame from another site.** Refused (§3): such a frame cannot put words before a person under the page's roof.
- **Printing**, and choosing a file or a colour: the System service's, later.
- **Sign-in prompts** (a server asking for a name and password): the next kind of prompt, which will have two fields where this has one.

## 3. Who does what

The engine notices that a page calls one of the three, and says so; the page's script waits. The core decides whether the person sees it. The shell draws it in the tab and sends the answer; the engine gives it to the page.

| What the shell reports | What the core does |
|---|---|
| `DialogRequested`: an id of the shell's own, who asks (the origin of the frame that called), which of the three, the page's words, and for `prompt` the text it offers | Applies the rules below; tells the shell the page's answer with `DialogAnswer`, now or when the person has answered |

**Dismissed with nobody shown anything** (the page is answered as if the person had closed it: `alert` returns, `confirm` is false, `prompt` is null) when:

1. the web view is no tab's, or its tab has no committed page;
2. **who asks is not the page**: the frame's origin is not the origin of the page the tab has committed;
3. the person silenced this page (below);
4. the tab has a prompt and three more waiting already (014 §3).

**Otherwise the tab gains a prompt** that names the site, says which of the three it is, and carries the page's words. The words are the page's and are shown as such, cut by the core to 2,000 characters; so is the text a `prompt` offers.

| The person | `alert` | `confirm` | `prompt` |
|---|---|---|---|
| presses **OK** | returns | true | the text in the field |
| presses **Cancel** (not offered for `alert`) | returns | false | null |

- **Not in the moment it appears**, and not from a tab behind: 014's rule, unchanged. A `confirm` that a moving finger or a key meant for the page could answer "OK" is the reason for that rule.
- **A page that says more can be silenced.** From a page's second dialog on, the prompt offers a third answer, to stop the page's messages. It dismisses this one, whatever else of the page's waits, and every later one, with nobody shown anything, until another page has loaded in the tab. Asked of a page's first dialog, it is asking nothing. It stops nothing else that waits in the tab: a permission the page asked for is still asked about.
- **What a page says is its own kind of thing.** A yes to it allows nothing, and a yes or a no to something else (a camera, a download) neither answers it nor dismisses it. The shell numbers its dialogs, its downloads and its pages' requests each from one: the core keeps the three apart.
- **A new page in the tab, the loss of its renderer, and the tab closing** dismiss what waited.
- **A tab behind keeps its dialog** until it is in front; its page's script waits meanwhile, as it does in front.
- **Nothing of it is logged**: a page's words are what was visited.

## 4. The contract (1.12)

Added; nothing is changed or removed.

```proto
// engine.proto
enum DialogKind {
  DIALOG_KIND_UNSPECIFIED = 0;
  DIALOG_KIND_ALERT = 1;
  DIALOG_KIND_CONFIRM = 2;
  DIALOG_KIND_PROMPT = 3;
}

// A page says something, or asks. Its script waits until the shell is told
// how to answer it (DialogAnswer).
message DialogRequested {
  uint64 request_id = 1;      // the shell's own, never used twice
  string origin = 2;          // redacted. The frame that called
  DialogKind kind = 3;
  string message = 4;         // redacted. The page's words
  string default_text = 5;    // redacted. What a prompt offers
}

// host.proto: WebViews. The page's answer.
message DialogAnswer {
  uint64 request_id = 1;
  bool accepted = 2;          // OK. False: the page is answered as if it had been closed
  string text = 3;            // redacted. With `accepted`, for a prompt
}

// browser.proto: a third kind of PromptView.
message DialogPrompt {
  string site = 1;            // redacted. Who says it, as the address field shows it
  DialogKind kind = 2;
  string message = 3;         // redacted. The page's words, cut to 2,000 characters
  string default_text = 4;    // redacted
  bool may_silence = 5;       // the page said something before: it can be stopped
}

message AnswerPrompt {
  ...
  string text = 4;            // redacted. For a prompt that asks for text
  bool silence = 5;           // for a DialogPrompt with may_silence: stop the page's messages
}
```

## 5. The engines

| | WebKit | System WebView |
|---|---|---|
| Notices | `WKUIDelegate`: `runJavaScriptAlertPanel…`, `…ConfirmPanel…`, `…TextInputPanel…`, each with the frame that called | `WebChromeClient`: `onJsAlert`, `onJsConfirm`, `onJsPrompt`, each handed a result to answer later |
| Unanswered | WebKit must be given its answer: a web view that is destroyed dismisses what waited | The same |
| "Leave site?" | Not implemented by the app, so never asked | `onJsBeforeUnload` answered at once: leave |

No dialog of the system's is involved anywhere, so the adapter suites run all of it against the real engines, on every platform.

**As built on Apple** (`Dialogs` in the WebKit adapter; three methods of the page's delegate):

- WebKit asks the page's delegate for each of the three, with the frame that called; the adapter reports it under an id of its own and gives WebKit the page's answer when the core has said. The page's script waits meanwhile, as on Android: in the suite, the line after an `alert` had not run half a second later.
- **Who called** is the frame's own origin, as for a permission (014): for a frame from another origin WebKit names the frame, and the core's rule dismisses it.
- A web view that is destroyed gives WebKit its answer for what waited: WebKit stops an app that never does.
- While a page waits on its dialog, its process answers nothing, assistive technologies included. On the iPhone simulator a UI test's look at the app says so in its log, and goes on. **On the Mac it does not go on**: the app's test, the first time it ran there, waited five minutes on the window's accessibility and failed without ever seeing the bar; a person's screen reader would have waited the same way.
- So **a page that waits is not offered to assistive technologies** (`WebViewHost`, `BrowserModel.pageWaits`): for as long as the tab shows what the page said, the view that holds the web view offers nothing of the page, as a window offers nothing of what is under a sheet. What there is to read and to answer is the bar; answered, the page is there again. With that the Mac's test found the bar a second after the press.

**As built on Android** (`Dialogs` in the adapter; the `WebChromeClient`'s `onJsAlert`, `onJsConfirm`, `onJsPrompt`, `onJsBeforeUnload`):

- The WebView is told each time that the app sees to it, so it shows no dialog of its own; the adapter reports what the page said under an id of its own and keeps the WebView's result to answer later. The page's script waits meanwhile: on the emulator, the line after an `alert` had not run half a second later, and ran when the core's word came.
- **Who called** is the origin of the address the WebView names, with nothing of its path. Seen on the emulator: for a frame from another origin, the WebView names the frame's address, so the adapter reports the frame and the core's rule dismisses it.
- "Leave site?" is answered at once: leave. On the emulator, a finger on a link of a page that asks to keep the person went where the link led, and nothing was reported.
- A web view that is destroyed dismisses what its pages said, before it is closed.
- **What a page says is reported after the page has arrived** (2026-11-24): like what a page asks to use, and for the same reason ([014](014-permissions.md) §6). The WebView says what a page says at once, and that a page has committed by a message that may still be waiting: a page that speaks as it loads could be heard before its commit, and the commit would then have dismissed what it said. It is taken up behind the messages that already wait. The script waits either way: the WebView's result is kept, as before. For dialogs this was not seen to go wrong; it is the same order of words as for the camera, where it was.

## 6. The shells

The same bar (014 §7), taller. The site's name with "says" on a line of its own, so that the words under it are plainly the page's; the page's words, on as many lines as they need up to a few, and scrolled beyond that; for a `prompt`, a field with the text it offers; **Cancel** and **OK** (for an `alert`, **OK** alone); and from a page's second dialog, a quieter third button that stops its messages. No button is the default for Return while the page has the keyboard; in the field of a `prompt`, Return is OK.

**As built on Apple** (`DialogBar`; `BrowserModel.answerPrompt`): the same bar on the Mac and the iPhone, stacked as on Android, with the neutral buttons every button with words is. The field is the system's own, with its border; Return in it is OK, and nothing else is answered by a key. Each prompt is a bar of its own, so a field starts from what its page offers.

**As built on Android** (`DialogBar`; `BrowserModel.answerPrompt`): as above. "Stop This Page's Messages" has a row of its own, apart from the answers to what the page says now. The field starts from what the page offers and is the person's from then on; another prompt starts from its own. Cancel sends nothing of what was typed. The keyboard's Done, in the field, is OK; nothing else is answered by a key. Something this build has no bar for (a later contract's kind) is not shown.

## 7. Tests

| Where | What |
|---|---|
| Core (`wt-app/tests/dialogs.rs`, 12 through `FakeShell`) | What a page says is shown in its tab, under the site's name, and the page's script waits. OK and Cancel answer each of the three as §3's table says: a prompt with the text in its field, which may be nothing; what was typed and then cancelled is not passed on. Dismissed with nothing shown: a web view that is no tab's, a tab with no page, a frame from another site, another of the site's names, another scheme, no origin, and what is none of the three; the page's own origin is itself however a shell spells it. A page not over `https` may speak, and is named with its scheme. Words at length are cut to 2,000 characters, never inside one, and so is the text a prompt offers; only a prompt offers text. Four wait, a fifth is dismissed, and they are shown one at a time in order. A page's first cannot be stopped and its second can; stopped, that one, those that waited and those that follow are dismissed, also after the page changes its own address, until another page, whose first is its first. A new page and a lost renderer dismiss what waited; a closed tab takes it with it. A tab behind keeps its dialog, and it is not answered in the moment it appears. Said twice, shown once. A dialog, a download and a request that the shell gave the same number are each their own: a yes to one allows nothing of another, a no dismisses nothing of another, and stopping a page's messages stops nothing else. Sixteen of the rules checked by breaking them |
| System WebView adapter, on the JVM (`DialogsTest`, 5) | Through the engine, as the core speaks to it: what a page says is reported with who called (its origin, and nothing of its path), which of the three, its words and what it offers, and the page waits; the core's word is the page's answer, with a prompt's text; each dialog has a name of its own and is answered once, and a word about one there is not is for nothing; a web view that is destroyed dismisses what its pages said, and not another's; an address as an origin, and one that is none of the web's as none. Six of the rules checked by breaking them |
| System WebView adapter suite (`SystemWebViewDialogTest`, 6, on the emulator) | With a finger on each of the test site's buttons. An alert is reported with its words and the page's origin, and the page's script waits for the core's word. A confirm is true or false as the core says. A prompt offers its text, and gets the answer or nothing. A frame from another origin is reported as itself. A page that asks to keep the person is left all the same, with nothing reported. A web view destroyed while its page waits |
| WebKit adapter suite (`WebKitDialogTests`, 7, on the Mac and the iPhone simulator) | Against real WebKit, the test site's page pressed by a script of the test's that does not wait for it. An alert is reported with its words and the page's origin, and the page's script waits for the core's word. A confirm is true or false as the core says. A prompt offers its text, and gets the answer or nothing. A frame from another origin is reported as itself. A web view destroyed while its page waits; and, at the seam where WebKit's answer can be seen, that it is dismissed and another web view's is not. A word about a dialog there is not. Five of the rules checked by breaking them |
| The Android bar (`DialogBarTest`, 4, drawn on the JVM against `ScriptedCore`) | Something said is shown under the site's name and is only acknowledged: no Cancel, no field, no "stop", and no button has the keyboard. A question has two answers, and a page that said something before can be stopped; its first cannot. A prompt has a field with what the page offers; OK sends what is in it, Cancel nothing of it, the keyboard's Done is OK, and another prompt starts from its own. Something this build has no bar for is not shown. Pictures of the bar for each of the three, light and dark, each looked at |
| The Android app, on the emulator | With the real core and WebView, in fake mode: a finger on the page's button; the question is in the tab under the site's name while the page waits, and Android's accessibility checks pass over it; OK, and the page has true; it asks for a word, the field holds what it offers, and the page gets what was typed; it says something more and is stopped, and what it asks after is answered as closed, with nothing shown |
| The Apple bar (`DialogModelTests`, 5 on the Mac and 4 on the iPhone, against `ScriptedCore`) | The model has what the page says, and each answer is sent by the prompt's name: OK with a prompt's text, Cancel, and "stop"; the prompt stays until the core says it is gone. Something this build has no bar for is not shown, and each of the three it knows is. The bar's first line is who says it. The page is said to wait only while what it said is shown: not for a camera's question nor a server's, and not once it is answered; and on the Mac, the view that holds a page that waits offers none of it to assistive technologies, and all of it again afterwards. Pictures of the bar for each of the three, light and dark, on the Mac and the iPhone, each looked at. Five of the rules checked by breaking them |
| The Apple app, on the Mac and the iPhone simulator | With the real core and WebKit, in fake mode: the page's button is pressed; the question is in the tab under the site's name, and Apple's audit passes over it; OK, and the page has true; it asks for a word, the field holds what it offers, the audit passes, and the page gets it; it says something more and is stopped, and what it asks after is answered as closed, with nothing shown. On the Mac the test failed when it first ran, and passed once a page that waits was no longer offered to assistive technologies (§5) |

## 8. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | In the tab, not over the window, and never while its tab is behind | A page's dialog that takes the whole window, or the front, is how a page holds a person or passes for the browser |
| D2 | The site is named by the core, on its own line, and the words are the page's under it | "example.com says" must not be something a page can write |
| D3 | A frame from another site is dismissed unseen | Its words would stand under another site's name. The engines that still allow it are the exception now |
| D4 | Silencing is offered from the second dialog, and lasts for the page | One dialog is a page talking; a second may be a page that will not stop. A person should not have to close the tab to end it |
| D5 | Leaving is never asked about | It is the one dialog whose only use against a person is to keep them |

## 9. What is not verified

- **Another login set**, as everywhere until W2.
- **Words at length, on a screen.** That 2,000 characters of a page's words scroll within the bar and leave the answers where they are is how the bar is built; no test or picture has that many words in it.
- **Android: a phone**, and a hardware keyboard: the bar ran on an emulator, and Return on a real keyboard was not tried.
- **A dialog that waits behind another prompt.** The page is kept from assistive technologies while the tab *shows* what it said. A page whose dialog waits behind another of the tab's prompts (a camera's, say) has stopped just the same, and is still offered: a shell is told only of the prompt in front. Not tried.
- **A screen reader itself.** That VoiceOver, on a Mac, reads the bar and is not kept waiting by the page is taken from what the app's test saw of the window's accessibility; nobody listened to it.
- **Typing in a prompt's field, in an Apple app.** The app's test accepts what the page offers; a person's own text reaching the page is the adapter's suite's and the model's, not the app's.
- **Return on a Mac**, in the field and outside it: how the bar is built, not tried by a test.

## 10. Done checklist

- [x] Contract 1.12 and its version in every place it is written.
- [x] The core: the rules of §3. 12 tests through `FakeShell`.
- [x] The test site's pages: one that says and asks by its buttons, and one that holds it in a frame from another origin.
- [x] Android: the adapter, on the JVM (5) and the device (6); the bar and its tests (4); pictures; a device test of the app (111 on the emulator).
- [x] Apple: the adapter and its suite against real WebKit (7); the bar and its model's tests (3); pictures; a UI test, passed on the iPhone simulator.
- [x] The Mac's UI suite, once the dialog on its screen had been answered: its test found that a page that waits must not be offered to assistive technologies (§5), which was fixed. `Tools/test.sh all` green, and `android-device`.
- [ ] Windows shell.

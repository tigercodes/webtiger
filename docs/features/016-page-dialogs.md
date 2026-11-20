# 016 — A page says something, or asks (W1)

| | |
|---|---|
| **Status** | Designed; nothing is built. To be read before it is: §8 lists what it decides |
| **Phase** | W1 of [000](000-master-plan.md) §9: a browser a day's ordinary use needs no other for. Pages say things (`alert`), ask yes or no (`confirm`) and ask for a line of text (`prompt`); today WebKit drops all three, and the System WebView shows them in a dialog of its own |
| **Contract** | 1.12, to be added: `DialogRequested`; the host request `DialogAnswer`; `DialogPrompt`, a third kind of `PromptView`; two fields of `AnswerPrompt` |
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
- **A page that says more can be silenced.** From a page's second dialog on, the prompt offers a third answer, to stop the page's messages. It dismisses this one, whatever else of the page's waits, and every later one, with nobody shown anything, until another page has loaded in the tab.
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

## 6. The shells

The same bar (014 §7), taller. The site's name with "says" on a line of its own, so that the words under it are plainly the page's; the page's words, on as many lines as they need up to a few, and scrolled beyond that; for a `prompt`, a field with the text it offers; **Cancel** and **OK** (for an `alert`, **OK** alone); and from a page's second dialog, a quieter third button that stops its messages. No button is the default for Return while the page has the keyboard; in the field of a `prompt`, Return is OK.

## 7. Tests

| Where | What |
|---|---|
| Core (through `FakeShell`) | Each of the three is shown under the site's name with the page's words, and the page waits; OK and Cancel are answered as §3's table says, a prompt with its text; an alert that is "cancelled" still returns. Dismissed with nothing shown: a web view that is no tab's, a tab with no page, a frame from another site, a kind unknown. Words longer than 2,000 characters are cut, and not inside a character. Dialogs wait their turn, and a fifth is dismissed. The second dialog of a page can be silenced, the first cannot; silenced, this one, those that wait and later ones are dismissed, until another page. A new page, a lost renderer and a closed tab dismiss what waited. A tab behind keeps its dialog. The half second. A dialog, a download and a permission request with the same number from the shell are not taken for each other |
| Adapter suites (WebKit on the Mac and the iPhone simulator; System WebView on the emulator) | A page of the test site calls each of the three by a button and writes what it got: each is reported with its words, who asks and which it is, and the page waits; answered, the page has what §3's table says; a prompt's offered text is reported and its answer returned; a web view destroyed while its page waits; leaving a page that asks to hold the person is not held up |
| The shells | Against a scripted core: the bar for each of the three, with the site's name, the words, the field and the buttons each has; what each button sends; the third button only when the core says it may; long words scroll. Pictures. The apps, with the real core and engines: a page asks, the bar shows its words under the site's name, each answer reaches the page |

## 8. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | In the tab, not over the window, and never while its tab is behind | A page's dialog that takes the whole window, or the front, is how a page holds a person or passes for the browser |
| D2 | The site is named by the core, on its own line, and the words are the page's under it | "example.com says" must not be something a page can write |
| D3 | A frame from another site is dismissed unseen | Its words would stand under another site's name. The engines that still allow it are the exception now |
| D4 | Silencing is offered from the second dialog, and lasts for the page | One dialog is a page talking; a second may be a page that will not stop. A person should not have to close the tab to end it |
| D5 | Leaving is never asked about | It is the one dialog whose only use against a person is to keep them |

## 9. What is not verified

Written when built.

## 10. Done checklist

- [ ] Contract 1.12 and its version in every place it is written.
- [ ] The core: the rules of §3.
- [ ] The test site's page.
- [ ] Android: the adapter, on the JVM and the device; the bar; pictures; a device test.
- [ ] Apple: the adapter and its suite; the bar; pictures; a UI test.
- [ ] `Tools/test.sh all` green, and `android-device`.
- [ ] Windows shell.

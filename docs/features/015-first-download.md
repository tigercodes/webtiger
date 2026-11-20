# 015 — A site's first download asks (W1)

| | |
|---|---|
| **Status** | Built and tested in the core, the Android shell (on an emulator) and the Apple shell, where it ran in the app on the iPhone simulator; **the Mac's UI test of it has not run** (§8). Windows is not started |
| **Phase** | W1 of [000](000-master-plan.md) §9: "downloads", the part [013](013-downloads.md) §2 left for the prompt that permissions needed. That prompt is [014](014-permissions.md)'s |
| **Contract** | 1.11: `DownloadPrompt`, a second kind of `PromptView`; `PERMISSION_KIND_DOWNLOADS`, among what a site was allowed |
| **Builds on** | 013: a download waits for the core's word in every shell. 014: a tab's prompt, what was allowed per origin, and the list where it is taken back |
| **Last updated** | 2026-11-20 |

## 1. User story

A page I am reading starts a download. Before anything is saved, WebTiger asks me, in that tab, whether this site may download, and names the site and the file. I say yes, and it is saved; the next file from that site is saved without a question. I say no, and nothing is saved, and the page does not get to ask me again. A file whose address I typed myself is just downloaded: I asked for it.

## 2. What is in it, and what is not

**In:** the core asks before a site's first download; "Allow" is remembered for the site, in the login set, and is in the list of what sites were allowed (014), where it is taken back; "Don't Allow" drops the download.

**Not in:**

- **Asking about each file**, or about a kind of file (an app, a script): Guard's, later (013 §2).
- **"Never for this site"**: as for the camera (014 §2), with a site's own sheet.
- **Anything in an engine adapter.** Every adapter already reports a download and waits for the core's word (013 §6): the word now comes later, when a person has to be asked.

## 3. The rule

A download a page asks for (`DownloadRequested`) is, in this order:

1. **dropped**, as before, when its web view is no tab's, or the web view started one less than a second ago (013 §3);
2. **started without asking when the person asked for the address**: the person sent the tab somewhere (an address typed, or chosen from history or from what was offered), and no page has come of it. Whatever that address turns out to be, a file or a redirect to one, they asked for it. It is good for one download: the next is the page's that is still there. A page that arrives, and a load that fails, end it;
3. **started without asking when the site was allowed** before, in this login set;
4. **dropped when the person said no to this page already**: until the tab commits another page, its downloads are dropped with nobody asked;
5. **asked about** otherwise: the tab gains a prompt (014 §4) that names the site and the file, and the download waits. "Allow" starts it, and writes down that the site may download. "Don't Allow" drops it, and is remembered for this page only.

**The site** is the origin of the page the tab has committed: it is the page that asks, wherever the file comes from. A tab with no page (a window a page opened only to download) asks in the name of the origin the file comes from: there is nothing else to name.

The rest is 014's, unchanged: one prompt at a time and three behind it, more are dropped; a prompt is not answered in its first half second, nor from a tab behind; what else waited for the same answer has it (every download of that site that waited, in whatever tab; and after a no, the others in that tab).

**What waited goes with its page.** A new page in the tab, the loss of its renderer, and the tab closing all drop the downloads that waited there: the shell is told to drop each (`DownloadCancel`). Nothing is listed of a download that was never started.

**Nothing changes for a download once it has started** (013): its record, its progress, its end.

## 4. The contract (1.11)

Added; nothing is changed or removed.

```proto
// common.proto
enum PermissionKind {
  ...
  // Since 1.11. Downloads (015). Never asked for by an engine: the core
  // asks before a site's first download. It is among what a site was
  // allowed (SitePermissions.kinds).
  PERMISSION_KIND_DOWNLOADS = 4;
}

// browser.proto
message PromptView {
  uint64 prompt_id = 1;
  oneof kind {
    PermissionPrompt permission = 2;
    DownloadPrompt download = 3;            // since 1.11
  }
}

// A page asks to download a file, and its site was not yet allowed to (015).
message DownloadPrompt {
  string site = 1;                          // redacted. Who asks, as the address field shows it
  string file_name = 2;                     // redacted. The name the file would be saved under (013 §4)
}
```

`AnswerPrompt` answers it: `allow` starts the download and is remembered for the site; not allowing drops it.

An engine that reports a request for `PERMISSION_KIND_DOWNLOADS` is refused: it is not an engine's to ask for.

## 5. The shells

The same bar (014 §7), with a sentence of its own: "*site* wants to download “*file*”." and the same two buttons. The file's name is the core's (013 §4): made safe, and shown in quotation marks so that where it begins and ends can be seen. In the list of what sites were allowed, "Downloads" is one more thing a site may do.

**As built**, in both shells: the model takes a download's prompt as it takes a permission's, the bar has the sentence, and the list has the word; the empty list's sentence now speaks of downloads too. Nothing else changed in a shell, and nothing in an adapter.

**It is how the bar was first seen in an Apple app with a real page.** No page can be made to ask for a camera on this Mac (014 §10); a download can be made to ask anywhere. Apple's audit, run over the bar on the iPhone simulator, said that "Don't Allow" could be cut short at larger text sizes: the bar's buttons now let their words take more lines, as the list's do, and are as tall as anything else that is pressed.

## 6. Tests

| Where | What |
|---|---|
| Core (`wt-app/tests/first_download.rs`, 14 through `FakeShell`) | A site's first download asks, naming the page's site and the file by the name the core made, and nothing is started or listed. "Allow" starts it under the core's name and lists it; the site is among what was allowed; its next download, and one from another page of it, start unasked, also after the app starts again; another site is asked about. "Don't Allow" drops it, lists nothing and writes nothing, and the page's later downloads are dropped unasked until another page has loaded. An address the person sent the tab to is downloaded without asking, also when it sends the browser on to a file; that is good for one, and is not lent to a page that arrived or to the page that is still there when the load failed. A tab with no page asks in the name of where the file comes from. A page not over `https` may ask to download, and is named with its scheme. What waited is dropped with a new page, a lost renderer and a closed tab. What else waited for the same answer has it: the site's download in a tab behind, after a yes; the page's other downloads, after a no. One a second from a web view, also while the first waits; four wait and a fifth is dropped; said twice, asked once. A download and a page's request that the shell gave the same number are not taken for each other, whichever came first, nor is the download what a page withdraws. To download is not an engine's to ask for. Taken back, the site asks again, and what is coming goes on. Fifteen parts of the rule checked by breaking them. 013's tests (15) pass as they were, each page's site allowed first as a person would allow it |
| The shells, against a scripted core | Android (`PromptTest`, 1 more; `PermissionsListTest`): a download the site was not yet allowed is asked about by the site and the file, and the buttons send their answer by the prompt's own name; "Location and Downloads" in the list; the empty list's new sentence. Apple (`PromptModelTests`, 1 more; `PermissionsModelTests`): the same. Pictures of the bar asking about a download, light and dark, on Android, the Mac and the iPhone; the Permissions list's pictures drawn again; each looked at |
| The Android app, on the emulator | With the real core, WebView and the system's fetcher, in fake mode: a finger on a link to a file; the bar names the site and the file while nothing is saved; Android's accessibility checks over it; "Allow", and the file is fetched and listed, as in 013 |
| The Apple app, on the iPhone simulator | With the real core and WebKit, in fake mode: a link to a file is pressed; the bar says the site wants to download “notes.txt”; Apple's audit over the bar; "Allow", and the file is listed and finishes, as in 013. On the Mac the same test has not run (§8) |

## 7. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | Every site's first download asks, also one a person clicked for | The plan's rule (013 §2). A click on a page is not a wish to save a file: the page decides what the click does |
| D2 | An address the person typed does not ask | It is the one case where the person, not a page, chose the address |
| D3 | The site is the page's, not the file's | The page is what the person can see and judge. Files come from wherever a page says |
| D4 | It is one more thing "a site was allowed", in the same store and the same list | One place to see and take back what sites may do |

## 8. What is not verified

- **Which word comes first from an engine.** When an address the person typed turns out to be a file, an adapter reports the download and that the load stopped. The core's rule does not depend on which comes first: the person's wish outlives a load that stopped. The tests give the order the System WebView adapter gives (the load stops, then the download); the other order was not tried.
- **A page that starts a download in the moment between the person sending the tab somewhere and that page arriving** borrows the person's wish, once: the core cannot tell whose download it is. Not tried against an engine.
- **The Mac's UI suite.** Its test of a download was changed to answer the question first, and has not run since: a dialog of the system's is on this Mac's screen, and the gate does not start the Mac's UI suite while one is ([014](014-permissions.md) §10). The same test passes on the iPhone simulator.
- **"Don't Allow" in an app.** The apps' tests say yes; a no, and what it does to a page's later downloads, is the core's test.
- **A phone.** Android ran on an emulator.

## 9. Done checklist

- [x] Contract 1.11 and its version in every place it is written.
- [x] The core: the rule of §3. 14 tests through `FakeShell`; 013's 15 still pass, with the site allowed.
- [x] Android: the bar's sentence, the list's word; pictures; the device test of a download answers the question (104 on the emulator).
- [x] Apple: the bar's sentence, the list's word; pictures; the UI test of a download answers the question, passed on the iPhone simulator.
- [ ] The Mac's UI suite, when the dialog on its screen has been answered. Every other stage of `Tools/test.sh all` passed, run one by one.
- [ ] Windows shell.

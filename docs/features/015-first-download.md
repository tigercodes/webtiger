# 015 — A site's first download asks (W1)

| | |
|---|---|
| **Status** | Designed; nothing is built. To be read before it is: §7 lists what it decides |
| **Phase** | W1 of [000](000-master-plan.md) §9: "downloads", the part [013](013-downloads.md) §2 left for the prompt that permissions needed. That prompt is [014](014-permissions.md)'s |
| **Contract** | 1.11, to be added: `DownloadPrompt`, a second kind of `PromptView`; `PERMISSION_KIND_DOWNLOADS`, among what a site was allowed |
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
2. **started without asking when the person asked for the address**: the load in flight in the tab is the person's own (an address typed, or chosen from history or from what was offered) and has not yet become a page. Whatever that address turns out to be, a file or a redirect to one, they asked for it;
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

## 6. Tests

| Where | What |
|---|---|
| Core (through `FakeShell`) | A page's first download asks, naming the site and the file, and nothing is started or listed; "Allow" starts it under the core's name, lists it, and the site's next download starts unasked, also after the app starts again; "Don't Allow" drops it, lists nothing, and the page's later downloads are dropped unasked until another page; an address the person typed is downloaded without asking, also when it redirects, and a page's own download a moment later is asked about; a tab with no page asks in the name of where the file comes from; what waited is dropped with a new page, a lost renderer and a closed tab; what else waited for the same answer has it; the limits of 013 and 014 still hold; an engine cannot ask for downloads as a permission; the site is in the list of what was allowed, and taken back it asks again. 013's tests, with the site allowed first |
| The shells | The bar's sentence for a download, against a scripted core, and a picture; "Downloads" in the Permissions list. The apps' tests of a download (013) press "Allow" first |

## 7. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | Every site's first download asks, also one a person clicked for | The plan's rule (013 §2). A click on a page is not a wish to save a file: the page decides what the click does |
| D2 | An address the person typed does not ask | It is the one case where the person, not a page, chose the address |
| D3 | The site is the page's, not the file's | The page is what the person can see and judge. Files come from wherever a page says |
| D4 | It is one more thing "a site was allowed", in the same store and the same list | One place to see and take back what sites may do |

## 8. What is not verified

Written when built.

## 9. Done checklist

- [ ] Contract 1.11 and its version in every place it is written.
- [ ] The core: the rule of §3. 013's tests still pass, with the site allowed.
- [ ] Android: the bar's sentence, the list's word; the device test of a download.
- [ ] Apple: the bar's sentence, the list's word; the UI test of a download.
- [ ] `Tools/test.sh all` green, and `android-device`.
- [ ] Windows shell.

# 013 — Downloads (W1)

| | |
|---|---|
| **Status** | Built and tested in the store, the core, the Apple shell and the Android shell (on an emulator). Windows is not started. What it leaves out is in §2, and what was not seen in §9 |
| **Phase** | W1 of [000](000-master-plan.md) §9: "downloads". The `download` table of the store (000 §5.6) and the downloads half of the Library (000 §4), in their first form |
| **Contract** | 1.9: `DownloadRequested`, `DownloadProgress`, `DownloadFinished`, `DownloadFailed`; the host requests `DownloadStart` and `DownloadCancel`; `QueryDownloads`, `CancelDownload`, `ForgetDownloads` |
| **Last updated** | 2026-11-19 |

## 1. User story

I click a link to a file. WebTiger saves it in my Downloads folder under a sensible name, shows me that it is coming and how far it has got, and when it is there I can see it in the list of what I downloaded. I can stop one that is coming, and take one off the list.

## 2. What is in it, and what is not

**In:** a response an engine cannot show, or that says it is an attachment, or a link that says "download", becomes a download; the file's name is made safe; the file is saved in the system's place for downloads; the download is in a list with its progress and how it ended; one that is coming can be stopped; the list can be cleared.

**Not in, each with where it goes:**

- **Asking before a site's first download**, and remembering the answer for the site. It is a prompt like a permission's, and is built with those (W1's permission prompts). Until then every download a page starts is started, at most one a second from a web view (as links for the system are, 007), and always shown: nothing is saved that the list does not say.
- **Choosing where a file goes**, and "Save As": settings, and the System service's file picker.
- **Opening a file, or showing it in its folder**, from the list: the System service again, with the Library (W3).
- **Resuming** a download that was cut off, and one that outlives a quit. A download that was coming when the app ended is listed as failed.
- **Checking what was downloaded** (a file's kind against its name, a known-bad list): Guard's, later.
- **Private windows** (W2): their downloads are in no list after the window closes.
- **On Android, what only the page can fetch**: a `blob:` or `data:` address, or the answer to a form that was posted. The System WebView does not download; it says a download was asked for, and the app fetches the address again (§6). Nor does it pass on the name a link's `download` gives: the name is made from the address and the kind.
- **On Android, a download that outlives the app.** The system goes on fetching after the app has gone, and the core lists what was coming as interrupted (§9).

## 3. Who does what

The engine notices a download and says so. The core decides whether it starts, names the file, and keeps the record. The shell (or its engine) moves the bytes into the system's place for downloads, and says how it is going.

| What the shell reports | What the core does |
|---|---|
| `DownloadRequested`: a web view's page asked for a download, with an id of the shell's own, the address, the name the site suggests, the kind and the size if known | Checks the web view is one it has, and that this is not the second in a second from it. Makes the name safe (§4). Writes the record. Asks the shell to start it under that name (`DownloadStart`), or to drop it (`DownloadCancel`). A shell that will not start it leaves a download that failed |
| `DownloadProgress`: so many bytes of so many | Keeps the numbers for the list. Not written to the store: a download that does not finish is "failed" there |
| `DownloadFinished`: the name the file was saved under (the shell adds " (2)" when the name is taken) | Writes how it ended, and the name |
| `DownloadFailed`: with a code (`download.cancelled`, `download.network`, `download.disk`, `download.failed`) | Writes how it ended. What a shell says that is not a code is written as `download.failed` |

| Request from a shell's screen | What the core does | Answer |
|---|---|---|
| `QueryDownloads { window_id, before_id }` | A page of the login set's downloads, newest first, each with its name, site, size, how far it is and how it ended | `DownloadsPage` |
| `CancelDownload { download_id }` | Asks the shell to stop it. It ends as `download.cancelled`, and what was saved of it is removed by the shell | `Ack`; `download.unknown` |
| `ForgetDownloads { window_id, download_id or everything }` | Takes records off the list. A download that is still coming is stopped first. The files stay: they are the person's | `Ack` |

- **A window's snapshot says how many downloads are coming** (`WindowSnapshot.downloads_active`), so a shell can show that something is, without asking.
- **A download is its login set's.** The list a window shows is its Space's; another login set's downloads are not in it.
- **Nothing of a download is logged**: its address and its file's name are marked for redaction, like an address.

## 4. The name of the file

The core makes it, from what the site suggests (`Content-Disposition`, or a link's `download`), else from the last part of the address when it is the web's (a `blob:` or `data:` address names nothing), else `download`. A shell never writes a name a page gave it.

- No folders: everything up to the last `/` or `\` goes, also when it was written as an escape in the address. No control characters, and none of `< > : " | ? *`, which one system or another refuses.
- Nothing that turns text round: the marks that make `gpj.exe` read as `exe.jpg` go.
- Not hidden, and not a name a system reserves: leading dots go, and so do trailing dots and spaces; `CON`, `NUL`, `COM1` and their kind gain an underscore.
- Not too long: 200 bytes at most, cut in the name and not in its ending (`.tar.gz` stays whole).
- Never empty: `download`. And a name with no ending takes the one its kind is known by (`report` of `application/pdf` is `report.pdf`); a kind that says nothing adds nothing.

## 5. The store

Migration 5 adds `download`, the table 000 §5.6 names.

```sql
CREATE TABLE download (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    login_set_id BLOB NOT NULL REFERENCES login_set (id),
    url TEXT NOT NULL,
    file_name TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    total_bytes INTEGER NOT NULL,          -- -1 when the site did not say
    state TEXT NOT NULL,                   -- 'coming', 'finished', or a failure's code
    started_at INTEGER NOT NULL,
    ended_at INTEGER
) STRICT;
```

A download ends once: what is said of one that has ended changes nothing. Downloads still `coming` when the core starts are ended as `download.interrupted`: the app that was fetching them is gone. Forgetting a record leaves no trace of it in the store's files, as forgetting a visit does (008), and touches no file of the person's.

## 6. The engines

| | WebKit | System WebView |
|---|---|---|
| Notices | `WKNavigationDelegate`: a response it cannot show, or an attachment, becomes a `WKDownload` | `DownloadListener.onDownloadStart`: the address, the kind, the size, and what the site suggests for a name |
| Moves the bytes | WebKit itself, in the page's own session: its cookies, a posted form's answer, a `blob:` | The system's `DownloadManager`, asked with the login set's cookies for the address. It fetches again: only what an address alone can fetch (§2) |
| Where | `~/Downloads` on a Mac; the app's own Downloads folder on an iPhone, which the Files app shows | The device's Downloads |
| Progress | `WKDownload.progress` | Asked of the `DownloadManager` |

**As built on Apple** (`Downloads` in the WebKit adapter; where the app saves, in `BrowserSession`):

- WebKit asks the adapter where a download's file goes; that is the moment the adapter reports it, with what the site suggests, and it waits for the core's word. Nothing is saved before it. `DownloadStart` names the file, and WebKit fetches it there; `DownloadCancel` means there is no download.
- A response is a download when WebKit cannot show its kind, or when it says it is an attachment; a link that says "download" is one once the core has allowed its address. The load that became a download ends as one that stopped, and the page on screen stays.
- **The name is the core's, and the adapter holds it to a name all the same**: of whatever it is given, only the last part is used, so nothing is written outside the folder. A name that is taken, by a file or by a download that is coming, gains " (2)", " (3)", and the adapter says which it used.
- **Where**: the real app saves in the system's place for downloads (`~/Downloads` on a Mac; on an iPhone the app's own Downloads folder, which the Files app is told to show). A profile's go in the profile's own folder. In fake mode, and for every engine a test makes, the folder is one of the engine's own among the temporary files: nothing but the app itself, started for real, can name a person's Downloads.
- How far a download has got is passed on four times a second at most.
- **The kind is what WebKit makes of it.** For a file ending `.bin` that its site calls `application/octet-stream`, a Mac's WebKit says so and an iPhone's says `application/macbinary`.
- **On an iPhone simulator the first download of a run takes WebKit a while to begin** (23 s measured here, then 7, then under a second); nothing is held afterwards, and dropping one and deleting its login set are quick.

**The list, on Apple** (`DownloadsView`, `DownloadsModel`, `LibraryView`):

- **Behind the History button, beside History.** The sheet that button opens has two lists and a switch between them, where a list's title was; the button's name is now "History and Downloads". The row of tabs gains no button: it has four already, and is W2's to lay out again. The Library (W3) will be the home of both lists.
- A dot on that button says a download is coming, and assistive technologies are told "Downloading".
- Each download by its file's name, the site it came from, and how it stands: so much of so much, with a bar, while it is coming and its size is known; so much "so far" when it is not; its size when it has finished; a sentence when it failed. The core speaks codes (`download.network`); the words are the shell's, and a code this build has no words for still says "The download failed."
- One that is coming has a button to stop it; one that has ended, a button to take it off the list. "Clear List" takes them all off. No file is touched by either.
- The list asks the core again every second while it is shown: how far a download has got is not pushed, and one that starts while the list is open belongs in it.
- On an iPhone Apple's audit said the switch's words could be cut short at larger text sizes: a switch and two buttons are too many for one row there. A row that broke in two only when it had to did no better (the audit then said of every control in it that its text size could not be changed), so on an iPhone the switch has a row of its own and the buttons the next; on a Mac, whose sheet is wider, they share one.

**As built on Android** (`Downloads` and `SystemFetcher` in the adapter; where the app saves, in `BrowserSession`):

- The System WebView does not download. Its `DownloadListener` says one was asked for: the adapter reports it, under an id of its own, and fetches nothing until the core has said under what name. `DownloadStart` hands the address to the system's `DownloadManager`; `DownloadCancel` means there is no download, or stops the one that is coming and has the system remove what it had saved.
- **The fetch is the login set's.** The system fetches in a process of its own, which knows nothing of the WebView's profiles: the adapter hands it the cookies the login set's profile has for the address, and what the WebView says it is (`User-Agent`). So it gets what an address and those cookies get: a file a sign-in guards, and not a `blob:` or a posted form's answer (§2). It sends no `Referer`.
- **The load that became a download is over**, as one that stopped (`net.cancelled`), at the moment the WebView says it is a download; the page on screen stays. On the emulator the adapter reported a load started, the download, and the load stopped, and nothing after. A link that says "download" starts no load at all.
- **What the site suggests is read as the platform reads it** (`URLUtil.guessFileName`: from `Content-Disposition`, else from the address and the kind). The WebView does not pass on what a link's `download` says: of the test site's link, which says "kept page.html", it reports `landed.html`. The core makes the name either way (§4).
- **The name is the core's, and the adapter holds it to a name all the same**: only its last part is used. A name that is taken is not written over: the system chooses another (`notes-1.txt`, where Apple's adapter writes `notes (2).txt`), and the adapter says which, read from where the system saved the file.
- **Where**: the real app saves in the device's Downloads, and the system's own notice of the download stays when the file is whole: until the list can open a file (§2), that notice is how a person does. In fake mode, and for every engine a test makes, the files go to the app's own folder for downloads: nothing but the app itself, started for real, writes where a person's files are. The app asks for no permission for either: none is needed since Android 10, the oldest it runs on.
- How far a download has got is asked of the system every 400 ms while one is coming, and passed on when it has changed.
- The system's reason for a failure becomes one of the contract's codes: `download.disk` (the file could not be written, no room, no storage), `download.network` (the data, the redirects, the server's answer, a fetch that cannot go on), else `download.failed`. A fetch the system no longer knows of, removed from under the app, is `download.cancelled`. The reason's own text never leaves the shell.

**The list, on Android** (`DownloadsList`, `DownloadsModel`, `Library`): the same list behind the same button, with the same switch, dot, words and buttons as on Apple, and it asks the core every second while it is shown.

- The switch has a row of its own and the buttons the next, as on an iPhone. The system's Back closes the sheet.
- Sizes are written as the system writes them, which since Android 8 is in thousands: "1.2 MB of 4.8 MB".
- A name is cut in its middle after two lines, so that its ending stays.

## 7. Tests

| Where | What |
|---|---|
| The name (`wt-core`, 9) | What stays as it is; no folder, by either separator; nothing a file system refuses or hides; nothing that turns the name round; the names a system keeps for devices, and what only looks like one; the address names it when nothing is suggested, escapes read, and an escape that hides a folder is a folder; never empty, and not named by an address that is not the web's; a name with no ending takes its kind's; a long name is cut in the name, not in its ending, and never inside a character. Eight of the rules checked by breaking them |
| Store (`wt-store`, 6) | The migration from version 4, which keeps what was there. A download is written as it starts and once as it ends, under the name the shell used; a failure keeps the name asked for, and a size not said is none; listed newest first, a page at a time, by login set; forgetting one, and all, of one login set's; those left `coming` are interrupted, and what had ended is as it ended |
| Core (`wt-app/tests/downloads.rs`, 15 through `FakeShell`) | A download starts under a name the core makes, from what the site suggests or from the address, and is listed with its site, kind and size; the snapshot says one is coming. How far it has got is in the list; how it finished is kept, under the name the shell used; a word about one that has ended changes nothing. A size the site did not say is the one the shell learns. A failure by its code; words that are not a code are not kept. One a second from a web view, to the millisecond, and another tab's page is not held back; a web view the core does not have gets none; said twice, one starts. A shell that will not start it leaves a download that failed. Stopping ends it and has the shell drop it, by the shell's name for it; one that is not coming cannot be stopped. Forgetting one, and all, and stopping one that is coming first; naming nothing. No such window. A page at a time, newest first. A download outlives the tab that asked for it. Started again over the same store, what was coming is listed as interrupted and nothing is started again. Nine of the rules checked by breaking them |
| WebKit adapter suite (`WebKitDownloadTests`, 8, on the Mac and the iPhone simulator) | The test site has a file served as an attachment, one of a kind no engine shows, a link that says "download", an attachment whose suggested name is a path, and a file that says who is signed in. An attachment is reported with its address, the name the site suggests, its kind and size, and nothing is saved until the core says; it is then whole under the name the core gave, and the page that asked is still the page. A kind nothing shows is a download. A link that says download is one, and the web view goes nowhere. A download the core drops saves nothing, and a word for it afterwards is refused. A name that is taken gains a number, twice. A name that is a path is held to its last part, and nothing is written outside the folder. A download is fetched as its login set: signed in, the file says who; another login set's web view is nobody. A word about a download there is not |
| Where the app saves (`ProfileTests`, 2 more) | The usual app names the system's place; a profile's downloads are in its own folder; an engine nobody gave a folder saves among the temporary files, each in its own |
| System WebView adapter, on the JVM | `DownloadsTest` (7), through the engine with a fetcher that fetches nothing: what a page offers is reported, and nothing is fetched until the core says, then with the address, the login set and the core's name; each download has an id of the shell's own, and is fetched as its web view's login set; how a fetch goes is said when it changes, and once that it is whole, under the name the system used; a failure by its code, and a fetch the system has forgotten; a fetch the system will not start is a download that failed; one the core drops is never fetched, one that is coming is stopped, and a word about one there is not is refused; the downloads are asked after for as long as one is coming, and no longer. The tracker (1 more): a load that became a download is over, and a download with no load on its way ends nothing. Where the app saves (1): the device's Downloads, and the app's own folder in fake mode. Seven of the rules checked by breaking them |
| System WebView adapter suite (`SystemWebViewDownloadTest`, 8, on the emulator) | With a finger on each link, the system's own `DownloadManager` fetching, into the app's own folder. An attachment is reported with its address, the name the site suggests, its kind and size, and nothing is saved until the core says; it is then whole under the name the core gave, and the page that asked is still the page. A kind nothing shows is a download. A link that says download is one, the web view goes nowhere, and the name it says is not passed on. A download is fetched as its login set: signed in, the file says who; another login set's web view is nobody. A download the core drops fetches nothing, and a word for it afterwards is refused. A name that is taken is not written over, and the adapter says which name was used. A name that is a path is held to its last part. A word about a download there is not |
| The Apple list | `DownloadsModelTests` (5): the list is what the core says, newest first, and older when asked for; stopping asks the core and the list is asked for again; forgetting one leaves the list at once, and forgetting all asks again; while the list is shown it is asked for about every second, and no more once it is gone; how a download stands, in words, for each state and each code, and never a code's own text. Pictures of the list (coming with a bar, failed, finished) and of the empty list, light and dark, on the Mac and the iPhone |
| The Apple apps, with the real core and engine | On the Mac and the iPhone simulator, in fake mode: a link to a file that says it is to be saved is clicked; behind the History button, under Downloads, the file is listed by its name and finishes, with its size; Apple's audit over the list; taken off the list, the list is empty; the page that offered the file is still the page |
| The Android list | `DownloadsModelTest` (5): as the Apple model's first three, and: a list taken further down is asked for whole again; only the answer to the last question is shown, with answers let go out of order. `DownloadsListTest` (8, drawn on the JVM): a download by its name, its site and how it stands, with the button to stop one that is coming and to take off one that has ended; how a download stands, in words, for each state and each code, and never a code's own text; stopping asks the core, forgetting takes it off; "Clear List" takes all off, says there is nothing, and is then off; shown, the list is asked for every second, and what the core then says is what it shows; a picture of it asks once; the switch beside History shows the downloads and goes back, and nothing is asked about downloads until they are shown; a dot on the button while the snapshot says one is coming. Pictures of the list (each of five states) and of the empty list, light and dark |
| The Android app, with the real core and engine | On the emulator, in fake mode, the system fetching: a finger on a link to a file that says it is to be saved; the file is whole in the app's own folder, and nothing else is written; the page that offered it is still the page, and nothing failed; behind the History button, under Downloads, it is listed by its name, its site and its size, as one that has finished; Android's accessibility checks over the empty list and the list; taken off the list, the list is empty and the file is where it was; the switch goes back to History, and the system's Back closes the sheet |

No test writes into a person's own Downloads folder: the suites give the adapter a folder of their own, and on Android the app's.

**By hand, once** (2026-11-19, the emulator, the app started for real and not in fake mode, a page served from the Mac): a finger on a link to an attachment; the file was in the device's Downloads, whole, and nowhere else; the list said its name, its site and its size; the system's notice of the download was in the status bar. The file and the app were then removed from the emulator.

## 8. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | The engine moves the bytes where it can (WebKit); the system does where it cannot (Android) | A download in the page's own session is the only kind that gets a file a sign-in guards, or one a page made itself. Fetching in the core would get neither, and would need the core to hold cookies |
| D2 | The core names the file | A name a page chooses is the oldest trick there is. One rule, tested once, for every shell |
| D3 | No asking in this feature; one a second, and always listed | Asking needs the prompt that permissions need, which is not built. A download nobody can see is the harm; one in a list can be stopped and deleted |
| D4 | The files are not deleted with their records | The list is WebTiger's; the files are the person's |
| D5 | Progress is not written to the store | It changes many times a second, and means nothing after a quit |

## 9. What is not verified

- **Another login set, through the core.** As for zoom (012 §8): one Space and one login set until W2. That a login set's downloads are its own is the store's test.
- **A shell that says it finished what it never started**, or names a file it was not given: the core writes down the name the shell says it used, and cannot see the folder.
- **Stopping a download that is coming.** The suite's files arrive at once; stopping was tried only before a download had started. That WebKit removes what it had saved, and that the adapter says "cancelled", is written and not seen.
- **How far a download has got**: no test waits for a report of it, for the same reason.
- **A person's Downloads folder.** No test writes there, on purpose: that the real app saves in `~/Downloads`, and that the Files app shows the iPhone's folder, was not looked at. A sandboxed build of the Mac app will need the entitlement for that folder, which comes with the distribution work.
- **A tab a page opened only to download** stays, empty: the core closes such a tab when its address is refused (006), and not yet when it becomes a download.
- **The list while a download is coming**, in the app: the test's file arrives before the list is opened. The bar, the count "so far" and the stop button are drawn in the catalogue's pictures and tested in the model; nobody pressed Stop on a download in the app.
- **The dot on the button**: drawn from the snapshot's count, which the core's tests cover. No picture has it; on Apple no test looked for it, on Android the window's test does.
- **Android: a phone.** The adapter, the list and the app ran on one emulator (Android 16, System WebView 133). That the device's Downloads needs no permission was seen there once, by hand (§7); on Android 10 to 15 it is what the platform documents, not seen.
- **Android: a download that outlives the app.** The system's `DownloadManager` goes on fetching when the app has gone. The core, started again, lists what was coming as interrupted, and nothing asks the system what became of it: the file may be whole in Downloads while the list says it failed. Not tried; to be put right with resuming (§2).
- **Android: stopping, and how far.** As on Apple, the device's files arrive at once: that the system removes what it had saved of a fetch that is stopped, and the reports of how far one has got, are tested with a fetcher that fetches nothing.
- **Android: what a sign-in guards.** The cookie the suite signs in with is a plain one that lasts. A cookie for the session only, one a script cannot read, and a site that wants the page's address with the request (the fetch sends none) were not tried.
- **Android: the system's reasons for a failure.** The table that turns them into codes was read, not run: no fetch failed on the device.

## 10. Done checklist

- [x] The name's rules, in `wt-core`. 9 tests.
- [x] The store: migration 5; the records. 6 tests.
- [x] Contract 1.9 and its version in every place it is written.
- [x] The core: the rules of §3. 15 tests through `FakeShell`.
- [x] The test site's files.
- [x] Apple: the adapter and its suite (7); where the app saves (2).
- [x] Apple: the list and its model's tests (5); pictures; a UI test on the Mac and the iPhone simulator.
- [x] Android: the adapter, on the JVM (9) and the device (8); the list and its tests (5 and 8); pictures; a device test of the app.
- [x] `Tools/test.sh all` green, and `android-device` (93 on the emulator).
- [ ] Windows shell.

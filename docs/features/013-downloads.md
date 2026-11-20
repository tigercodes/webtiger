# 013 — Downloads (W1)

| | |
|---|---|
| **Status** | In progress: the name of the file (§4), the store's records (§5), the contract (1.9) and the core (§3) are built and tested. No shell reports a download yet, and each answers the core's two requests as a shell built before 1.9 does; the engines and the screens are planned (§10). The design below is to be read before it is built: §8 lists what it decides |
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
- **On Android, what only the page can fetch**: a `blob:` or `data:` address, or the answer to a form that was posted. The System WebView does not download; it says a download was asked for, and the app fetches the address again (§6).

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

## 7. Tests

| Where | What |
|---|---|
| The name (`wt-core`, 9) | What stays as it is; no folder, by either separator; nothing a file system refuses or hides; nothing that turns the name round; the names a system keeps for devices, and what only looks like one; the address names it when nothing is suggested, escapes read, and an escape that hides a folder is a folder; never empty, and not named by an address that is not the web's; a name with no ending takes its kind's; a long name is cut in the name, not in its ending, and never inside a character. Eight of the rules checked by breaking them |
| Store (`wt-store`, 6) | The migration from version 4, which keeps what was there. A download is written as it starts and once as it ends, under the name the shell used; a failure keeps the name asked for, and a size not said is none; listed newest first, a page at a time, by login set; forgetting one, and all, of one login set's; those left `coming` are interrupted, and what had ended is as it ended |
| Core (`wt-app/tests/downloads.rs`, 15 through `FakeShell`) | A download starts under a name the core makes, from what the site suggests or from the address, and is listed with its site, kind and size; the snapshot says one is coming. How far it has got is in the list; how it finished is kept, under the name the shell used; a word about one that has ended changes nothing. A size the site did not say is the one the shell learns. A failure by its code; words that are not a code are not kept. One a second from a web view, to the millisecond, and another tab's page is not held back; a web view the core does not have gets none; said twice, one starts. A shell that will not start it leaves a download that failed. Stopping ends it and has the shell drop it, by the shell's name for it; one that is not coming cannot be stopped. Forgetting one, and all, and stopping one that is coming first; naming nothing. No such window. A page at a time, newest first. A download outlives the tab that asked for it. Started again over the same store, what was coming is listed as interrupted and nothing is started again. Nine of the rules checked by breaking them |
| Adapter suites | The test site gains a file served as an attachment, one of a kind no engine shows, and a link that says "download". Each is reported once, with its name and size; the file arrives whole in a folder of the test's own; cancelling removes what was saved; a name that is taken gains " (2)" |
| The shells' screens | The list's model against `ScriptedCore`; pictures of the list coming, finished and failed; the apps with the real core: click, see it listed, see it finish |

No test writes into a person's own Downloads folder: the suites give the adapter a folder of their own.

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
- The engines and the screens: written when built.

## 10. Done checklist

- [x] The name's rules, in `wt-core`. 9 tests.
- [x] The store: migration 5; the records. 6 tests.
- [x] Contract 1.9 and its version in every place it is written.
- [x] The core: the rules of §3. 15 tests through `FakeShell`.
- [ ] The test site's files.
- [ ] Apple: the adapter and its suite; the list; pictures; a UI test.
- [ ] Android: the adapter, on the JVM and the device; the list; pictures; a device test.
- [ ] `Tools/test.sh all` green, and `android-device`.
- [ ] Windows shell.

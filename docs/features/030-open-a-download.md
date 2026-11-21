# 030 — Opening a file from the list of downloads (W1)

| | |
|---|---|
| **Status** | Built and tested in the core, the Android shell and the Mac. On Android it was seen by hand in the app started for real, on an emulator. On the Mac the file itself is never opened by a test (that starts another app); in the Mac's app, in fake mode, the button is pressed, and the core and the shell answer with nothing opened. The iPhone is not built: its list does not offer it |
| **Phase** | Brought forward from W3 ([013](013-downloads.md) §2: "Opening a file, or showing it in its folder, from the list: the System service again, with the Library"). Since [027](027-cookies-stay-with-their-site.md), Android has no notice of the system's to open a file from: the list is the only way inside WebTiger |
| **Contract** | 1.22: `OpenDownload`; the host service `OpenFile` |
| **Builds on** | 013: the list of downloads, and the name each file is saved under. [007](007-outside-links.md): the System service, which hands something to whatever the system has for it |
| **Last updated** | 2026-11-21 |

## 1. User story

A file I downloaded is in the list. I press it, and it opens: in whatever my device opens that kind of file with. If nothing on the device opens it, or the file is no longer there, the list says so, in words.

## 2. What is in it, and what is not

**In:** a download that has finished can be opened from the list; the system opens it, with whatever it has for that kind of file; a file that is gone, and a kind nothing opens, are said.

**Not in:**

- **Showing the file in its folder**, and the rest of the Library (W3).
- **Opening it inside WebTiger** (a picture or a PDF in a tab): a file on the device is not a page, and opening one as a page is a decision of its own.
- **Opening a file as soon as it has come.** Nothing opens unasked.
- **Choosing what opens it.** The system's choice, and where there are several the system asks.

## 3. The rule

**The shell** hears a press on a download in the list, and asks the core (`OpenDownload`, with the download). It opens nothing by itself.

**The core** decides whether it may be opened:

- a download the core does not have, or one that is not the window's Space's, is refused (`download.unknown`);
- one that is still coming, or that failed, is refused (`download.not_finished`): there is no whole file;
- otherwise it asks the shell's System service to open the file (`OpenFile`), by the name the file was saved under, with the kind the site said it was, and answers as the shell answers.

**The shell's System service** finds the file where this shell saves downloads, by that name, and hands it to the system:

- the system took it: `Ack`;
- nothing on the device opens that kind of file: `system.no_handler`;
- the file is not there (moved, renamed or removed since): `download.missing`.

**What the system is handed is the file and nothing else**: not the address it came from, and no permission beyond reading that one file.

**A file is opened only by a person's press.** No page can ask for it, and nothing opens when a download ends. A file is something a site sent: opening it is the person's decision each time, and what opens it is the system's own choice of app, which is where the system's own warnings about a kind of file are.

**In fake mode a shell opens nothing** and answers `Ack` (as 007): no test starts another app.

## 4. The contract (1.22)

Added; nothing is changed or removed.

```proto
// browser.proto
// Opens a finished download's file with whatever the system has for it
// (030). Answered with Ack; "download.unknown"; "download.not_finished"; or
// what the shell's System service said: "system.no_handler",
// "download.missing".
message OpenDownload {
  uint64 window_id = 1;
  uint64 download_id = 2;
}

// host.proto
// System: opens a file this shell saved as a download, by the name it was
// saved under, with whatever the system has for its kind. The core asks only
// for a download that has finished, on a person's press. Answered with Ack
// once the system has taken the file; "system.no_handler" when nothing on
// the device opens it; "download.missing" when the file is not where it was
// saved. In fake mode a shell opens nothing and answers Ack.
message OpenFile {
  string file_name = 1;                     // redacted
  string mime_type = 2;
}
```

## 5. The shells

**Android** (`ShellHost.kt`: `FileOpener`, `SystemFileOpener`; `DownloadsModel`; the list's row).

- **A download that has finished is pressed**, anywhere on its row: a screen reader says "Open". One that is coming or that failed is not pressed.
- **The file is found** among the files the app itself saved in the device's Downloads, by its name, in the system's record of files (`MediaStore`): the newest, where a name was used twice. Without a permission the app is shown only its own files there, which is all it looks for.
- **The system is handed** the file's own address in that record, its kind (what the system makes of the file as it is saved; what the site said, where the system makes nothing of it), and leave to read that one file: none to change it, no app named, nothing of where the file came from. It opens in a task of its own.
- **Why it did not open** is said under the download, in words, until it is pressed again.
- **In fake mode nothing is opened**: files are saved in the app's own folder there, and the opener is one that opens nothing.

**The Mac** (`ShellHost.swift`: `FileOpener`; `DownloadsModel`; `DownloadsView`).

- **A download that has finished has a button, "Open"**, beside the one that takes it off the list. (A button of its own, and not the row: the row is what the app's tests read. When this was built the Mac's UI suite could not run to say what changing the row does; it has run since, with the button, and the button stays.)
- **The file is the one of that name in the folder the app saves downloads in.** A name is a name: one that is a path, or that leads out of the folder, is no file. The Mac opens it with whatever it has for its kind; where it has nothing, the Mac says so itself, in its own dialog, and offers to choose an app.

**The iPhone and iPad**: not built. An iPhone has no way to hand a file of an app's to whatever opens its kind; what it has is a preview and a sheet for sharing, which are a design of their own. The list offers nothing there.

## 6. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | The core is asked, and asks the shell back | Which downloads may be opened (finished, the Space's own) is the core's to say; where the file is and how the system is asked is the shell's. The same shape as a link for the system (007) |
| D2 | By the file's name, not by a path | The core knows the name the shell saved under and nothing of the device's folders. Each shell knows where it saves |
| D3 | The system opens it, not WebTiger | A downloaded file is not a page. What may open which kind of file, and what is warned about, is the system's |

## 7. Tests

| Where | What |
|---|---|
| Core (`wt-app/tests/downloads.rs`, 4 more: 19, through `FakeShell`) | A download that has finished is opened by the name it was saved under (the one the shell used, when the name asked for was taken) and the kind the site said, and nothing of where it came from; pressed again, asked again. One that is coming, one that failed, one the core does not have, an id no download could have, and a window there is not, are refused, and the shell is asked to open nothing. What the shell says of opening is the answer: nothing opens it, or the file is gone; the list is as it was. A download is opened only from the list it is on: from a Space with logins of its own, another Space's download is not known. Six parts of the rule checked by breaking them: all caught |
| Android (`ShellHostTest`, 4 more; `DownloadsModelTest`, 2 more; `DownloadsListTest`, 2 more; on the JVM) | A file goes to the opener by its name and kind, and what came of it is the answer; an engine is asked nothing. In fake mode no file is opened and the core is told it was. The system is handed the file, its kind and leave to read it, and nothing else: no leave to write, no app named, no extras; the kind is the system's own where it has one; a file that is not there starts nothing. A kind nothing opens, and an activity that may not be started, are "no handler". The model asks the core for a finished download, with its window, and not for one with no whole file; why a file did not open is kept by the core's code, each download's its own, until it is pressed again and opens. In the list, only the finished row is pressed, and says "Open" to a screen reader; why it did not open is said under it in words, never by its code. Twelve parts of this checked by breaking them: all caught |
| The Apple shell (`ShellHostTests`, 3 more; `DownloadsModelTests`, 3 more) | The same of the host and the model. And the Mac's own opener, about files that are not there: a file that is gone, a folder, an empty name, and a name that is a path or leads out of the folder are each no file, and are answered "missing". Why a file did not open is said in words. The Mac's picture of the list was made again, light and dark, with the button on the finished download and on no other: each looked at |
| The Android app, on the emulator (`AppWindowTest`, in its test of a download) | With the real core, in fake mode: the finished download's row is pressed; the core says it may be opened and asks the shell, which in fake mode opens nothing and says it did; nothing is said under the download; Android's accessibility checks pass over the list with the row that can be pressed |
| The Mac's app (`MenuTests`, in its test of a picture saved from the menu; 2026-11-21) | With the real core and real WebKit, in fake mode: the picture that was saved is whole in the list, and has the button that opens it; pressed, the core says the file may be opened and asks the shell, which in fake mode opens nothing and says it did; the row says what it said, and the button is still there. Not checked by breaking it: that a refusal is said in the row is the model's test |
| By hand (2026-11-20, the emulator, the app started for real, the test site served from the Mac) | Two files downloaded: a text file and one of a kind nothing shows. In the list, the text file pressed: the system's own "Open with" came up, offering its apps; nothing was chosen, and Back returned to WebTiger. The other file pressed: the system took it (a part of the system on that emulator claims every file of that kind, and did nothing visible with it). The text file removed from the device, then pressed: "The file isn't where it was saved any more." under it. The files and the app's data were removed afterwards |

## 8. What is not verified

- **A file actually opening in another app.** No test starts another app. By hand on the emulator the system's chooser was reached, and nothing was chosen in it.
- **"Nothing on this device opens this kind of file", on a device.** The emulator has a part of the system that takes every file of an unknown kind, so the words were not seen there; that they are said when the system refuses is the JVM's test.
- **The Mac: a file opened.** In the app the button is pressed only in fake mode, where nothing is opened; started for real it would hand the file to another app, which no test does and which was not done by hand either. What is tested of the Mac's own opener is its refusal of what is not a file in its folder.
- **A phone**; and what a phone's own apps make of being handed a file this way.
- **A kind of file the system warns about** (an app to install): what the system then says is the system's, and was not looked at. WebTiger asks for no leave to install anything.
- **iPhone and iPad**: not built.

## 9. Done checklist

- [x] Contract 1.22 and its version in every place it is written.
- [x] The core: the rule of §3 (4 tests).
- [x] Android: the list's press, and the System service's part; by hand on the emulator.
- [x] The Mac: the list's button, and the System service's part; in the app, in fake mode, the button pressed with nothing opened.
- [ ] iPhone and iPad.
- [x] `Tools/test.sh all` green, stage by stage, and `android-device` (144 on the emulator), on 2026-11-21. The Mac's UI suite, which had not run when this was built (a dialog of the system's was on the Mac's screen), ran with it.

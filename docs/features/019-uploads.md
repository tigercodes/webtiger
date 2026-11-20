# 019 — A page asks for a file (W1)

| | |
|---|---|
| **Status** | Built and tested on the Mac (the WebKit adapter, against real WebKit) and on Android (the System WebView adapter, on an emulator), each with a picker of the tests' own. The system's own picker was run by hand on Android, on the emulator; **the Mac's open panel has not been run** (§7). On an iPhone WebKit shows its own picker. Windows is not started |
| **Phase** | W1 of [000](000-master-plan.md) §9: "uploads" |
| **Contract** | None. Nothing about browsing is decided here: the engine lets a page ask only when the person pressed something on it, and what the page gets is what the person picks in the system's own picker |
| **Builds on** | [001](001-foundations.md): the engine adapters. [014](014-permissions.md): an engine brings nothing of the system's up unless the app itself, started for real, made it so |
| **Last updated** | 2026-11-20 |

## 1. User story

A page has a "Choose File" button: a form that wants a photo, a site I send a document to. I press it, the system's own file picker opens, and I choose a file, or several where the page lets me. The page has what I chose, and nothing else of my files. If I close the picker, the page has nothing.

## 2. What is in it, and what is not

**In:** a page's file input (`<input type="file">`, one file or several) opens the system's picker on a Mac and on Android, and the page is given what was picked.

**Not in:**

- **An iPhone or iPad.** WebKit shows its own menu there (the photo library, the camera, the files), and gives the page what was chosen. Nothing is built, and nothing is in the way.
- **A whole folder** (`webkitdirectory`). The picker is not offered folders: sending every file of a folder is more than a press on a button says.
- **The camera or the microphone as the source** (`capture`), on Android: the picker of documents is shown, as for any file input.
- **Files dropped on a page.** Each engine does what it does today.
- **Naming the site in the picker.** The system's picker is shown for the tab the person just pressed in.
- **The core.** An upload is not in the contract. That a tab with an upload running is kept awake (000 §5.7) is W3's.

## 3. Who does what

The engine notices that the page asks for a file, which it lets a page do only when the person has just pressed something on it. The adapter asks the shell's **file picker**; the picker shows the system's own; the adapter gives the engine what was picked, or nothing.

**The picker is a seam**, as what an engine may bring up of the system's always is (014 D5):

- An engine is made with a picker. The one it has unless it is given another **picks nothing**: the page is told that nothing was chosen, and nothing of the system's is shown. Every engine a test makes has that one, and so has fake mode: a system's file picker is a dialog of the system's, and no test may put one on the screen.
- Only the app itself, started for real, gives its engine the system's picker.

**What the page may ask for** is passed on as the engine says it: one file or several. The kinds of file a page says it accepts are passed to the system's picker where it takes them (Android); they narrow what is offered, and promise nothing about what is picked.

**One at a time.** While a picker is open for a web view, another request of that web view's page is answered with nothing.

**A web view that is destroyed** while its picker is open is told nothing was chosen; what the person then picks goes nowhere.

## 4. The engines

| | WebKit on a Mac | System WebView |
|---|---|---|
| Where it is told | The UI delegate's open panel | `WebChromeClient.onShowFileChooser` |
| Left alone | Nothing happens | Nothing happens |
| What it says | Whether several may be chosen, and whether folders | Whether several may be chosen, the kinds accepted, whether to capture |
| The answer | The files' addresses, or none | The files' addresses, or none |
| The system's picker | An open panel, as a sheet of the window | The system's document picker, started by the window |

**As built on the Mac** (`WebKitFiles.swift`: `FilePicker`, `Files`, the page's UI delegate; `FilePicker.system` in the shell), and as seen against real WebKit:

- WebKit asks the page's delegate when a file input is pressed with the mouse; the adapter asks the engine's picker, saying whether several may be chosen, and gives WebKit the files' addresses. The page then has each file's name and size and can read it.
- **Nothing picked** is told to WebKit as nothing: the page's input says so (its `cancel` event), and the page may ask again.
- An engine as it is made has the picker that picks nothing; the app, started for real and not in fake mode, gives its engine the Mac's open panel, as a sheet of the window in front, with folders never offered.

**As built on Android** (`Files.kt`: `FileChooser`, `Files`, `AppFiles`; `onShowFileChooser`; the window's two launchers), and as seen on the emulator (System WebView 133):

- The WebView says whether several files may be chosen and the kinds the page accepts, as the page wrote them (`text/plain`, `.txt`); the adapter passes both to the chooser, and gives the WebView the addresses that were picked. A file of the app's own, named by its `file:` address, was read by the page.
- **The system's picker** is the system's picker of documents, for one file or for several, started by the app's window. The kinds a page accepts are given to it as media types; an ending is given as the type the system knows it by, and with nothing it can go by every file is offered.
- **By hand, on the emulator**, with the app started for real: a press on "Choose File" brought up the system's picker; a file put in the emulator's Downloads was picked there; the page then said its name, its size and its words. The same with the input for several. The emulator was put back afterwards.

On both, one picker at a time for a web view, and a web view that is destroyed while its picker is open is told nothing; what the person then picks goes nowhere.

## 5. Tests

| Where | What |
|---|---|
| The test site | A page with a file input for one file and one for several (which says it accepts text), which says in its title what it was given: each file's name and size, and the words of the first; or that the picker was closed with nothing chosen |
| WebKit adapter suite (`WebKitFileTests`, 6 on the Mac, 2 on the iPhone simulator) | On the Mac, against real WebKit, the input pressed with the mouse's own events and a picker of the test's that picks files the test wrote: a press asks the picker once, for one file, and the page has the file by its name, its size and its words; several files where several may be chosen; nothing picked, and the page is told so and may ask again; an engine given no picker gives the page nothing. On both, at the seam where a picker can be held open: one at a time for a web view, at once and with no second picker; a web view that is destroyed is given nothing of what is then picked; a page that may have one file is given no more; nothing picked is nothing. Seven of the rules checked by breaking them |
| System WebView adapter, on the JVM (`FilesTest`, 8) | Through the engine, with a chooser that answers when the test says: a page that asks is given what was picked, and the chooser is told whether several may be chosen and the kinds accepted; one file and no more; nothing picked is nothing, and the page may ask again; one chooser at a time for a web view, and an answer is given once; a web view that is destroyed is told nothing, and what is then picked goes nowhere; an engine given no chooser picks nothing; the kinds a page accepts as the system's picker takes them; the app's chooser picks nothing until a window gives it the way to the system's. Nine of the rules checked by breaking them |
| System WebView adapter suite (`SystemWebViewFileTest`, 3, on the emulator) | With a finger on the test site's inputs, against the real WebView: a press asks the chooser once and the page has the file; several files, and the kinds the page accepts as the WebView passes them on; nothing picked, and the page is told so and may ask again |
| The apps, in fake mode | A press on the page's file input shows nothing of the system's, and the page is told that nothing was chosen: on the Mac (no sheet and no dialog) and on the emulator (the app's window is still the one on the screen) |
| By hand | Android's own picker, as §4 says. No test shows a picker of the system's |

## 6. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | No contract, and no rule of the core's | There is nothing to decide that an engine and the person's own choice do not already decide. A rule can be added when there is one to have |
| D2 | An engine picks nothing unless made otherwise | A test that raised a dialog of the system's stopped this Mac's UI suite for hours (007 §8). The seam makes it impossible, not merely not done |
| D3 | No folders | The page gets every file in it; a button that says "Choose File" does not say that |

## 7. What is not verified

- **The Mac's open panel.** No test shows a dialog of the system's, and nobody ran the app for real on this Mac: the dozen lines that show the panel were read, not run. A release build that is sandboxed needs the entitlement for files the person chooses; today's builds have no sandbox.
- **Android's picker**, by hand once, on an emulator: one file picked, in each of the two inputs. Picking several at once, a file from another app's provider (a drive in the cloud), and a phone were not tried.
- **A page in a frame** that asks, and a page that asks without a press: each engine's own rule; not tried.
- **Large files**, and what a page does with a file while it is being sent: the engine's.
- **A picker left open while the tab is closed**, with the system's picker: the rule is tested at the seam; the system's picker was not left open by hand.

## 8. Done checklist

- [x] The test site's page.
- [x] Apple: the Mac's adapter and its suite (6); the system's picker for the app (not run); fake mode shows nothing, checked in the app.
- [x] Android: the adapter, on the JVM (8) and the device (3); the system's picker for the app, run by hand on the emulator; fake mode starts nothing, checked in the app.
- [x] `Tools/test.sh all` green, and `android-device` (130 on the emulator).
- [ ] Windows shell.

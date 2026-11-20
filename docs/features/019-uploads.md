# 019 — A page asks for a file (W1)

| | |
|---|---|
| **Status** | Designed. Nothing is built yet: on a Mac and on Android a page's "Choose File" button does nothing, because neither engine picks a file by itself; on an iPhone WebKit shows its own picker |
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

What each does in fact, as observed, is written here when built.

## 5. Tests

| Where | What |
|---|---|
| The test site | A page with a file input for one file and one for several, which says in its title what it was given: each file's name and size, and the words of the first |
| Adapter suites (WebKit on the Mac; System WebView on the emulator) | With a picker of the test's own that picks a file the test wrote: a press on the input asks the picker once, saying whether several may be chosen; the page then has the file, by its name and with its words; several files where several may be chosen; nothing picked, and the page has nothing and may ask again; an engine given no picker shows nothing and gives nothing; a web view destroyed while its picker is open |
| The shells | The system's pickers are not shown by any test. In fake mode a press on the input shows nothing of the system's, checked in the apps |

## 6. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | No contract, and no rule of the core's | There is nothing to decide that an engine and the person's own choice do not already decide. A rule can be added when there is one to have |
| D2 | An engine picks nothing unless made otherwise | A test that raised a dialog of the system's stopped this Mac's UI suite for hours (007 §8). The seam makes it impossible, not merely not done |
| D3 | No folders | The page gets every file in it; a button that says "Choose File" does not say that |

## 7. What is not verified

- **The system's own pickers.** No test shows one: the Mac's open panel and Android's document picker are each a few lines that were read, and are to be run by hand.
- The rest: written when built.

## 8. Done checklist

- [ ] The test site's page.
- [ ] Apple: the Mac's adapter and its suite; the system's picker for the app; fake mode shows nothing.
- [ ] Android: the adapter, on the JVM and the device; the system's picker for the app; fake mode shows nothing.
- [ ] `Tools/test.sh all` green, and `android-device`.
- [ ] Windows shell.

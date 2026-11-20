# 024 — Several Spaces (W2)

| | |
|---|---|
| **Status** | In progress. The store and the core are built and tested: Spaces are made, named and listed, a window is switched between them, a tab is in the Space it was opened in, and each Space's tabs come back (§3 to §5). The Apple and Android shells have the chip, the list and the page that makes a Space (§6): tested in the apps on the iPhone simulator against real WebKit and on the Android emulator against the real System WebView, where two Spaces' logins are apart and a shared one's are shared; the Mac's app has not been run (§9). No shell names a Space yet |
| **Phase** | W2 of [000](000-master-plan.md) §9: "Spaces with their login sets … shared logins". The first of W2: the model has several Spaces, and a window shows one of them. Colours and icons, the tree of Groups and Sites, Favourites, both layouts, several windows and private windows are W2's later features |
| **Contract** | 1.18: `QuerySpaces`, `SpacesPage`, `SpaceSummary`, `CreateSpace`, `SpaceCreated`, `RenameSpace`, `SwitchSpace`; `WindowSnapshot.space_id` and `space_name` |
| **Builds on** | [001](001-foundations.md): the store's `space` and `login_set`, many Spaces to one login set from the first schema (000 WT-D9). [005](005-tabs.md): a window's tabs. [009](009-session-restore.md): the tabs come back |
| **Last updated** | 2026-11-20 |

## 1. User story

I keep work and home apart. I make a Space and call it "Work". It has its own logins: signed in to the same site, I am somebody else there than at home. Or, for a second project at the same job, I make a Space that shares Work's logins.

My window shows one Space at a time: its tabs. I switch the window to the other Space and see that one's tabs; when I come back, the tabs I left are as I left them, and the one that was in front is in front.

## 2. What is in it, and what is not

**In:** the core knows several Spaces, in an order; a Space is made with a name, and with logins of its own or another Space's; it is renamed; a window has the Space it shows, and is switched to another; a tab belongs to the Space it was opened in, and its pages run in that Space's logins; each Space's tabs come back when the app starts again, and the window shows the Space it showed. The shells get a way to switch, and to make a Space.

**Not in** (each its own feature, later in W2):

- **A Space's colour and icon**, and its defaults (engine, Guard, search, tidying).
- **Deleting a Space**, with what becomes of its tabs and its logins; **clearing a Space's data**.
- **Moving a tab to another Space.** With other logins it is another page.
- **Groups, Sites, Favourites, temporary tabs**: the tree. Until then a Space's tabs are a row, as now.
- **Several windows**, each with its own Space; **private windows**.
- **Putting the Spaces in another order.**

## 3. The rule

**A Space** has a name, a place in the order, and a login set. The Space the app made itself has no name of its own: a shell shows its own word for it ("Home"), until the person names it.

**Making one** (`CreateSpace`): with a name, and either logins of its own (a new login set, which is the default) or the logins of a Space there is. It is put after the others. A name is what is left of what was typed when the white space at its ends is gone and every run of it inside is one space (a name is one line, with nothing in it that is not shown); it may not be empty, and is cut at 60 characters. Two Spaces may have the same name. There may be 32 Spaces: one more is refused (`space.too_many`). A Space to share logins with that is not there is refused (`space.unknown`), and nothing is made.

**Sharing is chosen when a Space is made**, and not changed after: changing a Space's logins is moving everything in it to other logins, which is not this feature.

**Renaming** (`RenameSpace`) gives a Space a name, by the same rule for names. The app's own Space can be named like any other.

**A window shows one Space.** It starts with the Space it showed when the app last ran, or the first. Its snapshot says which (`space_id`) and what it is called (`space_name`), and its tabs are that Space's tabs.

**Switching** (`SwitchSpace`) changes the Space a window shows:

- the tabs of the Space that is left stay as they are: their pages go on, and nothing is asked of their web views. (Putting them to sleep is W3.) Which of them was in front is remembered;
- the window then shows the tabs of the other Space, with the one in front that was in front when it was last shown. A Space with no tabs is given one empty tab, as a new window is;
- switching to the Space the window shows changes nothing. A Space that is not there is refused (`space.unknown`).

**A tab belongs to the Space it was opened in**: the window's Space when the person opened it, and the asking tab's when a page or a menu opened it (a link in a new tab, a window a page asked for, a search for selected words). Its web view is made in that Space's login set, as every web view is made in a login set already.

**What is kept per login set stays so**: history, downloads, what sites were allowed. The Library shows those of the login set of the Space the window shows; two Spaces that share logins share them.

**The tabs come back per Space** (009): each Space's tabs are saved as a window's are now, and given back when the Space is first shown again. Only the Space a window shows has its tabs made when the app starts: the others' are made when they are switched to, and a Space that is not shown in a run keeps what was kept of it. A tab that came back is loaded when it comes in front of what a window shows: one that comes in front of a Space nobody is looking at sleeps on until that Space is shown.

**More than one window.** The core has had several windows since W0, and one of them has the session (009). Until several windows are a feature of their own: a window that is opened shows the Space the window with the session shows, or showed; each window has its own tabs of a Space, as each has its own tabs now; and the session is that one window's, every Space of it.

**What waits for a tab's answer waits with it.** A prompt of a tab in a Space that is not shown is still that tab's, and is shown when the tab is (014).

## 4. The contract (1.18)

Added; nothing is changed or removed.

```proto
// browser.proto
message QuerySpaces {}                      // answered with SpacesPage

message SpacesPage {
  repeated SpaceSummary spaces = 1;         // in their order
}

message SpaceSummary {
  uint64 space_id = 1;
  string name = 2;                          // redacted. Empty: the app's own Space, unnamed
  // Spaces with the same number here share their logins. It is a number of
  // the core's own for this run, and says nothing else of the login set.
  uint32 logins = 3;
}

message CreateSpace {                       // answered with SpaceCreated, or an error
  string name = 1;                          // redacted
  uint64 share_logins_with = 2;             // a Space's id; 0 for logins of its own
}

message SpaceCreated {
  uint64 space_id = 1;
}

message RenameSpace {                       // answered with Ack, or an error
  uint64 space_id = 1;
  string name = 2;                          // redacted
}

message SwitchSpace {                       // answered with Ack, "window.unknown" or "space.unknown"
  uint64 window_id = 1;
  uint64 space_id = 2;
}

message WindowSnapshot {
  ...
  uint64 space_id = 5;                      // the Space the window shows
  string space_name = 6;                    // redacted. Its name; empty as above
}
```

Errors: `space.unknown`, `space.too_many`, `space.name_empty`; and `space.random`, when the system has no random bytes for a new login set, which nobody has seen happen.

`SpaceSummary.logins` is the place in the list of the first Space with those logins: a number of that answer's.

A window's snapshot is published again when its Space changes, and when the Space it shows is named: a shell draws the name from the snapshot. A shell that shows the list of Spaces asks for it when it needs it, and again after it has made or renamed one.

**Fake mode** has the Spaces a run makes and no others, as it has no session. A Space made in it with logins of its own gets the same login set in every run.

## 5. The store

The first schema has what this needs: `space (id, name, position, login_set_id)` and `login_set`. New here: listing the Spaces in order; making one, with a new login set or another's; renaming one; and, with the session (009), which Space the window showed. That last is one of the app's settings (020's table), and is read only while it names a Space there is. No migration.

## 6. The shells

- **A way to switch**: where the layout has it (000 §6.1). With the tab strip, which is the layout there is: a chip at the leading end of the strip that names the Space the window shows, and opens the list of Spaces. On a phone, the same chip.
- **The list**: the Spaces in their order, the one shown marked; a press switches. At its end, "New Space…".
- **Making one**: its name, and a choice between logins of its own and those of a Space there is, which says what that means in a sentence.

### 6.1 The Apple shell (the Mac and the iPhone)

- **The chip** stands before the tabs and does not scroll with them. It says the Space's name, or "Home" for the Space the app made while nobody has named it, cut to one line; to a screen reader it is "Space: ‹name›", a button.
- **In a window as narrow as a phone's** (a compact width: an iPhone, an iPad's narrow split) the chip is the Space's initial, a capital, and no more: the strip is one row there, and it was full. With the name in it a second tab was pushed out of reach, and the button for a new tab out of sight beside one ordinary tab: Android's pictures showed it, and two of its app's tests failed on it. A screen reader is still told the name, and the list shows it. When Spaces have their colours and icons, this is where the icon goes; and a phone's own layout (000 §6.4) has no strip at all.
- **The list** is a sheet, as History and the list of search engines are: a heading, the Spaces in their order as rows to press, the one the window shows marked with words ("Shown"), not by colour alone; under a row whose logins are another Space's too, "Shares logins with ‹the first other Space that has them›". Pressing a row switches and closes the sheet. Then "New Space…" and "Done".
- **Making one** is the same sheet's second page: a field for the name, which has the keyboard; "Logins" with two choices, each a sentence. "Its own: signed out everywhere to begin with. What you sign in to in this Space stays in it." "The same as another Space: signed in wherever ‹that Space› is. The two share history and downloads." With the second, a choice of which Space, among the first of each that has logins of its own; it starts at the one whose logins the Space the window shows has. "Make Space" is not to be pressed while the name is empty; "Back" goes to the list. A Space that is made is the one the window then shows. What the core refuses is said in words above the buttons, and the page stays.
- The shell decides nothing: the list is the core's answer to `QuerySpaces`, asked for each time the sheet opens, and a name goes to the core as it was typed.
- "Make Space" is a plain bordered button whose words are heavier, not the system's prominent one: white on the accent's blue is too faint at that size, by Apple's own audit on the iPhone.

### 6.2 The Android shell

The same chip, list and page, in Compose. The chip is the Space's initial in a window narrower than 600 dp, by the window's size and not by the device (000 §6.5). The list is over the window, as the list of search engines is, and the system's Back leaves the page that makes a Space for the list, and the list for the window. The two choices of logins are radio buttons, and so is the choice of which Space to share with, listed under the second when there is more than one.

**The client names the answers it knows.** Android's `NativeCore` took an answer of a kind it did not name for no answer at all, so the list of Spaces came back empty on a device, where the tests drawn on the JVM, against a scripted core, could not see it. The two new answers are named, the client's own test asks the real core for both, and a test now reads the contract's kinds of answer from the generated type and fails for one the client does not name: it is the third time a new answer was lost this way, and the first time a test would have said so before a device did.

## 7. Tests

| Where | What |
|---|---|
| Store (`wt-store/src/space.rs`, 6) | Spaces are listed in the order they were made; one is made after the others, with logins of its own and nobody else's, or with those of a Space there is, of the app's own Space, or of one that shares already; the logins of a Space that is not there make nothing; a Space is renamed, the app's own among them, and keeps its logins and its place, and one that is not there is not; all of it is still there when the store is opened again, and the first Space is still the app's own. The Space the window shows is kept, the last one written, over a restart; and one that is no Space, or no number, is none. Five of the rules checked by breaking them |
| The rule for a name (`wt-core/src/space.rs`, 2) | What was typed, tidied: no white space at its ends, one space for each run inside, nothing that is not shown; nothing left is no name; cut at 60 characters, by characters, with no space left at the cut |
| Core (`wt-app/tests/spaces.rs`, 20, through `FakeShell`) | A Space is made after the others and listed with its name as tidied, and the list says which share logins: with a Space, with the app's own, with one that shares already. A name is cut at 60. An empty name and the logins of a Space that is not there are refused, and so is one Space more than 32, with logins of its own or another's; nothing is made. A Space is named, the app's own among them, and the window that shows it says so; a window that shows another is told nothing; no name and no such Space are refused. A window is switched: a Space with no tabs has one empty tab; back again the snapshot is what it was, tab for tab, with the same one in front; and nothing at all was asked of the shell meanwhile. The Space that is shown changes nothing; a Space or a window that is not there is refused. A tab is in the Space it was opened in and in no other, and its web view is made in that Space's login set: of its own, or the one it shares; fake mode's are the same every run. A link's tab, a page's window and a search, asked for by a page whose Space is not shown, open in that page's Space and its logins, and nothing of them is in the Space that is shown. A page that closes its own tab there closes only that. What a tab asks waits with it, and is there when its Space is shown. The history a window lists is that of the logins of the Space it shows; two Spaces that share logins share it. A download that is coming shows in the windows that show its logins. Closing a window closes the tabs of every Space it showed. Another window opens on the first Space with tabs of its own. Over a store on disk: each Space's tabs come back, in order, with the one in front that was, and the window shows the Space it showed; only the tab in front of that Space is loaded, and the other Space's tabs are made and its front tab loaded when it is shown, in its own logins. A Space that is not shown in a run keeps the tabs it had. What changed in a Space while it was not shown is kept too. A window opened once the first is closed shows the Space that one showed, with the session. A tab that comes in front of a Space nobody looks at sleeps on, and is loaded when the Space is shown. The Spaces, their names and their logins are there when the app starts again. Nineteen of the rules checked by breaking them: seventeen were caught, three of them only by tests written after they were not; two changed nothing that can be seen (the dispatcher sends no snapshot that is the same as the last; the store refuses logins of a Space that is not there by itself) |
| The Apple shell's model (`SpacesModelTests`, 7, against a scripted core) | The window says which Space it shows, and the one nobody named has the shell's word, "Home"; a name is shown as it is. Where there is no room for a name a Space is its initial: a capital, and a picture is one letter. The list is the core's answer, asked for each time it is opened, and nothing is asked before. A Space that is pressed is asked of the core, and the model shows nothing by itself. A Space is made with its name as it was typed and logins of its own, or another's, and is then shown. One the core will not make says why, shows nothing, and says nothing of it once made after all, or once the list is opened anew. A refusal is said in words, each code's own, and what has none is said in the words for "it could not be made": never the code. Seven of the rules checked by breaking them |
| The Apple shell's pictures (004) | The list, the page that makes a Space with logins of its own and with another's, and a Space the core would not make: on the Mac and the iPhone, light and dark (16). Every picture of the window was recorded again with the chip in it (38 on each system), and each was looked at: the chip stands before the tabs, and nothing else moved. The iPhone's were recorded once more with the initial in the chip: the Mac's, which is never narrow, did not change |
| The Apple apps (`SpaceTests`, 1, fake mode: the real core and real WebKit) | A site signs somebody in, in the first Space, with a cookie for the session only. The chip says "Home"; the list has the one Space, marked as shown; nothing is made of no name. "Work" is made: the window shows it, with one empty tab, and the same site says nobody is signed in. Back in the first Space its tab is as it was left, and it is signed in as it was. "Project" is made to share the first Space's logins: it is signed in as the first is. The list says who shares; and Work's tab is as it was left, signed in as nobody. Apple's accessibility audit passes over the list and over the page that makes a Space. **On the iPhone simulator. On the Mac it is written and has not run** (§9) |
| The Android shell's window (`SpacesTest`, 10, drawn on the JVM against a scripted core) | The chip says "Space: " and the name of the Space the core says the window shows, "Home" for the one nobody named, and nothing is asked until it is pressed. In a window of a phone's width it shows the Space's initial, in one of a tablet's its name, and a screen reader is told the name in both; an initial is a capital, and a picture is one letter. It opens the list, which is the core's answer: each Space by its name in the core's order, who shares logins and with whom, and one marked "Shown"; "Done" closes it with nothing asked; opened again it is asked for again. A Space that is pressed is asked of the core and the list goes; the chip goes on saying what the core says. Nothing is made of no name or of white space; a Space is made with its name as typed and logins of its own, then shown. One is made to share the logins of the Space shown, named by the first Space that has them, among the Spaces that have their own; or another's. One the core will not make says why in words, the page stays, and nothing is said of it once the list is opened anew. Each code has its words, and what has none is "it could not be made": never the code |
| The Android client (`NativeCoreTest`, 2 more, against the real core's library) | The list of Spaces, a Space that is made, and a window switched to it each come back as what they are. Every kind of answer the contract has is one the client names; an error is not taken for an answer. Checked by taking one off the client's list: both fail |
| The Android shell's pictures (004) | The list, the page that makes a Space with logins of its own and with another's, and a Space the core would not make, light and dark (8). Every picture of the window was recorded again with the chip in it (44), twice: first with the name, where the button for a new tab was out of sight beside one tab, then with the initial, where it is back. And a window as wide as a tablet's, with the name in the chip, light and dark (2). Each was looked at |
| The Android app (`AppSpacesTest`, 1, on the emulator: the real core and the real System WebView) | What the Apple apps' test shows: the first Space signed in for the session; "Work" made, shown with one empty tab, signed in as nobody; the first Space's tab as it was left, signed in as before; "Project" made to share the first Space's logins, signed in as it is; the list says who shares; Work as it was left. Android's accessibility checks pass over the list and the page that makes a Space |

## 8. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | A window shows one Space; a tab is in one Space | 000 §5.6. What is open in "Work" is not in the way at home |
| D2 | Logins of its own by default; sharing is chosen at making, and stays | 000 WT-D9. Keeping apart is what a Space is for; sharing is the exception, said once |
| D3 | The tabs of a Space that is not shown stay awake, for now | Sleeping is W3, with its budgets. Until then nothing a person left is lost or reloaded |
| D4 | Only the shown Space's tabs are made at start | A start that makes every Space's web views is slower for each Space a person has |
| D5 | 32 Spaces, and 60 characters to a name | Limits a person does not meet, that a list and a chip can still show |
| D6 | The window's snapshot carries the name of the Space it shows | The chip that names it is drawn from the snapshot, as everything in a window is: a shell does not keep a list of its own to look the name up in |
| D7 | Which Space the window showed is one of the app's settings, not a column | There is one window with a session. When several windows are kept, each will need its own row, and that is the migration to make then |

## 9. What is not verified

- **The Mac's app.** Its UI suite has not run with any of this: a dialog of the system's, about a simulator's daemon and none of WebTiger's, is on this Mac's screen, and the suite does not start while one is (001). The same test passes on the iPhone simulator, and the Mac's pictures and model tests pass.
- **An iPhone, and an iPad.** The simulator is not a device; and no picture or test here is of an iPad's width.
- **An Android phone.** The emulator (Android 16, WebView 133) is not a phone, and one version of the WebView is not the others.
- **A phone's width, beyond the pictures.** With the initial in the chip the strip holds what it held before less the chip's own width, about a button's: a tab with a long name still pushes the new-tab button along the strip, as it did. No test measures how many tabs fit; the two of Android's app that open a second tab and go back to the first pass again. Whether an initial is enough to tell two Spaces apart at a glance is not something a test can say: "Work" and "Web" have the same one, until Spaces have colours and icons.
- **Naming a Space, in a shell.** The core names one (`RenameSpace`), and no shell offers it yet.
- **Windows.** Not started.
- **Many Spaces, many tabs.** The session is worked out for every Space a window has shown each time the snapshots are published. With a few Spaces that is what it was; nothing has measured it with thirty-two full ones.
- **A store that fails while a Space is made or named** is answered with the store's code; no test makes the store fail there.

## 10. Done checklist

- [x] The store: listing, making, renaming (5 tests).
- [x] The store: the window's Space with the session (1 test).
- [x] Contract 1.18 and its version in every place it is written.
- [x] The core: the rule of §3 (20 tests, and 2 for a name).
- [x] iPhone: the chip, the list, making a Space; model tests, pictures, and the app's test on the simulator.
- [ ] The Mac: the same code, model tests and pictures; **the app's UI suite has not run**.
- [x] Android: the chip, the list, making a Space; the window's tests, pictures, and the app's test on the emulator.
- [ ] Naming a Space, in the shells.
- [ ] `Tools/test.sh all` green, and `android-device`.
- [ ] Windows shell.

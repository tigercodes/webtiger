# 048 — The command bar: Spaces and commands in the address field (W2)

| | |
|---|---|
| **Status** | **Built and tested in the core, the Apple shell and the Android shell, on the recommendations of §6, which are provisional:** they are put to the owner and not yet answered. Typed in the address field, a word of a Space's name or of a command's words brings a row for it; chosen, the Space is shown or the command done |
| **Phase** | W2 of [000](000-master-plan.md) §9: "the command bar". 000 §6.2: "One field for addresses, searches, open tabs, Sites, Spaces, history, settings and plugin commands. Everything in the app is reachable by typing" |
| **Contract** | 1.40 (§5). Nothing is kept |
| **Builds on** | [011](011-suggestions.md): what is offered for what is typed, from history. [038](038-an-open-tab-is-offered.md): the open tabs of the row among them, Sites that sleep and Sites of shut Groups too. [024](024-several-spaces.md): Spaces. [034](034-the-macs-menus.md): the window's commands, which a menu and a key reach |
| **Last updated** | 2026-11-23 |

## 1. User story

I type in the one field at the top of the window. Part of a page's name brings me a tab or a place I have been; "work" brings me my Work Space; "side" brings me "Tabs at the Side". I do not look for a menu.

## 2. What is in it, and what is not

**Built already, by other features:** addresses and searches (the field itself); open tabs and Sites (038); history (011).

**In:** two more kinds of thing in the same list: **a Space,** by its name, which chosen is shown in the window; **a command** of the window, by its words, which chosen is done.

**Not in, each with where it goes:**

- **Settings:** there are no settings yet to reach.
- **Plugins' commands** (000 §5.10): with plugins.
- **A field of its own over the page,** as Arc's is (D1).
- **Commands that need more than a choice:** "Move to Group" and a Group's name. They open what they open now.

## 3. The rule

**The address field is the command bar.** What is typed is asked of the core, as it is now; the answer has tabs, Spaces, commands and places, in that order, and no more rows than it had.

**A Space is offered** when its name has every word typed, but not the Space the window shows. Chosen, the window shows it (as from the list of Spaces). A private window is offered none: it has no Space (037).

**A command is offered** when its words have every word typed. **The words are the shell's, and the matching is the core's** (D2): with each question a shell sends the commands that can be done now, each by a name of the core's kind (`layout.side`) and its words in the person's language; the core says which match, by name. Chosen, the shell does what its menu's item does. A command that can do nothing now (it is grey in the menu) is not sent, and so not offered (D4).

**Typed text is never done as a command.** Return in the field goes to an address, or searches, as it does now: it is the field's own rule that what is typed is never executed. A command is done only by choosing its row: a press, or the arrow keys and Return on the row.

**Nothing typed is kept, and none of it leaves the device:** as for every other row of the list.

## 4. Where it is kept

Nowhere.

## 5. The contract (1.40)

Added; nothing is changed or removed.

```proto
message SuggestAddresses {
  …
  // The commands that can be done now, for the core to match with what is typed.
  repeated OfferedCommand commands = 4;
  // The shell shows a Space among what is offered.
  bool spaces = 5;
}
message OfferedCommand {
  string name = 1;     // the shell's own name for it, given back when it matches: "layout.side"
  string words = 2;    // as its menu's item says it, in the person's language: the app's words, not the person's
}

message AddressSuggestion {
  …
  uint64 space_id = 5;   // not 0: a Space, by `title` its name. Choosing it shows it (SwitchSpace)
  string command = 6;    // not empty: a command, by `title` its words. Choosing it does it, in the shell
}
```

**As built, where the design did not say:**

- **Only a shell that asks is offered either.** A shell that sends no commands is offered none, and one that does not say it shows Spaces is offered none: a shell from before 1.40 is answered as it was, row for row. (The design had Spaces offered to every shell: one that did not know of them would have drawn a Space as a place with no address.)
- **The Space the app made itself is not offered.** It has no name of its own: its word is the shell's, and the core has none to match. A shell that wants it found sends it as a command.
- **A command's name is of the kind the core's codes are** (`layout.side`): one that is not, and one with no words, are passed over, so that nothing a shell sends by mistake is handed back as a command. At most 64 are read from a question.
- **The words are matched as a page's are** (038): every word typed is a word of them or the start of one, whatever its case. Not the middle of a word; and not a command's name, which is no word of its.
- **The rows** (D5): the open tabs have their part of the list, as they had: three, or as many as the shell asked for when that is fewer. Spaces and commands have what the tabs left, less one row where there is a place to offer: **history is never crowded out whole by them**. Where there is not room for two Spaces and three commands, each kind has a row before either has a second. In the six rows a shell asks for by default, with three tabs open that match: a Space, a command, and a place.

**The shells, as built** (D1, D6):

- **The commands a window sends** are those its menus do in one press, each only while it can be done: a new tab; on a Mac a new private window; the closed tab brought back; the tab in front closed, kept or let go, made a Favourite or stopped being one, sent home; the tabs at the side or along the top, where there is room; find, zoom, and the lists of what was visited; the tab moved a place, each way it can go; the next tab and the one before. By the words their menus have, in the person's language: "Keep as Site" while the tab is none, "Stop Keeping" once it is.
- **A row says what it is at its end:** "Space", "Command", as an open tab's says "Switch to Tab". It has a name and no address under it.
- **A command is done as it was when it was offered:** the window keeps what it sent with its last question. A command the window did not send, were the core to offer one, is done by nobody.
- **Choosing a row gives the field back what it showed:** what was typed to find the row is not kept there.
- **On a Mac the last tab's "Close Tab" closes the window,** as its menu's does (034).
- **A window that is not drawn sends no commands** (Android): it has no words to send. **A private window sends its commands and asks for no Spaces.**
- **Nothing is typed first, and no key opens it:** it is the address field, which has its key already.

## 6. Decisions

**All of these are put to the owner,** and built as recommended. The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | **The address field is the command bar:** one field, and no second one over the page | The plan says one field. Its list is built, with tabs and history in it; a second field would be a second list to keep like the first | A field of its own in the middle of the window, opened by a key, as Arc's is: more room for rows, and two places to type |
| D2 | A command's words are the shell's, its matching the core's: the shell sends the commands with the question | The words are in each system's catalogue (040) and the core has none; the matching is the list's, which is the core's, and is then the same on every system | The core lists commands by name and the shell finds the words: two things that match typed text. Or the shell matches alone: two orders in one list |
| D3 | A command is offered among the other rows whenever its words match; no mark is typed first | "Everything in the app is reachable by typing" | Commands only after a mark, as editors have ">": a thing to learn before the first one is found |
| D4 | Only what can be done now is offered | A row that does nothing when it is chosen is worse than no row | Every command, the ones that can do nothing drawn grey |
| D5 | The order: tabs, Spaces, commands, places; at most three tabs (as now), two Spaces and three commands, in a list no longer than it was | Where a person is going comes before what they might do; history fills what is left, and is never crowded out whole | Commands first |
| D6 | Which commands: those of the window that need nothing more than to be chosen (new tab, new private window, bring back a closed tab, close, keep or let go, Favourites, return home, the layout, find, zoom, History, move a tab, the next and the previous tab) | They are the ones a menu does in one press | Every item of every menu |

## 7. In what order

1. **The core:** Spaces and commands matched and ordered, against the scripted engine. **Built.**
2. **The shells:** the commands sent; a Space's row and a command's row; choosing them. **Built.**

## 8. Tests

| Where | What |
|---|---|
| The core, through the contract (`command_bar.rs`, 3, against the scripted engine) | **A Space:** offered by a word of its name or the start of one, every word typed in any order and any case, in the Spaces' order, two at most; with its name and its number and nothing else of a row. Not by the middle of a word, a word it has not, or nothing typed. Not to a shell that does not say it shows Spaces. Never the Space the window shows; choosing one shows it. The Space the app made itself is found by nothing. A private window is offered none; a window there is not is refused as it was. **A command:** offered by the shell's words for it, in the shell's order, three at most, as its name and its words. Not by its name, the middle of a word, or nothing typed; none to a shell that sends none. A name that is no code, and a command with no words, are passed over; the sixty-fifth is not read. **The rows:** tabs, then Spaces, then commands, then places. In six rows: three tabs, a Space, a command, a place. With room for all: three tabs, two Spaces, three commands and the places. A row short: a place keeps its row and each kind has one before either has a second. The tabs keep their part: in four rows the fourth is a place's, in two there are two tabs. Only commands, or only Spaces, have the rows the other would have had. With no place to offer none has a row kept. **A shell that asks for neither** is answered as it was |
| The tests there were | The 634 the core had pass unchanged but for seven places where a question is written out whole, which now name the two new fields' defaults. One of them, 038's, held the first way of keeping a row for history to account: it took a row from the tabs, in every shell's list |
| The rule broken, a part at a time | 25 parts, in the model and where the question is answered: 25 caught |
| The Apple shell's model (`CommandBarModelTests`, 6, against a scripted core) | **What a question carries:** for an empty tab alone, a new tab and closing it, by their menus' words, and that the window shows Spaces; for a page, keeping, a Favourite and zoom too; with what only the window can do, and room for a sidebar, a private window, the layout, find and the lists, each where its menu has it, and nothing is done by being offered; with an empty tab again, nothing to find in, keep or zoom; for a Site that is a Favourite and away, with a closed tab and a tab beside it, each command says what it would do now ("Stop Keeping", "Remove from Favourites", "Return to Site", "Tabs at the Top"), and a tile moves right, or left where the row runs from right to left. A private window asks for no Spaces and offers no Favourite; a window that is not open has no commands. **Choosing:** a Space's row asks the core to show it, ends the typing and puts the list away; a command's row does what its menu's item does (a tab opened, the zoom bar, find with the keyboard asked of the window, the lists); a command the window did not send asks nothing; the command done is the one that was offered. What a row says it is. On a Mac the last tab takes the window with it, a tab with another to show does not, and a sleeping Site is no tab to be left with. Broken in 38 parts, 38 caught: 34 at once, one by an assertion added for it, three once they were written so that they built |
| The apps (`CommandBarTests`, 2, on the Mac and the iPhone simulator, with the real core and the real engine) | **A command:** "keep" brings "Keep as Site", a row that says it is a command, and Apple's audit faults nothing; chosen, the tab is a Site, the field has the page's address again and the list has gone; the same word then brings "Stop Keeping"; "new t" opens a tab; on the Mac "side" puts the tabs down the side and "tabs to" brings them back. **A Space:** in a Space named Work its own name brings no Space; from the first Space "wor" brings Work, a row that says it is a Space, and chosen it is shown. Nothing was loaded by any of it |
| Apple's pictures | The state with places offered has a Space's row and a command's row: four pictures recorded again, two on each system, and looked at |
| The Android shell on the JVM (`CommandBarTest`, 7) | As Apple's model, with the window drawn so that it has its words: what a question carries in each state; a window with room for a sidebar offers the layout by where its tabs are, a row of the sidebar moves down and a tile right; where the row runs from right to left, towards its end is left; a private window asks for no Spaces, and a window that is not drawn sends no commands. Drawn: a Space's row and a command's are a name and what they are, and nothing else; chosen, the Space is asked of the core and the list goes; a command is done; one nobody sent asks nothing; find and zoom open their bars, and the lists open over the window. Broken in 39 parts, 39 caught |
| Android's pictures | The same state: two pictures taken away, made again and looked at |
| The Android app on the emulator (`AppCommandBarTest`, 1, the real core and the real WebView) | "keep" brings "Keep as Site, Command"; chosen, the tab is a Site and the list has gone; then "Stop Keeping"; "new t" opens a tab. Nothing was loaded |

## 9. What is not verified

- **The choices of §6** are the recommendations, not the owner's word.
- **A Space's row on the emulator:** not chosen there. The device suite's tests share one core, and a Space made by one of them is there for all that come after; it is chosen on the JVM, against a scripted core, and in Apple's apps.
- **A command chosen by the keyboard** (the arrow keys and Return on its row): by the same lines as a place's row, which is tested (011); no test marked a command's row with a key.
- **Find, chosen from the list:** that the keyboard is then in the find bar is asked of the window, and tested as asked; no test of the app looked at where the keyboard was.
- **A private window on a Mac, opened from the list;** the commands that move a tab; "Return to Site": each is sent and done by the model's tests, and not chosen in an app.
- **An iPad, a real phone, a screen reader.** Apple's audit ran over a list with a command's row and with a Space's.
- **Windows:** the two new strings are in its resources, which were never built on a PC.
- **Words in another language.** The matching is the one a page's title has (038): by letters and digits, in lower case. A language that does not part its words with spaces is matched by the start of what was typed only.
- **A long list of commands.** Sixty-four are read at each letter typed: not measured.
- **Two commands a shell gives one name:** both are offered, as sent.

## 10. Done checklist

- [ ] The owner's word on §6.
- [x] The core and the contract.
- [x] The shells.
- [x] The core: `Tools/test.sh all` green, and `android-device`, on 2026-11-23: `lint`, `core` (637, and the strings tool's 24), `unit` (325 on the Mac), `build`, the Mac's UI suite (54), `ios` (307 on the iPhone simulator), the iPhone's UI suite (41), `android` (437 and 32 on the JVM) and `android-device` (167 on the emulator).
- [x] The shells: `Tools/test.sh all` green, and `android-device`, on 2026-11-23: `lint`, `core` (637, and the strings tool's 24), `unit` (331 on the Mac), `build`, the Mac's UI suite (56), `ios` (313 on the iPhone simulator), the iPhone's UI suite (43), `android` (444 and 32 on the JVM) and `android-device` (168 on the emulator).

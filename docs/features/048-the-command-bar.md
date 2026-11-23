# 048 — The command bar: Spaces and commands in the address field (W2)

| | |
|---|---|
| **Status** | **Designed; nothing built.** The choices of §6 are put to the owner |
| **Phase** | W2 of [000](000-master-plan.md) §9: "the command bar". 000 §6.2: "One field for addresses, searches, open tabs, Sites, Spaces, history, settings and plugin commands. Everything in the app is reachable by typing" |
| **Contract** | To be 1.40 (§5). Nothing is kept |
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

## 5. The contract (to be 1.40)

Added; nothing is changed or removed.

```proto
message SuggestAddresses {
  …
  // The commands that can be done now, for the core to match with what is typed.
  repeated OfferedCommand commands = 4;
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

## 6. Decisions

**All of these are put to the owner; none is built.** The first column is the recommendation.

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | **The address field is the command bar:** one field, and no second one over the page | The plan says one field. Its list is built, with tabs and history in it; a second field would be a second list to keep like the first | A field of its own in the middle of the window, opened by a key, as Arc's is: more room for rows, and two places to type |
| D2 | A command's words are the shell's, its matching the core's: the shell sends the commands with the question | The words are in each system's catalogue (040) and the core has none; the matching is the list's, which is the core's, and is then the same on every system | The core lists commands by name and the shell finds the words: two things that match typed text. Or the shell matches alone: two orders in one list |
| D3 | A command is offered among the other rows whenever its words match; no mark is typed first | "Everything in the app is reachable by typing" | Commands only after a mark, as editors have ">": a thing to learn before the first one is found |
| D4 | Only what can be done now is offered | A row that does nothing when it is chosen is worse than no row | Every command, the ones that can do nothing drawn grey |
| D5 | The order: tabs, Spaces, commands, places; at most three tabs (as now), two Spaces and three commands, in a list no longer than it was | Where a person is going comes before what they might do; history fills what is left, and is never crowded out whole | Commands first |
| D6 | Which commands: those of the window that need nothing more than to be chosen (new tab, new private window, bring back a closed tab, close, keep or let go, Favourites, return home, the layout, find, zoom, History, move a tab, the next and the previous tab) | They are the ones a menu does in one press | Every item of every menu |

## 7. In what order

1. **The core:** Spaces and commands matched and ordered, against the scripted engine.
2. **The shells:** the commands sent; a Space's row and a command's row; choosing them.

## 8. Tests

To be filled in as built.

## 9. What is not verified

To be filled in as built.

## 10. Done checklist

- [ ] The owner's word on §6.
- [ ] The core and the contract.
- [ ] The shells.
- [ ] `Tools/test.sh all` green, and `android-device`.

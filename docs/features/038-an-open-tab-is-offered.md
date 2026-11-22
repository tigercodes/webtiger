# 038 — An open tab is offered while typing (W2)

| | |
|---|---|
| **Status** | Built and tested in the core, on the Mac, the iPhone simulator and an Android emulator: open tabs of the row the window shows are offered first while typing, and choosing one brings it in front. The choices of §6 are provisional: the owner's to confirm |
| **Phase** | W2 of [000](000-master-plan.md) §9: "the command bar", its first part: open tabs in what the address field offers |
| **Contract** | 1.27: `AddressSuggestion.tab_id` |
| **Builds on** | [011](011-suggestions.md): what is offered while typing. [005](005-tabs.md): a tab brought in front. [024](024-several-spaces.md): the row a window shows |
| **Last updated** | 2026-11-22 |

## 1. User story

I type a few letters of a page I want, and it is already open in another tab. The list under the address field says so, first: "Switch to Tab". I choose it and that tab comes in front, as it was; the page is not loaded a second time in the tab I was typing in.

## 2. What is in it, and what is not

**In:** the tabs of the row the window shows are offered while typing, before the places of history; choosing one brings it in front.

**Not in:**

- **The rest of the command bar** (000 §6): Sites, Spaces, commands, settings in the same list. Each comes with what it lists.
- **Tabs of a Space the window does not show, and of other windows.** Choosing one would change the Space, or the window: with the command bar's Spaces (D3).
- **Closing the empty tab that was typed in**, when the tab chosen was typed for in a tab opened only to go somewhere (D5).
- **A key for it** (Chrome's and Firefox's "% " to search tabs only).

## 3. The rule

The core is asked what to offer, as before (`SuggestAddresses`, 011), and answers with open tabs first, then places:

- **Which tabs:** those of the row the window shows (the Space it shows, 024; a private window's own row, [037](037-private-windows.md)), but not the tab in front, and only those that show a page (an empty tab, and one whose first page has not arrived, have nothing to match).
- **What matches:** as for a place (011 §3): every word typed is found in the tab's title or its address, as a word or the start of one, whatever its case. A word is letters and digits. (One difference from history's index: an accent is not looked past. "cafe" finds a visited "Café" and not an open tab called so.)
- **The order:** tabs first, in the order of the row; at most three (D2). Then places, as 011 orders them.
- **A place that an offered tab shows is not offered again** (by its address): the tab stands for it.
- **How many:** the same number in all as before (six, unless the shell asks): tabs take up to three of them.
- Nothing of this is read from history, or written anywhere: a tab is what the window has open. A private window, which is offered nothing from history, is offered its own open tabs.

**Choosing a tab's row** asks the core to bring that tab in front (`ActivateTab`, 005). The tab that was typed in stays as it was, and its field shows its page's address again, as when any other tab comes in front (011 §5).

**Why the core.** Which tabs a window has, which row it shows and what matches are the core's; a shell that searched the tabs it draws would have a rule of its own, and a second one for the order.

## 4. The contract (1.27)

Added; nothing is changed or removed.

```proto
// browser.proto
message AddressSuggestion {
  …
  // Not 0: an open tab of the window that shows this page (038). Choosing it
  // brings that tab in front (ActivateTab) and loads nothing.
  uint64 tab_id = 4;
}
```

A shell that does not know the field loads the address in the tab in front, as it would a place's: nothing is lost but the switch.

## 5. The shells

A tab's row is a row of the same list (011 §5), with the tab's title and address as a place has them, and the words "Switch to Tab" at its end, said to a screen reader with the row. Choosing it, by a press, or on a Mac by the arrow keys and Return, brings the tab in front and puts the list away. Nothing is decided in the shell: a row with a tab is a row whose choice is `ActivateTab`.

**The words are in the text's own colour.** As first drawn on Apple they were small and grey, as a caption is. The iPhone's accessibility audit, run by the app's test over the list, found them too faint for their size; they are now as dark as the address beneath the title. (Android's are in the colour Material gives a caption, which its own checks pass.)

## 6. Decisions

Provisional until the owner says: built as the recommendation.

| # | Decision | Why | Other ways |
|---|---|---|---|
| D1 | Open tabs come first | It is what Safari, Firefox and Chrome do: a page that is open is the likeliest meant, and loading it again makes a second copy | After the places; or only when nothing in history matches |
| D2 | At most three tabs, in the row's order | The list is six: history must still be seen. The row's order is the one a person can see | By when each was last in front, which the core does not keep yet |
| D3 | Only the row the window shows | Choosing a row should not change the Space or the window under a person | Every tab of every Space and window, with the Space named |
| D4 | The tab in front is not offered | It is where the person is | — |
| D5 | The tab that was typed in stays, also when it is empty | The smallest first step. Firefox and Chrome close an empty tab that was only opened to go somewhere; that is a rule of the core's to add (a word on `ActivateTab`), and is recommended next | Close it now |

## 7. Tests

| Where | What |
|---|---|
| The words (`wt-core`, 4: `words.rs`) | Every word typed is a word or the start of one, in any of the texts, whatever its case; the middle of a word is not its start, and one word missing is a miss. Nothing typed, and only punctuation, find nothing. Letters of any script are words; an accent is not looked past. No more words are looked for than history's index looks for (the two now share the number) |
| The core (`wt-app/tests/offered_tabs.rs`, 8, through `FakeShell`) | An open tab is offered first, as that tab, with its title and its address as the field shows it; the place it also is is not offered a second time; the places follow as 011 orders them; the tab in front is never offered as a tab. What matches: seven things typed that do, five that do not, and the title a page has now, not the one it had. Not offered: an empty tab, a tab whose page never arrived, a tab on a page that is not the web's. At most three, the first of the row that match, the tab in front stepped over; the list no longer than it was (six; the shell's own number is the whole list's). Only the row the window shows: not another window's tabs, nor those of a Space of the same window that is not shown, and each window and Space the other way round. A private window is offered its own tab and nothing of history, and an ordinary window nothing of a private one's. Choosing it (`ActivateTab`) brings the tab in front, loads nothing, and leaves the tab that was typed in where it was. A window there is not is answered with `window.unknown` |
| By breaking it | Thirteen parts of the rule, each then run against these tests and 011's: all caught |
| The Apple shell's model (`SuggestionModelTests`, 1 more, on the Mac and the iPhone simulator, over a scripted core) | Choosing a row that is a tab asks the core to bring that tab in front; no address is sent; the field is no longer being typed in and shows no typed text; the list is gone. By the arrow keys and Return the same. A place beside it is still gone to |
| Android's model and list (`SuggestionModelTest`, 1 more; `SuggestionListTest`, 1 more, drawn by Compose on Robolectric) | The same of the model. In the list: of three rows only the tab's says "Switch to Tab", with its title and address; pressing it asks the core for the tab and loads nothing; the list is gone |
| Pictures (`window-suggestions`, light and dark, on the Mac, the iPhone simulator and Robolectric) | Recorded again and looked at: a tab's row first, its title cut short where it meets the words, "Switch to Tab" whole at the row's end; the places below as they were. The old pictures were seen not to match first |
| The Mac's and the iPhone's app (`FeatureTests`, 1 more, fake mode: the real core and real WebKit) | A page open in the first tab; in a second tab, gone elsewhere, the start of a word of the first's title is typed: one row, which says the title and "Switch to Tab"; Apple's accessibility audit over the list; chosen (on the Mac by the arrow key and Return), the first tab is in front with its page and its address, the list is gone, there are still two tabs, and the test site was not asked for the page again. The audit is told two things, each written where it is told: a tab that the strip has scrolled part of the way out of view has no contrast to measure (with two tabs an iPhone's strip scrolls; the test checks that the tab in front is in view), and a row's title is one line and is cut sooner beside the words at the largest text sizes |
| Android's app on the emulator (`AppWindowTest`, 1 more, fake mode: the real core and the real WebView) | The same, and back again: from the first tab the second is the one offered. The accessibility checks over the list |

## 8. What is not verified

- **Many tabs.** Three are offered of however many match; the core looks at every tab of the row for each letter typed. With hundreds of tabs that was not measured.
- **An accent** typed without, in a tab's title: not found (§3), though history finds it. Nobody has complained; it is written down.
- **A tab that came back asleep** after a quit (009): it has its page's address and title and is offered like any other; choosing it loads it, as bringing it in front always does. Not tested here.
- **The iPad and a keyboard on Android**: the arrow keys in the list are the Mac's only.
- **The largest text sizes.** The audit says the title beside the words may be cut there, which it is by design; nobody looked at the list at those sizes.
- **A screen reader** reading the row: the words are part of the row's label on Apple (the app's test reads the label) and of its text on Android; nobody listened.

## 9. Done checklist

- [x] Contract 1.27 and its version in every place it is written.
- [x] The core: the rule of §3.
- [x] The Apple shell: the row, and choosing it.
- [x] The Android shell: the row, and choosing it.
- [x] `Tools/test.sh all` green, and `android-device`, on 2026-11-22: `lint`, `core` (550), `unit` (272 on the Mac), `build`, the Mac's UI suite (38), `ios` (254 on the iPhone simulator, one skipped as before), the iPhone's UI suite (29), `android` (381 and 32 on the JVM) and `android-device` (156 on the emulator).

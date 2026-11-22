# 035 — From tab to tab by key (W2)

| | |
|---|---|
| **Status** | Built and tested in the core and in the Mac's app: the core says which tab is the next, the one before, the last, or at a place; the Mac's Window menu and its keys ask. Other shells have the request and no keys yet |
| **Phase** | W2 of [000](000-master-plan.md) §9: "keyboard maps per platform", the keys that go from tab to tab |
| **Contract** | 1.25: `StepTab` |
| **Builds on** | [005](005-tabs.md): a tab brought in front. [034](034-the-macs-menus.md): the Mac's menus |
| **Last updated** | 2026-11-21 |

## 1. User story

My hands are on the keyboard. I go to the next tab, and to the one before, and round again at the end of the row; I go straight to the third tab, or to the last. On the Mac the menu bar has these too, with their keys.

## 2. What is in it, and what is not

**In:** one request to the core, which brings another tab of a window in front by where it is in the row; on the Mac, the Window menu's items and their keys.

**Not in:**

- **The iPad and Android with a keyboard.** The request is the same for every shell; their keys are not added here.
- **Going through tabs in the order they were last used** (as Control-Tab does in some browsers, by choice). The row's order is the only one.
- **Moving a tab** along the row: with the tree and drag and drop (W2).

## 3. The rule

The core is asked (`StepTab`, for a window) and decides which tab comes in front, by the row the window shows (the tabs of the Space it shows, [024](024-several-spaces.md)):

- **the next** tab, and from the last, the first; **the one before**, and from the first, the last;
- **the tab at a place**, counted from one: nothing happens when the row is shorter;
- **the last** tab.

Coming in front is what it always is (005): a tab that came back from the session is loaded now (009); a find stops in the tab that was in front (010). With one tab in the row, or when the tab asked for is in front already, nothing changes and nothing is refused. A window the core does not have is refused (`window.unknown`), and so is a request that says no way to go (`tab.step`).

**Why the core.** Which tab is "next" is a rule about browsing: it must be the same on every system, and it changes when rows become trees and groups (W2). A shell that worked it out from the tabs it draws would have its own.

## 4. The contract (1.25)

Added; nothing is changed or removed.

```proto
// browser.proto
// Brings another tab of a window in front, by where it is in the row the
// window shows (035). Answered with Ack, also when nothing changes;
// "window.unknown"; or "tab.step" when no way to go is said.
message StepTab {
  uint64 window_id = 1;
  enum To {
    TO_UNSPECIFIED = 0;
    TO_NEXT = 1;      // from the last, the first
    TO_PREVIOUS = 2;  // from the first, the last
    TO_LAST = 3;
    TO_PLACE = 4;     // the tab at `place`
  }
  To to = 2;
  // For TO_PLACE: the place in the row, counted from one. Beyond the row,
  // nothing happens.
  uint32 place = 3;
}
```

## 5. The Mac

The Window menu, after the system's items for a window's size:

| Item | Key | Grey when |
|---|---|---|
| Show Next Tab | ⌃Tab | the window has one tab |
| Show Previous Tab | ⌃⇧Tab | the window has one tab |

And keys with no item, as every browser on the Mac has them: ⌘1 to ⌘8 for the tab at that place, ⌘9 for the last; ⇧⌘] and ⇧⌘[ beside ⌃Tab and ⌃⇧Tab. They are carried by buttons nobody sees, as ⌘= is (034).

**The system's own tabs are turned off.** A Mac makes tabs of an app's windows by itself, and gives the Window menu "Show Next Tab" and "Show Previous Tab" for those, with these very names and keys. The browser's tabs are its own: the system's way is turned off as the app starts (`NSWindow.allowsAutomaticWindowTabbing`), and with it those items, and "Show Tab Bar" in the View menu.

**The model asks, and works nothing out** (`TabSteps.swift`): the way to go, and the place; no tab is named.

## 6. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | One request with a way to go, not four | The four are one question: which tab, by the row |
| D2 | Nothing to do is not refused | A key pressed with one tab open is no mistake |
| D3 | ⌘9 is the last tab, not the ninth | As Safari, Chrome and Firefox on the Mac have it |

## 7. Tests

| Where | What |
|---|---|
| Core (`wt-app/tests/step.rs`, 6, through `FakeShell`) | The next tab and the one before, going round at both ends, the row itself unchanged. The tab at a place, counted from one; the tab already in front; a place beyond the row, place nought and the largest number there is: nothing changes, nothing is refused; the last, from anywhere and from the last. With one tab nothing changes and the shell is asked for nothing. A window there is not is refused, first; no way to go, and a way this core does not know, are refused (`tab.step`). The row is the one of the Space the window shows: with three tabs in a Space that is not shown, nothing is stepped to, there or here. A tab that came back asleep after a quit is loaded when it is stepped to, once (over a store on disk). Eleven parts checked by breaking them: all caught, one only after its test was made to look at the Space that is not shown |
| The Apple shell's model (`TabStepModelTests`, 1, on the Mac and the iPhone simulator) | The core is asked with the window, the way and the place; before the window is open, and for a place below one, nothing is asked; no tab is named by the model |
| The Mac's app (`CommandTests`, 1 more, fake mode: the real core and real WebKit) | With one tab both items are grey. With three: Show Next Tab from the last goes round to the first, whose page is then shown; Show Previous Tab goes round to the last. ⌃Tab twice, ⌃⇧Tab; ⌘2, ⌘9 (the last), ⌘1; ⌘7 goes nowhere; ⇧⌘] and ⇧⌘[. No tab is opened or closed by any of it. Four parts checked by breaking them in the app: all caught |

## 8. What is not verified

- **The iPad, and Android with a keyboard.** No keys there.
- **Keyboards that are not American.** ⇧⌘] and ⇧⌘[, and the number keys, are where they are on an American keyboard; where a layout needs Shift or Option for them, what the keys do was not tried.
- **That turning off the system's tabs takes nothing a person wanted.** A person who had "Prefer tabs: always" set in the system's settings gets windows, as in other browsers. Seen only as the absence of a second pair of items in the Window menu: the test presses each item by its name, which fails when two have it.
- **Many tabs.** Three in the app; the core's rule has no limit of its own.
- **A tree.** The row is flat. When tabs are in groups and trees (W2), "next" is the core's to say anew.

## 9. Done checklist

- [x] Contract 1.25 and its version in every place it is written.
- [x] The core: the rule of §3.
- [x] The Mac: the Window menu's items, and the keys.
- [x] `Tools/test.sh all` green, and `android-device`, on 2026-11-21: `lint`, `core` (525), `unit` (268 on the Mac), `build`, the Mac's UI suite (36), `ios` (250 on the iPhone simulator, one skipped as before), the iPhone's UI suite (27), `android` (360 and 32 on the JVM) and `android-device` (149 on the emulator).

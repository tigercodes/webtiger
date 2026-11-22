# 036 — Keys on Android with a keyboard (W2)

| | |
|---|---|
| **Status** | Built and tested, on the JVM and on an emulator: with a keyboard, the window answers a browser's keys wherever the keyboard's focus is. Not run on a device with a keyboard of its own (§6) |
| **Phase** | W2 of [000](000-master-plan.md) §9: "keyboard maps per platform", Android's, for the commands there are so far |
| **Contract** | No change (1.25's `StepTab` is used) |
| **Builds on** | [003](003-android-shell.md): the window. [034](034-the-macs-menus.md): the Mac's keys for the same commands. [035](035-tabs-by-key.md): from tab to tab, by the core's word |
| **Last updated** | 2026-11-21 |

## 1. User story

My tablet has a keyboard, or my phone is on a desk with one. The keys I know from a browser work in WebTiger: a new tab, close it, bring it back, the next tab, the address field, reload, back and forward, find, zoom, history.

## 2. What is in it, and what is not

**In:** on Android, with a keyboard that has a Control key, the window answers the keys of §3, wherever the keyboard's focus is in it: in the page, in the address field, on a button.

**Not in:**

- **The system's list of an app's keys** (the helper a keyboard's Search key and `/` bring up): it could be told these; not yet.
- **Changing a key.**
- **A menu.** Android has no menu bar.
- **Keys while a list is over the window** (History and Downloads, the Spaces): the window does not answer them then.

## 3. The keys

As Chrome on Android and every desktop browser on a PC keyboard have them.

| Key | What |
|---|---|
| Ctrl+T | New tab |
| Ctrl+W | Close the tab in front (the core never leaves the window empty, 005) |
| Ctrl+Shift+T | Reopen the closed tab ([033](033-a-closed-tab-comes-back.md)); nothing when there is none |
| Ctrl+Tab, Ctrl+Shift+Tab | The next tab, the one before ([035](035-tabs-by-key.md)) |
| Ctrl+1 to Ctrl+8, Ctrl+9 | The tab at that place; the last |
| Ctrl+L | The keyboard goes to the address field |
| Ctrl+R | Reload |
| Alt+Left, Alt+Right | Back, forward |
| Ctrl+F | Find on the page |
| Ctrl+H | History and Downloads |
| Ctrl+Plus (and Ctrl+=), Ctrl+Minus, Ctrl+0 | Zoom in, zoom out, the page's own size |

**The window takes these keys before the page.** A page that has a use of its own for one of them does not get it. (Desktop browsers leave some of them to a page that asks; which, and how a page asks, is not settled here.)

**Nothing is decided in the shell but which key asks for what.** Each key calls what a button of the window calls; which tab is next is the core's.

**A key held down asks once.** What a held key goes on sending is taken, and asks nothing: a held Ctrl+T does not fill the window with tabs. A key the window answered is given to nothing else; any other key goes to whatever has the keyboard, as before.

## 4. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | The activity asks the window about every key before anything in the window has it | A key must do the same whether the focus is in the page, in a field, or nowhere. As first built the window waited for the key to come through whatever had the focus: with the focus nowhere, no key came (the JVM's test of the window found it) |
| D2 | Which key asks for what is one function, of the key and its modifiers | It can be tested as a table, with no window |
| D3 | A PC's keys (Control), not a Mac's | An Android keyboard has Control where a Mac has Command; Chrome on Android does the same |

## 5. Tests

| Where | What |
|---|---|
| The table (`KeysTest`, 2 of its 6, on the JVM) | Each key of §3 asks for what it should, Plus by each of the keys that type it; Ctrl+9 is the last tab and Ctrl+1 to Ctrl+8 the tab at that place. Left alone: the same keys by themselves and with Shift, with the system's key held, with Control and Alt together; editing's keys (Ctrl+A, C, V, X, Z); a key whose Shift form is another command; an arrow with Control |
| The window (`KeysTest`, the other 4, drawn by Compose on Robolectric over a scripted core, keys given as the activity hears them) | With nothing in the window given the keyboard's focus, thirteen keys each ask the core what their button asks, in order, with the window or the tab named. A key that is not the window's is not answered; nor one coming up; a key held down is answered once, and what it sends while held is taken and asks nothing. Ctrl+F opens the find bar; Ctrl+L puts the keyboard in the address field; Ctrl+H opens the lists, and while they are over the window no key is answered and nothing is asked. With no browser window drawn, and after one is gone, no key is answered |
| The app, on the emulator (`AppKeysTest`, 1, fake mode: the real core and the real WebView; keys sent as the system sends a keyboard's) | With the keyboard in a page: Alt+Left and Alt+Right go back and forward; Ctrl+T opens a tab; Ctrl+Shift+Tab, Ctrl+Tab, Ctrl+1, Ctrl+9 go from tab to tab as the core says. Ctrl+L puts the keyboard in the address field; with it there, Ctrl+W closes the tab in front and Ctrl+Shift+T brings it back with its page. Passed three runs one after another, and in the whole suite twice |
| By breaking it | Fourteen parts, each run against `KeysTest`: all caught |

## 6. What is not verified

- **A real keyboard.** The keys were sent by the test, as the system sends them; nobody typed. A keyboard whose layout puts these keys elsewhere, or types a letter with Control and Alt, was not tried.
- **A phone or a tablet.** An emulator.
- **Ctrl+R, Ctrl+F, Ctrl+H and the zoom keys in the app on the emulator.** Tested in the window on the JVM; the device's test presses the keys of tabs, of going back and forward, and Ctrl+L.
- **A page that wants one of these keys.** It does not get it; no test has such a page, and what a page should be let keep is not settled.
- **Keys while a list other than History and Downloads is over the window** (the Spaces, a link's menu): the window answers them, under the list. Not looked at.
- **The system's list of an app's keys.** Not told.

## 7. Done checklist

- [x] The table of keys, as a function, and tested as one.
- [x] The window answers them, in a JVM test of the window.
- [x] On the emulator: with the focus in a page, and in the address field.
- [x] `Tools/test.sh android` (366 and 32 on the JVM) and `android-device` (150 on the emulator, twice) green, with `lint`, on 2026-11-21. Nothing of the core's or of Apple's changed: those stages were not run again.

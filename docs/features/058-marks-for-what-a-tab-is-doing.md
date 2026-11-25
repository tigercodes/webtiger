# 058 — Marks for what a tab is doing (W3)

| | |
|---|---|
| **Status** | **Built and tested on Apple's systems and Android, on the recommendations of §6.** The owner's word of 2026-11-24 was to use what is suggested where something waits on a choice, and to go on: they are built on, and stay the owner's to change. A tab that plays a sound has a speaker after its name, one whose page holds the camera or the microphone a recording mark, and each says so in words |
| **Phase** | W3 of [000](000-master-plan.md) §9. 000 §5.7: "A tab is kept awake while it plays audio, holds the camera or microphone, … The sidebar shows why." The eighth of W3 |
| **Contract** | None: a tab's reasons have been in what the core tells a shell since 1.45 ([052](052-what-keeps-a-tab-awake.md)) |
| **Builds on** | [052](052-what-keeps-a-tab-awake.md): the reasons, and its D5, which left the marks for later and recommended a speaker for sound. [056](056-what-is-awake.md): the list that says every reason in words. [043](043-the-sidebar.md), [047](047-the-phones-switcher.md): the three places a tab is drawn |
| **Last updated** | 2026-11-24 |

## 1. User story

Something is playing, and I have thirty tabs. I want to see which one without opening each. And if a page has my camera or my microphone, I want to see that from wherever I am in the window, not only when its tab is in front.

## 2. What is in it, and what is not

**In:** a tab that plays a sound has a small speaker after its name; a tab whose page holds the camera or the microphone has a small recording mark there. In the strip, in the sidebar and in a phone's switcher: wherever a tab is drawn by its name. A screen reader is told in words, with the tab's name.

**Not in, each with where it goes:**

- **Marks for the other reasons** (typed text, a download, a question, a load): none (D1). Each is said in words in the list of what is awake (056), and each but typed text has a place already: a load has the tab's dot, a download the Library's button and list, a question its bar in the tab.
- **Pressing the speaker to silence the tab:** with the media controls (W7). The mark is not a button.
- **A Favourite's tile:** it has no name beside it and no room for a mark; it says the words to a screen reader, and shows nothing more (D4).
- **A tab that is asleep:** it has no page to be doing anything, and has no mark.

## 3. The rule

Nothing here is decided by a shell: the core says which reasons a tab has ([052](052-what-keeps-a-tab-awake.md)), in its own order, and a shell draws at most one mark for them.

**Which mark.** The camera or the microphone before sound, when a tab has both: it is the one a person must not miss, and a call is both. One mark, not two: a tab's name is short enough already.

**Where.** After the name, before the button at the tab's end. It does not move the name. A tab whose name is as long as a tab's may be is no wider for it on Apple's systems: the name gives up the room, and the mark is never the thing cut short. A tab with a shorter name is wider by the mark.

**As built, where the design did not say:**

- **The pictures.** On Apple's systems the system's own: a speaker with two waves, and a ring round a dot. On Android two drawn for the app to the same shapes.
- **On Android a tab is wider by the mark** whatever its name's length: in the strip a name has its own most width, and the mark stands after it. In the sidebar the name has the row but for the mark.
- **In the sidebar and the phone's switcher** the mark is where it is in the strip: they draw a tab with the same part.
- **A reason this shell does not know** (a newer core's) has no mark and no words in the row.

**In words.** "Playing sound", or "Using the camera or microphone", said of the tab beside its name, with what else is said of it (that it is a Site, away, a Favourite): the words the list of what is awake uses (056).

## 4. The contract

None. `TabSummary.keeps_awake` (1.45).

## 5. In what order

1. **Apple's shell:** the mark in the strip, the sidebar and the phone's switcher; the words; pictures; a test in the app with the suite's page that plays a sound. **Built.**
2. **Android's shell:** the same. **Built.**

## 6. Decisions

**All of these are recommendations, to be used on the owner's word of 2026-11-24** (§ Status).

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | Two marks: a speaker for sound, a recording mark for the camera or the microphone. None for the other reasons | A speaker is what every browser has (052 D5). A page that can see or hear the person is the other thing they must be able to find at a glance. The rest are of no urgency, and have their words in the list (056) | A speaker alone (052 D5's other way). Or a mark for every reason: six small pictures to learn, most of them for things that need nothing done |
| D2 | One mark to a tab: the camera or the microphone before sound | A call is both, and the one that matters is that it can see and hear | Both, side by side |
| D3 | The mark is after the name, and is never the thing cut short | A long name gives up the room; the mark is what must be seen. (A tab with a short name is wider by the mark, and the row moves by that much when a song starts) | Before the name, where the site's icon is: seen even when the name is cut short, and it moves the name. Or a place kept for the mark in every tab, so that nothing ever moves, and every tab is wider |
| D4 | A Favourite's tile shows no mark, and says the words | A tile is as small as it can be pressed, and is a picture already | A dot on the tile's corner |
| D5 | The mark is not a button | Silencing a tab is a command with its own rules (what of a call?), and belongs with the media controls | Pressing the speaker silences the tab |

## 7. Tests

| Where | What |
|---|---|
| Apple's marks and words (`TabDoingTests`, 5, on the Mac and the iPhone simulator) | Sound and the camera or the microphone have a mark; typed text, a download, a question, a load, and a reason the shell does not know have none; every reason the contract has is one or the other. A tab has one mark, the camera's before the sound's, whatever the order they are said in and whatever else is among them. Each mark is a picture the system has, by its name, and has its own words. What is said of a tab beside its name: the words of its mark, after what the tab is and where; one thing where it has both; nothing for a reason with no mark; and a sleeping tab says what it did. **The mark is drawn:** a tab's own label, drawn alone and small, differs with a speaker and with a recording mark from the same label with none by more than a picture's comparison lets pass; the two marks are two pictures; with both reasons it is the camera's; and typed text draws nothing |
| Apple's pictures (`CatalogueSnapshotTests`, a state more: two pictures on each system) | The window with tabs that are doing something, in both themes. On the Mac: the tab in front, in a call, with the recording mark; beside it one that plays a sound, whose long name is cut short before its speaker. On the iPhone the strip is narrower: the tab in front with its mark, and the next cut by the strip's end. Every other picture of the window was recorded again and is the one it was, to the last dot |
| The apps, in fake mode, with the real core and the real engine (`DoingTests`, 1, on the Mac and the iPhone simulator) | The suite's page of fields, whose button starts a sound that is silence: the tab says nothing; the button pressed in the page, it says "Playing sound"; another tab in front, it says so from behind, and the tab in front says nothing; Apple's audit, with that plain page in front; in front again and its sound stopped, it says nothing |
| Apple's shell broken, a part at a time | 10 parts, 10 caught. The first run caught 9. With the mark not drawn at all, every test passed, the window's pictures among them: a picture's comparison lets a hundredth of it differ, and two small marks in a whole window are less than that. So a tab's own label is now drawn alone, with a mark and without, and the two must differ (above). (The first try at that break was no break for the new test: it left the mark drawn for a tab numbered nought, which the test's tab is) |
| Android's marks, words and strip (`TabDoingTest`, 4, on the JVM) | As Apple's first three. And the strip drawn, as a scripted core says it: a tab that plays a sound, a Site that is away and holds the camera, a tab with both, a tab with other reasons and a tab with none say "Playing sound", "Site, Away, Using the camera or microphone", "Using the camera or microphone", and nothing twice; one speaker is drawn and two recording marks; when the core says the sound has stopped and the camera is given back, nothing is said and no mark is left |
| Android's pictures (`CatalogueSnapshotTest`, a state more: two pictures) | The window with tabs that are doing something, in both themes: the tab in front with the recording mark drawn for the app. (The speaker drawn for the app was looked at in a picture that was recorded on the way and not kept, with a tab that plays a sound in front.) Every other picture was recorded again and is the one it was |
| Android's shell broken, a part at a time | 10 parts, 10 caught, at the first run: on Android a mark that is not drawn is a part of the strip that is not there, and the strip's test counts them |
| Android, on an emulator (`AppDoingTest`, 1), in fake mode, with the real core and the real WebView | The suite's page: its button touched in the page, the tab says "Playing sound" in the strip and has its speaker; another tab in front, it says so and has it still; in front again and its sound stopped, it says nothing and has none |

## 8. What is not verified

- **The choices of §6** are the recommendations, used on the owner's word; the owner has not looked at the marks.
- **The camera's mark, in an app.** No test lets a page have a camera or a microphone (001: no test may bring up a dialog of the system's, and this Mac has neither to give). That a tab whose page holds one says so and has the mark is the model's tests and the pictures; that the core knows a page holds one is 052's.
- **A sound that is heard.** The suite's sound is silence, as a sound file: nothing was listened to.
- **The speaker on a phone's picture.** A phone's strip shows one tab whole: the kept pictures have the recording mark on it. Android's speaker was seen in a picture that was not kept (§7); the iPhone's is the system's own picture, and is half seen at the strip's end.
- **The sidebar and the switcher, by picture.** They draw a tab with the same part as the strip; no picture of them has a tab with a mark.
- **A tab that jumps.** A tab with a short name is wider by the mark when a song starts (§3): not measured, and not tried with anyone.

## 9. Done checklist

- [ ] The owner's word on §6 (used as suggested, on the word of 2026-11-24).
- [x] Apple's shell.
- [x] Android's shell.
- [ ] The gate.

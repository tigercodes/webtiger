# 058 — Marks for what a tab is doing (W3)

| | |
|---|---|
| **Status** | **Designed, not built.** The choices of §6 are recommendations. The owner's word of 2026-11-24 was to use what is suggested where something waits on a choice, and to go on: they will be built on, and stay the owner's to change |
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

**Where.** After the name, before the button at the tab's end. It does not move the name, and a tab that gains a mark is no wider: in a strip a tab's place is what it was, and the name gives up the room.

**In words.** "Playing sound", or "Using the camera or microphone", said of the tab beside its name, with what else is said of it (that it is a Site, away, a Favourite): the words the list of what is awake uses (056).

## 4. The contract

None. `TabSummary.keeps_awake` (1.45).

## 5. In what order

1. **Apple's shell:** the mark in the strip, the sidebar and the phone's switcher; the words; pictures; a test in the app with the suite's page that plays a sound.
2. **Android's shell:** the same.

## 6. Decisions

**All of these are recommendations, to be used on the owner's word of 2026-11-24** (§ Status).

| # | Recommendation | Why | Other ways |
|---|---|---|---|
| D1 | Two marks: a speaker for sound, a recording mark for the camera or the microphone. None for the other reasons | A speaker is what every browser has (052 D5). A page that can see or hear the person is the other thing they must be able to find at a glance. The rest are of no urgency, and have their words in the list (056) | A speaker alone (052 D5's other way). Or a mark for every reason: six small pictures to learn, most of them for things that need nothing done |
| D2 | One mark to a tab: the camera or the microphone before sound | A call is both, and the one that matters is that it can see and hear | Both, side by side |
| D3 | The mark is after the name, and the tab is no wider for it | The row does not jump when a song starts | Before the name, where the site's icon is: seen even when the name is cut short, and it moves the name |
| D4 | A Favourite's tile shows no mark, and says the words | A tile is as small as it can be pressed, and is a picture already | A dot on the tile's corner |
| D5 | The mark is not a button | Silencing a tab is a command with its own rules (what of a call?), and belongs with the media controls | Pressing the speaker silences the tab |

## 7. Tests

To be filled in as built.

## 8. What is not verified

To be filled in as built.

## 9. Done checklist

- [ ] The owner's word on §6 (to be used as suggested, on the word of 2026-11-24).
- [ ] Apple's shell.
- [ ] Android's shell.
- [ ] The gate.

# 040 — One catalogue of words, one tool, every platform

| | |
|---|---|
| **Status** | Built and tested on the Mac: one Rust tool writes Apple's, Android's and Windows' files from the catalogue; the Swift code asks for every string by a generated name. **Windows' file has not been built on a PC** (§8). The owner's instruction of 2026-11-22 (WT-D30) |
| **Phase** | Foundations, kept up: [001](001-foundations.md)'s strings, made ready for three shells |
| **Contract** | No change: no word crosses the contract. The core speaks codes; the shells speak words |
| **Builds on** | [001](001-foundations.md): `Localization/strings.json`, and the script that wrote Apple's and Android's files from it |
| **Last updated** | 2026-11-22 |

## 1. User story

I change a label, a button's name, a message or a title in one place, and every platform has the change: the Mac, the iPhone and the iPad, Android, and Windows when its shell is there. It costs nothing while the app runs, and nothing in safety.

## 2. What there was, and what was missing

Every user-visible string has been in `Localization/strings.json` since W0, each with a key and a note, and a Python script wrote Apple's String Catalog and Android's `strings.xml` from it. Swift and Kotlin both ask for a string by its key. So one edit already reached the Mac, the iPhone and Android. What was missing:

1. **A generator both machines can run.** The Windows shell is built on a PC, where Python is not part of what the project needs. The Rust toolchain is on both: the core is built on both.
2. **Windows' file.** There was none.
3. **A string that is deleted or renamed was not a build error on Apple.** Kotlin names a string as `R.string.tabs_new`, which does not compile when the string is gone. Swift named it by a text, `"tabs.new"`, which compiles whatever it says: a key that is not in the catalogue is drawn on the screen as it is written.
4. **Nothing checked that a string takes the same arguments everywhere.** A translation, or a platform's own wording, with one `%@` fewer than the English reads past its arguments on Apple, and ends the app on Android.

## 3. The design: all of it at build time

**One tool.** `Core/tools/strings-gen`, a small Rust program with its own workspace (as `tools/proto-gen` is, so the core's builds never compile it), reads the catalogue and writes every platform's file. `Tools/strings/generate.sh` runs it; `--check` writes nothing and fails when a file is stale, which `Tools/test.sh lint` asks. It depends on `serde` and `serde_json` at the versions the core's lock file already holds: neither machine fetches anything to run it.

| Platform | What is written | How the code asks |
|---|---|---|
| Apple | `Packages/Sources/WTUI/Resources/Localizable.xcstrings`, the String Catalog, as before; and `Packages/Sources/WTUI/Words.generated.swift`, every string by name | `Words.tabsNew`: a name that does not compile when the string is gone (§4) |
| Android | `Android/app/src/main/res/values*/strings.xml`, as before | `R.string.tabs_new`, as before |
| Windows | `Windows/Strings/<language>/Resources.resw` | By the resource's name, `tabs_new`. **Not built on a PC yet** (§8) |

**Nothing changes while the app runs.** Each shell looks a string up exactly as its system has it, from files built into the signed app. There is no call into the core for a label, no file read that was not read before, and nothing fetched.

**Two things this is not, on purpose:**

- **The shells do not ask the core for words.** Every label would cross into Rust each time a screen is drawn, and a screen that is drawn without the core (the page that says the core could not start; a picture of a screen in a test) would have no words. It would gain nothing over a file written at build time.
- **Words are not fetched from a server**, to change them without a release. Whoever could change that server's answer could make a button say what it does not do. A browser's words say what is safe and what is not: they ship with the code that acts on them.

## 4. Names on Apple

The tool writes `Words.generated.swift`: one constant for each string, named from its key (`tabs.new` is `Words.tabsNew`; `space.new.shared.detail` is `Words.spaceNewSharedDetail`), with the English and the note as its comment. The Swift code asks by name:

| Before | Now |
|---|---|
| `Text("tabs.new", bundle: .module)` | `Text(Words.tabsNew)` |
| `String(localized: "space.home", bundle: .module, locale: locale)` | `Words.spaceHome.string(locale)` |
| `String(format: String(localized: "find.count", bundle: .module), current, total)` | `Words.findCount.string(current, total)` |

Deleting or renaming a string in the catalogue is then a build error at every place that used it, as it is on Android. A string whose key is made while the app runs (an error's words are `error.` and the core's code) is asked for by `Word(named:)`, which answers nothing when the catalogue has no such string, so that the place that asks says what it shows instead.

What a name is at run time is what the text was: the key the String Catalog is asked for. Nothing is looked up differently.

## 5. The catalogue

```json
"find.count": { "en": "%1$d of %2$d", "note": "…" },
"tabs.close": { "en": "Close Tab", "android": { "en": "Close tab" }, "note": "…" }
```

- **A key** is dotted lower-case words (`tabs.new`): the Apple key as it is, and with `_` for `.` the Android and Windows name.
- **`en` and `note`** are required. Another language is another field, by its tag (`fr`, `pt-BR`).
- **A platform's own wording** (new): `apple`, `android` or `windows`, each an object of language to text, for a string that should read differently there (a system's own capitals, a system's own word). Without it a platform has the common text. No string uses one yet.
- **Arguments** are written Apple's way (`%@`, `%d`, `%1$@`; `%%` is a per cent sign). Android's file has them as `%s` and `%1$d`; Windows' as `{0}` and `{1}`.

**What the tool refuses**, with the key and the reason:

- a key that is not dotted lower-case words; the same key twice; a field that is no language, no platform and no note; a missing `en` or `note`;
- two keys that make one Android or Windows name, or one Swift name; a name that starts with a digit;
- a `%` that is no argument; a string that numbers some of its arguments and not others;
- **a string that does not take the same arguments in every language and on every platform as its English does**: the same number, of the same kinds, whatever their order.

## 6. Decisions

| # | Decision | Why |
|---|---|---|
| D1 | A Rust tool in `Core/`, not Python or a script of one system's | It is the one toolchain both machines must have |
| D2 | Generated files stay checked in, and `lint` fails when one is stale | As every generated file here: no build runs a generator, and a stale label is caught on either machine |
| D3 | Names on Apple are generated constants | A missing string should stop the build, as on Android |
| D4 | A platform's own wording is in the same entry | One place still says everything a string is; the arguments are checked against the English |
| D5 | Windows' file is written now, though no Windows shell is | The tool is where the format lives; the PC's first build then finds its strings there. Said to be unverified until then |

## 7. Tests

| Where | What |
|---|---|
| The tool (`Core/tools/strings-gen`, 24, run by `Tools/test.sh core`) | **The catalogue:** strings are read by key with every language; a platform has its own wording where it is given one, and the common text where it is not; thirteen things refused, each by its reason (a key that is not dotted lower-case, a missing `en` or `note`, a field that is no language, platform or note, a platform's wording that is no object or names no language, a key or a field written twice, two keys that are one resource, a name that starts with a digit, a field the file does not have). **Placeholders:** a string as its text and its arguments, of every kind the catalogue writes; six stray `%` refused; numbering some and not others refused. **The same arguments everywhere:** another order with numbers passes; one fewer, one more, another kind, the kinds swapped, a platform's wording with one fewer, one argument used as two kinds, a stray `%` in a translation: each refused, by the string's key. **Android:** a language's folder; the source language's file with notes, another's without; Android's own wording; placeholders as Android's, numbered when several; aapt's and XML's escapes. **Windows:** a resource for each string with its note as the comment, another language without; arguments from nought in the text's order, braces doubled only in a string that is formatted; Windows' own wording. **Apple:** the String Catalog, whole, compared as text; Apple's own wording; a key as a Swift name; the names file, whole; two keys that are one name refused. **The program:** the checked-in catalogue makes four files; in a folder of its own: a check with nothing written fails and writes nothing, a run writes six files, a second run writes none, a file meddled with is the one stale and the one rewritten; a language taken out of the catalogue leaves no file of the tool's behind, and a file that is not the tool's is not touched; a wrong catalogue writes nothing and says which string; bad arguments are refused |
| What it wrote against what the Python script had written | Apple's String Catalog: **the same bytes** (the tool's first check did not list it). Android's `strings.xml`: one line differs, the one that names what wrote it. Seen in the repository's own difference before anything else was changed |
| The Swift names (`WordTests`, 3, on the Mac and the iPhone simulator) | Every key the names know is a string the app's bundle has, in words that are not the key; a name is its key. A key made while the app runs is answered for only when the catalogue has it: an error's code falls back to the nearest it has, and then to the general words; a colour with no word has none. Arguments are put in |
| The switch to names | 25 Swift files, by a script and then by the compiler: every text that was a key is a name, and every parameter that took one takes a `Word`. Six places asked the bundle for a string directly, three of them by a key made at run time, comparing the answer with the key to tell that there was none: those ask `Word(named:)`, and the others ask by name. **Every picture of every screen is unchanged** (the Mac's and the iPhone's snapshot suites pass without recording): a string that no longer resolved would be drawn as its key |

## 8. What is not verified

- **Windows.** `Windows/Strings/en/Resources.resw` is written and is what the tool's tests say it is. It has not been compiled into an app: there is no Windows shell yet, and no PC ran the tool. Its place, its `{0}` arguments and the doubling of braces are the tool's to change when the shell's first build says otherwise. On the PC the tool is run with cargo from `Core/` (the line is in `Tools/strings/generate.sh`); a script of the PC's own was not written here, where it could not be run.
- **A platform's own wording in an app.** The tool writes it and checks it; no string uses one yet, so no app has shown one.
- **Another language.** The catalogue has English. The tool's tests have a second language; the apps have never been built with one.
- **A string nobody uses.** Deleting a string that is used stops the build. A string that nothing asks for any more stays in the catalogue: nothing says so.
- **Android's names** were already compiled against (`R.string`). A string Android looks up by a code at run time is the test's to catch (`WordsTest`), as before.

## 9. Done checklist

- [x] The tool, with its tests; the Python script gone; `lint` runs the tool.
- [x] Apple's file is, byte for byte, what it was; Android's differs by the line that names the tool.
- [x] Windows' file is written.
- [x] The Swift code asks by name.
- [x] `Tools/test.sh all` green, and `android-device`, on 2026-11-22: `lint`, `core` (564, and the tool's 24), `unit` (275 on the Mac), `build`, the Mac's UI suite (38), `ios` (257 on the iPhone simulator, one skipped as before), the iPhone's UI suite (29), `android` (381 and 32 on the JVM) and `android-device` (156 on the emulator).

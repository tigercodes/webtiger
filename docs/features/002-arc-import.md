# 002 — Importing from Arc

| | |
|---|---|
| **Status** | W0. Reader and command line built against the published shapes of Arc's files and synthetic fixtures. **Not yet run against a real Arc folder** (§8) |
| **Phase** | W0 (reader, portable file, command line); W2 (the import screen that reads the portable file into Spaces) |
| **Decision** | 000 WT-D16: the Arc importer comes first |
| **Deadline** | Arc stops working on 22 December 2026 |

## 1. Why now

Arc shuts down on 22 December 2026. Its own export is a list of links: no folders, no Spaces, no titles. What people built in Arc lives in files on their own machine, and those files stay readable after the shutdown only if someone kept them and something can read them.

WebTiger cannot hold Spaces yet (that is W2). So W0 does the part that cannot wait: it reads Arc's folder and writes everything worth keeping into one file in WebTiger's own format. W2's import screen reads that file. A person who runs the command today loses nothing by Arc going away.

## 2. User story

> I used Arc for three years. I run one command. It tells me it found 4 Spaces, 212 pinned tabs in 31 folders, 9 Favorites and 6 link rules, and writes a file. Arc's folder is untouched. When WebTiger has Spaces, I open that file and my sidebar is back.

## 3. The command

```
webtiger import arc [--from <folder>] [--out <file>] [--no-today] [--no-archive] [--no-boosts]
```

| Option | Default |
|---|---|
| `--from` | Arc's data folder: `~/Library/Application Support/Arc` on a Mac; the `Arc` folder under `%LOCALAPPDATA%\Packages\TheBrowserCompany.Arc*\LocalCache\Local` on Windows. A copy of the folder made on another machine works the same |
| `--out` | `webtiger-arc-import.json` in the current folder. An existing file is not overwritten unless `--force` is given. On a Mac the file is readable by its owner alone |
| `--no-today`, `--no-archive`, `--no-boosts` | Leave out the unpinned "Today" tabs, the archive, or legacy Boosts |

It prints what it found and what it left out, as counts. It never prints an address or a title.

**What it never does:** write anywhere in Arc's folder; read Arc's account or sync keys (`StorableAuthState.json`, `StorableFirebaseEncryptionKey.json`); read cookies, passwords or history; contact any server.

## 4. What is read

| Arc file | Becomes |
|---|---|
| `StorableSidebar.json` | Spaces in sidebar order; each Space's icon, colour and profile; its pinned tree (folders, nested folders, pinned tabs); its unpinned "Today" tabs; Favorites |
| `StorableLinkRouting.json` | Link rules (Arc's Air Traffic Control) |
| `StorableArchiveItems.json` | The archive: title, address, when it was archived, which Space it came from |
| `boosts/<id>/` | Tweaks, from legacy Boosts: name, the pages it applies to, its CSS and script |
| `User Data/Local State` | The names of Arc's profiles, for naming login sets |

**Not read, and said so in the summary:**

- **Boosts made in Arc's visual editor (Boosts 2.0).** Their content is not in Arc's folder. It sits in a database cache elsewhere on the machine, in a format that needs its own reader. Tracked as the first follow-up (§9).
- **Easels and Notes.** Their content is on Arc's servers. The sidebar has only their names; the summary counts them.
- **Split views.** The tabs are kept, in order, where the split was. The split itself is not (WebTiger's split view is W5).
- **Cookies, passwords, history, extensions.** These are Chromium profile data, and the general Chromium importer's job in W7.

## 5. How Arc's sidebar file is read

Arc's files are Swift `Codable` output, and three of its habits explain nearly every shape in them:

1. **A dictionary is a flat array** that alternates key and value: `[key, value, key, value]`.
2. **An enum is an object with one key**, the case name, with any payload under `_0`.
3. **A time is seconds since 1 January 2001**, not 1970.

The reader walks the JSON as a tree of values, not into fixed structures, so a key it does not know is skipped instead of failing the file. The rules it follows, each of which exists because a published importer was caught out without it:

| Rule | Why |
|---|---|
| The main sidebar is the container that follows the `{"global": {}}` key | Each Little Arc window adds a container of its own, sometimes before the main one, so "the second element" is wrong |
| A Space's pinned and unpinned containers are found by their marker, in **both** `containerIDs` (markers are strings) and `newContainerIDs` (markers are objects), merged | Both lists are present in the same file, in either order; tools that read only one of them miss data in some files |
| Order comes from `childrenIds`, never from the order of the `items` array | The flat array is not in sidebar order |
| A parent that lists no children adopts the items whose `parentID` names it | Seen in real files |
| Ids already visited are skipped, and depth stops at 32 | Real files contain dangling ids, repeated ids and cycles |
| The default profile is recognised by the **presence** of a `default` key | Its value is `{}` in older files and `true` in newer ones |
| A row's name is its own `title`, else its tab's `savedTitle` | `title` is what the user renamed it to |
| An address is kept only if it is `http` or `https` | `arc://`, `chrome://`, `file:` and `javascript:` rows exist and mean nothing outside Arc |
| A file over 64 MiB is refused | A sidebar file is a few megabytes at most |

**Favorites** belong to an Arc profile, not to a Space: every Space on the same profile shows the same row. WebTiger's Favourites belong to a Space (000 §5.6). The reader therefore gives each Space a copy of its profile's Favorites, which is what the person saw in Arc. One profile can be listed once per machine it was used on; those lists are merged, without repeating an address.

**Profiles become login sets.** Two Spaces share logins in Arc exactly when they name the same profile folder, and they share a login set here on the same test. This is the many-Spaces-to-one-login-set shape of WT-D9, which Arc had and several Arc-likes did not.

**Colour.** A Space's colour is the theme's `primaryColorPalette.midTone`, else its single background colour, else the first colour of its gradient. Arc stores colours in extended sRGB, with components that can fall outside 0 to 1; they are clamped. Only one colour is kept: WebTiger has no gradient themes.

**Icon.** An emoji is kept as it is. A name from Arc's own icon set (`planet`) or an SF Symbol name is kept as a name, for the import screen to map onto WebTiger's icons.

## 6. The portable file

One JSON document. It is WebTiger's own interchange format, not a copy of Arc's: the same file is what WebTiger will write when it exports, and what every other importer will produce.

```json
{
  "format": "webtiger.portable",
  "version": 1,
  "source": { "app": "arc" },
  "login_sets": [
    { "key": "default", "name": "Personal" },
    { "key": "profile-1", "name": "Work" }
  ],
  "spaces": [
    {
      "name": "Work",
      "icon": { "emoji": "💼" },
      "colour": "#6CBAD9",
      "login_set": "profile-1",
      "favourites": [ { "kind": "site", "title": "Mail", "address": "https://mail.example.com/" } ],
      "sites": [
        { "kind": "group", "name": "Docs", "children": [
          { "kind": "site", "title": "Handbook", "address": "https://docs.example.com/handbook", "added_at": 1722470400000 }
        ] }
      ],
      "tabs": [ { "title": "A search", "address": "https://search.example.com/?q=x", "last_active_at": 1722556800000 } ]
    }
  ],
  "link_rules": [ { "match": "contains", "pattern": "jira.example.com", "target": "space", "space": 0 } ],
  "tweaks": [ { "name": "Quiet wiki", "matches": ["*://*.wikipedia.org/wiki/*"], "css": "…", "script": "…", "run_at": "document_start", "enabled": true } ],
  "archive": [ { "title": "Old page", "address": "https://example.com/old", "archived_at": 1722643200000, "space": 0 } ],
  "notes": [ { "code": "arc.easel_skipped", "count": 2 } ]
}
```

| Field | Meaning |
|---|---|
| `format`, `version` | A reader refuses a format it does not know and a version newer than it understands |
| `login_sets[].key` | Names a login set inside this file only. No cookie or credential is in the file |
| `spaces[].sites` | The pinned tree. A node is a `group` (a folder, which may nest) or a `site` |
| `spaces[].tabs` | Temporary tabs, in sidebar order |
| `spaces[].icon` | `{"emoji"}`, `{"name"}` (the source app's icon name) or `{"symbol"}` (an SF Symbol name); absent when there is none |
| `link_rules[].target` | `space` (with `space`, an index into `spaces`), `most_recent_space`, or `quick_window` (Arc's Little Arc) |
| `link_rules[].match` | `contains` or `equals` |
| Times | Milliseconds since 1970-01-01 UTC |
| `notes` | What was left out or changed, as a code and a count. Never an address or a title |

The file holds addresses and titles: a person's browsing. It is written with the user's own permissions and nowhere else, and the command says where.

## 7. Tests

- **Fixtures** in `Fixtures/import/arc/`: folders written by hand to the shapes §5 lists. They are synthetic. Real Arc files in public repositories either carry no licence or are someone's actual browsing, and neither belongs in this repository.
- One fixture per rule in §5's table, so each rule has a test that fails without it: a Little Arc container before the main one; the two container lists in opposite orders; children whose order differs from the flat array; adopted children; a cycle; a dangling id; both spellings of the default profile; a renamed tab; non-web addresses; an oversized file.
- Favorites merged across machines without repeats; Spaces on one profile sharing a login set.
- Colours out of range, a gradient theme, no theme; each icon form.
- Link rules with each destination form; a rule that names a Space that no longer exists.
- Legacy Boosts with and without a script; a Boost folder with a file over the size limit.
- A folder with no sidebar file, an empty file, and a file that is not JSON, each a clear error and no output file.
- The summary and the errors contain no address and no title (checked against the fixtures' own).
- A Boost's configuration cannot name a file outside its own folder, by path or by link.
- The import leaves Arc's folder exactly as it was: same files, sizes and dates.
- The reader is fuzzed with the other parsers in W10 (000 §8.8); until then a test replaces every value in the fixture, one at a time, with each kind of JSON value and requires a result or a refusal, never a panic.

## 8. What is not verified

The shapes in §5 come from importer source code, one real file from 2024 and write-ups of 2026 builds. **This Mac has no Arc data folder, so the reader has not been run against a real one.** Specifically unverified:

- Any file written by a 2026 build of Arc, or by Arc for Windows. One Windows exporter repairs unescaped backslashes before parsing, which suggests some Windows files are not valid JSON; this reader does not repair them, and reports the file as unreadable.
- Whether a pinned tab's saved address changes when the tab wanders.
- Link rules with more than one condition.

A copy of a real Arc folder closes this (000 §13.1). The first run against one is expected to find something.

## 9. Follow-ups, in order

1. **Run against a real Arc folder**, macOS and Windows, and fix what it finds.
2. **Boosts 2.0**: read Arc's database cache for Boost documents (their CSS, script and hidden elements) and write them as Tweaks.
3. **Favicons**: Arc caches one per sidebar row; carry them across so Sites have icons before they are first opened.
4. **The import screen** (W2): read the portable file into Spaces, Groups, Sites, Favourites and link rules, with a preview and an undo.

## 10. Done checklist

- [x] This plan.
- [x] `wt-import`: the portable format and the Arc reader.
- [x] `webtiger import arc`.
- [x] Fixtures and the tests of §7 (36 tests: the whole synthetic folder against its expected file; one test per rule of §5; every value of the fixture replaced in turn by each kind of JSON value, about 5,900 broken files, none of which may panic the reader).
- [ ] Run against a real Arc folder (needs the folder; §8).

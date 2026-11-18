# Arc import fixtures

`full/` is a whole Arc data folder, **written by hand**. Nothing in it came from a real Arc installation: every id, address and title is made up, and every host is under `example.com`. Its shapes follow what `docs/features/002-arc-import.md` §5 lists, including the awkward ones (a Little Arc container before the main one, the two container lists in opposite orders, children out of flat-array order, a dangling id, a cycle, adopted children, both spellings of the default profile).

`full.expected.json` is what `webtiger import arc` must write for it. To accept a deliberate change, run the importer's tests with `WEBTIGER_UPDATE_FIXTURES=1` and review the diff.

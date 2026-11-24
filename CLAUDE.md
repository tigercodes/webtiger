# WebTiger

An open-source browser: one Rust core, native shells (Swift on Apple, C++/WinRT on Windows, Kotlin on Android), each system's own web engine. The plan is `docs/features/000-master-plan.md`; what is built so far is `docs/features/001-foundations.md` (and 002 the Arc importer, 003 the Android shell, 004 the design catalogue). W1's features start at 005 (tabs). **This repository is public: its documents and comments must stand alone and never point into a private repository.**

Two words, kept apart everywhere: the **core** is the Rust side; an **engine** is a web engine (WebKit, Chromium).

## Commands

```sh
Tools/test.sh lint     # generated code fresh; rustfmt, clippy -D warnings; SwiftFormat, SwiftLint
Tools/test.sh core     # cargo test (Core/)
Tools/test.sh unit     # Swift package tests on macOS: the real core, real WKWebView
Tools/test.sh ios      # the same on the iOS simulator
Tools/test.sh build    # the apps, macOS and iOS simulator
Tools/test.sh ui       # the apps' UI tests in fake mode (`ui-mac` and `ui-ios` are its halves)
Tools/test.sh all      # everything above, and `android` where a JDK and the Android SDK are found
Tools/test.sh android  # the Android shell: JVM tests against the real core, its pictures, and the debug APK
Tools/test.sh android-device  # the Android shell on a device: adapter suite, isolation, the window (not in `all`)
```

- Fast loop for core work: `cd Core && cargo test -p <crate>`.
- After changing anything under `Core/crates` that the Apple shell uses: `Tools/apple/xcframework.sh` (`--mac` is quicker). `Tools/test.sh` rebuilds it when the core's sources are newer.
- After changing `Core/proto`: `Tools/proto/generate.sh`. After `Design/tokens.json`: `Tools/design/tokens.py` (Swift and Kotlin). After `Localization/strings.json`: `Tools/strings/generate.sh` (Apple's String Catalog and `Words.generated.swift`, Android's `strings.xml`, Windows' `Resources.resw`; `Core/tools/strings-gen`, 040). After `Fixtures/testsite/site.json` (the pages every shell's suite loads): `Tools/testsite/site.py` (Swift and Kotlin). Generated files are checked in and `lint` fails when they are stale.
- The Xcode project is generated: `Tools/bin/xcodegen generate`. Never edit `WebTiger.xcodeproj`; edit `project.yml`.
- The Windows shell builds only on the Windows PC (`Tools/test.ps1`, not yet written). Do not write Windows code that has not been built there.
- The Android shell builds on a Mac with a JDK (21 or later; Android Studio's will do) and the Android SDK and NDK; `Tools/test.sh android` finds them. `Tools/test.sh android-device` needs a device: one that is attached, or an emulator, which `Tools/android/emulator.sh start` boots from a system image that is already installed (it downloads nothing). Without either, nothing that needs a device may be claimed as tested; and what ran on an emulator is not thereby tested on a phone.

## Layout

```
Core/crates/wt-core      ids, addresses (fix-up, registrable domain), navigation policy, clocks. No I/O
Core/crates/wt-store     SQLite: schema and migrations
Core/crates/wt-app       the models shells subscribe to, and the contract's dispatcher
Core/crates/wt-ffi       the C ABI, the core's thread, the fast path. The only crate with unsafe
Core/crates/wt-proto     the contract as prost types (generated)
Core/crates/wt-import    importers and the portable file format
Core/crates/wt-cli       the `webtiger` command line
Core/crates/wt-testkit   a fake shell with a scripted web engine
Core/proto               the contract. One Envelope; Request/Response/Event/HostRequest/HostResponse
Packages/                the Apple shell: WTContract (generated) → WTCoreClient → WTWebKit → WTUI; WTDesign
Apps/WebTiger            app targets: almost nothing in them
Android/core-engine      the contract (Wire, generated at build time), the core's library through JNI, CoreClient
Android/app              the System WebView adapter, the Compose window; src/androidTest runs on a device
```

## Rules

- **The core decides, the shell draws, the engine renders.** If a shell has to decide something about browsing, add a request to the contract; do not write the logic in Swift, C++ or Kotlin.
- **The core speaks codes; the shells speak words.** Errors and statuses cross the contract as `area.reason` codes. Every user-visible string is in `Localization/strings.json`; an error code's words are keyed `error.<code>`. One edit there changes every platform, at build time (040): Swift asks by name (`Text(Words.tabsNew)`, never a key's text), Kotlin by `R.string.tabs_new`. No shell asks the core for a word, and no word is fetched.
- **Nothing a person typed or visited goes in a log or an error**: no address, title, or typed text. Mark such contract fields `[(wt.redact) = true]`.
- **The address field never lies.** An address is shown as the page's only once that page has committed. Typed text is never executed (`javascript:` is a search). See `wt-core/src/address.rs` and the two rules at the top of `Core/proto/webtiger/core/v1/engine.proto`.
- Rust: no `unwrap`, `expect`, `panic` outside tests (clippy denies them). `unsafe` only in `wt-ffi`, each block with a `SAFETY` comment. No `RefCell` borrow held across an `await`.
- A request changes the models before its function returns; the future it returns is only the wait for the shell. Host requests are sent in order, at once (pipelined), never after an `await` on an earlier one.
- Swift 6, strict concurrency, warnings are errors. `InternalImportsByDefault` is on: write `public import` only for what public declarations use.
- No private platform APIs.
- Contract changes within v1 only add. Bump `CONTRACT_MINOR` in `wt-proto`, the version line in `envelope.proto` and `WT_CONTRACT_MINOR` in `webtiger_core.h` together; tests check they agree.
- A new dependency is a decision: say why in `Core/Cargo.toml` (every dependency is declared there once) and keep `Core/deny.toml` passing.
- British spelling in documents and comments.

## Testing

- Core behaviour is tested headless through `wt-testkit`'s `FakeShell`: it drives the whole contract in process against a scripted engine. Prefer that to mocking pieces.
- Engine adapters are tested by the **adapter suite** against a real engine and the test site, served from inside the test's process (`Packages/Tests/WTWebKitTests`; `Android/app/src/androidTest`, on a device). A new page goes in `Fixtures/testsite/site.json`, never in one shell's server. The same behaviours are the bar for every adapter; isolation between login sets is tested as a security property.
- Screens are tested against `ScriptedCore`, without the Rust core.
- **Snapshots** (`Packages/Tests/WTUITests/__Snapshots__/<system>/`, `Android/app/src/test/snapshots/`, 004): a screen's states as pictures, compared within a stated tolerance. `WEBTIGER_RECORD_SNAPSHOTS=1 Tools/test.sh unit` (or `ios`, or `android`) records them. Look at every picture you record before committing it: a blank one matches itself for ever.
- **Fake mode**: `WEBTIGER_FAKE=1` keeps the store in memory and makes ids deterministic; `WEBTIGER_FAKE_START=<address>` is opened in every new window. UI suites run in it. It keeps no session and opens no other app. On Apple's systems what pages keep (cookies, storage) is in memory too, so one test leaves nothing for the next; on Android it is on disk (the WebView has no login set in memory), and a device test does not count on a clean one. It fetches no page's icon unless a suite says `WEBTIGER_FAKE_ICONS=1`, and then from the loopback site only. Its clock moves a second each time it is read; `WEBTIGER_FAKE_CLOCK_STEP=<seconds>` (on Android `--ei webtiger.fake.clock_step`) makes that step longer, for a suite that must see a day pass (049).
- **A profile** (Apple): `WEBTIGER_PROFILE=<name>` runs the real app with its data in a folder of that name, apart from the usual one; `WEBTIGER_PROFILE_FRESH=1` empties it first. For a test that must outlive a quit (009), and for running the app without touching your own data.
- Test hostnames must sit under a real public suffix (`shop.example.com`, not `shop.example`): the address fix-up treats an unknown suffix as a search.
- No test may put a dialog of the system's on the screen, or start another app. XCUITest presses Cancel on a system dialog that is in its way, whoever it was for, and hangs on two: `Tools/test.sh ui` does not start the Mac's suite while one is there (`Tools/apple/system-dialogs.swift` counts them). Answer it by hand. `all` runs the Mac's suite before anything starts the iPhone simulator: a process of the simulator's own crashes now and then, and the Mac says so in such a dialog (which process, why, and what stopped it: `docs/features/007-outside-links.md` §8).
- Never dump a whole macOS accessibility tree in a UI test (`app.debugDescription`): it includes the system menu bar and its recent items. Query the app's own window.

## Definition of done

Code; unit tests; a UI or snapshot test if it touches a screen; the adapter suite if it touches an engine; the feature's `docs/features/NNN-*.md` updated; `Tools/test.sh all` green.

## Process

- One `docs/features/NNN-<feature>.md` per feature, written before the code: user story, design, tests, what is not verified, a done checklist.
- Conventional commits: `feat(guard): …`, `fix(tabs): …`, `test(adapter): …`, `docs: …`. Contributions are signed off (DCO).
- Report honestly in documents: what was measured, what was not, and what could not be verified on this machine.

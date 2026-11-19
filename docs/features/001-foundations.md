# 001 — Foundations (W0)

| | |
|---|---|
| **Status** | In progress. The core, the contract and the Apple shell are built and tested. The Android shell is built, tested on the JVM, and runs on an emulator, where its adapter passes the adapter suite ([003](003-android-shell.md)); no physical device has run it. The Windows shell, the design catalogue and several spikes are not started (§2) |
| **Phase** | W0 of [000](000-master-plan.md) §9 |
| **Decisions** | The eighteen decisions of 000 §11 were accepted by the owner on 2026-11-18, with macOS 26 as the minimum |
| **Last updated** | 2026-11-18 (Android on an emulator) |

## 1. What W0 is for

W0 proves the stack before any feature is built on it: one Rust core that every shell links, one contract between them, a shell on each platform that loads a page **through** the core in a login set of its own, and a way to test all of it without a person watching. It also answers the questions 000 could not (the spikes), and it makes Arc data safe before Arc shuts down ([002](002-arc-import.md)).

## 2. Where it stands

| Deliverable (000 §9, W0) | State | Verified by |
|---|---|---|
| Repository layout, licence, contributor documents | Done | — |
| Core skeleton: `wt-core`, `wt-store`, `wt-app`, `wt-proto`, `wt-testkit` | Done | 82 Rust tests |
| The contract (1.0) and `wt-ffi`: C ABI, core thread, fast path | Done | 21 Rust tests through the C functions; 8 Swift tests through the real library |
| Apple shell loads a page through the core, in the Space's login set | **Done on macOS and the iPhone simulator** | 28 tests against real WKWebView (macOS 27.2, iOS 27 simulator; one is skipped on iOS, where a test process has no service workers); 5 UI tests on macOS and on the iPhone 17, iPhone Duo and iPad Air simulators |
| Android shell loads a page through the core, in the Space's login set | **Done on an emulator** (Android 16, System WebView 133). Not run on a physical device ([003](003-android-shell.md)) | 152 JVM tests, 31 of them driving the real core through JNI; 66 on the emulator: the adapter suite against the real WebView (19, 10 for new windows and 5 for links for the system), the system's opener (2), isolation (12), sleep and wake (3), and the app's window in fake mode (15) |
| Windows shell | **Not started.** It cannot be built on this Mac; it needs the Windows PC | — |
| Fake mode | Done (core, Apple and Android shells) | The Apple apps' UI tests and the Android device suite's window tests run in it |
| Design tokens and strings, one source for every shell | Done for Apple (Swift, String Catalog) and Android (Kotlin, `strings.xml`) | `lint` fails when the generated files are stale |
| Design catalogue | **Done for Apple and Android** ([004](004-design-catalogue.md)); Windows not started | Apple: 6 snapshot tests on macOS and 7 on the iPhone simulator, over 54 pictures (light and dark), and a UI test that opens it and runs the accessibility audit. Android: 11 snapshot tests over 27 pictures drawn on the JVM, and device tests that open it and run Android's accessibility checks over it |
| `Tools/test.sh` | Done for the core, the Apple shell and the Android shell (`android`, which `all` runs where its tools are, and `android-device`) | Run in full on a Mac; `android-device` on the Mac with an emulator |
| CI | **Green**: four jobs on GitHub, all passing (run 44419083409, 2026-11-18). The core (Linux); the Android shell's JVM tests and APK (Linux, its first build off a Mac); the Apple shell's lint, unit tests and build; the package's tests on the iPhone simulator and the apps' UI tests. The Apple jobs run on the one image that has Xcode 27 (`xcode-27`, a preview, macOS 27.0.1, a screen of 1024 × 768 at one pixel a point: F19). The Android device suite is not run on GitHub | GitHub Actions; a failing UI test's screen recording and any picture that did not match are kept |
| Arc importer | Built, against published shapes and a synthetic folder. **Not run against a real Arc folder** | 36 Rust tests |
| Spikes Q1 to Q11 | Q2 and Q5 answered for WebKit and the System WebView; Q3 and Q7 answered as far as a Mac can; Q1, Q6, Q10 and Q11 started; Q4, Q8, Q9 not started (§10) | The answers that could be put to an engine are tests (§11.3, §11.4) |

**W0's exit** (000 §9) is: `Tools/test all` green, each app loads a page through the core, every question answered. Today that is met for the Mac and the iPhone, and for Android on an emulator; Windows is not started.

## 3. The repository, as built

```
Core/
  Cargo.toml              one workspace; every dependency declared once, with its reason
  crates/wt-core          ids, addresses, navigation policy, clocks. No I/O, no async
  crates/wt-store         SQLite: schema v1 (login_set, space), migrations
  crates/wt-app           the models, and the contract's dispatcher
  crates/wt-ffi           the C ABI, the core's thread, the fast path (the only crate with unsafe)
  crates/wt-proto         the contract as prost types (generated, checked in)
  crates/wt-import        the portable file and the Arc reader (002)
  crates/wt-cli           `webtiger`
  crates/wt-testkit       a fake shell with a scripted web engine
  proto/webtiger/core/v1  the contract: envelope, common, config, browser, engine, host
  tools/proto-gen         prost-build, in a workspace of its own
Packages/                 the Apple shell, one Swift package
  WTContract              the contract as Swift (generated, checked in)
  WTCoreClient            NativeCore (the real core) and ScriptedCore (for tests)
  WTWebKit                the WebKit engine adapter
  WTDesign                design tokens (generated)
  WTUI                    the window, its model, the strings, the design catalogue
Apps/WebTiger             the app targets and their UI tests
Design/tokens.json        → Tools/design/tokens.py
Localization/strings.json → Tools/strings/strings.py
Fixtures/import/arc       a synthetic Arc folder and its expected import
Fixtures/testsite         the test site's pages, once for every shell → Tools/testsite/site.py
Tools/                    bootstrap, proto, apple (xcframework), android (emulator), design,
                          strings, testsite, test.sh
```

## 4. The core

### 4.1 Shape

- **One thread per core** (`wt-ffi/src/core.rs`), running a current-thread tokio runtime. Envelopes arrive through a channel. When the channel is empty the thread sleeps: an idle core has no timer and wakes for nothing (000 §7).
- **The dispatcher is in `wt-app`, not behind the C ABI.** It takes envelopes and gives envelopes, and needs no thread of its own. So the tests and the command line drive the *whole* contract in process, on a paused clock (`wt-testkit`), and `wt-ffi` is left thin: a thread, handles, and borrowed bytes.
- **The view models are the contract's own types.** `wt-app` publishes `WindowSnapshot` as prost generates it. There is no second set of model types to keep in step.
- **A request changes the models before its function returns.** What it returns is only the wait for the shell. Requests therefore take effect in the order they arrive, and one that waits never holds up the next.
- **Host requests are pipelined.** Sending a tab somewhere asks for the web view and for the load at once, in that order, without waiting for the first answer. A shell must perform a web view's host requests in the order they arrive.
- **Snapshots are coalesced without a timer.** The thread takes everything that has arrived, then publishes each changed model once. Fifty progress reports that arrive together leave as one snapshot; a report that changes nothing publishes nothing.
- **Panics are fenced** at three levels: around each request's start, around each request's future, and around the thread. A panic is answered as `core.panic` and the core carries on.

### 4.2 The address field never lies (000 §5.13)

This is built in from the first commit, because it is the one thing that cannot be retrofitted.

- **Typed text is never executed.** `javascript:`, `data:`, `file:` and every scheme but `http`, `https` and `about:blank` become a search for those words (`wt-core/src/address.rs`).
- **The shown host is the real host**: always the ASCII form, so a look-alike international name reads as `xn--…`; never the part before an `@`.
- **A page cannot put an address in the field that it has not become.** The field shows where the *user* sent the tab until a page commits. A load a page starts by itself changes nothing until it commits. Both are tested in the core (`wt-app/tests/browsing.rs`) and at the engine (`WebKitAdapterTests`).
- **What typed text is** follows the Public Suffix List: `example.com` is an address, `readme.txt2` and `3.14` are searches, `intranet` is a search and `intranet/` is a host.

## 5. The contract, version 1.0

(W0's contract. W1 has added to it since: 1.1 is tabs, in [005](005-tabs.md) §4; 1.2 and 1.3 are a page that asks for a new window, in [006](006-new-windows.md) §4; 1.4 is a link for the system, in [007](007-outside-links.md) §4; 1.5 is history, in [008](008-history.md) §5.)

`Core/proto/webtiger/core/v1`. The C ABI is `Core/crates/wt-ffi/include/webtiger_core.h`.

```c
uint64_t wt_core_start(const uint8_t *config, size_t len, wt_sink sink, void *context);
int32_t  wt_core_send(uint64_t core, const uint8_t *envelope, size_t len);
int32_t  wt_core_stop(uint64_t core);
uint32_t wt_core_version(void);
int32_t  wt_guard_check(uint64_t core, const wt_request *request, wt_verdict *out);
int32_t  wt_nav_check(uint64_t core, const wt_navigation *navigation, wt_nav_verdict *out);
```

| Direction | Message | Notes |
|---|---|---|
| shell → core | `Ping`, `Subscribe`, `Unsubscribe` | The health check; a window's snapshots |
| shell → core | `OpenWindow`, `CloseWindow`, `SubmitAddress`, `TabCommand` | W0's whole browser |
| shell → core | `EngineEvent` | What a web view did. Sent with envelope id 0 |
| core → shell | `CoreStatus`, `Snapshot` | Events |
| core → shell | `WebViewCreate`, `WebViewLoad`, `WebViewCommand`, `WebViewDestroy`, `HostPing` | Host requests; each gets one answer |

Rules beyond 000 §5.5, each with a test:

- **A request sent with envelope id 0 is a notification** and gets no response. Engine events use it: a page reports dozens, and none needs an answer.
- **`Subscribed` is sent before the first snapshot**, so a shell knows a subscription before it hears from it.
- **Codes, never text.** A shell's error whose code is not `area.reason` is replaced by `host.failed`; an engine failure whose code is not one is replaced by `net.failed`.
- **A bad message is refused at once.** `wt_core_send` decodes on the caller's thread and returns `WT_MALFORMED`.
- **The fast path's lookup is lock-free**: running cores sit in a fixed table of atomically swapped slots, not a map behind a lock. `wt_nav_check` fails closed (CANCEL); `wt_guard_check` fails open (ALLOW), and until Guard exists (W4) it allows everything. It is here so every shell wires its request callback from the first build.

### 5.1 What each engine reports

A shell maps its engine's callbacks to the contract's events, so the core sees one order whatever the engine. The WebKit column is built and tested. **The WebView2 column is the plan, to be confirmed or corrected when that adapter is built. The Android column below is the first plan; it was corrected from Chromium's WebView source when the adapter was written, and 003 §5.1 has the mapping that was built and then observed on a real WebView** (in short: today's WebView calls `onPageStarted` at commit, so the commit is `doUpdateVisitedHistory`).

| Event | WKWebView (built) | WebView2 (planned) | Android WebView (planned) |
|---|---|---|---|
| `NavigationStarted` | `didStartProvisionalNavigation` | `NavigationStarting` | `onPageStarted` |
| `NavigationCommitted` | `didCommit` | `ContentLoading` | `onPageCommitVisible` |
| `NavigationFinished` | `didFinish` | `NavigationCompleted`, success | `onPageFinished` |
| `NavigationFailed` | `didFail…`, as a code; and a refusal by the policy, which WebKit itself does not report | `NavigationCompleted`, with its `WebErrorStatus` as a code | `onReceivedError` for the main frame |
| `AddressChanged` | `url` changing while nothing is loading; caught up at `didFinish` | `SourceChanged` without a new document | `doUpdateVisitedHistory` |
| `TitleChanged` | `title` | `DocumentTitleChanged` | `onReceivedTitle` |
| `ProgressChanged` | `estimatedProgress` | none; from the load's stages | `onProgressChanged` |
| `HistoryChanged` | `canGoBack`, `canGoForward` | `HistoryChanged` | read after each commit |
| `RendererGone` | `webViewWebContentProcessDidTerminate` | `ProcessFailed` | `onRenderProcessGone` |
| The navigation policy | `decidePolicyFor` → `wt_nav_check` | `NavigationStarting` → `wt_nav_check` | `shouldOverrideUrlLoading` → `wt_nav_check` |

Two rules every adapter keeps, found while building the first (and now in `engine.proto`):

1. **An address is reported as the page's only once that page has committed.** WKWebView's `url` changes when a load is *asked for*, when it starts, when it is redirected and when it fails. Reporting those would have let a page put an address in the field it had not become. The adapter reports a change only while nothing is loading.
2. **A navigation the shell refuses on the core's verdict is reported as ended** (`net.cancelled`), because the engine then reports nothing and the tab would wait for ever.

## 6. The Apple shell

`WTUI → WTWebKit → WTCoreClient → WTContract`, with `WTDesign` beside. The app targets hold one file.

- **`NativeCore`** wraps the C ABI. Its handle is immutable, so the fast path takes no lock on the Swift side either. One task answers host requests in order. **A request is handed to the core before `send` returns** (F20): what was done first, the core hears first.
- **`ShellHost`** answers the core's host requests: the System service itself ([007](007-outside-links.md)), the rest through the engine adapter.
- **`WebKitEngine`** answers the WebViews host service. A login set is `WKWebsiteDataStore(forIdentifier:)`; a private one is `.nonPersistent()`. The engine holds each login set's store itself, one for all of its web views (F17).
- **`BrowserWindowView`**: back, forward, reload or stop, the address field with its security mark, a progress line, and the page, an empty tab, or a failed page in words. ⌘L goes to the address field.
- The package builds **without** the Rust core too (`ScriptedCore`), so a screen can be worked on first.

**Verified:** the package's tests on macOS (93) and on the iPhone 17 simulator (93); the app's UI tests on macOS (11) and the iPhone 17 simulator (10); the five there were before tabs also on the iPhone Duo and iPad Air 11-inch simulators. The Duo's run is where the audit found the failed page's words cut short at the largest text sizes (004 §5.1). **Not yet looked at:** what the iPad's window should be (it is the phone's, larger), Stage Manager, multiple windows, the Duo folded.

## 7. Fake mode

`WEBTIGER_FAKE=1`: the store is in memory, the login set's id and the clock are fixed, nothing on the device is read or written. `WEBTIGER_FAKE_START=<address>` is opened in every new window. The engine stays real: UI tests load pages from a site on the loopback address, served from inside the test process. (The Mac's UI-test runner is sandboxed and needs the network-server entitlement to do that; `project.yml` gives it.)

The showcase of Spaces and pages, memory-pressure injection and a pinned clock (000 §8.10) arrive with the features that need them.

## 8. Testing

```sh
Tools/test.sh lint | core | unit | ios | build | ui | all
```

| Suite | Count | What it shows |
|---|---|---|
| `wt-core` | 30 | Address fix-up, the registrable domain, the navigation policy, ids |
| `wt-store` | 26 | Migrations, a newer store refused, SQLite's build flags (7); history: visits, the words to find them by, and forgetting them from the files (14, [008](008-history.md)); the tabs a window had (5, [009](009-session-restore.md)) |
| `wt-app` | 133 | A tab's life and the contract's rules, through `FakeShell` (45); several tabs (21, [005](005-tabs.md)); a page that asks for a new window (19, [006](006-new-windows.md)); a link for the system (12, [007](007-outside-links.md)); history (20, [008](008-history.md)); the tabs come back (16, [009](009-session-restore.md)) |
| `wt-ffi` | 21 | The header matches the exports; the core through its C functions, from many threads |
| `wt-proto` | 3 | The contract's version is the same in three places |
| `wt-import`, `wt-cli` | 36 | 002 |
| Swift package | 93 on the Mac and on the iPhone simulator | The real core through Swift (12); the WebKit adapter, isolation and restore (28), a page that asks for a new window (11, [006](006-new-windows.md)) and a link for the system (7, [007](007-outside-links.md)); the window's model (13) and the History list's (5, [008](008-history.md)); the shell's own host (5, and one more on the Mac); the design catalogue as pictures (6, and one more on the iPhone for the largest text size); `ScriptedCore` (5) |
| Android (JVM) | 152 | The real core through JNI from Kotlin (31); the adapter's rules, the error codes, the window's model and the History list's; the design catalogue as pictures (11) |
| Android (on an emulator) | 66 | The adapter suite against the real System WebView (19); a page that asks for a new window (10, [006](006-new-windows.md)); a link for the system and the system's opener (7, [007](007-outside-links.md)); isolation between login sets (12); sleep and wake (3); the app's window in fake mode (15), with Android's accessibility checks. `Tools/test.sh android-device`, not part of `all` |
| App UI tests | 11 on the Mac, 10 on the iPhone | A page loads through the core; typing an address; a failure in words; Apple's accessibility audit over the empty tab and the failed page; the design catalogue opens and passes the audit; tabs open, switch and close (005), and the Mac's ⌘T and ⌘W; a link that opens a window opens a tab, and a window refused its address leaves none (006); a link for the system leaves the page as it was and starts no other app (007); history lists what was visited, finds it, opens it again and forgets it (008) |

**The adapter suite** (`Packages/Tests/WTWebKitTests`) is the bar every engine adapter must clear (000 §8.3): event order, redirects, address changes, history, failures as codes, the navigation policy, host requests for web views that do not exist, release on destroy, and **isolation**: sign in under one login set and another sees no cookie, no local storage, no database, no cache, no service worker and no copy in the HTTP cache; the web views of one private login set share it, and it is forgotten with the last of them; a sign-in for the session outlives the last web view of its login set; a deleted login set leaves nothing.

`lint` also runs `buf lint` and `buf format` over the contract and `cargo deny check` over the core's dependencies (`Core/deny.toml`: permissive licences and MPL-2.0 only, crates.io only, no known advisories). Both tools are pinned by checksum in `Tools/bootstrap.sh` and both pass.

**Not yet in place:** fuzzing and performance tests, which are later phases'. Snapshot tests began with the design catalogue (004).

## 9. Decisions taken while building

Small ones, each made where the plan was silent or wrong. None changes a decision of 000 §11.

| # | Decision | Why |
|---|---|---|
| F1 | `psl` (the list compiled into code) instead of `publicsuffix` (000 §5.1) | Nothing is parsed at start-up |
| F2 | Address fix-up lives in `wt-core` for now; `wt-omnibox` arrives with suggestions (W1) | No crate with one function in it |
| F3 | Searches go to DuckDuckGo until the first-run chooser exists (W1, WT-D17) | It needs no account and no agreement. A placeholder, not a product decision |
| F4 | `file:` addresses and top-level `data:` navigations are refused; other schemes are for the system, which W0 only refuses | WebTiger opens no local files yet; the System host service is W1 ([007](007-outside-links.md)) |
| F5 | A page that asks for a new window gets its address loaded in the same tab | W0 has one tab. Replaced by [006](006-new-windows.md): it opens in a tab of its own |
| F6 | No envelope is logged. Fields are marked `(wt.redact)` already; the generated redactor arrives with logging | Nothing logged, nothing leaked |
| F7 | The store has one connection, held by the dispatcher | The one-writer, many-readers arrangement is for W1's load |
| F8 | The contract's C code for Windows (nanopb) is generated when the Windows shell is built; Android generates Kotlin at build time with Wire | No generated code without something that compiles it |
| F9 | Debug builds of the Mac app are unsandboxed and ad-hoc signed | Entitlements and the sandbox are settled with distribution (WT-D14) and the Chromium pack (Q1) |
| F10 | The design tokens' palette is provisional | The brand is not settled |
| F11 | On the Mac, clicking the address field places the caret; ⌘L selects the whole address | A SwiftUI text field cannot select on click. The Mac's address field becomes AppKit in W1 (000 §5.1) |
| F12 | The test site is one definition, served by each suite from inside its own process | The pages are `Fixtures/testsite/site.json`; `Tools/testsite/site.py` writes them into the Swift and the Kotlin suite's loopback server (checked in, and checked by `lint`) and can serve them by hand. A server beside the suites was the plan (000 §8.2) and would not do: the Mac's UI-test runner is sandboxed, and an emulator sees the Mac at an address that is not a secure context, so the Cache API and service workers would be missing from the very pages that test them |
| F13 | WKWebView's default user agent is kept | Sites that sniff for Safari are a W1 matter |
| F15 | A finish reported after a failure does not clear the failure | Android's WebView finishes an error page of its own after a load fails; found while building the Android adapter, fixed in the core with a test |
| F16 | An address field that has the keyboard still follows the page until something is typed in it; and a window that opens on a page gives the page the keyboard | Found on Android, where a new window's field takes the focus by itself when a keyboard was used last, and then showed nothing over a loaded page (003 §6). The same happened to the Mac app on GitHub's runner, where its UI test failed on an empty field. Both rules are in both shells |
| F17 | The WebKit adapter holds each login set's data store, and deletes a login set by clearing it through that store first | Q5 found that a cookie with no expiry lives in its store's memory and goes with the store: left to the web views, a Space whose last tab slept could wake signed out. A private login set's web views each got a store of their own, so a private window's second tab would not have been signed in where its first was. And deleting a store's files straight after letting the store go left its cookies, for a moment, to a store made anew for the same login set (§10, Q2) |
| F18 | A window keeps a snapshot that arrives before it knows which subscription is its own | The core sends a subscription's first snapshot at once, with its answer (§5), and a shell takes answers and events off the core by different paths. The Android session was built to keep such a snapshot; the Apple one dropped it, and a window would then have shown nothing until the page next changed |
| F19 | Apple's accessibility audit does not judge contrast on a screen of one pixel a point, nor a fault it gives no element for | The audit measures contrast from the screen. On GitHub's Mac (1024 × 768, one pixel a point) small text is drawn in greys that never reach its colour, and plain black on white was faulted; the recording kept from its run shows the screen. The same windows pass on a Mac whose points have four pixels. And macOS 27.0, which that runner has, reports a mismatch in the hierarchy with no element to name for every state of the window, where 27.2 reports none: a fault that cannot be named cannot be mended |
| F14 | Words a person must read are in the primary colour, the retry button is neutral, and the light accent is `#C2410C` | Apple's accessibility audit failed the system's secondary grey, a tinted button label and the first accent (4.3:1 on white; the new one is 5.2:1). The audit now runs in the UI tests on both platforms |
| F20 | The Swift client sends a request before `send` returns; only the answer is waited for | The window's model used to start a task for each gesture and send from inside it. Two tasks may run in either order, and against the real core sixty requests sent in a row arrived shuffled, every time it was tried: a Stop could overtake the load it was meant to stop. Found when a test of the tab strip's three requests failed once on the iPhone simulator. Android's client was already in order: its requests are sent from coroutines the main thread starts one after the other |
| F21 | The Android client names every kind of answer the contract has | It named the four it knew when it was written, and took any other for no answer at all (`response.empty`). A new tab's answer came back so from the day tabs were built, and nobody waited for it; a page of history did, and the History list on the emulator was blank. Each kind the contract gains is added, and a test through the real core asks for each |

## 10. Spikes

| # | Question | State | Answer so far |
|---|---|---|---|
| Q1 | CEF on the Mac | **Pack size answered; the pack's shape read from CEF's own sources; nothing built or run** | §11.5. Not yet: in-process loading, memory once loaded, crash containment, the hardened runtime. They need the distribution built and run, which was not done |
| Q2 | Isolation on all four engines | **WebKit answered; Android answered for WebView 133** | On macOS 27.2 and the iOS 27 simulator, two `WKWebsiteDataStore(forIdentifier:)` stores share no cookie, no local storage, no IndexedDB database, no Cache API cache and (on macOS, where a test process has them) no service worker; a non-persistent store is gone with its web view; `remove(forIdentifier:)` leaves nothing, and refuses while a web view still uses the store (it succeeds a moment after the last one is released). **Android** (System WebView 133 on an Android 16 emulator, 003 §5.2 and §8.1): two WebView profiles share no cookie, no local storage, no IndexedDB database, no Cache API cache and no service worker, and nothing reaches the default profile. A profile that has been used cannot be deleted until the app starts again; its data can be cleared at once, completely only with `WebStorageCompat.deleteBrowsingData`. **Both:** the HTTP cache is each login set's own, and its web views share it: a script that may be kept is fetched once for a login set and again for another. **Deleting on WebKit has a trap:** WebKit lets a store's session go a moment after the store, and a store made anew for the same login set in that moment is given the old one's cookies, though the files are gone. The adapter clears a login set through its store before deleting it (F17), and waits no more than twenty seconds for WebKit to say the clearing is done: WebKit sets no limit, and on GitHub's runner a test once hung for its whole ten minutes, by the timings in that clearing. **Not yet tested:** permissions; the cost of each extra store; the lowest Android WebView that has profiles; WebView2 and CEF |
| Q3 | WebKit rule lists: compile time, chunk size | **Answered for this Mac** | §11.1. Not yet: the oldest supported iPhone; exceptions across chunks; the corpus difference |
| Q4 | Request callback cost | Not started; needs the Windows and Android shells | — |
| Q5 | Sleep and restore fidelity | **WebKit and Android answered** | §11.3. Not yet: cookies with no expiry across a restart of the app (the case 000 §4.4 flags); WebView2 and CEF |
| Q6 | Baselines for the budgets | **Started** | §11 |
| Q7 | QuickJS size and memory | **Answered for macOS and the iOS simulator** | §11.2. Not yet: Android, Windows Arm64, a WebAssembly wall |
| Q8 | Passwords and AutoFill | Not started; needs a person at the Mac, and the entitlements | One fact recorded on the way: in a Mac process **without** the browser entitlement, a WKWebView page on a secure context has service workers, the Notification interface, the Credential Management interface and storage estimates, and has **no** `PushManager` and no WebUSB (macOS 27.2). That confirms 000 §4.7's "no web push" for the Mac. **On the iOS 27 simulator the same page has no service workers and no Notification interface either.** On iPhone and iPad, WebKit gives service workers only to an app that holds Apple's browser entitlement, so that entitlement is not a nicety for being the default browser: without it a large part of the modern web does not work (§13) |
| Q9 | WinUI 3 with WebView2 profiles | Not started; needs the Windows PC | — |
| Q10 | iPhone Duo | **Started** | The app runs on the iPhone Duo simulator and its five UI tests pass there, the accessibility audit among them (§11). Custom chrome with reserved regions and vertical bars is not built |
| Q11 | DRM video; WebUSB and friends | **Started**: what each engine offers a page is recorded; nothing was played | §11.4. Not yet: a protected video actually playing in WKWebView; Widevine in the System WebView on a real device; WebUSB, WebSerial and WebHID in WebView2 (the Windows PC) |

## 11. First measurements

Release build, this Mac (Apple silicon, 36 GB; **not** the 8 GB reference machine of 000 §7), macOS 27.2, 2026-11-18. Two runs each. The start-up figure is coarse: the window was polled for through accessibility.

| Measure | Value | Budget (000 §7) |
|---|---|---|
| Mac app bundle | 8.1 MB | ≤ 25 MB download |
| The core's static library, one architecture, before the linker strips it | 9.6 MB | — |
| The app's own process, empty tab | 28 to 29 MB footprint | ≤ 120 MB, standard session |
| The app's own process, one page loaded | 36 MB footprint | ≤ 120 MB, standard session |
| Process start to a window on screen | 210 to 370 ms | ≤ 500 ms |
| Rust core build for four Apple targets | 56 s | — |

These are a single tab with no Spaces, no history and no Guard. They say the starting point is light. They do not say the budgets are met.

### 11.1 Q3: WebKit rule lists

EasyList and EasyPrivacy as published on 2026-11-18 (135,478 lines), converted with `adblock-rust` 0.13.6's own converter (`content-blocking` feature), then compiled with `WKContentRuleListStore` in a command-line process on this Mac.

| Step | Result |
|---|---|
| Conversion to WebKit rules | 130,940 rules from 130,796 filters, in about 50 ms; 14.0 MB of JSON |
| Rules WebKit refused | None: every chunk compiled |
| Compile time, all rules | 2.2 s, whatever the chunk size |
| Compiled size on disk | 44 to 46 MB |

| Chunk size | Chunks | Slowest chunk | Peak memory of the compiling process |
|---|---|---|---|
| 30,000 | 5 | 0.55 s | 269 MB |
| 50,000 | 3 | 0.86 s | 292 MB |
| 150,000 (one list) | 1 | 2.17 s | 439 MB |

**What it says.** These two lists fit under WebKit's 150,000-rule ceiling today, with little room; the default set of 000 §5.8 will not. Chunking does not make compiling faster. It cuts the **peak memory**, which is what gets an app killed on a phone, and it lets a list that changed be recompiled alone. 30,000 is a reasonable start, and W4 should try smaller. The compiled lists are about 45 MB of cache on disk: not part of the download, but worth showing in "storage used". **Not measured:** an iPhone (the question asks for the oldest supported one), exceptions that must reach across chunks, and the difference in what is blocked against the request engine.

### 11.2 Q7: QuickJS

`rquickjs` 0.12.2 (QuickJS-NG), release build, one runtime with a full context per plugin as 000 §5.10 plans.

| Measure | macOS | iOS simulator |
|---|---|---|
| A whole stripped program that embeds it | 0.99 MB | — |
| Creating a runtime and context | 0.3 ms | 1.9 ms for the first; 100 in 7 ms |
| Memory of an idle runtime | 104 KB | 104 KB |
| A "hello" plugin loaded | +4 KB | +4 KB |
| 100 runtimes | 10.4 MB | 10.4 MB |
| An endless loop under a 50 ms limit | interrupted | interrupted |
| An allocation loop under an 8 MiB ceiling | stopped | stopped |

**What it says.** The plan's numbers hold: about 1 MB of code, about 100 KB per plugin that is awake, and both guard rails (a time limit per event, a memory ceiling) exist and work. **One correction:** `rquickjs` ships ready-made bindings for macOS, Windows (x64 and Arm64) and Linux, but **not for iOS or Android**. Those need its `bindgen` feature, which needs libclang when the core is built. It built and ran for the iOS simulator that way. **Not measured:** Android, Windows Arm64, a real iPhone, and what a WebAssembly wall around the runtime would cost.

### 11.3 Q5: what a sleeping tab keeps

A tab that sleeps has no web view (000 §5.7). What it wakes with is what the engine hands over when the web view is let go, and takes back in a new one: WebKit's `interactionState`, the System WebView's `saveState`. Both were put to it on real engines (`WebKitRestoreTests`, macOS 27.2 and the iOS 27 simulator; `SystemWebViewRestoreTest`, WebView 133 on an Android 16 emulator), with the first web view destroyed before the second was made.

| | WebKit | System WebView |
|---|---|---|
| The state | Plain data, 1 KB for three pages of history | One byte array, 2.4 KB for the same |
| The page, its title, back and forward | Yes | Yes |
| The signed-in state | Yes | Yes |
| The scroll position | Yes, to the pixel | Yes, **once the engine has recorded it**: about a second after the last scroll. Put to sleep 0.6 s after, the tab woke at the top |
| Text typed into a form | **No.** Waiting longer before the sleep changes nothing | Yes, on the same condition as the scroll position |
| A sign-in for the session (a cookie with no expiry), while the app runs | Kept **only while something holds the login set's data store**. With no holder it goes when WebKit lets the session go, a moment later that cannot be counted on | Kept: a profile stays loaded for the life of the process |

**What it says.** Sleeping a tab is cheap and faithful on both engines: the budget of 250 KB a tab (000 §7) is a hundred times what the state needs. Two things follow for W3. On WebKit, **unsaved form text is lost by sleeping**, so "has unsaved form text" must stay among the reasons a tab is kept awake (000 §5.7), and the adapter must hold each login set's data store, which it now does (§9, F17): otherwise the last tab of a Space, sleeping, could sign the Space out. On Android, a tab put to sleep under memory pressure within a second of the user's last scroll or keystroke wakes without it; a tab that sleeps after minutes in the background does not have that problem.

### 11.4 Q8 and Q11: what each engine offers a page

Asked of a page on the loopback address (a secure context) in each adapter's web view, by a test that records the answers and asserts nothing about them (`testWhatWebKitOffersWithoutTheBrowserEntitlement`, `whatTheSystemWebViewOffersAPage`). The Mac's process holds no browser entitlement. 2026-11-18.

| A page is offered | WKWebView, macOS 27.2 | System WebView 133, Android 16 emulator |
|---|---|---|
| Service workers | Yes (on the iOS 27 simulator: **no**, without the browser entitlement) | Yes |
| The Notification interface | Yes (iOS simulator: no) | **No** |
| Web push (`PushManager`) | **No** | **No** |
| WebUSB, WebSerial, WebHID, Web Bluetooth | **None** | **None** |
| Credential Management, storage estimates | Yes | Yes |
| A DRM key system | **FairPlay: yes** (`com.apple.fps` is granted to the page). Widevine: no | Widevine: **refused here**, which settles nothing (below) |
| Clear Key | Yes | Yes |

**What it says.** 000 §4.7's rows hold where they could be checked: no web push in either engine, none of the device interfaces in either, and no web notifications at all on Android. **FairPlay is offered to a page in a WKWebView of an ordinary app**, which is the first half of 000 §4.4's "DRM video: FairPlay (W0 verifies)"; the second half, a protected stream playing, needs a stream and its licence server and was not tried. **Widevine on Android is not answered**: the WebView asks the app before it grants a page protected media (`onPermissionRequest`), the adapter does not answer permission requests yet (W1), and an emulator image may have no Widevine to offer in any case.

### 11.5 Q1: the Chromium pack, as far as reading goes

CEF 156.0.31 (Chromium 156.0.8078.100), the *minimal* distribution for Apple silicon, downloaded and checked against its published SHA-1 on 2026-11-19. **It was read, not built and not run**: everything below is from its files, its headers and the sources of the loader it ships.

| | |
|---|---|
| To download | 134.9 MB compressed (the 135 MB of 000 §13) |
| On disk | 352 MB, of which the framework is 340 MB: its one library 240 MB, its resources 84 MB (220 folders of localised strings among them), its other libraries 16 MB |
| Its signature | Ad hoc, by the linker, with no team. A pack WebTiger ships is signed by WebTiger |
| Where an app may load it from | Anywhere: `cef_load_library(path)` takes a path, and `CefSettings` has `framework_dir_path`, `browser_subprocess_path` and `main_bundle_path` for an app whose framework and helpers are not in its bundle |
| Where its helpers look for it | Three folders up from their own executable (`../../..`), for the framework and for the sandbox's library. That is fixed in the loader CEF ships |
| What it asks of the app | The app's `NSApplication` must implement CEF's `CefAppProtocol`. To live in an app that runs its own loop, `external_message_pump` and `CefDoMessageLoopWork` |

**What it says.** A pack outside the app bundle (000 §4.6, WT-D3) is a shape CEF provides for, not one to be forced: **the pack is a folder with the framework and the helper apps side by side**, which is where the helpers look, and the app names that folder in three settings. Two things follow that the plan had not said. The pack must hold WebTiger's own helper apps as well as CEF's framework, so it is built and signed here, not fetched from CEF. And WebTiger's application object must speak CEF's protocol from its launch, though no Chromium is loaded then: a few methods, compiled in always.

**Not answered, and not answerable by reading:** whether the stock loaders are in fact content with that folder; what Chromium costs in memory once loaded, in WebTiger's process and in its helpers'; what a crash in each process takes down; and the hardened runtime. For the last, this Mac has no signing identity at all, so library validation against a team's signature cannot be tried here even when the rest is.

**iPhone Duo.** The app builds for the iPhone Duo simulator and its five UI tests pass there, on the display the simulator starts with. Nothing in the app yet adapts to the fold, and no test folds the device: that is W5.

## 12. What is not done, in order

1. ~~The Android shell on a device.~~ Done on an emulator: the first launch, the adapter suite against a real WebView, and isolation between profiles (003 §8.1). **A physical device, a foldable and other WebView versions** are still to come (003 §8.2).
2. **The Windows shell**, on the Windows PC: C++/WinRT, WinUI 3, WebView2 with a profile per login set, nanopb, the adapter suite, `Tools/test.ps1`.
3. ~~`Tools/testsite`, shared by all three adapter suites.~~ Done for the two that exist (F12); the Windows suite will read the same definition. The plan's longer list (000 §8.2: several hostnames, third-party trackers, downloads, uploads, permissions, media) arrives with the features that need each page.
4. ~~The design catalogue, snapshot-tested.~~ Done for the Apple and the Android shell (004); the Windows one comes with that shell.
5. **The spikes**, starting with those a Mac can answer alone. Q2 and Q5 are answered for WebKit and for the System WebView as far as §10 says. Q1 is read (§11.5) and waits to be built and run.
6. **The Arc importer against a real Arc folder** (002 §8).

## 13. For the owner

- **Apple's browser entitlement, requested now** (000 §13.5). On iPhone and iPad a WKWebView has no service workers without it (§10, Q8), so it gates W1, not just the default-browser prompt. Apple grants it on request and it can take time.
- **A copy of a real Arc data folder** is the one thing that turns the importer from "built to the published shapes" into "known to work". Arc stops on 22 December.
- ~~An Android emulator image.~~ A second Mac has one, and the Android shell runs there (003 §8.1). **A phone and a foldable** (or its emulator image) are what Android still wants.
- **The contact address** for `SECURITY.md` and `CODE_OF_CONDUCT.md`. Both currently point at GitHub's private reporting and say the address is set when the repository goes public.
- **Everything is committed and pushed** (`main` at `tigercodes/webtiger`, 2026-11-18): the code first, then these documents once the owner had reviewed them.

## 14. Done checklist

- [x] Repository, licence (MPL-2.0), README, contributing (DCO), security policy, trademark note, `CLAUDE.md`.
- [x] Core skeleton and contract 1.0; `Tools/proto/generate.sh` for Rust and Swift.
- [x] `wt-ffi`: C ABI, header checked against the exports, fast path.
- [x] Apple shell: a page loads through the core on macOS and the iPhone simulator, in fake mode and live.
- [x] The WebKit adapter passes the adapter suite, isolation included.
- [x] Design tokens and strings generated for Apple.
- [x] `Tools/test.sh all` green on this Mac for the core and the Apple shell.
- [x] Arc importer and `webtiger import arc` (002).
- [x] Android shell built and tested on the JVM (003).
- [x] Android shell run on an emulator, with the adapter suite and isolation (003 §8.1).
- [ ] Android shell on a physical device.
- [ ] Windows shell.
- [x] Design catalogue for the Apple and the Android shell, snapshot-tested (004).
- [ ] Design catalogue for Windows.
- [x] Spikes Q2 and Q5 for WebKit and the System WebView (§10, §11.3), but for permissions, the cost of a store, and session cookies across a restart.
- [ ] Spikes Q1, Q4, Q8, Q9, Q11; the rest of Q3, Q6, Q7, Q10; Q2 and Q5 for WebView2 and CEF.
- [x] CI green on GitHub (run 44419083409). The Android device suite is not part of it.

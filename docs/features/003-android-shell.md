# 003 — The Android shell (W0)

| | |
|---|---|
| **Status** | Built as far as this Mac allows. The app compiles, its debug APK is assembled with the core for both ABIs, and 102 tests pass on the JVM, 28 of them driving the real core through JNI. **It has never run on a device or an emulator** (§8): the adapter against a real WebView, isolation between login sets and the window on a screen are all unverified |
| **Phase** | W0 of [000](000-master-plan.md) §9; item 1 of [001](001-foundations.md) §12 |
| **Stack** | 000 §5.1: Kotlin, Jetpack Compose, Material 3, `androidx.webkit`, Wire, JNI through `wt-ffi` |
| **Last updated** | 2026-11-18 |

## 1. What it is for

W0 asks each shell to load a page **through** the core, in a login set of its own, and to be testable without a person watching. This is the Android half: the core's library with its JNI glue, a Kotlin client over it, the System WebView behind the WebViews host service with one WebView profile per login set, and one Compose window.

The Mac that built it has the Android SDK and NDK and **no emulator, no system image and no device**. So the work was shaped to put as much as possible where it can be tested here: everything the adapter decides about a WebView's callbacks is in one class with no Android in it, and the window is tested over the real core with only the WebView itself replaced.

## 2. Where it stands

| Deliverable | State | Verified by |
|---|---|---|
| JNI glue, `wt-ffi/src/android.rs` (the `jni` feature) | Done | 23 Kotlin tests through the real library, built for this Mac; clippy with the feature on |
| Gradle project, `Android/`: `app` and `core-engine` | Done | `./gradlew :app:assembleDebug` |
| The contract as Kotlin (Wire, at build time, from `Core/proto`) | Done | Compiled; used by every test |
| `CoreClient`, `NativeCore`, `ScriptedCore` | Done | 28 tests |
| The core for `arm64-v8a` and `x86_64`, built with the NDK and packed in the APK | Done | Both libraries are in the APK, aligned for 16 KB pages. **Never loaded on Android** |
| System WebView adapter: WebViews host service, a profile per login set | Written | Its logic: 43 JVM tests without a WebView. **Against a real WebView: nothing** |
| Event mapping (001 §5.1, Android column) | Corrected (§5.1) from the WebView's source | 26 tests of the rules. **Not confirmed on a device** |
| Compose window: address field, back, forward, reload or stop, progress, three content states | Done | 8 tests drawn on Robolectric; 10 of its model. **Never seen on a screen** |
| Strings and tokens from the shared sources | Done | `--check` covers them; the Apple outputs are byte for byte what they were |
| Fake mode | Done | 6 tests |
| `Tools/test.sh android` | Done, not yet part of `all` | Run in full on this Mac |
| The adapter suite (001 §8) against a real WebView, isolation included | **Not written**: it could not have been run | — |
| An app UI test on a device | **Not written**, for the same reason | — |

## 3. How it is built

```
Core/crates/wt-ffi/src/android.rs   the JNI exports, behind the `jni` feature
Android/
  gradle/libs.versions.toml         every version, pinned
  core-engine/                      the contract (Wire), the core's library (cargo + NDK),
                                    CoreClient, NativeCore, ScriptedCore
  app/                              the engine adapter (webview/), the window (ui/),
                                    the tokens (design/), the activity
```

```sh
Tools/test.sh android     # clippy for the glue; the JVM tests; the debug APK
cd Android && ./gradlew :core-engine:testDebugUnitTest :app:testDebugUnitTest
cd Android && ./gradlew :app:assembleDebug
```

Gradle needs `JAVA_HOME` (a JDK 21) and `ANDROID_HOME`, or `Android/local.properties`. `Tools/test.sh android` finds Homebrew's JDK and the SDK in their usual places when neither is set. It installs nothing: `android.builder.sdkDownload=false` makes a missing platform or NDK an error.

| | Version | Note |
|---|---|---|
| Gradle | 9.9.0 | The wrapper checks the distribution's SHA-256; the wrapper jar is Gradle's own (its checksum is the published one) |
| Android Gradle Plugin | 9.5.1 | Kotlin is built in |
| Kotlin (Compose compiler plugin) | 2.4.21 | |
| Compose BOM | **2026.06.01** (Compose 1.11.4, Material 3 1.4.0) | The newest that compiles against android-36. Compose 1.12 needs android-37, which this Mac does not have |
| `androidx.webkit` | 1.18.0 (2026-10-21) | Profiles (`MULTI_PROFILE`) |
| `androidx.activity:activity-compose` | 1.13.0 | |
| kotlinx.coroutines | 1.11.0 | `-android` in the app, for `Dispatchers.Main` |
| Wire | 7.1.2 | Generates at build time (001 §9, F8): no generated Kotlin is checked in |
| JUnit, Robolectric, AndroidX Test core | 4.13.2, 4.17, 1.7.0 | Test only |
| compileSdk, targetSdk / minSdk | 36 / 29 (Android 10) | |
| NDK | 28.2.13676358 | Its clang builds the core for API 29 |

**The core for Android.** `:core-engine:cargoBuild<Variant>` runs `cargo build -p wt-ffi --features jni --release --target <target>` for `aarch64-linux-android` and `x86_64-linux-android` with the NDK's clang as linker and C compiler (SQLite), into `Core/target/android`, and hands the two libraries to the build as `jniLibs`. The app names the two ABIs, so no library can put it on a 32-bit device. **The core for the tests.** `:core-engine:cargoBuildHost` builds the same crate with the same feature for this machine into `Core/target/jvm`; the unit tests of both modules load it. Cargo decides what is stale.

Measured here (Apple silicon, 2026-11-18), from nothing: `Tools/test.sh android` takes 48 s, of which the core for both ABIs in release is 32 s. `libwebtiger_core.so` is 3.2 MB per ABI in the APK. The debug APK is 37 MB, unshrunk; it says nothing about the budget (000 §7), which is for a release build.

## 4. The glue and the client

### 4.1 `wt-ffi/src/android.rs`

Six exports for the Kotlin object `CoreLibrary`, one for each function of the C ABI: `version`, `start(config, listener)`, `send`, `stop`, `navCheck`, `guardCheck`.

- `start` copies the configuration and keeps the listener as a JNI global reference beside the VM. Both are the sink's context, freed when `stop` has returned OK for the core, or at once when the core refuses to start. The call into the core is made with no `Env` held, so a refusal, which the core delivers before `start` returns, reaches the listener the same way as everything else.
- The sink attaches the core's thread to the VM for the thread's life (it is detached when the thread ends, which is when the core stops), and calls `onEnvelope([B)V` inside a local frame of its own. No local reference outlives a call. What the listener throws is cleared there.
- **Only the caller whose `stop` returned OK frees the listener.** Two threads can stop one core; the loser is told `WT_UNKNOWN_CORE` while the core's thread may still be inside the sink.
- **Nothing throws with words of its own, and nothing is logged.** A failure is a status, a zero handle, or the fast path's safe verdict (`CANCEL` for a navigation, `ALLOW` for a request). An exception the VM itself raised (out of memory) is left pending for Kotlin.
- Every export is fenced. Addresses cross as Java strings and are converted from the VM's modified UTF-8, so an address outside the Basic Multilingual Plane arrives whole (tested).
- A message longer than `WT_MAX_MESSAGE_LEN` is refused without being copied.

### 4.2 `core-engine`

The same shape as the Apple shell's `WTCoreClient` (001 §6):

- **`CoreClient`**: `request` (suspends for the answer; an error answer is thrown as `CoreError.Refused` with its code), `report` (a notification), `events` (a flow, one collector), `navigationVerdict`, and `stop`. One addition for Android: **`requestVerdict`**, because the System WebView asks about every request a page makes (`wt_guard_check`), which WKWebView does not.
- **`NativeCore`**: its handle is immutable, so the fast path takes no lock. One coroutine, in the context it is given (the main thread, for a WebView), answers host requests one after the other, in the order the core made them. A host that throws is answered `host.failed`: the exception's words never travel. `stop` is safe from the core's own thread (it hands the stop to another).
- **`ScriptedCore`**: records requests and reports, answers from a script, emits what a test tells it to.
- The contract's constants in Kotlin (the version, the statuses, the fast path's values, the resource types) are checked against `webtiger_core.h` by a test, as the Rust ones are (001 §5).

## 5. The System WebView adapter

`app/…/webview/`:

| File | What |
|---|---|
| `SystemWebViewEngine.kt` | The WebViews host service: which web view, which login set, which code when it cannot. No WebView in it |
| `PageTracker.kt` | What a WebView's callbacks mean as the contract's events. No Android in it |
| `WebViewPage.kt` | The WebView itself, its settings, and the two clients that forward its callbacks to the tracker |
| `NetErrors.kt` | The WebView's error numbers as the contract's codes |

### 5.1 What the WebView reports: the Android column of 001 §5.1, corrected

001's column was a plan. It was checked against the WebView's own source (Chromium's `AwWebContentsObserver`, `AwContentsClientBridge`, `AwContentsClientCallbackHelper`, `AwComputedFlags` and `AwContentBrowserClient::ShouldOverrideUrlLoading`, as on `main` on 2026-11-18), because there was no device to check it on. Four rows were wrong:

| Event | 001's plan | As built | Why |
|---|---|---|---|
| `NavigationStarted` | `onPageStarted` | The first of: the adapter's own load or reload; `shouldOverrideUrlLoading` allowing a main-frame navigation; `onPageStarted`; the first `onProgressChanged` below 100 | **A current WebView calls `onPageStarted` when a page commits**, not when its load starts. An older one calls it at the start of a load the app asked for, and again for each redirect. Neither is "a load began" |
| `NavigationCommitted` | `onPageCommitVisible` | `doUpdateVisitedHistory`, after one of the start signals | `onPageCommitVisible` waits for a frame to be drawn, which a web view that is not on screen may never do. `doUpdateVisitedHistory` is posted for every main-frame navigation that committed, in order with the other callbacks |
| `NavigationFinished` | `onPageFinished` | `onPageFinished` or `onProgressChanged(100)`, whichever comes first, for a load that committed | `onPageFinished` is not called for every load (a page restored whole has none; a stopped page may have none) and is called for loads that never committed |
| `NavigationFailed` | `onReceivedError` for the main frame | That; and `onReceivedSslError`, always answered with cancel, as `net.tls`; and `net.cancelled` for a load that ends without committing (stopped, a download, a 204) and for a refusal | The WebView never reports a cancelled load as an error: it reports a finished page |
| `AddressChanged` | `doUpdateVisitedHistory` | `doUpdateVisitedHistory` with no start signal before it | The same callback is the commit of a new document and of a fragment or History API change; what came before tells them apart |
| `TitleChanged` | `onReceivedTitle` | Confirmed. Dropped while the tab shows a failure | The engine's error page has a title of its own |
| `ProgressChanged` | `onProgressChanged` | Confirmed. Ignored while the engine loads its error page | |
| `HistoryChanged` | read after each commit | Confirmed: `canGoBack`, `canGoForward` read in `doUpdateVisitedHistory` | |
| `RendererGone` | `onRenderProcessGone` | Confirmed; returns true; the dead WebView is destroyed and another is made, in the same profile, at the next load | A WebView whose renderer died cannot be used again |
| The navigation policy | `shouldOverrideUrlLoading` → `wt_nav_check` | Confirmed, with limits: the WebView asks about navigations a page starts and about every redirect; **not** about a load the app asked for, a form's POST, or a step back or forward; in a frame only about addresses that are not http(s) | |

The two rules of `engine.proto` are kept, each with tests:

1. **An address is reported as the page's only from a commit.** `NavigationStarted` carries where a load is heading; `NavigationCommitted` and `AddressChanged` come only from `doUpdateVisitedHistory`. `WebView.getUrl()` is never read: it is the *visible* address, which for a load the app asked for is the one not yet committed.
2. **A navigation refused on the core's verdict is reported as `net.cancelled`** by the adapter. The WebView then reports a finished page for the refused address, which the tracker recognises and drops.

Other rules found on the way, all in `PageTracker` with a test each:

- **A failed load is followed by the engine's own error page**, which commits at the failed address and finishes like any page. The failure is reported once; the error page's commit is reported as a commit (the address field then shows the address that failed, as the engine's history does); its finish, title and progress are dropped. A `NavigationFinished` there would turn the failed tab into an idle one showing the engine's error page.
- **Trying again after a failure loads the address again** rather than reload whatever the engine is showing. The same after a renderer died.
- **Stop is reported by the adapter**, as cancelled or finished, because the WebView may say nothing after `stopLoading()`.
- A load that ended without committing and then commits after all (a stop that came too late; an HTTP error with nothing to show) is a new document, not an address change.

`androidx.webkit` 1.18 also has `NavigationListener` (`WebViewFeature.NAVIGATION_LISTENER`), which says started, redirected, completed, `didCommit()` and `isSameDocument()` outright. It needs a newer WebView than profiles do, so the callbacks above would remain as the fallback. It is the better source where it exists and was left for when a device can show both paths working (§11).

### 5.2 A login set is a WebView profile

`WebViewCompat.setProfile(webView, <the login set's UUID>)`, before anything else is done with the web view, as the API requires; the profile is made if it is new. Checked with `WebViewFeature.isFeatureSupported(MULTI_PROFILE)` by the engine, and again where the web view is made.

- **Without profiles no web view is made.** `WebViewCreate` is refused with `engine.no_profiles`, and the tab fails. Every web view sharing the default data store would be one login set's cookies in every other.
- **A private (ephemeral) login set is refused**, `engine.no_private`. A WebView profile is always written to disk; the contract says a private one is kept in memory. W0's core never asks for one.
- A login set that is not a lower-case UUID is refused (`webview.bad_login_set`): the name goes to the engine as given.
- A device with no usable WebView (none installed, or one being updated) is `engine.unavailable`.
- The default profile is never touched: no `CookieManager.getInstance()`, no web view without `setProfile`.

Other codes are the Apple adapter's: `webview.unknown`, `webview.exists`, `webview.bad_address`, `webview.bad_command`.

**`WebViewLoad` asks the core's policy once more** (`navigationVerdict`) before `loadUrl`. The core sends only addresses its own policy allows; the check is here because of what `WebView.loadUrl` does with one that is not: a `javascript:` address is run as script in the page that is showing.

### 5.3 Errors as codes

| WebView | Code |
|---|---|
| `ERROR_HOST_LOOKUP`, with a network | `net.dns` |
| `ERROR_HOST_LOOKUP`, with no network (`ConnectivityManager`) | `net.offline` |
| `ERROR_CONNECT`, `ERROR_IO` | `net.connect` |
| `ERROR_TIMEOUT` | `net.timeout` |
| `ERROR_FAILED_SSL_HANDSHAKE`; `onReceivedSslError` for the page's own load | `net.tls` |
| A load that ended without committing; a refusal | `net.cancelled` |
| Everything else | `net.failed` |

The WebView has no error for "no network": it folds it into a failed lookup. The adapter asks the system whether there is a network, which is why the app declares `ACCESS_NETWORK_STATE`. An error's own description never leaves the shell.

### 5.4 The request callback

`shouldInterceptRequest` calls `requestVerdict` (`wt_guard_check`) for every request, on the WebView's own threads, so each shell has its callback wired from the first build (001 §5). Until Guard exists it allows everything and the callback returns null. The WebView says only whether a request is the page itself, so the resource type sent is `DOCUMENT` or `OTHER`. What a blocked request should look like to its page is W4's to settle; for now it is an empty 403.

### 5.5 Settings

JavaScript and DOM storage on; a wide viewport and pinch zoom; **no file access and no content-provider access**; the default user agent (as on Apple, 001 F13). A certificate the system does not trust is never accepted. A page that asks for a new window gets its address in the same web view (the WebView's default, and 001 F5).

## 6. The window

`app/…/ui/`, the Apple window's shape (001 §6):

- **`BrowserModel`** mirrors `BrowserModel.swift`: the tab as the core last published it, and the address field's text, which follows the core unless the user is typing. Editing starts from the whole address; submitting sends what was typed, as typed.
- **`BrowserSession`**: one core and one engine for the process. A window's first snapshot is kept if it arrives before the window knows its subscription's id.
- **`BrowserWindow`**: back, forward, reload or stop; the address field with its security mark; a progress line; and the page, an empty tab, or a failed page in words. The system's Back goes back in the page while it can. After an address is submitted the keyboard's focus goes to the page.
- **`WebViewHost`** gives the engine's web view a place. Web views live in the engine, outside the composition, and the activity handles its own configuration changes (size, fold, density, keyboard, theme, language), so none of those reloads a page (000 §6.5). A web view is made with the application as its context and borrows the activity while it is on screen.
- Words: `error.<code>` is the resource `error_<code>` with dots as underscores, found by name; a code with no words of its own gets the general ones, and a code's text is never shown.
- The window draws edge to edge and keeps clear of the bars, the cutout and the keyboard.
- Nothing the app keeps is backed up or transferred to another device (`allowBackup` off, data extraction rules): whether a browser's cookies and history may leave the device is not yet decided.

**Strings** (`Tools/strings/strings.py`) are written to `res/values/strings.xml` from `Localization/strings.json`, and to `values-<language>` for each other language: apostrophes, quotes, backslashes, a leading `@` or `?` and XML's own characters are escaped, and `%@` becomes `%s` (numbered when there is more than one). **Tokens** (`Tools/design/tokens.py`) are written to `design/Tokens.kt`. Both are checked in and `Tools/test.sh lint` fails when they are stale.

## 7. Fake mode

`WEBTIGER_FAKE` for Android is two intent extras, read from the launch that starts the process:

```sh
adb shell am start -n com.tigercodes.webtiger/.MainActivity \
  --ez webtiger.fake true --es webtiger.fake.start http://10.0.2.2:8080/
```

They become `CoreConfig.test_mode = TEST_MODE_FAKE` and `fake.start_address`; no data folder is named. **They are honoured in a debuggable build only**: an intent from another app must not be able to start a shipped WebTiger without its data.

## 8. What is not verified

No device, no emulator, no system image, and none could be installed. So:

- **The app has never been launched.** Not on a phone, a tablet, a foldable or an emulator.
- **`libwebtiger_core.so` has never been loaded on Android.** It links, it exports the twelve symbols, it needs only `libc`, `libm` and `libdl`, and it is aligned for 16 KB pages. The same source built for the Mac passes the tests.
- **Nothing in `WebViewPage.kt` has run**: creating a WebView, `setProfile`, the settings, the two clients, the dead-renderer replacement, the request callback.
- **The event mapping is derived, not observed.** It follows the WebView's source as read on 2026-11-18. The order of the callbacks around a failure (the error and the error page's commit) could not be established from the source; both orders are handled and tested. Which WebView versions call `onPageStarted` when is known only from the source.
- **Isolation between login sets** (Q2 for Android): that two profiles share no cookie, storage, cache or service worker; the lowest WebView that has profiles; what an extra profile costs. Nothing measured.
- **The adapter suite** (001 §8) for this adapter: not written.
- **The window on a screen**: layout, the keyboard, focus (whether tapping the field selects the whole address; whether a page's form gets the keyboard after the focus was sent to the page), insets, dark theme, TalkBack, rotation and folding. The Robolectric tests draw it, find its controls by tag and by label, and click them; they show no pixels.
- **Fake mode on a device**, and the `am start` line above.
- `net.offline`: the system's answer about the network was never asked on a device.
- **Performance** (Q4, Q6): the cost of the request callback and of the JNI crossings; start-up; memory.
- A release build: R8, shrinking (the keep rules are written, never exercised), signing.
- The Android build on Windows or Linux: the Gradle tasks choose the NDK's toolchain for each, and only the Mac's was run.

What **was** run, on this Mac, 2026-11-18:

| Command | Result |
|---|---|
| `cd Core && cargo fmt --all --check` | clean |
| `cd Core && cargo clippy --workspace --all-targets --locked -- -D warnings` | clean |
| `cd Core && cargo clippy -p wt-ffi --features jni --all-targets --locked -- -D warnings` | clean |
| `cd Core && cargo test --workspace --locked` | 138 passed (as before) |
| `Tools/test.sh android`, from nothing | OK, 48 s |
| … `:core-engine:testDebugUnitTest` | 28 passed |
| … `:app:testDebugUnitTest` | 74 passed |
| … `:app:assembleDebug` | `app-debug.apk`, with `libwebtiger_core.so` for `arm64-v8a` and `x86_64` |
| `Tools/strings/strings.py --check`, `Tools/design/tokens.py --check` | up to date; the Apple outputs' checksums unchanged |
| `./gradlew :app:lintDebug` | no errors. Apart from notices that newer versions exist, two warnings are left on purpose: no launcher icon (the brand is not settled, 001 F10), and `toolbar_open_location` unused (it is the Mac's ⌘L) |

| Suite | Count | What it shows |
|---|---|---|
| `CoreLibraryTest` | 8 | The glue: constants against the header; order and one thread; bad messages; a refused start; a listener that throws; stop from the wrong thread; the fast path failing safe; several cores from several threads |
| `NativeCoreTest` | 15 | The contract through Kotlin: ping direct and through the host; a page loading (`WebViewCreate` then `WebViewLoad`, engine events, the snapshot); fake mode's start address; refusals and refused starts by code; the store in its folder; the navigation policy; stopping |
| `ScriptedCoreTest` | 5 | The scripted core |
| `PageTrackerTest` | 26 | §5.1, one situation each |
| `SystemWebViewEngineTest` | 14 | The host service and its codes; no web view without profiles |
| `NetErrorsTest` | 3 | §5.3 |
| `BrowserModelTest` | 10 | The window's model against `ScriptedCore` |
| `BrowserWindowTest` | 8 | The window drawn (Robolectric): its states, its controls, what they send |
| `WordsTest`, `LaunchOptionsTest` | 3, 4 | Codes into words; fake mode's extras |
| `ThroughTheCoreTest` | 6 | The model, the session, the engine and the tracker over the **real core**: a page loads in the Space's login set; a page cannot put an address in the field it has not become; a failure by its code; fake mode; no profiles |

## 9. Decisions taken while building

| # | Decision | Why |
|---|---|---|
| A1 | The adapter's rules are in `PageTracker`, which has no Android in it | With no device, a rule that cannot be tested on the JVM is a rule nobody has checked |
| A2 | The commit signal is `doUpdateVisitedHistory` | §5.1 |
| A3 | No profiles, no web view (`engine.no_profiles`) | Sharing a data store silently is the one failure a login set must not have |
| A4 | A private login set is refused on Android | The WebView has no profile that is kept in memory |
| A5 | The request callback is wired now and allows everything | 001 §5: every shell wires it from the first build |
| A6 | Fake mode in debuggable builds only | §7 |
| A7 | No backup and no device transfer of the app's data | A privacy default until the question is decided |
| A8 | The core is built in release for debug apps too | One library, the one that ships, and measurements that mean something |
| A9 | Compose BOM 2026.06.01, not the newest | The newest needs a platform this Mac does not have |
| A10 | No launcher icon | The brand is not settled (001 F10) |
| A11 | The WebView's activity lifecycle (`onPause`, `onResume`), downloads, file choosers, permission prompts, full screen, new windows as tabs, opening external addresses in the system | Not W0: they arrive with the host services that own them (000 §5.5) |

## 10. Found in the core and the contract, from the Android side

Found while the shell was built. Items 1, 2, 4 and the first half of 8 were dealt with straight afterwards, as each says; the rest are open.

1. **`NavigationFinished` after `NavigationFailed` makes a failed tab idle.** The System WebView does report a finish after a failure. The adapter drops it, but the core would be safer ignoring a finish that no start preceded. **Fixed:** the core now keeps a failed tab failed until a new load starts (001 F15), with a test that fails without the fix.
2. **A commit after a failure** is what Android's error page produces. The core takes it as intended (the tab stays failed, the address becomes the failed one). **Pinned:** the same core test reports the commit and the finish after the failure.
3. **`LoginSet.ephemeral` has no Android answer** (A4). Private windows on Android need a decision: a profile made for the window and deleted with it is the nearest, and it is written to disk meanwhile.
4. **Words are missing** for `engine.no_profiles` and `engine.no_private`. **Added** to `Localization/strings.json`, for both shells; the two tests that used `engine.no_profiles` as their example of a code without words now use a code that has none.
5. **`wt_request.resource_type`** cannot be filled on Android beyond document or other.
6. **`wt_nav_check` is not asked about every navigation on Android** (§5.1, last row). The core's own loads are the core's; a POST and a step back or forward are not seen.
7. `NavigationStarted.url` is empty when the first sign of a load is its progress. The core ignores it today.
8. `Tools/test.sh lint` and CI do not build `wt-ffi` with the `jni` feature, so `android.rs` is linted only by `Tools/test.sh android`. **CI's core job now runs clippy with the `jni` feature.** CI still has no Android job (Gradle, the SDK and the NDK).
9. Wire's generated messages print their fields, addresses included, in `toString()`. Nothing in the shell logs or prints a message; the redactor of 001 F6 will need a Kotlin half.

## 11. Follow-ups, in order

1. **Run it**: an emulator image or a device (001 §13). Then, in order: the app starts in fake mode; a page loads; §8's list.
2. The adapter suite for the System WebView, with `Tools/testsite` (001 §12), isolation first.
3. Q2 for Android: the lowest WebView with profiles, and what to do below it.
4. `NavigationListener` as the source of navigation events where the WebView has it.
5. `android` in `Tools/test.sh all` and a CI job, once a run on a device has shown the suite means what it says.
6. A UI test on the emulator in fake mode, as the Apple apps have.

## 12. Done checklist

- [x] JNI glue behind the `jni` feature; fenced; no local reference kept; nothing logged; clippy and rustfmt clean.
- [x] The 138 Rust tests still pass.
- [x] Gradle project: `app`, `core-engine`; versions pinned; the wrapper's checksum set.
- [x] The contract as Kotlin from `Core/proto`; the core for both ABIs in the APK.
- [x] `CoreClient`, `NativeCore`, `ScriptedCore`.
- [x] WebViews host service with a profile per login set; refused, not shared, without profiles.
- [x] Event mapping corrected and written down (§5.1); the two rules kept.
- [x] Compose window and its model.
- [x] Strings and tokens generated for Android; `--check` covers them.
- [x] Fake mode.
- [x] JVM tests: the real core through JNI; the window's model; the error codes.
- [x] `./gradlew :app:assembleDebug`.
- [x] `Tools/test.sh android`.
- [ ] Anything on a device or an emulator (§8).
- [ ] The adapter suite and isolation, against a real WebView.
- [ ] `android` in `Tools/test.sh all`; CI.

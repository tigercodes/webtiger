# 003 — The Android shell (W0)

| | |
|---|---|
| **Status** | Built, and **run on an emulator** (Android 16, System WebView 133): the app starts, a page loads through the core, and 66 tests pass on the device, among them the adapter suite against the real WebView and isolation between login sets. 152 more pass on the JVM, 28 of them driving the real core through JNI. **Not yet run on a physical device, on any other WebView version, or folded** (§8) |
| **Phase** | W0 of [000](000-master-plan.md) §9; item 1 of [001](001-foundations.md) §12 |
| **Stack** | 000 §5.1: Kotlin, Jetpack Compose, Material 3, `androidx.webkit`, Wire, JNI through `wt-ffi` |
| **Last updated** | 2026-11-18 (run on an emulator; the device suite) |

## 1. What it is for

W0 asks each shell to load a page **through** the core, in a login set of its own, and to be testable without a person watching. This is the Android half: the core's library with its JNI glue, a Kotlin client over it, the System WebView behind the WebViews host service with one WebView profile per login set, and one Compose window.

The Mac that first built it had the Android SDK and NDK and **no emulator, no system image and no device**. So the work was shaped to put as much as possible where it can be tested without one: everything the adapter decides about a WebView's callbacks is in one class with no Android in it, and the window is tested over the real core with only the WebView itself replaced. A second Mac with an emulator image then ran it (§8), and the device suite was written there: what the first build derived from the WebView's source, the second observed.

## 2. Where it stands

| Deliverable | State | Verified by |
|---|---|---|
| JNI glue, `wt-ffi/src/android.rs` (the `jni` feature) | Done | 23 Kotlin tests through the real library, built for this Mac; clippy with the feature on |
| Gradle project, `Android/`: `app` and `core-engine` | Done | `./gradlew :app:assembleDebug` |
| The contract as Kotlin (Wire, at build time, from `Core/proto`) | Done | Compiled; used by every test |
| `CoreClient`, `NativeCore`, `ScriptedCore` | Done | 28 tests |
| The core for `arm64-v8a` and `x86_64`, built with the NDK and packed in the APK | Done | Both libraries are in the APK, aligned for 16 KB pages. The `arm64-v8a` one is loaded and used on the emulator; **the `x86_64` one has never been loaded** |
| System WebView adapter: WebViews host service, a profile per login set | Done | Its logic: 53 JVM tests without a WebView. Against the real WebView: the adapter suite, 19 tests on the emulator, and 10 more for a page that asks for a new window (§8.1) |
| Event mapping (001 §5.1, Android column) | Corrected (§5.1) from the WebView's source, then **observed** on WebView 133 | 28 JVM tests of the rules; the device suite. One rule was missing and was found there (§5.1, the refused navigation) |
| Isolation between login sets (Q2 for Android) | **Holds** on WebView 133 | 12 tests on the emulator (§8.1): no cookie, stored value, database, cache, service worker or copy in the HTTP cache is shared; nothing reaches the default profile. Deleting a profile has a rule of its own (§5.2) |
| Compose window: address field, back, forward, reload or stop, progress, three content states | Done | 9 tests drawn on Robolectric; 11 of its model; 8 on the emulator's screen with the real core and WebView. Three faults showed only on a device and are fixed (§6) |
| Strings and tokens from the shared sources | Done | `--check` covers them; the Apple outputs are byte for byte what they were |
| Fake mode | Done | 6 JVM tests; the device's window tests start the app in it |
| `Tools/test.sh android` | Done; part of `all` where a JDK and the Android SDK are found | Run in full on both Macs |
| `Tools/test.sh android-device` | Done, not part of `all` | Run in full on the Mac with the emulator, from a cold emulator and a warm one |

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
Tools/test.sh android          # clippy for the glue; the JVM tests; the debug APK
Tools/test.sh android-device   # on a device: the adapter suite, isolation, the window (§8.1)
Tools/android/emulator.sh start | stop | status
cd Android && ./gradlew :core-engine:testDebugUnitTest :app:testDebugUnitTest
cd Android && ./gradlew :app:assembleDebug
cd Android && ./gradlew :app:connectedDebugAndroidTest
```

Gradle needs `JAVA_HOME` (a JDK 21 or later) and `ANDROID_HOME`, or `Android/local.properties`. `Tools/test.sh android` finds Homebrew's JDK 21 or the JDK inside Android Studio (25, which builds and tests the same), and the SDK in its usual place, when neither is set. It installs nothing: `android.builder.sdkDownload=false` makes a missing platform or NDK an error.

**A device.** `Tools/android/emulator.sh start` uses whatever device is attached. With none it boots an emulator without a window: the AVD `wt-phone` (or `WEBTIGER_AVD`), which it makes from the newest system image installed. It downloads nothing either. It starts the adb server with `ADB_MDNS=0`: adb 37's mDNS discovery aborted the server on macOS 27.2 here, repeatedly, whenever the local network carried a service name it could not parse, and every device then read as offline.

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
| AndroidX Test runner, JUnit extension | 1.7.0, 1.3.0 | The device suite only |
| Compose's accessibility checks (`ui-test-junit4-accessibility`) | The BOM's (1.11.4) | The device suite only. It brings Google's Accessibility Test Framework |
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

001's column was a plan. It was checked against the WebView's own source (Chromium's `AwWebContentsObserver`, `AwContentsClientBridge`, `AwContentsClientCallbackHelper`, `AwComputedFlags` and `AwContentBrowserClient::ShouldOverrideUrlLoading`, as on `main` on 2026-11-18), because there was no device to check it on. Four rows were wrong. The table as built was then run against WebView 133 on an emulator (§8.1), and held; what that WebView was seen to do is listed after it.

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

**What WebView 133 was seen to do** (Android 16 emulator, 2026-11-18; recorded with a log line in each callback, since removed):

| Situation | Callbacks, in order |
|---|---|
| A load the adapter asked for | progress 10 · `onPageStarted` · `doUpdateVisitedHistory` · `onReceivedTitle` · progress 100, twice · `onPageFinished` |
| Back or forward | The same, with the title after the progress. Nothing announces it before progress 10 |
| A page pushing its own address | `doUpdateVisitedHistory` with the new address and nothing before it; `onPageFinished` then names the pushed address |
| A load that fails (connection refused) | progress 10 · `onPageStarted` · `onReceivedError` · `onPageFinished` · `doUpdateVisitedHistory` · `onReceivedTitle` with the engine's words · progress 100, twice. So the error comes before its error page commits |
| A load stopped before it commits | progress 10 · `onPageFinished` · progress 100 |
| A redirect the policy refuses | progress 10 · `shouldOverrideUrlLoading` · `onPageFinished` for the refused address · progress 100 |
| A link the page follows, refused | `shouldOverrideUrlLoading` · progress 10 · progress 100, twice. **No `onPageFinished`** |

The last row was a fault. The tracker expected a refused navigation to end with a finished page, as the redirect does, and took the progress that came instead for a load nobody had announced: the core was told `started` and `finished` after the `net.cancelled`. A refused navigation's own progress is now dropped (`PageTracker.progress`), with JVM tests for the sequence as observed and the device suite's `aNavigationTheCoreRefusesNeverCommits` against the real thing.

`androidx.webkit` 1.18 also has `NavigationListener` (`WebViewFeature.NAVIGATION_LISTENER`), which says started, redirected, completed, `didCommit()` and `isSameDocument()` outright. It needs a newer WebView than profiles do, so the callbacks above would remain as the fallback. It is the better source where it exists and was left for when a device can show both paths working (§11).

### 5.2 A login set is a WebView profile

`WebViewCompat.setProfile(webView, <the login set's UUID>)`, before anything else is done with the web view, as the API requires; the profile is made if it is new. Checked with `WebViewFeature.isFeatureSupported(MULTI_PROFILE)` by the engine, and again where the web view is made.

- **Without profiles no web view is made.** `WebViewCreate` is refused with `engine.no_profiles`, and the tab fails. Every web view sharing the default data store would be one login set's cookies in every other.
- **A private (ephemeral) login set is refused**, `engine.no_private`. A WebView profile is always written to disk; the contract says a private one is kept in memory. W0's core never asks for one.
- A login set that is not a lower-case UUID is refused (`webview.bad_login_set`): the name goes to the engine as given.
- A device with no usable WebView (none installed, or one being updated) is `engine.unavailable`.
- The default profile is never touched: no `CookieManager.getInstance()`, no web view without `setProfile`.

Other codes are the Apple adapter's: `webview.unknown`, `webview.exists`, `webview.bad_address`, `webview.bad_command`.

**Deleting and clearing a login set: what the WebView allows** (observed on WebView 133, pinned by two device tests; the contract's LoginSets host service is not built yet, so this is the ground it will stand on):

- **A profile that has been used since the app started cannot be deleted until the app starts again.** `ProfileStore.deleteProfile` throws ("Cannot delete in-use profile") while a web view uses the profile, and goes on throwing after the web view is destroyed, for as long as the process lives. Asking for the profile (`getOrCreateProfile`) is enough to hold it. WebKit differs: its store can be removed a moment after its last web view is released (001 §10, Q2).
- **Its data can be cleared while it lives.** `CookieManager.removeAllCookies` on the profile's own cookie manager, and `WebStorageCompat.deleteBrowsingData` on its storage (`WebViewFeature.DELETE_BROWSING_DATA`, which WebView 133 has), left nothing: no cookie, stored value, database, Cache API entry or service worker.
- **The older call is not enough.** `WebStorage.deleteAllData`, the only one a WebView without `DELETE_BROWSING_DATA` has, cleared the stored value and the database and **left the Cache API's entries and the service worker**.

So on Android "delete this Space's logins" (000 §5.6) is two steps: clear the data at once, and delete the profile at the next start. A private window made of "a profile deleted on close" (000 §4.4) has the same shape: cleared on close, deleted at the next start.

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
- **The address field at rest shows the start of the address.** A text field scrolls to its caret, and the caret used to be put at the end: on a screen, an address longer than the field showed its tail and lost its scheme and the start of its host. That is the wrong half to lose (000 §5.13). At rest the caret is now at the start, so the site stays in view and the tail is cut.
- **A tap on the field selects the whole address.** The tap that brings the keyboard also places a caret where it fell, in the text the field showed until then, and that undid the selection the focus had just made (and put the short form back in place of the whole address). On Robolectric the same click left the selection alone, so the JVM test passed while the app was wrong. The stale caret is now dropped. Both are tested on the emulator's screen.
- **A field that has the keyboard still follows the page, until something is typed in it.** Out of touch mode (a keyboard was used last), Android gives a new window's first focusable control the focus, and that is the address field. The field then held what it held when the focus came, which at launch is nothing, while the page loaded under it: an empty field over a page. "What the user types is not overwritten" had been built as "a field with the focus is not overwritten". The model now knows the difference (`BrowserModel.hasTypedAddress`), on Android and in the Apple shell, whose model had the same rule and whose Mac UI test failed the same way on GitHub's runner.
- **A window that opens on a page gives the page the keyboard.** The field that took the focus by itself, with nothing typed in it, lets go when the page arrives; an empty tab keeps the keyboard in the field, ready for an address. The device suite sets the touch mode it means, and has a test that starts the app out of it.
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

The line above was run against the emulator, with a page served from the Mac (`10.0.2.2` is the Mac's loopback address as the emulator sees it). The device's window tests start the app the same way, with a site inside the test's own process on the device's `127.0.0.1`: its pages are then a secure context, which the Cache API and service workers need and `10.0.2.2` is not.

## 8. What was run on a device, and what is not verified

### 8.1 On the emulator

An emulator without a window, made by `Tools/android/emulator.sh` from the Google APIs image for Android 16 (API 36, `arm64-v8a`, revision 7) with the Pixel 8 profile, on a Mac with Apple silicon; its System WebView is **133.0.6943.137**. 2026-11-18.

`Tools/test.sh android-device` runs the tests in `Android/app/src/androidTest` there: 66 pass. From a cold emulator the whole step took 40 s, the boot included.

| Suite | Count | What it shows |
|---|---|---|
| `SystemWebViewAdapterTest` | 19 | The adapter suite (001 §8) against the real WebView and a site on the device's loopback address, with a scripted core: started, committed, finished in that order; a redirect commits where it landed; a pushed address is an address change, not a page; back and forward; a link the page follows; a refused connection and a name that does not resolve, as codes, with the engine's error page reported neither as finished nor by its title; trying again after a failure; stop; the navigation policy for a link, a redirect and an external address; a `javascript:` address never given to the WebView; no address reported before it commits; **every request put to the core** (a request the core blocks never reaches the site); **a renderer that is killed** is reported, and the next load makes another web view; the host service's codes |
| `SystemWebViewIsolationTest` | 12 | Isolation as a security property (000 §8.4): two login sets share no cookie, no stored value, no IndexedDB database, no Cache API cache, no service worker and no copy in the HTTP cache, and two web views in one login set share them all; nothing a login set stores reaches the default profile; a sign-in for the session outlives the last web view of its login set; a private login set is refused; deleting and clearing, as §5.2; and a record of what the WebView offers a page (001 §11.4) |
| `SystemWebViewNewWindowTest` | 10 | A page that asks for a new window ([006](006-new-windows.md)), with a finger on the page: the adapter makes the web view the WebView asks for, in the asking page's login set, names it with an id of the shell's own and reports it; `window.open` keeps the hold between the two pages; `window.close()` is reported; a page that asks as it loads, or clicks its own link, gets nothing; a refused web view is destroyed; the navigation policy is asked about where the new window goes, and a refusal is reported as one, which a stopped load is not |
| `SystemWebViewOutsideLinkTest` | 5 | A link for the system ([007](007-outside-links.md)), with a finger on the page: reported with a gesture before its refusal; a button's script; a link in a frame; a page that goes there by itself; a link that opens a window |
| `SystemOpenerTest` | 2 | The system's own opener: the activity it starts for a letter and for a number, caught before it leaves the app; a scheme nothing opens is said to be so |
| `SystemWebViewRestoreTest` | 3 | What a sleeping tab keeps (Q5; 001 §11.3): `saveState` is one small byte array, and a new web view in the same profile wakes from it with the page, its history, its login set, and, once the engine has had a second to record them, the scroll position and the text typed into a form |
| `AppWindowTest` | 15 | The app, started in fake mode, with the real core and the real WebView in its window: a page loads through the core; a typed address loads, and the window's Back and the system's Back go back; a failure is shown in words, over the engine's own error page; the address field at rest, tapped, and when it took the focus by itself (§6); **turning the device reloads nothing**: the same activity, the same web view, no second request; tabs open, switch and close, each with a web view of its own ([005](005-tabs.md)); a link that opens a window, touched, opens a tab beside the one it was in, and closing it brings that one forward, and a window opened on a letter to write leaves no tab behind (006); a touch on a link to write a letter leaves the page as it was and starts no activity (007); history lists what was visited, finds it, opens it again and forgets it ([008](008-history.md)); the design catalogue opens in place of a browser (004); **Android's accessibility checks** (Google's Accessibility Test Framework, through Compose's test rule) pass over a page, a failed page and the catalogue, which has every state the window has |

Also done by hand there, and looked at in screenshots: the app started with the `am start` line of §7 and a page served from the Mac; a link, back and forward; the keyboard that comes up is the address one, with Go; a load that timed out (the emulator's network was not up yet, seconds after its boot) showed the failed page in words, and Try Again loaded the page; `example.com` typed into the field loaded over https with the padlock and the short form of the address; the dark theme, switched while the app ran, redrew the window without making the activity again.

Faults found by running it, all fixed with tests: the refused navigation's progress (§5.1), and the three in the address field (§6). The first launch of the core's library, the JNI glue, `setProfile`, the two WebView clients and the request callback found none.

### 8.2 Not verified

- **A physical device.** Everything above is one emulator image. No phone, tablet or foldable has run the app.
- **Any other WebView.** Only 133. Which older WebViews call `onPageStarted` at the start of a load rather than at its commit is still known only from the source; the lowest WebView with profiles (Q2) is not established; nothing was run on a WebView without `DELETE_BROWSING_DATA`.
- **The `x86_64` library**, on anything.
- **Folding, tablets, several windows.** The fold-continuity rule (000 §6.5) is tested here only as a rotation.
- **TalkBack itself.** The accessibility checks run (§8.1) and found one fault, in the catalogue: an empty text for a screen reader to stop at in every colour's row. Nobody has used the app with TalkBack.
- **The keyboard after a page takes the focus**: whether a page's own form gets the keyboard once the focus was sent to the page. A tapped address field shows the whole address selected, with one caret handle where two selection handles would be expected.
- `net.offline`: the system's answer about the network was never asked with the network off. `net.tls`: no site with a bad certificate was loaded.
- Site permissions between login sets (as for WebKit, 001 §10 Q2); what an extra profile costs.
- **Deleting a profile at the next start** (§5.2): the refusal within one run is pinned; the deletion after a restart was not exercised, since a test's process cannot restart itself.
- **Performance** (Q4, Q6): the cost of the request callback and of the JNI crossings; start-up; memory. The emulator's debug build started cold in 0.6 to 0.9 s by `am start -W`, which says nothing about the budget.
- A release build: R8, shrinking (the keep rules are written, never exercised), signing.
- The Android build on Windows: the Gradle tasks choose the NDK's toolchain for each system; the Mac's and Linux's (on GitHub) were run.
- **CI for the device suite.** GitHub's workflow runs `Tools/test.sh android` on Linux, and it passed at its first run there: the first build of the Android shell off a Mac, with the NDK's Linux toolchain, and the catalogue's pictures matching those drawn on a Mac. The device suite runs only where there is an emulator, which is not there.

### 8.3 What was run without a device

On both Macs, 2026-11-18 (the time is the first Mac's):

| Command | Result |
|---|---|
| `cd Core && cargo fmt --all --check` | clean |
| `cd Core && cargo clippy --workspace --all-targets --locked -- -D warnings` | clean |
| `cd Core && cargo clippy -p wt-ffi --features jni --all-targets --locked -- -D warnings` | clean |
| `cd Core && cargo test --workspace --locked` | 139 passed |
| `Tools/test.sh android`, from nothing | OK, 48 s |
| … `:core-engine:testDebugUnitTest` | 31 passed |
| … `:app:testDebugUnitTest` | 121 passed |
| … `:app:assembleDebug` | `app-debug.apk`, with `libwebtiger_core.so` for `arm64-v8a` and `x86_64` |
| `Tools/strings/strings.py --check`, `Tools/design/tokens.py --check` | up to date; the Apple outputs' checksums unchanged |
| `./gradlew :app:lintDebug` | no errors. Apart from notices that newer versions exist, two warnings are left on purpose: no launcher icon (the brand is not settled, 001 F10), and `toolbar_open_location` unused (it is the Mac's ⌘L) |

| Suite | Count | What it shows |
|---|---|---|
| `CoreLibraryTest` | 8 | The glue: constants against the header; order and one thread; bad messages; a refused start; a listener that throws; stop from the wrong thread; the fast path failing safe; several cores from several threads |
| `NativeCoreTest` | 18 | The contract through Kotlin: ping direct and through the host; a page loading (`WebViewCreate` then `WebViewLoad`, engine events, the snapshot); fake mode's start address; refusals and refused starts by code; the store in its folder; the navigation policy; stopping |
| `ScriptedCoreTest` | 5 | The scripted core |
| `PageTrackerTest` | 32 | §5.1, one situation each, two of them as observed on the emulator; a refusal is told apart from a load that was stopped (006); a link for the system is reported, and then refused (007) |
| `SystemWebViewEngineTest` | 18 | The host service and its codes; no web view without profiles; the web view made for a page that asks for a window (006); the engine itself opens nothing outside (007) |
| `ShellHostTest` | 7 | The shell's own host: the System service goes to the opener, the rest to the engine; the intent the system's opener starts (007) |
| `NetErrorsTest` | 3 | §5.3 |
| `BrowserModelTest` | 14 | The window's model against `ScriptedCore`, its tabs among it |
| `BrowserWindowTest` | 10 | The window drawn (Robolectric): its states, its controls and its strip of tabs, what they send |
| `HistoryModelTest`, `HistoryListTest` | 5, 7 | The History list's model, and the list drawn: what it shows and what a person's choices send (008) |
| `WordsTest`, `LaunchOptionsTest` | 3, 5 | Codes into words; fake mode's and the catalogue's extras |
| `CatalogueSnapshotTest` | 11 | The design catalogue as pictures, drawn on the JVM, and the failed page at twice the text size (004) |
| `ThroughTheCoreTest` | 6 | The model, the session, the engine and the tracker over the **real core**: a page loads in the Space's login set; a page cannot put an address in the field it has not become; a failure by its code; fake mode; no profiles |

## 9. Decisions taken while building

| # | Decision | Why |
|---|---|---|
| A1 | The adapter's rules are in `PageTracker`, which has no Android in it | With no device, a rule that cannot be tested on the JVM is a rule nobody has checked. With one, a rule the device suite finds wanting is fixed and pinned in milliseconds |
| A2 | The commit signal is `doUpdateVisitedHistory` | §5.1 |
| A3 | No profiles, no web view (`engine.no_profiles`) | Sharing a data store silently is the one failure a login set must not have |
| A4 | A private login set is refused on Android | The WebView has no profile that is kept in memory |
| A5 | The request callback is wired now and allows everything | 001 §5: every shell wires it from the first build |
| A6 | Fake mode in debuggable builds only | §7 |
| A7 | No backup and no device transfer of the app's data | A privacy default until the question is decided |
| A8 | The core is built in release for debug apps too | One library, the one that ships, and measurements that mean something |
| A9 | Compose BOM 2026.06.01, not the newest | The newest needs a platform this Mac does not have |
| A10 | No launcher icon | The brand is not settled (001 F10) |
| A12 | The device suite's site is in the test's own process, on the device's `127.0.0.1` | Its pages must be a secure context for the Cache API and service workers, which the Mac's address as an emulator sees it is not; and a suite then needs nothing running beside it. Its pages are the one definition every shell's suite serves (`Fixtures/testsite/site.json`, 001 F12) |
| A13 | The device suite leaves its login sets' profiles behind | The WebView deletes none that was used since the app started (§5.2). They go with the app's data when the runner uninstalls it |
| A11 | The WebView's activity lifecycle (`onPause`, `onResume`), downloads, file choosers, permission prompts, full screen, (new windows as tabs have arrived since, [006](006-new-windows.md), and opening a letter, a number or a message in the system, [007](007-outside-links.md)) | Not W0: they arrive with the host services that own them (000 §5.5) |

## 10. Found in the core and the contract, from the Android side

Found while the shell was built. Items 1, 2, 4 and the first half of 8 were dealt with straight afterwards, as each says; the rest are open.

1. **`NavigationFinished` after `NavigationFailed` makes a failed tab idle.** The System WebView does report a finish after a failure. The adapter drops it, but the core would be safer ignoring a finish that no start preceded. **Fixed:** the core now keeps a failed tab failed until a new load starts (001 F15), with a test that fails without the fix.
2. **A commit after a failure** is what Android's error page produces. The core takes it as intended (the tab stays failed, the address becomes the failed one). **Pinned:** the same core test reports the commit and the finish after the failure.
3. **`LoginSet.ephemeral` has no Android answer** (A4). Private windows on Android need a decision: a profile made for the window and deleted with it is the nearest, and it is written to disk meanwhile. **And it cannot be deleted with the window** (§5.2): it can be cleared when the window closes and deleted when the app next starts.
4. **Words are missing** for `engine.no_profiles` and `engine.no_private`. **Added** to `Localization/strings.json`, for both shells; the two tests that used `engine.no_profiles` as their example of a code without words now use a code that has none.
5. **`wt_request.resource_type`** cannot be filled on Android beyond document or other.
6. **`wt_nav_check` is not asked about every navigation on Android** (§5.1, last row). The core's own loads are the core's; a POST and a step back or forward are not seen.
7. `NavigationStarted.url` is empty when the first sign of a load is its progress. The core ignores it today.
8. `Tools/test.sh lint` and CI do not build `wt-ffi` with the `jni` feature, so `android.rs` is linted only by `Tools/test.sh android`. **CI's core job now runs clippy with the `jni` feature.** CI still has no Android job (Gradle, the SDK and the NDK).
9. Wire's generated messages print their fields, addresses included, in `toString()`. Nothing in the shell logs or prints a message; the redactor of 001 F6 will need a Kotlin half.

## 11. Follow-ups, in order

1. ~~Run it.~~ Done on an emulator (§8.1). **A physical device** is still to come, and a foldable or its emulator image for 000 §6.5.
2. ~~The adapter suite for the System WebView, isolation first.~~ Done, serving the one test site every shell's suite serves (001 F12).
3. Q2 for Android: the lowest WebView with profiles, and what to do below it; permissions between profiles.
4. The LoginSets host service for Android as §5.2 found it must be: clear at once, delete at the next start.
5. `NavigationListener` as the source of navigation events where the WebView has it.
6. ~~`android` in `Tools/test.sh all`.~~ Done: `all` runs it where its tools are, and says so where they are not. A CI job for the device suite (GitHub's Linux runners can run an x86_64 emulator, which would also load the `x86_64` library for the first time). The JVM tests and the APK have their job, and it is green.
7. ~~The accessibility audit in the device's window tests, as the Apple apps have.~~ Done (§8.1).

## 12. Done checklist

- [x] JNI glue behind the `jni` feature; fenced; no local reference kept; nothing logged; clippy and rustfmt clean.
- [x] The Rust tests still pass (139).
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
- [x] The app on an emulator: it starts, and a page loads through the core (§8.1).
- [x] The adapter suite and isolation, against a real WebView; a UI test of the window on the device.
- [x] `Tools/test.sh android-device`, and `Tools/android/emulator.sh` for a device to run it on.
- [ ] A physical device; a foldable; other WebView versions (§8.2).
- [x] CI for `Tools/test.sh android`, on Linux.
- [x] `android` in `Tools/test.sh all`, where its tools are.
- [ ] CI for the device suite.

# WebTiger — Master Plan

**Product:** WebTiger
**Developer:** TigerCodes, LLC
**Bundle identifier:** `com.tigercodes.webtiger` (extensions and helpers use this prefix)
**Status:** Accepted 2026-11-18. The owner agreed to every recommended decision in §11, with macOS 26 as the minimum. Phase W0 is in progress: what is built, and what is not, is in [001](001-foundations.md).
**Platforms:** macOS, iPadOS, iOS (iPhone Duo included), Windows 11 (x64 and Arm64), Android (phones, tablets, foldables)
**Stack:** one Rust core under native shells: Swift (SwiftUI with AppKit and UIKit), C++20 with C++/WinRT and WinUI 3, Kotlin with Jetpack Compose
**Web engines:** WebKit and Chromium, each platform's own engine first (§4)
**Source:** open source under the Mozilla Public License 2.0 (WT-D8)
**Business model:** free. No account, no telemetry, no licence server
**Benchmark app:** Arc (The Browser Company), which stops working on 2026-12-22
**Documents:** one plan per feature in `docs/features/`; this is 000, and [001](001-foundations.md) records what is built

---

## 0. Summary

WebTiger is an open-source browser for people who liked Arc: Spaces that hold Groups and Sites, a command bar, split view, and a calm window. It adds what Arc never had. Every Space can keep its own logins. Tabs sit down the side or along the top. Ads and trackers are blocked from the first launch. The same app runs natively on five platforms. A plugin is one folder and a few lines of JavaScript.

**The recommendation in one paragraph.** Build one Rust core, **WebTiger Core**, that owns everything except pixels and web engines: Spaces, Groups, Sites, tabs, history, the blocker, link rules, plugins, sync, and the view models. Put three native shells on it (Swift for Mac, iPad and iPhone; C++/WinRT for Windows; Kotlin for Android), talking over one versioned protobuf contract. Do not fork a browser engine. Render with the engine each system already ships and patches: WKWebView on Apple platforms, WebView2 on Windows, Android System WebView on Android. Where a second engine can be embedded without forking it, offer it as an optional download. At 1.0 that is Chromium on the Mac.

**Five things to know before reading on.**

1. **"WebKit or Chromium" is settled by the platform before it is a setting.** At 1.0 the Mac offers both. iPhone and iPad are WebKit only: Apple allows other engines only in the EU and Japan, and no vendor has shipped one. Windows and Android are Chromium only: there is no production WebKit for Windows (Kagi cancelled its port in October 2026), and WebKit on Android is experimental. The design keeps every one of those doors open (§4.2).
2. **No engine fork.** Arc, Brave, Vivaldi and Helium each maintain a Chromium fork. That is a standing team and a 200 MB download. DuckDuckGo ships on the same four operating systems on system web views, and that is the shape proposed here. Its limits are listed honestly in §4.7.
3. **One core from the first day, Apple included.** Two implementations of the same logic need a conformance suite to hold them together. WebTiger starts with one.
4. **Plugins run in the core, not in a web engine.** Extension support differs on every engine and is absent on Android's. A small JavaScript runtime inside the core gives one plugin API on all five platforms (§5.10).
5. **Arc's shutdown sets the first deadline.** Arc's own export is a list of links. An importer that reads Arc's local data is the first thing built (Phase W0), and anyone with Arc data should copy it before 22 December (§13).

**Decisions for review.** §11 lists 18. Five shape everything else: the engine strategy (WT-D1), the engine matrix at 1.0 (WT-D2), one core for all five apps (WT-D4), the licence (WT-D8) and the plugin runtime (WT-D11).

---

## 1. Vision

WebTiger keeps the ideas people loved in Arc and drops what they complained about: an account before the first page, one heavy engine, a Mac-first product, a learning curve, and a closed codebase that was switched off.

Four non-negotiables, in priority order:

1. **Light.** The browser's own cost is small and stays small. A tab you are not looking at costs almost nothing. Every claim has a budget and a test (§7).
2. **Private.** Ads and trackers are blocked by default, with a switch per site, per Space and overall. A Space is a separate identity. Nothing about your browsing leaves the device unless you turn on sync, and then only encrypted.
3. **Organised.** Spaces hold Groups and Sites. The same structure shows as a sidebar or as a tab strip, on a 5-inch cover screen and on a 32-inch monitor.
4. **Open.** The source is public, your data is a documented file with a real export, and the plugin API is small.

One rule sits above the four: **a familiar browser first.** The Browser Company's own figures say only 5.5 % of Arc's daily users regularly used more than one Space; its founder called the rest a "novelty tax". WebTiger opens as an ordinary browser with one Space. Groups, more Spaces and tab tidying are there when wanted and never required.

### 1.1 What "better than Arc" means concretely

| Arc pain point | WebTiger answer |
|---|---|
| An account is required before the first page | **No account, ever.** Sync is optional and end-to-end encrypted (§5.11) |
| Chromium only; memory and heat grow with the sidebar | **System engines and sleeping tabs.** Budgets in §7 are tests, not hopes |
| Spaces share logins unless you also set up Profiles | **A new Space has its own logins by default.** Sharing with another Space is one choice at creation (§5.6) |
| Vertical tabs or nothing | **Sidebar or tab strip**, one model underneath, switchable at any time (§6.1) |
| Unpinned tabs are archived whether you like it or not | Tab tidying is a setting with a **Never** (WT-D15) |
| Mac first; Windows arrived late and thinner; phones got a different app | **Five native apps on one core**, held at the same feature level phase by phase |
| Closed source; frozen in 2025, switched off in 2026; export is a list of links | **Open source.** Data is a documented SQLite file plus a plain export, and nobody can turn the app off |
| Boosts were the only way to extend it, and a flaw in Arc's cloud let an attacker plant script in another user's Boosts (CVE-2024-45489) | **Plugins and Tweaks** that install only by the user's own action; no server can add or change code (§5.10) |

---

## 2. Scope

### 2.1 In scope (1.0)

- **Platforms and minimums (WT-D7):** macOS 26, iPadOS 26, iOS 26 (iPhone Duo through the iOS 27 SDK), Windows 11 22H2 on x64 and Arm64, Android 10 (API 29) with a current System WebView.
- **Engines** as §4.2: WebKit on every Apple platform, Chromium on Windows and Android, and Chromium as an optional pack on the Mac.
- **Organisation:** Spaces, Groups, Sites, Favourites, temporary tabs, an archive, a command bar, split view, Preview, a Quick window, link rules.
- **Logins per Space:** separate cookies, site storage, cache, permissions and history; private windows.
- **Two layouts:** sidebar and tab strip, on every platform that has the width.
- **Browsing basics:** everything in Phase W1's list (navigation, downloads, find, zoom, permissions, errors, uploads, session restore).
- **Memory:** sleeping tabs, live-tab limits, pressure handling, measured budgets.
- **Guard:** ad and tracker blocking, on by default, switchable per site, per Space and overall, with the other protections in §5.9.
- **Passwords and passkeys** through each platform's own service (§5.9, WT-D13).
- **Devices:** phones, tablets, Android foldables in every posture, iPhone Duo on both displays.
- **Plugins:** the plugin API, Tweaks, userscripts, developer tools.
- **Sync** of Spaces, Groups, Sites, settings and rules, end-to-end encrypted.
- **Import** from Arc, Chrome, Safari, Firefox and Edge.

### 2.2 Out of scope for 1.0 (tracked)

- **Forking or porting an engine.** No Chromium fork, no WebKit port.
- **Chromium on iPhone and iPad; WebKit on Windows; WebKit on Android.** Reasons and conditions in §4.2.
- **Chrome Web Store parity.** WebExtensions arrive after plugins, Apple first (§5.10).
- **A password manager of our own.** TigerPassword is the natural first provider through the plugin interface.
- **Built-in AI, VPN, wallet, rewards, news.** Never in the app. A plugin may add any of them.
- **Linux desktop.** The core builds and tests on Linux; a community shell is welcome.
- **visionOS, watchOS, TV, Wear OS.**
- **Sync of history and open tabs** (later, opt-in). **Cookies never sync.**
- **Web push** on engines that lack it (§4.7).
- **Managed configuration** (Group Policy, managed Google Play).

---

## 3. What the best browsers do, and what WebTiger takes

Checked 2026-11-18 (§15). Store ratings are the US App Store's. Every public memory or battery figure found was run by a vendor.

### 3.1 The survey

| Browser | Built on | Take | Leave |
|---|---|---|---|
| **Arc** | Chromium, closed | The sidebar hierarchy; command bar; split view; link preview; the small window for outside links; link routing; a colour per Space | The account; one heavy engine; forced archiving; being switched off |
| **Dia** (Arc's successor) | Chromium, closed, Atlassian | — | Spaces became profiles, Boosts and Easels were dropped; an AI product first |
| **Zen** | Firefox fork, MPL-2.0, desktop only | Proof of demand: 45,900 GitHub stars for an open Arc-like | Workspaces do not isolate logins; no tab strip; no DRM video; memory is its top open issue; no mobile |
| **Crest** | SwiftUI on a C# core, own Chromium build plus WebKit per site, MPL-2.0, Apple only | The nearest neighbour. Confirms the dual-engine model: one WebKit store and one Chromium profile per Space, DRM video routed to WebKit | A Chromium fork to maintain; no Windows or Android |
| **Nook, Ora** | WKWebView, Swift, GPL-3.0, Mac only | Worked answers for WebKit: a data store per Space, `adblock-rust` compiled to content rule lists, unloading tabs with their state | Nook leans on private WebKit APIs, which rules out the App Store; password managers are the most-requested gap in both |
| **Brave** (4.8★) | Chromium fork, MPL-2.0 | Blocking that works on the first launch with a per-site switch; `adblock-rust`; link cleaning | Wallet, rewards, VPN and AI in the box (it now sells a $59.99 build without them) |
| **Vivaldi** (4.7★) | Chromium fork | Blocking strength set per site; tab bar on any edge; tab hibernation; workspaces | Settings sprawl; a web-technology interface |
| **DuckDuckGo** | **System web views on Mac, iPhone, Windows and Android** | The architecture; tracker blocking by default; one-tap data clearing | Few power features: no groups, no sidebar, no extensions |
| **Orion** (4.0★) | WebKit, closed | A thin WebKit browser; no telemetry | Stability complaints; Windows and Linux ports cancelled in October 2026 as too costly |
| **Safari** | WebKit | Battery life; profiles | No built-in ad blocker. Vertical tabs arrive in the 27.3 beta |
| **Chrome** (4.7★) | Chromium | Compatibility; Memory Saver; tab search | Memory; no blocking; uBlock Origin delisted in August 2026 |
| **Edge** (4.7★) | Chromium | Sleeping tabs on a timer; vertical tabs | Clutter, telemetry |
| **Firefox** (4.6★) | Gecko, MPL-2.0 | Containers; the licence; it has begun shipping `adblock-rust` itself, switched off | — |
| **Helium** | ungoogled-chromium, GPL-3.0 | "Nothing you didn't ask for" as a pitch | No sync, no mobile |
| **AI browsers** | Chromium | — | ChatGPT Atlas was retired within a year; Firefox and DuckDuckGo added AI off-switches |

### 3.2 What follows from it

1. **Blocking on by default with a per-site switch** is what the best-rated privacy browsers share. With extension blockers weakened in Chrome and Edge, a built-in blocker matters more than it did.
2. **Vertical tabs and split view are now ordinary.** Chrome, Firefox, Edge, Brave and Vivaldi have them and Safari is adding them. What is still rare: isolated Spaces, both layouts over one model, and the same organisation on a phone.
3. **Nobody has one plugin story on all five platforms.** Mobile extensions are partial everywhere. This is the gap §5.10 aims at.
4. **A non-Chromium engine off its home platform is expensive.** Kagi retreated; Ladybird is pre-alpha. System engines put that cost on the OS vendor.
5. **Passwords come first.** iCloud Passwords, passkeys, 1Password and Bitwarden top the issue trackers of every Arc-like. WebTiger treats them as a Phase W1 matter, not a late feature (WT-D13).
6. **Memory claims are unproven in public.** Reproducible numbers and a visible per-tab cost are unclaimed ground (§7).
7. **Why not build on an existing Arc-like.** Zen is a desktop Firefox fork, Nook and Ora are Mac-only and GPL, Crest is Apple-only on a C# core. None has native Windows and Android apps or a core they could share.

### 3.3 Feature inventory: Arc and WebTiger

"W#" is the phase that delivers it (§9). "—" is not planned.

| Arc | WebTiger | Phase |
|---|---|---|
| Spaces | Spaces: name, icon, colour, own logins by default | W2 |
| Profiles (one can back several Spaces) | Folded into Spaces: a Space may share another Space's logins | W2 |
| Pinned tabs | **Sites** | W2 |
| Folders | **Groups**, nestable | W2 |
| Favorites | Favourites row at the top of a Space | W2 |
| Today tabs and auto-archive | Temporary tabs; tidying after a chosen time or never; Archive | W2, W3 |
| Collapsible sidebar | Sidebar, collapsible, either side; **plus a tab strip** | W2 |
| Command Bar | Command bar: addresses, tabs, Sites, Spaces, history, commands | W2 |
| Split View | Split view | W5 |
| Peek | **Preview** | W7 |
| Little Arc | **Quick window** | W7 |
| Air Traffic Control | **Link rules**: a pattern chooses the Space and the engine | W6, W7 |
| Boosts | **Tweaks**, with a visual editor | W8 |
| Library | Library: downloads, archive | W1, W3 |
| Automatic picture-in-picture, media controls | The engines' own picture-in-picture, with controls in the sidebar | W7 |
| Sidebar sync | Sync | W9 |
| Arc Search on phones | The full browser on phone and tablet | all |
| Tidy, Arc Max (AI) | Not built in; plugin territory | — |
| Easels, Notes | — | — |
| — | Engine choice per Site; Guard; sleeping tabs; plugins; foldables | W3–W8 |

---

## 4. Engines

**Terminology.** In this plan **engine** means a web engine (WebKit, Chromium). The Rust library is **the core**.

### 4.1 Principle: the system's engine first

Every platform here already ships a maintained, sandboxed, self-updating web engine with an embedding API. WebTiger uses it as the default. A second engine is added only where it can be embedded without forking it. It is always optional: downloaded when first asked for, loaded when first used.

- **Tier 1, system.** No download; patched by the OS vendor. The default.
- **Tier 2, pack.** An optional signed download that WebTiger keeps current. Supported.
- **Tier 3, experimental.** Opt-in and labelled.

### 4.2 The matrix

| Platform | WebKit | Chromium |
|---|---|---|
| macOS | **Tier 1.** WKWebView | **Tier 2.** A pack built on the Chromium Embedded Framework (CEF), Phase W6 |
| iPadOS, iOS | **Tier 1.** WKWebView | **Not planned** |
| Windows | **Not planned** | **Tier 1.** WebView2, the Evergreen runtime Windows 11 ships |
| Android | **Tier 3, after 1.0** | **Tier 1.** Android System WebView |

Why each gap exists, and what would close it:

- **Chromium on iPhone and iPad.** Apple permits other engines only in the EU (iOS 17.4 and later) and Japan (iOS 26.2 and later), under an entitlement with its own security terms. It means shipping and patching a full port of Blink. As of mid-2026 Google, Microsoft and Mozilla had prototypes and none had shipped. *Closes if* Apple widens the rule and a maintained Blink port exists to embed.
- **WebKit on Windows.** WebKit's Windows port is documented as a tool for developing and testing WebKit. A maintainer's November 2025 update lists no WebRTC and no camera or microphone capture, and asks for contributors. Kagi worked on Orion for Windows through 2026 and stopped in October, saying it will publish the code. *Closes if* that code, or upstream, becomes something a small project can keep secure.
- **WebKit on Android.** Igalia's WPE WebKit for Android is real (0.3.x on Maven Central) and young. *Planned as* an experimental pack after 1.0, delivered as an on-demand module.

The engine picker appears only where more than one engine is installed. The adapter (§4.4) is engine-neutral, so a gap can close without a change to the core.

### 4.3 Why not fork Chromium

| | System engines | A Chromium fork |
|---|---|---|
| Security patches | The OS vendor's. WebView2 and Android WebView update themselves about every four weeks | Ours to merge and ship within days, indefinitely |
| Download | A budget of 25 MB (§7) | 150–250 MB |
| Build | Minutes | A 100 GB checkout and hours per platform |
| Who it takes | The owner, with agents | A standing engine team |
| iPhone and iPad | The same approach | Not allowed anyway |
| What a fork buys | — | Chrome Web Store extensions on desktop, blocking in the network layer on every platform, site isolation on Android, engine-level fingerprinting defences |

Arc stopped because one small company could not carry that security upkeep. The cost of the system-engine choice is §4.7.

### 4.4 The engine adapter

Each shell implements one small interface per engine. The core drives it through the contract (§5.5) and knows engines only by the capabilities they report.

| Capability | WKWebView | CEF (Mac pack) | WebView2 | Android WebView |
|---|---|---|---|---|
| Logins per Space | `WKWebsiteDataStore(forIdentifier:)` | a request context per cache path | a profile (`ProfileName`) | a profile (`ProfileStore`) |
| Private window | a non-persistent store | an in-memory context | `IsInPrivateModeEnabled` | a profile deleted on close |
| Blocking requests | compiled content rule lists | a callback per request | `WebResourceRequested` | `shouldInterceptRequest` |
| Scripts at document start | `WKUserScript`, own content world | DevTools protocol, own world | `AddScriptToExecuteOnDocumentCreated`, page world | `addDocumentStartJavaScript`, page world |
| Sleep, keeping back and forward | `interactionState` | address only | suspend first; then address only | `saveState` / `restoreState` |
| Developer tools | Web Inspector | Chrome DevTools | DevTools | remote only |
| DRM video | FairPlay (W0 verifies) | no, in practice | PlayReady and Widevine | Widevine |
| Password autofill | the system's (W0 verifies on the Mac) | none | built in, per profile | the Autofill framework |
| Safe-browsing warnings | yes | no | SmartScreen | yes |
| Web extensions | `WKWebExtension` | partial | loads, without toolbar or popup UI | none |
| Processes | managed by WebKit, one per tab | Chromium's, with site isolation | Chromium's, with site isolation | one renderer per profile |

The same adapter test suite runs against every engine (§8, rule 3). A capability an engine lacks is hidden in the UI, never faked.

### 4.5 Choosing an engine

Where two engines are installed, the first match wins:

1. the Site's own setting;
2. a link rule (`*.figma.com` → Chromium);
3. the Space's default;
4. the global default;
5. the platform's Tier 1 engine.

In the window, an engine mark sits in the address field. Clicking it offers "Reopen in Chromium" and "Always open this site in Chromium". The command bar has the same two commands. When a page asks for something the current engine lacks, WebTiger offers the other one. When a page needs DRM video, it offers WebKit.

**Logins do not cross engines.** A Space has one set of logins per engine. Signing in under WebKit does not sign you in under Chromium. The choice is therefore remembered per Site, and the first switch says so in one sentence. Copying cookies between engines is out of scope: it is fragile and it weakens isolation.

### 4.6 The Chromium pack on the Mac

- **What.** CEF, BSD-licensed, is Chromium's content layer with an embedding API. Its stable builds follow Chromium's release line.
- **Delivery.** A separate signed and notarised download, kept outside the app bundle so the app's signature stays intact. The size is shown before the download; it is in the order of 100 MB compressed (W0 measures). The Mac App Store build cannot download code, so it is WebKit only (WT-D14).
- **Loading.** Nothing of Chromium loads until the first Chromium tab opens.
- **Embedding (WT-D3).** Recommended: in-process and windowed. CEF draws into a native child view, so input, text composition and accessibility are native. The cost: once loaded, Chromium's browser-side code stays in WebTiger's process until quit, and a crash there is a crash of the app. The alternative is a separate host process that renders off-screen into shared surfaces. It isolates crashes and memory, but every input event, composition, popup and accessibility node must then be forwarded by hand. W0 measures both.
- **Updates.** Independent of the app. A signed manifest names the minimum safe version; an older pack will not load pages until it updates.
- **Media.** CEF's official builds leave out H.264 and AAC, and DRM needs both a build flag and the services' own verification. So in 1.0, Chromium on the Mac is for sites that need Chromium: WebUSB, WebSerial, WebHID, Web MIDI and Chrome-only web apps. Video sites are sent back to WebKit, which has the system's codecs and FairPlay. A custom build that uses the system's decoders is a candidate after 1.0 and needs a codec-licensing review (§13).

### 4.7 What system web views cannot do

| Limit | Where | What WebTiger does |
|---|---|---|
| Web push and web notifications | WKWebView and Android WebView (W0 checks WebView2 and CEF) | The permission is not offered. A Site can be kept awake instead. Documented |
| WebUSB, WebSerial, WebHID, Web Bluetooth | WebKit does not implement them; Android WebView; WebView2 lacks WebUSB (W0 checks the others) | Mac: the Chromium pack. Elsewhere: unsupported, and the page is told so |
| Browser extensions | See §4.4 | Core-hosted plugins on every platform (§5.10) |
| A hook on every request | WKWebView has none | Compiled rule lists plus scripts (§5.8) |
| Site isolation | Android WebView: tabs of one Space share a renderer; Spaces are separate | Recover from a lost renderer per Space; documented |
| The engine's version is the system's | WKWebView moves with the OS; an Android device without Play services may lag | The engine's version is checked at start, with a plain message if too old |
| Restoring back and forward after sleep | WebView2, CEF | Suspend before discarding. A woken tab reloads its address; its earlier pages stay in the tab's history menu |
| Session cookies across a restart | WebKit drops them at quit | WebTiger saves and restores them for Sites, encrypted at rest (a setting, on by default) |
| Per-tab mute, process tuning | WKWebView's public API has neither | Media is paused instead. No private API is used, so the App Store stays open |
| Fingerprinting defences | All | Known fingerprinting scripts are blocked by list. No stronger claim is made |
| Memory per tab | WKWebView and Android WebView expose none | Shown where measurable (WebView2, CEF) |
| Proxy or DNS-over-HTTPS per Space | Uneven | The OS settings apply in 1.0 |

---

## 5. Architecture

### 5.1 Stack

| Layer | Choice |
|---|---|
| Core | Rust, pinned stable. tokio; rusqlite with bundled SQLite and FTS5; prost; `adblock-rust`; `rquickjs` (QuickJS-NG); `url`, `idna`, `publicsuffix`; rustls with the platform verifier |
| Contract | Protobuf under `Core/proto/webtiger/core/v1/`, a four-function C ABI and two fast-path calls (§5.5); `buf` lint and breaking-change checks |
| Apple shell | Swift 6, strict concurrency. SwiftUI, with AppKit for the Mac window, outline and toolbar, and UIKit where SwiftUI falls short. swift-protobuf. XcodeGen |
| Windows shell | C++20, C++/WinRT, WinUI 3 on Windows App SDK 2.x, WebView2, nanopb. VS 2026 Build Tools alone. MSIX |
| Android shell | Kotlin, Jetpack Compose, Material 3 adaptive, Jetpack WindowManager, `androidx.webkit`, Wire; JNI through `wt-ffi` |
| Design | `Design/tokens.json` → Swift, XAML and Kotlin |
| Strings | `Localization/strings.json` → String Catalog, `.resw`, `strings.xml`. The core speaks codes; the shells speak words |
| Services | None required. Static signed files (list bundles, the pack, the plugin index, update feeds). An optional sync relay in Go, by TigerCodes' service conventions |

### 5.2 Overview

```
┌──────────────────────┐ ┌──────────────────────┐ ┌──────────────────────┐
│ Apple shell (Swift)  │ │ Windows shell        │ │ Android shell        │
│ SwiftUI·AppKit·UIKit │ │ (C++/WinRT) WinUI 3  │ │ (Kotlin) Compose     │
│ engine adapters:     │ │ engine adapter:      │ │ engine adapter:      │
│  WKWebView           │ │  WebView2            │ │  System WebView      │
│  CEF pack (Mac)      │ │                      │ │  (WPE pack, later)   │
└──────────┬───────────┘ └──────────┬───────────┘ └──────────┬───────────┘
           │   protobuf over a C ABI, plus two fast-path calls │
           ▼                        ▼                          ▼
┌────────────────────────────────────────────────────────────────────────┐
│ WebTiger Core (Rust)                                                   │
│  wt-ffi   contract dispatch, sink, panic fence, fast path              │
│  wt-app   headless view models: sidebar, tab strip, address field,     │
│           command bar, Guard panel, Library, settings                  │
│  domain   wt-spaces · wt-tabs · wt-history · wt-omnibox · wt-guard ·   │
│           wt-router · wt-permissions · wt-downloads · wt-plugins ·     │
│           wt-sync · wt-import · wt-store · wt-core                     │
│  tools    wt-cli (`webtiger`) · wt-testkit (fake engine, fixtures)     │
└────────────────────────────────────────────────────────────────────────┘
```

The division of labour: **the core decides, the shell draws, the engine renders.** A shell owns windows, gestures, menus, and the web views themselves. It holds no browser logic that a second shell would have to repeat.

### 5.3 Repository layout

```
webtiger/
├── Core/           the Rust workspace: crates/, proto/, xtask/, fuzz/
├── Packages/       one SwiftPM package: WTDesign, WTContract, WTCoreClient,
│                   WTWebKit, WTChromium (Mac), WTUI
├── Apps/           Mac and iOS targets that only host WTUI's root views
├── Windows/        WebTiger.sln: the app, WebTiger.Core (contract client), tests
├── Android/        app, core-engine (JNI), core-design, core-ui, feature-*
├── Plugins/        the SDK: webtiger.d.ts, the manifest schema, templates, examples
├── Design/         tokens.json, icons
├── Localization/   strings.json
├── Fixtures/       filter lists and a request corpus, the test site, import samples
├── Tools/          bootstrap, test.sh and test.ps1, perf, testsite, lists, proto, release
└── docs/features/  one plan per feature; this file is 000
```

### 5.4 Core crates

| Crate | Job |
|---|---|
| `wt-core` | Ids, addresses (parsing, fix-up, registrable domain), clocks |
| `wt-store` | The SQLite schema, migrations, observation; one writer thread, a pool of readers |
| `wt-spaces` | Spaces, Groups, Sites, Favourites, ordering, login sets |
| `wt-tabs` | Tabs, windows, split layouts, the lifecycle policy (§5.7), session restore |
| `wt-history` | Visits, ranking, full-text search |
| `wt-omnibox` | Classifying input, suggestions, search providers |
| `wt-guard` | The blocker: lists, the request engine, the WebKit rule compiler, page fixes, link cleaning (§5.8) |
| `wt-router` | Link rules: an address chooses a Space and an engine |
| `wt-permissions` | Site permissions per login set |
| `wt-downloads` | Download records and policy |
| `wt-plugins` | The plugin host, manifests, permissions, the API (§5.10) |
| `wt-sync` | Encrypted records and containers (§5.11) |
| `wt-import` | Arc, Chrome, Safari, Firefox, Edge |
| `wt-app` | Headless view models; what the shells subscribe to |
| `wt-proto`, `wt-ffi`, `wt-sys` | Generated types; the C ABI, panic fence and host bridge; audited platform glue |
| `wt-cli` | `webtiger`: a headless driver, the importers, plugin tools |
| `wt-testkit` | A fake engine host, fixture pages, builders |

Rules: no `unwrap`, `expect` or `panic` outside tests; `unsafe` only in `wt-ffi` and `wt-sys`; clocks, transports and hosts are injected so scenario tests run in milliseconds.

### 5.5 The contract between core and shell

The contract is one protobuf `Envelope` carrying requests, responses, events, host requests and host responses, over four C functions. Screens subscribe and receive snapshots. Every call is asynchronous.

```c
typedef void (*wt_sink)(void *context, const uint8_t *envelope, size_t len);

uint64_t wt_core_start(const uint8_t *config, size_t len, wt_sink sink, void *context);
int32_t  wt_core_send(uint64_t core, const uint8_t *envelope, size_t len);
int32_t  wt_core_stop(uint64_t core);
uint32_t wt_core_version(void);

/* The fast path: synchronous, lock-free, callable from any thread. */
int32_t  wt_guard_check(uint64_t core, const wt_request *request, wt_verdict *out);
int32_t  wt_nav_check(uint64_t core, const wt_navigation *navigation, wt_nav_verdict *out);
```

**The one deliberate addition** is the fast path. A page makes hundreds of requests, and three of the four engines ask about each one on their own threads. Android also wants a navigation decision synchronously. Both calls read an immutable snapshot that the core swaps when lists or rules change. They never queue, lock or allocate on the way.

**What the core asks of a shell** (host requests):

| Service | Purpose |
|---|---|
| `Engines` | Which engines exist, their versions and capabilities; install, update and remove a pack |
| `WebViews` | Create, load, command (back, forward, reload, stop, find, zoom), suspend, snapshot, save state, destroy |
| `LoginSets` | Create, clear and delete an engine's data store for a login set |
| `RuleLists` | Compile and attach WebKit content rule lists; attach scripts |
| `Vault` | Keep a secret (the sync key, saved session cookies' key) in the OS vault |
| `Lifecycle` | Foreground, background, memory pressure, power, fold and window-size changes |
| `System` | Open an outside link, pick a file, show a notification, report default-browser state |

**What a shell reports** (events): navigation started, committed, finished or failed; title, icon and address changes; a new window asked for; a permission, download, sign-in or certificate prompt; audio and capture state; full screen; a lost renderer.

Shell tests replay recorded envelopes through a `ScriptedCore`, so a screen is tested without the core.

### 5.6 Data model and what the words mean

| Table | Holds |
|---|---|
| `login_set` | An identity: one data store per engine. Private ones are never written |
| `space` | Name, icon, colour, order, its `login_set`, defaults (engine, Guard, search, tidying) |
| `node` | The sidebar tree: a Group or a Site, its parent and position. A Site has a home address and optional engine and Guard settings |
| `tab` | A browsing context: Space, the Site it belongs to (if any), address, title, engine, state, last use, saved state and snapshot |
| `window` | Windows, their active Space and tab, layout mode and split |
| `visit` | History, per login set, with a full-text index |
| `site_setting` | Per login set and site: permissions, Guard switches, zoom, engine |
| `route_rule` | Link rules |
| `filter_list`, `custom_rule` | Guard's lists and the user's own rules |
| `plugin`, `plugin_storage` | Installed plugins, granted permissions, per-Space switches, their data |
| `download`, `archive` | The Library |
| `journal` | Every organising action, written first. It gives undo and feeds sync |

**The words:**

- **Space.** A named place with a colour and an icon. A new Space gets its own logins. At creation it may instead share another Space's (two work projects, one work account). The model is many Spaces to one login set from the first schema; Arc, Nook and Crest all arrived there late.
- **Group.** A folder in a Space. Groups nest and collapse. In the tab strip a Group is a coloured, collapsible chip.
- **Site.** A saved place that is also a tab. Opening it wakes its tab. It can wander from its home address and be sent back. Closing it puts it to sleep; the entry stays.
- **Temporary tab.** Opened by a link or by typing. It lives below the Sites, and is tidied into the Archive after the chosen time, or never. Dragging it into a Group makes it a Site.
- **Favourites.** A row of icon tiles at the top of a Space.
- **Private window.** A throwaway login set. Nothing is written; closing it wipes it.

Moving a Site between Spaces with different logins reloads it under the new ones, and says so. Deleting a Space offers to delete its logins if no other Space shares them. "Clear this Space's data" is one command.

### 5.7 Tab lifecycle and memory

```
 Live ──hidden──► Background ──idle, pressure or limit──► Asleep ──tidy time──► Archived
   ▲                  │                                     │       (temporary tabs only)
   └──────────────────┴────────────── activate ─────────────┘
```

- **Live.** A web view exists and is on screen.
- **Background.** A web view exists, off screen. The engine throttles it; WebView2 is suspended outright.
- **Asleep.** The web view is gone. A snapshot, the saved state and the scroll position remain. Cost: a database row and a small image on disk.
- **Archived.** Out of the sidebar, in the Library, restorable.

The policy lives in the core and runs on an injected clock, so it is tested in milliseconds. A tab goes to sleep when any of these holds: it has been in the background past the sleep time (30 minutes by default on desktop); the system reports memory pressure; the number of live tabs passes the device's limit (3 on a phone, 6 on a tablet or open foldable, set by memory on a desktop).

A tab is **kept awake** while it plays audio, holds the camera or microphone, is in a call, has a download or upload running, has unsaved form text, has developer tools open, or belongs to a Site marked "keep awake". The sidebar shows why.

Starting the app wakes one tab: the one on screen. Every other tab restores asleep.

Per engine: WKWebView saves `interactionState` and is released. Android WebView saves its state, and `onRenderProcessGone` is handled so a killed renderer costs one Space a reload, not the app. WebView2 is suspended, then closed. A CEF browser is closed; the pack itself stays loaded until quit.

### 5.8 Guard: ads and trackers

**One blocker, acting where each engine allows.** The core holds Brave's `adblock-rust` (MPL-2.0). Brave's 2026 rework cut its memory by 75 %, about 45 MB saved, which leaves roughly 15 MB for the default 100,000 filters.

| Engine | Requests | Page fixes (hiding, scriptlets) |
|---|---|---|
| CEF | `wt_guard_check` from the request callback | Scripts in an isolated world |
| WebView2 | `wt_guard_check` from `WebResourceRequested` | Scripts at document creation |
| Android WebView | `wt_guard_check` from `shouldInterceptRequest`, off the main thread | Scripts at document start |
| WKWebView | Content rule lists compiled from the same filters | CSS rules in the lists, plus `WKUserScript`s |

**WebKit is the hard case**, because it offers no per-request hook. The core converts filters to WebKit's rule format with `adblock-rust`'s own converter. The shell compiles them once per list version, off the main thread, and WebKit caches the result. Lists are cut into chunks under WebKit's 150,000-rule ceiling (Nook uses 30,000; W0 measures compile time on the oldest supported iPhone). Exceptions cannot reach across rule lists, so each chunk carries the exceptions that apply to it. Redirects, header changes and response rewriting have no WebKit equivalent; scripts cover part of that. Blocking on WebKit will be somewhat weaker than on the Chromium engines. The plan says so, and the corpus test in §8 measures the difference.

**Switches.** Three levels, the nearest wins: this site (within this Space's logins), this Space, everywhere. Each level can set ads, trackers, cookie notices and third-party cookies. The Guard mark in the address field shows a count and opens a panel with the site and Space switches. "Site broken? Turn Guard off here" is one click and a reload. On WebKit a site's switch is a small allow-list rule list, regenerated in milliseconds.

**Lists (WT-D10).** EasyList, EasyPrivacy, the uBlock Origin lists, Peter Lowe's, and a regional list by locale. Optional: cookie notices, URLhaus. The user can add lists and write rules.

**Updates.** A job in this repository builds a signed bundle each day: the lists, the request engine in its serialised form, and the WebKit rule chunks. It is published as static files. Clients fetch one bundle with a conditional request and no identifier. If the bundle is unreachable they fall back to the lists' own addresses.

### 5.9 Privacy beyond blocking

| Protection | How |
|---|---|
| Separate identities | A login set per Space is itself the strongest tracker defence the app has |
| Link cleaning | Tracking parameters are removed from navigations in the core; "Copy clean link" |
| HTTPS first | Top-level loads are upgraded; a plain-HTTP fallback asks |
| Third-party cookies | Blocked by default on every engine (WebKit already does; the others are set to) |
| Global Privacy Control | Sent where the engine lets a header be added; exposed to scripts everywhere |
| Safe-browsing warnings | The engine's own, on (§4.4) |
| Search suggestions | Off by default; local suggestions always |
| Telemetry | None. Crash reports are opt-in |
| History | Local, per login set. "Forget this site" removes its history and data together |

**Passwords and passkeys (WT-D13).** The platform's own service first: the system AutoFill on iPhone, iPad and, if W0 confirms it reaches web forms, the Mac; the Autofill framework and Credential Manager on Android; WebView2's built-in save and fill and Windows Hello on Windows. Passkeys on Apple platforms need the browser passkey entitlement, requested in W0. Third-party managers are the gap on the desktop. The plan closes it with a credential-provider plugin interface, with TigerPassword as the first provider, and with web extensions on Apple later.

### 5.10 Plugins

**The goals.** A plugin is one folder. It needs no build step. It runs unchanged on all five platforms and under both engines. It costs nothing while idle.

**Why the core hosts them.** Web extension support is different on every engine and missing on Android's (§4.4). A plugin's logic therefore runs in a QuickJS runtime inside the core, and reaches pages through content scripts that each engine injects.

| Option | Verdict |
|---|---|
| **A. QuickJS in the core** | **Recommended.** About 1 MB of code. An interpreter with no JIT, so it runs on iOS. One runtime per plugin with a memory ceiling and a time limit per event |
| B. A hidden web view per plugin | Rejected. 30–80 MB each, and a different API per engine |
| C. WebAssembly components | Later, for heavy work. Any language, a strong sandbox, and a much higher bar for authors |
| D. WebExtensions only | Rejected as the base. Uneven on desktop, absent on Android. Kept as a compatibility layer |

**A whole plugin:**

```json
{
  "id": "com.example.hello",
  "name": "Hello",
  "version": "1.0.0",
  "main": "main.js",
  "permissions": ["tabs"],
  "commands": [{ "id": "hello", "title": "Say hello" }]
}
```

```js
webtiger.commands.on("hello", async () => {
  const tab = await webtiger.tabs.active();
  webtiger.ui.toast(`Hello from ${tab.title}`);
});
```

**What a plugin can add** (declared in the manifest): commands in the command bar; content scripts and styles for matching pages; toolbar buttons and menu items; sidebar panels and popovers (HTML in a sandboxed web view, created on demand, destroyed on close); address-field keywords and search providers; a new-tab page; themes (design-token overrides, including Space colours); filter lists; link rules; Space templates; a credential provider.

**Three easier kinds for people who do not write plugins:**

- **Tweaks.** "Tweak this site": pick an element to hide, change a colour or a font, add CSS or a line of script. A Tweak is a one-file plugin and can be exported and shared.
- **Userscripts.** A `.user.js` file with the usual header installs as it is. A defined subset of the `GM_` functions is supported.
- **Templates.** `webtiger plugin new` writes a working folder for each kind above.

**The API.** `tabs`, `spaces`, `groups`, `sites`, `windows`, `commands`, `storage`, `net.fetch`, `clipboard`, `notifications`, `ui`, `guard`, `router`, `history`, `scripting`, and events for each. Types ship as `webtiger.d.ts`, so an editor completes them without a build.

**Permissions.** Named in the manifest, shown in plain words at install, and enforced in the core: `tabs`, `spaces:read`, `history:read`, `net:https://api.example.com/*`, `scripting:*://*.example.com/*`. All-sites scripting, history and network access are marked as sensitive. A plugin is switched on per Space, so one can run in Work and not in Personal. There is no cookie API, no file system and no native code in 1.0.

**Idle cost.** A plugin's runtime is created for an event and released after 30 seconds of quiet, as an extension's service worker is, at a fraction of the memory.

**Making one.** "Load from folder" in settings reloads on every save. A plugin inspector shows logs, events and storage. `webtiger plugin pack` writes a signed `.wtplugin`. The public index is a file in a Git repository that anyone can propose a change to; no store server exists.

**Security.** Nothing installs or changes plugin code except the user's own action. Sync carries which plugins are installed and their settings, never code from a server. A signed block list can disable a plugin found to be malicious. Content scripts run in an isolated world where the engine has one (WebKit, CEF); on WebView2 and Android WebView they share the page's world, so the bridge gives them message passing only, validated in the core. Whether the runtime needs a second wall (QuickJS compiled to WebAssembly) is an open question for W0 and the W10 review.

**Web extensions.** After the plugin system, Apple first, through `WKWebExtension`. Nook shows it works and what it costs: shims for the APIs WebKit lacks, and password managers that need special care. It ships with a tested list, password managers at the top.

### 5.11 Sync

- **What syncs:** Spaces, Groups, Sites, Favourites, settings, link rules, the user's filter rules, the plugin list and plugin settings. **Not** cookies or site data (never), passwords (the platform's job), or history and open tabs (later, opt-in).
- **How.** Journal entries become records, encrypted on the device under a key that never leaves the user's devices. A new device joins by scanning a code from one already joined. There is no account.
- **Where (WT-D12).** A container interface: sync writes to whichever container the user chooses. Recommended first container: a small Go relay that stores opaque records, run by TigerCodes and self-hostable. Second: a folder the user picks, which works with iCloud Drive, OneDrive, Dropbox or Syncthing and needs no server at all.
- **Conflicts.** Newest change per field wins; a deletion wins unless the item was changed after it; order uses fractional positions so two devices can both insert.

### 5.12 Distribution and updates

| Platform | Channels (WT-D14) |
|---|---|
| macOS | Direct: Developer ID, notarised, Sparkle; this build offers the Chromium pack. Mac App Store: WebKit only |
| iPadOS, iOS | App Store, TestFlight |
| Windows | Signed MSIX direct, winget, Microsoft Store |
| Android | Google Play; F-Droid (a build with no proprietary dependencies); a signed APK |

### 5.13 Security principles

1. **A browser is a security product.** The address field never lies: the registrable domain is emphasised, look-alike international names are shown in their encoded form, and prompts name the site that asked.
2. **No private APIs.** The App Store and Play stay open, and an OS update cannot break what was never promised.
3. **Signed, pinned supply.** List bundles, the Chromium pack, plugin packages, the block list and app updates are each signed. The signing roots are kept offline.
4. **No servers that see browsing.** The static files carry no identifier. The sync relay sees ciphertext.
5. **`SECURITY.md` and a disclosure address from the first public commit.** A review of the address field, prompts and the plugin wall is a W10 gate.

---

## 6. Design and UX

### 6.1 Two layouts, one model

The layout is a view setting, per window. Switching it moves nothing and loses nothing.

| | Sidebar | Tab strip |
|---|---|---|
| Spaces | Icons at the foot of the sidebar; swipe to change | A chip at the leading end of the strip |
| Favourites | A row of tiles at the top | Icon-only tabs |
| Groups and Sites | A tree | Coloured chips that open into tabs |
| Temporary tabs | Below a divider | After the Groups |
| Address field | At the top of the sidebar | In a toolbar under the strip |
| Collapsed | A rail of icons, or hidden until the pointer reaches the edge | — |

The sidebar can sit on either side. Zen refuses a tab strip; Chrome's vertical tabs are an afterthought. Doing both properly is a point of difference.

### 6.2 What every window has

- **Command bar.** One field for addresses, searches, open tabs, Sites, Spaces, history, settings and plugin commands. Everything in the app is reachable by typing.
- **Split view.** Two pages side by side, three on a wide window. A split is saved with its Group.
- **Preview.** A link clicked in a Site opens over it; one key keeps it as a tab, another dismisses it.
- **Quick window.** A link from another app opens in a small window, unless a link rule sends it to a Space. One key moves it into a Space.
- **Guard and engine marks** in the address field (§5.8, §4.5).
- **Undo** for every organising action, including closing a tab and deleting a Group.

### 6.3 First run

No account, and under a minute. Four choices, each with a default: import (Arc, Chrome, Safari, Firefox, Edge); a style; a search engine; make WebTiger the default browser.

The style is one of two presets (WT-D15):

- **Classic.** Tab strip. Tabs stay until closed.
- **Tidy.** Sidebar. Temporary tabs are archived after 12 hours.

Both start with one Space. Every part of either preset can be changed later, one setting at a time.

### 6.4 Phones

In a compact width the address field sits at the bottom, within reach of a thumb, with the option of the top. The tab switcher is a full-height sheet. The layout setting chooses its form: the same tree as the sidebar, or cards. Swiping the address field moves between tabs; swiping the switcher's header moves between Spaces.

### 6.5 Tablets and Android foldables

1. **Window size classes, never device checks.** Folded is compact and gets the phone layout. Unfolded is medium or expanded and gets the sidebar or the tab strip.
2. **Folding loses nothing.** The activity is not recreated on a size change. Web views are held outside the composition, keyed by tab, so a fold does not reload a page or move its scroll position.
3. **Postures** come from Jetpack WindowManager's `FoldingFeature`:
   - *Tabletop* (half open, hinge horizontal): the page on the upper half; the address field, the tab list and media controls on the lower.
   - *Book* (half open, hinge vertical): split view with the divider on the hinge.
   - Nothing interactive is placed under a hinge that hides pixels.
4. **Large-screen rules.** Android 17 ignores orientation and resize restrictions above 600 dp, so the app is resizable from its first build. Two WebTiger windows side by side, drag and drop of tabs and links between them, keyboard, mouse and stylus are all in Phase W5.
5. **iPad.** Both layouts, several windows, the menu bar, pointer and keyboard, an external display.

### 6.6 iPhone Duo

The facts and the principles, which hold for any app:

- The outer display (5.4 inches) is compact width. The inner display (7.6 inches) is regular in both directions and reports as a phone.
- On the outer display, and on the inner display in landscape, iOS moves bars into one vertical bar at the side.
- Folding changes the scene's size and traits, not the app. The process and its web views stay.
- Split views become 50/50 around the fold. Sheets and menus keep out of it.

What it means for a browser:

1. **The toolbar is made of standard items with icons and visibility priorities,** so iOS can stand them up vertically. Back, forward, the tab switcher and the Space switcher keep their places; the rest overflow.
2. **The address field and tab strip are custom chrome,** so they read the reserved regions themselves (`ReservedRegion`) to stay clear of the fold and the camera.
3. **The inner display gets the tablet layout:** sidebar or tab strip, and split view with one page each side of the fold.
4. **Two windows on the inner display** show two Spaces at once.
5. **Laptop pose:** the page above the fold; the address field, tab list and keyboard below.
6. **No function depends on a pose.** Apple's rule.

This Mac already has what the Duo needs: macOS 27.2.1 and Xcode 27.2. The minimum OS stays at 26, with the iOS 27 APIs behind availability checks.

### 6.7 Design system and accessibility

One token file drives all three shells. Each platform keeps its own idiom: Liquid Glass on Apple, Material 3 on Android, Fluent and Mica on Windows. The information architecture is identical everywhere. A Space's colour tints its window.

Accessibility is a requirement of every phase: VoiceOver, TalkBack and Narrator; full keyboard use; text scaling; contrast themes; reduced motion; WCAG 2.2 AA.

---

## 7. Performance and memory budgets

Targets, measured by automated tests on reference devices (an 8 GB M2 MacBook Air, an 8 GB Windows laptop, a mid-range Android phone, an iPhone 15-class phone). W0 measures the baselines and may revise a number once, with the reason recorded.

**The standard session:** 5 Spaces, 200 Sites, 40 temporary tabs, one live page from the local test site.

| Budget | Target |
|---|---|
| WebTiger's own processes in the standard session (shell and core; the engine's processes are counted separately) | ≤ 120 MB macOS · ≤ 150 MB Windows · ≤ 110 MB Android · ≤ 90 MB iOS |
| The core's share, with the default lists loaded | ≤ 30 MB |
| A sleeping tab | ≤ 4 KB of memory; ≤ 250 KB on disk |
| 100 sleeping tabs | add ≤ 5 MB in total |
| A Space with nothing live | no process; ≤ 100 KB |
| Cold start to a usable window | ≤ 500 ms Mac · ≤ 700 ms Windows · ≤ 800 ms Android · ≤ 600 ms iPhone |
| Start-up loads | exactly one page |
| Switch to a live tab | the next frame |
| Wake a sleeping tab | snapshot within 50 ms; the engine loading within 100 ms |
| Address-field suggestions over 100,000 history rows | ≤ 10 ms at p95 |
| A Guard decision | ≤ 25 µs at p95 on the phone |
| Page-load cost of Guard on callback engines | ≤ 3 % against Guard off |
| Sidebar with 5,000 rows | no dropped frame at 120 Hz |
| Idle with every tab asleep | no timers; at most one wake-up a second |
| Download size | ≤ 25 MB on every platform, the Chromium pack apart |
| A contract round trip for a 200-row sidebar page | ≤ 2 ms at p95 |

**Published, reproducible numbers.** Phase W3 adds a public run of the kind Brave published in September 2026 (50 sites, 10 runs, memory and energy), scripted in `Tools/perf` so that anyone can repeat it.

**Memory in the app.** A Memory view lists live tabs and why each is awake, with a figure per tab where the engine provides one, and a "sleep all others" command.

---

## 8. Testing strategy: how the agent tests while it builds

1. **Headless first.** `cargo test` in `Core/` is the fast loop. The core runs against a **fake engine host**: scripted pages, redirects, subresources and failures, on a manual clock. "Open 50 tabs, advance 31 minutes, 49 are asleep" is a unit test.
2. **A test site in the loop.** `Tools/testsite` serves, on several local hostnames, pages that sign in, set every kind of storage, register a service worker, load third-party trackers, redirect, download, upload, ask for permissions and play media.
3. **One adapter suite for every engine.** The same behaviours run against WKWebView on Mac and iOS, CEF, WebView2 and Android WebView: event order, new windows, downloads, permissions, isolation, deletion, blocking, sleep and restore, script injection, recovery from a lost renderer. It is a contract test: one suite, every implementation. It is how a "not planned" engine would later be admitted.
4. **Isolation is tested as a security property.** Sign in under Space A; Space B must see no cookie, storage entry, cache hit, service worker or permission. Deleting A's logins must leave nothing.
5. **A blocking corpus.** A fixture of requests with expected verdicts, against lists pinned by hash. The WebKit rule lists and the request engine are checked against the same corpus, and their difference is a tracked number.
6. **UI and snapshot tests** per shell: XCUITest; UI Automation on Windows; Compose tests with Roborazzi. The design catalogue is snapshot-tested in light, dark and contrast themes, at three text sizes, per size class and per fold posture. Accessibility audits run on every screen.
7. **Performance tests** assert §7 and fail the build when a budget is passed. A soak test opens and closes 10,000 tabs and requires flat memory.
8. **Fuzzing:** address fix-up, the list parser, plugin manifests, the importers, the contract decoder, link-rule patterns.
9. **Security tests:** address-field spoofing cases, prompt origins, download names, plugin permission enforcement, escape attempts from the plugin runtime.
10. **Fake mode:** `WEBTIGER_FAKE_*` knobs seed a showcase of Spaces and pages, inject memory pressure and pin the clock, so UI suites are deterministic.
11. **One command.** `Tools/test.sh` and `Tools/test.ps1`, with the same sub-commands on every platform: `lint`, `core`, `unit`, `ui`, `perf`, `all`.
12. **Definition of done:** code, unit tests, a UI or snapshot test if it touches a screen, the adapter suite if it touches an engine, and the feature's `docs/features/NNN-*.md` updated.

---

## 9. Phases

Each phase ends with a working build on every platform it touches. **The Mac leads; a phase is not done until the other shells reach the same level (WT-D16).** Sizes are relative. Estimates in weeks would be guesses, so none are given; the human-side steps in §13 are the real calendar.

### W0 — Foundations, spikes and the Arc importer (medium)

**Goal:** the stack proven on all three shells, the unknowns measured, and Arc data safe.

- **First, because of 22 December:** `webtiger import arc` in `wt-import` and `wt-cli`. It reads Arc's local data folder and writes WebTiger's portable format: Spaces, folders, pinned tabs, Favorites, routing rules and, where the data allows, Boosts as Tweaks. Fixtures come from a real Arc folder (§13).
- The repository of §5.3; the core skeleton; the contract and `wt-ffi`; three shells that each open one address in one isolated login set through the contract; fake mode; the design catalogue; `Tools/test`; CI.
- **Spikes, each ending in a recorded answer:**

| # | Question |
|---|---|
| Q1 | CEF on the Mac: in-process against a separate host; memory once loaded; crash containment; a pack outside the app bundle under the hardened runtime; pack size |
| Q2 | Isolation on all four engines, for every kind of storage, and deletion. The lowest Android WebView that has profiles. The cost of each extra login set |
| Q3 | WebKit rule lists: compile time and memory on the oldest supported iPhone; chunk size; the corpus difference against the request engine |
| Q4 | Request callbacks: the cost of `WebResourceRequested` and `shouldInterceptRequest` for every request; `adblock-rust`'s threading mode |
| Q5 | Sleep and restore fidelity per engine; saving WebKit's session cookies |
| Q6 | Baselines for every budget in §7 |
| Q7 | QuickJS on iOS, Android and Windows Arm64: size, memory per runtime, start time. Is a WebAssembly wall affordable? |
| Q8 | Passwords: does the Mac's AutoFill reach web forms in WKWebView? What each entitlement requires |
| Q9 | WinUI 3 with a WebView2 profile per Space in C++/WinRT; WinUI's memory baseline against the Windows budget |
| Q10 | iPhone Duo: custom chrome with reserved regions and vertical bars, in the simulator and in Device Hub |
| Q11 | DRM video in WKWebView; WebUSB, WebSerial and WebHID in WebView2 |

**Exit:** `Tools/test all` green; each app loads a page through the core; every question answered; §4 and §7 revised once where the answers require it.

### W1 — A browser (large)

**Goal:** a daily-drivable browser with one Space on every platform.

The store and journal; tabs and windows; navigation, new windows and popups; the address field with fix-up and search; history; downloads; find; zoom; permission prompts; error and certificate pages; sign-in prompts; uploads; context menus; outside links; **platform password autofill and passkeys**; session restore after a crash; the first-run flow with import.

**Exit:** the adapter suite passes on all four Tier 1 adapters; a day's ordinary use needs no other browser.

### W2 — Spaces, Groups, Sites; sidebar and tab strip (large)

Spaces with their login sets, colours and icons; shared logins; the tree with drag and drop; Favourites; temporary tabs; both layouts; the phone switcher; the command bar; keyboard maps per platform; several windows; private windows; undo.

**Exit:** the isolation tests pass; switching layout changes nothing in the model; 5,000 rows scroll within budget.

**Where W2 stands (2026-11-23).** Built: Spaces with their login sets and colours, and shared logins (024, 025); several windows and private windows (037); undo for a closed tab (033); the keys of each system (034 to 036); Sites (039), Groups (041) and Favourites (042); both layouts, on the Mac and on a touch screen's wide windows (043); a tab moved a place by a command, and on the Mac by dragging (045); the command bar: Spaces and commands in the address field's list ([048](048-the-command-bar.md)); a site's icon, on every system (046: where WebKit hands none over the app fetches the one a page names, as nobody); the phone's switcher (047); undo for what is done to the tree (044); the tidying of temporary tabs into a Space's Archive, for a Space that is given a time ([049](049-tidying-temporary-tabs.md): never, until a person says). Not built: a Space's icon; dragging in the tree on a touch screen (on the Mac a tab and a Group are dragged, along the top and down the side, [045](045-moving-in-the-row.md) §5b; elsewhere a tab is moved a place by a command); the phone's switcher as cards (it has the sidebar's tree, [047](047-the-phones-switcher.md); cards wait for pictures of pages). Of the exit: the isolation tests pass; that switching the layout changes nothing in the model is tested in the core (043 §8); **the 5,000 rows are measured in part** ([043](043-the-sidebar.md) §8a): the standard session is well inside the contract's budget; at 5,000 rows a change costs the core about 4 ms and tells the window the whole row again, which telling only what changed (W3) is to mend; the sidebars make their rows as they come into view; no frame was counted.

### W3 — Sleeping tabs and the budgets (medium)

The lifecycle of §5.7; snapshots; keep-awake reasons; live-tab limits; pressure handling; tidying and the Archive; the Memory view; every §7 budget as a failing test; the published run.

### W4 — Guard (large)

The request engine and fast path; lists and the daily bundle; the WebKit compiler; page fixes; the three levels of switch; the Guard panel; link cleaning; HTTPS first; third-party cookies; Global Privacy Control.

**Exit:** the corpus passes at its targets on every engine; a site's switch takes effect in one reload.

### W5 — Split view, tablets, foldables, iPhone Duo (medium)

Split view; Android postures and continuity; multi-window and drag and drop; iPad windows; the Duo on both displays and in every pose; Windows snap layouts, touch and pen. Runs alongside W6, which touches a different shell.

### W6 — Chromium on the Mac (large)

The pack: download, verification, updates, removal, the minimum-safe-version rule. The CEF adapter passing the adapter suite. The engine mark, the per-Site and per-Space choices, engine link rules, the "this page needs…" offers, DRM routing to WebKit.

**Exit:** any Site opens in either engine on the Mac; with the pack absent or unloaded, WebTiger's footprint is unchanged.

### W7 — Power and platform integration (medium)

Preview; the Quick window; link rules; picture-in-picture and media controls; print and PDF; share in and out; default-browser flows; Shortcuts and App Intents, jump lists, Android shortcuts; Spotlight for Sites; the importers for Chrome, Safari, Firefox and Edge.

### W8 — Plugins (large)

The host and its limits; manifests and permissions; the API; content scripts on every engine; panels; Tweaks and their editor; userscripts; the inspector; `webtiger plugin new`, `dev` and `pack`; the index and block list; six example plugins; the credential-provider interface with TigerPassword. Stretch: web extensions on Apple.

### W9 — Sync (medium)

Records, keys and pairing; the relay and the folder container; conflict rules; a test that replays two devices' journals in every order.

### W10 — Hardening, beta, 1.0 (large)

Soak, fuzz and performance gates; the security review; accessibility audits; localisation; opt-in crash reports; the update channels; contributor documents; the website; store submissions.

### After 1.0 (candidates, unordered)

Web extensions in breadth. WebKit on Android as an experimental pack. A CEF build with system codecs. History and open-tab sync. A Linux shell. WebAssembly plugins. Managed configuration. WebKit on Windows and Chromium on iPhone, each if its condition in §4.2 is met.

---

## 10. Milestone summary

| Phase | Outcome | Size |
|---|---|---|
| W0 | Stack proven on three shells; unknowns measured; Arc data importable | M |
| W1 | A browser you can use all day, one Space | L |
| W2 | Spaces, Groups, Sites; sidebar and tab strip | L |
| W3 | Sleeping tabs; budgets enforced and published | M |
| W4 | Guard on every engine | L |
| W5 | Split view; foldables and iPhone Duo | M |
| W6 | Chromium on the Mac | L |
| W7 | Preview, Quick window, link rules, system integration | M |
| W8 | Plugins, Tweaks, userscripts | L |
| W9 | Sync | M |
| W10 | Hardening and 1.0 | L |

W1 to W4 are the critical path. W5 to W9 can be reordered by what is wanted first.

---

## 11. Decisions

**All eighteen were accepted as recommended on 2026-11-18.** The alternatives stay in the table as the record of what was weighed.

| # | Decision | Recommendation | Main alternative |
|---|---|---|---|
| **WT-D1** | Engine strategy | **System engines; no fork.** A second engine only where it embeds without a fork | Fork Chromium everywhere: full extensions and control, at the cost in §4.3 |
| **WT-D2** | Engines at 1.0 | **Accept §4.2:** both on the Mac; WebKit on iPhone and iPad; Chromium on Windows and Android | Delay 1.0 for a second engine on more platforms; none is ready |
| WT-D3 | CEF on the Mac | In-process, windowed, loaded on first use; official CEF builds, with video and DRM sent to WebKit. W0 confirms | A separate host process; a custom build with codecs |
| **WT-D4** | The core | **One Rust core for all five apps from the first commit, with the view models in it** | Apple-native logic first, ported to Rust later |
| WT-D5 | The contract | One protobuf contract over a four-function C ABI, plus two fast-path calls | UniFFI, whose C++ binding is third-party |
| WT-D6 | Shell stacks | Swift; C++/WinRT and WinUI 3; Kotlin and Compose | — |
| WT-D7 | Minimum OS | macOS 26, iOS and iPadOS 26, Windows 11 22H2, Android 10 with a current WebView | 27 on Apple: simpler, fewer users |
| **WT-D8** | Licence | **MPL-2.0**, contributions under the DCO, and a trademark policy for the name | GPL-3.0 (conflicts with the App Store unless every contributor signs a CLA); Apache-2.0 (forks may close) |
| WT-D9 | Spaces and logins | A new Space has its own; it may share another's; many-to-one in the schema | Strictly one each: simpler, and wrong for two work Spaces on one account |
| WT-D10 | Guard defaults | On. The lists in §5.8. A signed daily bundle built here | Off by default; fetch every list from its origin |
| **WT-D11** | Plugins | **QuickJS in the core plus content scripts.** Web extensions later, Apple first | WebExtensions as the base: uneven, and nothing on Android |
| WT-D12 | Sync | No account; encrypted on the device; a self-hostable Go relay first, a folder second; Phase W9 | The user's own cloud storage only: no server, clumsier on phones |
| WT-D13 | Passwords and passkeys | The platform's service; entitlements requested in W0; no vault of our own; a provider interface with TigerPassword first | Build a vault: a second product |
| WT-D14 | Distribution | §5.12, with the Mac App Store build WebKit only | Direct only on the Mac |
| WT-D15 | First run | One Space. Two presets: Classic (strip, no tidying) and Tidy (sidebar, 12 hours) | Arc's behaviour as the only default |
| WT-D16 | Phasing | Parity per phase, the Mac leading; the Arc importer first | The Mac to 1.0, the others after |
| WT-D17 | Search and money | DuckDuckGo is the default search engine, shown at first run with the others to pick instead (Google and Bing among them); suggestions off; no search deal, no telemetry, opt-in crash reports. (The default is the owner's decision of 2026-11-20; before it, the user had to pick one at first run) | A default search partner: an engine that pays to be the default |
| WT-D18 | Names | Guard, Tweaks, Preview, Quick window, Link rules; "core" for Rust, "engine" for the web | — |
| WT-D19 | A certificate that is not trusted | A person may go on past one, after a warning, by two deliberate presses; the exception is one certificate, one site, one login set, until the app quits, and the page is marked not secure ([026](026-past-a-certificate.md)). The owner's decision, 2026-11-20 | No way past: a router or a device of one's own could not be reached at all |
| WT-D20 | Cookies | A cookie is sent to the site it belongs to and to no other, on every system, whoever moves the bytes. Where a shell fetches something itself it follows whoever sends it on by hand ([027](027-cookies-stay-with-their-site.md)). The owner's decision, 2026-11-20 | The system's downloader on Android, which hands one site's cookies to the next |
| WT-D21 | Searching from other apps | On Android, "Search with WebTiger" is in other apps' menu of selected text; the words are only ever searched for, with the engine the person chose ([028](028-search-from-other-apps.md)). The owner's decision, 2026-11-20 | Nothing of WebTiger's that other apps can start |
| WT-D22 | The mark beside the address | It speaks of a page that has arrived: nothing is said of an address on its way, or of one that failed ([029](029-the-mark-waits-for-the-page.md), [026](026-past-a-certificate.md)). The owner's word, 2026-11-20 ("use suggestions"), on a question put to them | A padlock from the address's scheme alone, before any connection is made |
| WT-D23 | Opening a download | The system opens it, not WebTiger: with whatever the device has for its kind on a Mac and on Android, and in the system's own preview on an iPhone and an iPad, which have no other way ([030](030-open-a-download.md)). The owner's, 2026-11-21 | Showing a downloaded file in a tab; an iPhone list with nothing to press |
| WT-D24 | Links from other apps | WebTiger answers the system's request to show an address of the web, on Android and the Mac, and so is among the browsers the system knows; only an address of the web is opened, and never mended or searched for ([031](031-links-from-other-apps.md)). Being the browser links go to stays the person's choice, in the system's settings | Leaving it until the first run asks about the default browser |
| WT-D25 | Bringing back a closed tab on a phone | The button for a new tab, held, offers "Reopen Closed Tab", on the iPhone, the iPad and Android ([033](033-a-closed-tab-comes-back.md) §6). The owner's, 2026-11-21, on what was built as provisional | A line across the window that says a tab was closed, with "Undo" |
| WT-D26 | What a private window is | A login set of its own, thrown away with the window; nothing of it is written or offered back, and nothing from history is offered in it; a closed private tab is not brought back; a download's entry is listed until the app quits and its file stays; two private windows share nothing; it says "Private" in words; on Android its profile is deleted at close and looked for again at start ([037](037-private-windows.md) §4). The owner's, 2026-11-21 | Offering the ordinary history in a private window; one login set for all private windows; no private windows on Android |
| WT-D27 | Where a private window is on a phone | The list of Spaces ends with "Private Window"; the window then says "Private" where a Space's chip is, and the page behind that chip says what private means, with Close Private Window and the way back to the other window ([037](037-private-windows.md) §5.3). The owner's, 2026-11-22, on what was built as provisional | A button of its own in the strip; a gesture |
| WT-D28 | An open tab offered while typing | A page that is open in another tab of the row is offered first, at most three such, and choosing one brings that tab in front; an empty tab that was typed in to get there is closed ([038](038-an-open-tab-is-offered.md) §6). The owner's, 2026-11-22: "close the empty tab", on what was built with the empty tab left open; the rest as it was built and reported | The places of history first; the empty tab left open |
| WT-D29 | What a Site is | A tab that is kept: first in its row; closed, it sleeps at its home and stays; brought in front, it wakes there; closing one never wakes another; it is of the row it was kept in; none in a private window. The words are "Site", "Keep as Site", "Stop Keeping"; the key is ⌘D on a Mac and Ctrl+D on Android; in the strip a Site has a mark, and is drawn fainter asleep ([039](039-sites.md) §6). The owner's, 2026-11-22, on the seven choices as recommended | A closed Site sleeping where it was, or Arc's two closes; "Pin Tab"; no key yet |
| WT-D30 | One catalogue of words, at build time | Every user-visible string is in one file; one Rust tool in `Core/` writes every platform's file from it, and `lint` fails on either machine when one is stale; Swift asks by generated name, so a deleted string stops the build; a string must take the same arguments in every language and on every platform. No shell asks the core for a word while it runs, and no word is fetched from anywhere ([040](040-one-catalogue-of-words.md)). The owner's, 2026-11-22: no cost in speed or in safety | Words asked of the core at run time; words fetched from a server |
| WT-D31 | A Site's home that moves | Where a load of a Site's home lands is its home: an address that sends the browser on to another is at that other ([039](039-sites.md) §7b). The owner's, 2026-11-22: "the site that is redirected to should be the decided home page for that site" | The Site is away at once, and its home is the address it was kept at for ever |
| WT-D32 | Where undo for the tree is asked from | On the Mac in the Edit menu, ⌘Z, when the keyboard is not in a field of words; on a touch screen in the menu of the thing nearest to what was done ([044](044-undo-for-the-tree.md) D5, D6). The owner's, 2026-11-23, when told these held up dragging: "use suggested choices for the blocked items" | The File menu with a key of its own; a line across the window with "Undo" after each thing done |
| WT-D33 | A page's icon where the engine hands none over | On Apple's systems the app fetches the icon a page names itself: one request, with no cookie and no login, nothing kept of what comes back but the picture ([046](046-a-sites-icon.md) D1). The owner's, 2026-11-23, in the same words | In the page, by a script; or no icons on Apple's systems |
| WT-D34 | The switcher's button on a phone | At the end of the toolbar, after the address field; and the strip stays on a phone ([047](047-the-phones-switcher.md) D2, D3). The owner's, 2026-11-23, in the same words, on what was built as provisional | In the strip, with Find, Zoom and History gone into one button's menu; or no strip on a phone |
| WT-D35 | What pages keep, in fake mode | In memory, on Apple's systems, as the core's store is: each login set a store of its own, written nowhere, so that nothing of one run of a suite is there in the next ([001](001-foundations.md) §7). The owner's, 2026-11-23: "i like the fake mode suggestion", of a suggestion made when a test's kept cookie failed the tests after it | On disk, with every test taking care to keep nothing |

### 11.1 Built on a recommendation, waiting for the owner

Four features of W2 are built on the recommendations of their own documents, the owner not yet having answered (the fourth, moving in the row, without dragging); a fifth, undo for the tree, in the core only. Each choice is one line here; its reasons and the other ways are in the feature's §6. An answer to any of them changes that feature, and nothing else.

**Groups** ([041](041-groups.md) §6, put on 2026-11-22):

1. One level of Groups first; Groups in Groups later.
2. A Group holds Sites only: putting a tab in one keeps it.
3. Shutting a Group hides its Sites and changes none of them: none is put to sleep.
4. Removing a Group leaves its Sites, in no Group.
5. A Group's colours are a Space's, or none.
6. In the strip a Group is a chip before its Sites; a press shuts or opens it; shut, it says how many it holds.
7. The commands: "Move to Group" (the Groups by name, "No Group", "New Group…") for a tab; "Edit Group…" and "Remove Group" for a Group. No command makes an empty Group.

**Favourites** ([042](042-favourites.md) §6, put on 2026-11-22):

1. A Favourite is a Site with a mark, not a bookmark that opens a new tab at each press.
2. A Favourite is in no Group.
3. Favourites are a Space's, not the same in every Space.
4. Until sites have icons, a tile shows the first letter of the Site's name. (On a phone the Space's chip is a letter in a square too: they look alike until there are icons.)
5. At most twelve in a row.
6. The command: "Add to Favourites" and "Remove from Favourites", where "Keep as Site" is.

**The sidebar** ([043](043-the-sidebar.md) §6, put on 2026-11-22):

1. The layout is each window's, and a new window has the one last chosen.
2. A new installation has the strip, not the sidebar.
3. The Mac first, then the iPad and Android's wide windows; a phone has no sidebar.
4. The sidebar on the leading side, at one width; the other side, resizing and collapsing later.
5. The toolbar and the address field at the top of the sidebar.
6. The command: "Tabs at the Side" and "Tabs at the Top"; on the Mac in the View menu, ⌃⌘S.
7. On a touch screen the switch is a button beside Find, Zoom and History; with a keyboard ⌃⌘S on an iPad and Ctrl+Shift+S on Android.

**Undo for the tree** ([044](044-undo-for-the-tree.md) §6, put on 2026-11-22; built, in the core and the shells; 5 and 6 are decided, WT-D32):

1. Two ways back, apart: "Reopen Closed Tab" as it is, and an undo of its own for what is done to arrange a row.
2. The whole arrangement is kept and put back, not the one thing that was done.
3. For the run only: nothing of the arrangement to undo after a quit.
4. The command says what it undoes ("Undo Remove Group").
5. ~~On the Mac in the Edit menu, ⌘Z, when the keyboard is not in a field of words.~~ Decided: WT-D32.
6. ~~On a touch screen in the menu of the thing nearest to what was done.~~ Decided: WT-D32.

**Moving in the row** ([045](045-moving-in-the-row.md) §6, put on 2026-11-22; 1 to 4 built in the core and 6 in the shells; 5, dragging, built on the Mac and not yet on a touch screen):

1. A tab that is moved becomes what its new neighbours are: among Sites a Site, in a Group that Group's, among Favourites a Favourite.
2. That goes both ways: a Site moved among the tabs that are no Sites is a Site no longer. (Which is why undo for the tree should be in the apps before dragging is.)
3. By the keyboard, a step moves a tab within its part of the row and never out of it.
4. A Group is moved whole among the Groups, and stays after the Sites that are in no Group.
5. Dragging in the sidebar and in the strip alike; on a touch screen after a long press.
6. The commands "Move Tab Left" and "Move Tab Right" in the tab's menu and the Mac's Window menu, with no key of their own yet.

**A site's icon** ([046](046-a-sites-icon.md) §6, put on 2026-11-23; built as recommended, on every system; the first is decided):

1. ~~**On Apple's systems the app fetches a page's icon itself** (WebKit hands none over): one request for the picture the page named, with no cookie and no login. It is a request the web engine did not make, and the picture is decoded in the app.~~ Decided: WT-D33.
2. An icon is kept for a host in the login set it came from, and is not shown in another.
3. A Site shows its home's icon, wherever it has wandered.
4. The page's own icon nearest 64 points; PNG, ICO, JPEG and GIF; no SVG yet.
5. Kept on disk: every Site's, and the thousand used most lately of the rest.
6. Drawn before a tab's name; on a tile, the icon alone; a Site keeps its mark.

**The phone's switcher** ([047](047-the-phones-switcher.md) §6, put on 2026-11-23; built as recommended, on the iPhone and on Android):

1. The sidebar's tree in a sheet first; cards when there are pictures of pages.
2. ~~Its button at the end of the toolbar, after the address field: a square with the number of tabs in it.~~ Decided: WT-D34.
3. ~~The strip stays on a phone.~~ Decided: WT-D34.
4. The number is of all the row's tabs, those of shut Groups among them.
5. The sheet covers the page, with the Space's name at its head.

**The command bar** ([048](048-the-command-bar.md) §6, put on 2026-11-23; built as recommended, in the core and in Apple's and Android's shells):

1. The address field is the command bar: one field, and no second one over the page.
2. A command's words are the shell's and its matching the core's: the shell sends the commands with the question.
3. A command is offered among the other rows whenever its words match; no mark is typed first.
4. Only what can be done now is offered.
5. The order: tabs, Spaces, commands, places.
6. Which commands: those that need nothing more than to be chosen.

**Tidying temporary tabs** ([049](049-tidying-temporary-tabs.md) §6, put on 2026-11-23; built as recommended in the core and in the shells of Apple's systems and Android):

1. A Space is made with no tidying, until the first run offers its two styles.
2. A tab's time is counted from when it was last in front.
3. The times: never, 12 hours, a day, a week, 30 days.
4. The time is the Space's.
5. Never the tab in front of a window; nor a tab with no page.
6. The Archive keeps a Space's thousand newest.
7. A tab brought back is at the end of the temporary tabs, in front, and loads its address.
8. Tidying is not undone by ⌘Z: bringing back is its undoing.
9. A Space's time is chosen where the Space is edited, and not where it is made.
10. The Archive is the Library's fourth list.
11. A row of the Archive pressed is a tab again, and the Library closes.
12. A private window's Library has no Archive.
13. On a phone the page that edits a Space does not raise the keyboard when it opens.

Also the owner's to say, from earlier: the words, the keys and the mark for a Site that has wandered ([039](039-sites.md) §7b: "Return to" and the Site's name; ⇧⌘H and Alt+Home; an outline for its mark).


---

## 12. Risks

| Risk | Mitigation |
|---|---|
| "WebKit or Chromium" disappoints off the Mac | Said plainly in §0 and in the app. The adapter keeps each gap closable. WT-D2 is the owner's to accept |
| System web views lack something a daily browser needs | §4.7 is the list. W0 verifies the uncertain rows. Web push is the visible loss on Apple and Android |
| Password managers on the desktop | W0 Q8. The provider interface and TigerPassword in W8. This is the top request in every Arc-like's tracker |
| The Chromium pack falls behind on security | It tracks CEF's stable line in CI. The signed minimum version stops an old pack loading pages |
| CEF in-process costs memory or stability | Loaded only on use. W0 measures. The separate-host design is the fallback |
| Two engines, two sets of bugs | The adapter suite. Capabilities are reported, never assumed. One engine per Site, remembered |
| WebKit blocking is weaker than uBlock Origin | The corpus tracks the difference. Scripts cover what rules cannot. The per-site switch is one click |
| WinUI 3's baseline breaks the Windows budget | W0 Q9 measures before any Windows screen is built |
| App Review: the browser entitlement, downloaded plugins | Requests go in during W0. Plugins are interpreted and permissioned; content scripts run in WebKit. Fallbacks: a JavaScriptCore runtime behind the same interface, or bundled plugins only on iOS |
| A malicious plugin | Permissions in plain words; per-Space switches; no cookie API; a signed block list; the runtime's limits; the W10 review |
| An address-field or prompt flaw | §5.13; spoofing tests from W1; an outside review before 1.0 |
| Arc users expect every Arc feature | §3.3 is the contract. The rest is plugin territory or "after 1.0" |
| Filter-list licences and hosting | Lists are data fetched at run time. A human check in §13 before bundling any |
| Old or vendor-modified Android WebViews | The version is checked at start. Profiles are feature-tested. A plain message asks for an update |
| One maintainer of an open-source browser | System engines carry the patching. A narrow scope per platform. Contributor documents and tests that let others help |
| Agent-written tests that test nothing | Mutation spot-checks and human review of test intent |

---

## 13. Human-side checklist

**Before 22 December 2026**

1. **Copy Arc's data** from any machine that still has it: `~/Library/Application Support/Arc/` on a Mac, and Arc's data folder under `%LOCALAPPDATA%` on Windows. This Mac has no such folder. Arc's own export is a list of links, and account data requests close on 15 December.
2. Keep one copy for fixtures: the importer is written against real data.

**To start W0**

3. ~~Decide WT-D1, D2, D4, D8 and D11.~~ Done: all eighteen decisions accepted on 2026-11-18.
4. Create the public repository with the chosen licence, `SECURITY.md`, a code of conduct and a trademark note.
5. **Apple:** App IDs under `com.tigercodes.webtiger`. Request the default-browser entitlement and the browser passkey entitlement; both are granted by Apple on request and can take time.
6. **Windows PC:** Visual Studio 2026 Build Tools with the C++ and Windows App SDK workloads, the Rust toolchain, and the WebView2 SDK.
7. **Devices:** an Android foldable or its emulator images; an iPhone Duo or time in Device Hub, since the Duo simulator cannot be folded from a test.

**Before the phases that need them**

8. **Signing keys:** an offline root, with separate keys for list bundles, the Chromium pack, plugin packages and the block list; a Sparkle key; the Android upload key.
9. **Hosting** for the static files, and bandwidth for a pack of about 135 MB per architecture (measured in W0: 001 §10, Q1) (W4, W6).
10. **A licence check** on bundling filter lists, and a codec-licensing review before any custom CEF build (W4, after 1.0).
11. **Store records:** App Store Connect, Play Console, Partner Center, F-Droid metadata (W10).
12. **The relay:** whether TigerCodes runs one (W9).

---

## 14. Repository conventions (for the agent)

- One `docs/features/NNN-<feature>.md` per feature, written before the code: user story, UX, data and sync impact, tests, done checklist. This file is `000`.
- **The repository is public.** Its documents stand alone and never point into a private repository.
- Conventional commits: `feat(guard): …`, `fix(tabs): …`, `test(adapter): …`.
- `CLAUDE.md` at the root records commands, module rules and the definition of done once W0 lands.
- Generated files are checked in, and `lint` fails when they are stale.
- No secrets in the repository. No private API in any shell.

---

## 15. Sources checked (2026-11-18)

**Platform constraints**

- Apple, BrowserEngineKit and alternative browser engines: <https://developer.apple.com/documentation/browserenginekit>
- No alternative engine shipped on iOS; the Edge team's Blink prototype: <https://macrumors.com/2026/06/17/webkit-rule-costs-ios-users-browser-performance>, <https://open-web-advocacy.org/blog/28-percent-faster--the-blink-prototype-that-shows-why-apples-ios-browser-engine-ban-must-end/>
- Japan, iOS 26.2: <https://9to5mac.com/2025/12/17/apple-announces-sweeping-app-store-and-iphone-changes-in-japan>
- Kagi ends Orion for Windows and Linux: <https://alternativeto.net/news/2026/10/kagi-ends-development-of-orion-for-linux-and-windows-open-sources-code-to-community/>
- WebKit's Windows port: <https://iangrunert.pages.dev/2025/11/06/webkit-windows-port-update-november-2025.html>, <https://docs.webkit.org/Ports/Introduction.html>
- WPE WebKit for Android: <https://github.com/Igalia/wpe-android>
- CEF: sandbox and packaging <https://chromiumembedded.github.io/cef/sandbox_setup>; codecs <https://support.google.com/webdesigner/answer/10043691>; Mac App Store <https://developer.apple.com/forums/thread/807949>
- WebView2: profiles <https://learn.microsoft.com/microsoft-edge/webview2/concepts/multi-profile-support>; extensions <https://github.com/MicrosoftEdge/WebView2Feedback/blob/main/specs/Extensions.md>; the WinUI 3 overload <https://learn.microsoft.com/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.controls.webview2.ensurecorewebview2async>
- Android WebView: profiles <https://developer.android.com/reference/androidx/webkit/ProfileStore>; process model <https://chromium.googlesource.com/chromium/src.git/+/main/android_webview/SECURITY.md>
- WebKit: `WKWebExtensionController` <https://developer.apple.com/documentation/webkit/wkwebextensioncontroller>; the 150,000-rule limit <https://developer.apple.com/forums/thread/734111>; web push in WKWebView <https://developer.apple.com/forums/thread/760767>
- Android WebView and web push: <https://pushpad.xyz/blog/can-i-use-the-web-push-api-with-hybrid-mobile-apps>
- WebUSB support by engine: <https://developer.mozilla.org/docs/Web/API/WebUSB_API>
- `adblock-rust`: <https://github.com/brave/adblock-rust>, <https://brave.com/privacy-updates/36-adblock-memory-reduction>
- `rquickjs`: <https://docs.rs/crate/rquickjs/latest>
- Android 17 large-screen rules: <https://developer.android.com/about/versions/17/changes/ff-restrictions-ignored>
- iPhone Duo: <https://developer.apple.com/documentation/technologyoverviews/preparing-your-app-for-iphone-duo>

**Browsers**

- Arc's shutdown: <https://9to5mac.com/2026/10/22/arc-browser-gets-a-sunset-date-and-users-have-two-months-to-move-on/>, <https://cyberinsider.com/arc-browser-to-stop-working-on-december-22-as-team-shifts-to-dia/>
- Arc's usage figures: <https://browsercompany.substack.com/p/letter-to-arc-members-2025>. Boosts: <https://nvd.nist.gov/vuln/detail/CVE-2024-45489>
- Zen: <https://github.com/zen-browser/desktop>, <https://docs.zen-browser.app/user-manual/workspaces>, <https://docs.zen-browser.app/faq>
- Nook: <https://github.com/nook-browser/Nook>. Ora: <https://github.com/the-ora/browser>. Crest: <https://github.com/pauljoda/Crest>
- Brave: its benchmark <https://brave.com/blog/brave-outperforms-other-browsers/>; Origin <https://www.privacyguides.org/news/2026/06/07/brave-launches-paid-minimalist-brave-origin-browser/>
- Vivaldi 8.3: <https://ppc.land/vivaldi-8-3-blocks-ads-and-trackers-at-three-strengths-set-site-by-site/>
- DuckDuckGo's blocker: <https://piunikaweb.com/2026/07/08/duckduckgo-introduces-built-in-ad-blocker/>
- Chrome: vertical tabs <https://techcrunch.com/2026/04/07/chrome-is-finally-getting-vertical-tabs/>; uBlock Origin delisted <https://cyberinsider.com/google-removes-ublock-origin-from-chrome-web-store-in-final-manifest-v2-purge/>
- Safari 27.3: <https://9to5mac.com/2026/11/03/ios-27-3-and-macos-27-3-add-vertical-tabs-adaptive-dark-mode-and-more-to-safari/>
- Firefox and `adblock-rust`: <https://www.privacyguides.org/news/2026/04/24/firefox-quietly-adds-braves-rust-based-adblocker/>
- Helium: <https://github.com/imputnet/helium>. ChatGPT Atlas: <https://www.eweek.com/news/openai-atlas-browser-shutdown-chatgpt/>. Ladybird: <https://ladybird.org/newsletter/2026-10-31/>

**Not verified, and marked for W0:** the size of the CEF pack; whether a CEF pack loads from outside the app bundle; DRM video in WKWebView for a third-party browser; password AutoFill in WKWebView on the Mac; WebSerial and WebHID in WebView2; the lowest Android WebView with profiles; every number in §7.

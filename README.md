# WebTiger

An open-source web browser that is light, private and organised, with native apps for macOS, iPadOS, iOS, Windows and Android.

**Status: early development (the work is in phase W2 of the plan's ten).** Nothing here is released, or ready to rely on as a browser yet. The plan is in [`docs/features/000-master-plan.md`](docs/features/000-master-plan.md), which says under each phase where it stands; what exists and how it is tested is described in [`docs/features/001-foundations.md`](docs/features/001-foundations.md), and each feature has a document of its own beside them. So far the Mac app is built and tested on a Mac, the iPhone and iPad app on Apple's simulator, and the Android app on an emulator: none on a phone or a tablet. The Windows app is not built yet.

## What it is meant to be

- **Light.** Tabs you are not looking at sleep, and cost almost nothing. Memory and start-up budgets are tests that fail the build.
- **Private.** Ad and tracker blocking built in, switchable for one site, one Space, or everywhere. No telemetry.
- **Organised.** Spaces hold Groups and Sites. Each Space can have its own logins, so two accounts on one service never mix.
- **Open.** MPL-2.0. A plugin is one folder with a manifest and a script.

## How it is built

WebTiger uses each system's own web engine: WebKit on Apple platforms, WebView2 on Windows, the System WebView on Android. It does not fork a browser engine. On the Mac a second engine, Chromium, is planned as an optional download.

One **Rust core** decides everything a browser decides: what typed text means, what the address field may show, which navigation is allowed, what belongs to which Space. Thin **native shells** draw the windows and host the web views: Swift on Apple platforms, C++/WinRT on Windows, Kotlin on Android. Core and shell speak one protobuf contract over a small C ABI.

```
Core/        the Rust core: crates, the contract (proto), tools
Packages/    the Apple shell, as one Swift package
Apps/        the Mac and iOS app targets
Android/     the Android shell: Kotlin and Compose
Design/      design tokens, one source for every shell
Localization/ every user-visible string, one source for every shell
Fixtures/    test data
Tools/       bootstrap, code generation, the test entry point
docs/features/ one plan per feature, written before the code
```

## Building and testing (macOS)

Needs Xcode 27 and [rustup](https://rustup.rs). Everything else is fetched, pinned by checksum, on first use.

```sh
Tools/bootstrap.sh          # pinned build tools into Tools/bin
Tools/apple/xcframework.sh  # the Rust core for the Apple apps
Tools/test.sh all           # every check
```

`Tools/test.sh` also takes one step at a time: `lint`, `core`, `unit`, `ios`, `build`, `ui`, and `android` (which needs a JDK 21 and the Android SDK and NDK).

## Leaving Arc?

Arc stops working on 22 December 2026. `webtiger import arc` reads Arc's data folder and writes your Spaces, folders, pinned tabs, Favorites and link rules to one file that WebTiger will read. See [`docs/features/002-arc-import.md`](docs/features/002-arc-import.md). **Keep a copy of your Arc folder** whether or not you use this: `~/Library/Application Support/Arc` on a Mac.

## Contributing, security, licence

- [CONTRIBUTING.md](CONTRIBUTING.md): how to propose a change. Contributions are accepted under the Developer Certificate of Origin.
- [SECURITY.md](SECURITY.md): how to report a vulnerability privately.
- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
- Licence: [Mozilla Public License 2.0](LICENSE). The name "WebTiger" and its logo are not covered by that licence; see [TRADEMARKS.md](TRADEMARKS.md).

Every file in this repository is covered by the following notice, unless the file itself says otherwise:

> This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

#!/usr/bin/env bash
# Every platform's strings from the shared catalogue, Localization/strings.json (040):
#   Apple     Packages/Sources/WTUI/Resources/Localizable.xcstrings, the String Catalog, and
#             Packages/Sources/WTUI/Words.generated.swift, every string by name
#   Android   Android/app/src/main/res/values*/strings.xml (the key error.net.dns is the
#             resource error_net_dns)
#   Windows   Windows/Strings/<language>/Resources.resw
# written by Core/tools/strings-gen, a Rust program, so that the Mac and the PC run the same
# one. The output is checked in. After an edit to the catalogue on either machine, run it once.
#   Tools/strings/generate.sh            # write what changed
#   Tools/strings/generate.sh --check    # write nothing; fail if a file is stale (Tools/test.sh lint)
# On the PC, from Core/: cargo run --quiet --locked --manifest-path tools/strings-gen/Cargo.toml -- --root ..
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# From Core/, so rustup builds it with the pinned toolchain; a target directory of its own, so
# it never waits on the core's build or invalidates it.
cd "$ROOT/Core"
CARGO_TARGET_DIR="$ROOT/Core/target/strings-gen" \
  cargo run --quiet --locked --manifest-path tools/strings-gen/Cargo.toml -- --root "$ROOT" "$@"

#!/usr/bin/env bash
# The Rust core for the Apple shell: builds wt-ffi (its C ABI,
# Core/crates/wt-ffi/include/webtiger_core.h) as a static library and wraps it in
# Packages/Frameworks/WebTigerCore.xcframework, which Packages/Package.swift links into
# WTCoreClient (WEBTIGER_CORE). The apps need it; the package's tests run without it against
# ScriptedCore.
#   Tools/apple/xcframework.sh            # macOS (arm64 + x86_64), iOS and the iOS simulator
#   Tools/apple/xcframework.sh --mac      # macOS only: the suites and the Mac app
#   Tools/apple/xcframework.sh --remove   # no Rust core
# Needs: the Rust toolchain Core/rust-toolchain.toml pins (rustup adds the Apple targets it
# lacks) and Xcode's command-line tools. The Rust build is the whole core in release with fat
# LTO; the app's linker strips what the apps don't call.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FRAMEWORK="$ROOT/Packages/Frameworks/WebTigerCore.xcframework"
WORK="$ROOT/Core/target/apple"
MODE=all
for argument in "$@"; do
  case "$argument" in
    --mac) MODE=--mac ;;
    --remove) MODE=--remove ;;
    *) echo "usage: $0 [--mac] | --remove" >&2; exit 2 ;;
  esac
done
# The apps' deployment targets (project.yml, Packages/Package.swift), for rustc and for the C the
# core's crates compile (SQLite). Unset, cc builds that C for this Mac's macOS, and every link of
# the app warns that the objects are newer than it.
export MACOSX_DEPLOYMENT_TARGET=26.0 IPHONEOS_DEPLOYMENT_TARGET=26.0

# SwiftPM caches a manifest's evaluation by its text, so it wouldn't see the framework come or
# go: the cached evaluations are forgotten (the next build makes them again). Xcode's build
# description caches which platforms the framework had, so the gate's derived data forgets that too.
forget_manifests() {
  rm -f "$HOME/Library/Caches/org.swift.swiftpm/manifests/manifest.db"*
  rm -rf "$ROOT/.build/DerivedData/Build/Intermediates.noindex/XCBuildData" \
    "$ROOT/.build/DerivedData-ios/Build/Intermediates.noindex/XCBuildData"
}

if [[ "$MODE" == "--remove" ]]; then
  rm -rf "$FRAMEWORK"
  forget_manifests
  echo "Removed $FRAMEWORK: the package builds without NativeCore."
  exit 0
fi
case "$MODE" in
  all) TARGETS=(aarch64-apple-darwin x86_64-apple-darwin aarch64-apple-ios aarch64-apple-ios-sim) ;;
  --mac) TARGETS=(aarch64-apple-darwin x86_64-apple-darwin) ;;
esac

cd "$ROOT/Core"
installed="$(rustup target list --installed)"
for target in "${TARGETS[@]}"; do
  grep -qx "$target" <<<"$installed" || rustup target add "$target"
done

started=$(date +%s)
for target in "${TARGETS[@]}"; do
  echo "== wt-ffi, $target"
  cargo rustc -p wt-ffi --release --locked --target "$target" --crate-type staticlib
done
echo "Rust build: $(( $(date +%s) - started )) s for ${#TARGETS[@]} targets"

rm -rf "$WORK"
mkdir -p "$WORK/headers" "$WORK/macos" "$WORK/prelinked"

# Only the C ABI leaves the library: each target's is prelinked into one object exporting wt_*,
# everything else in it made local, its own SQLite among them. Left global, the core's SQLite
# would answer for the system's in any framework the app links that uses it.
printf '_wt_*\n' > "$WORK/exports.txt"
prelink() {
  local target="$1" arch platform sdk minos
  case "$target" in
    aarch64-*) arch=arm64 ;;
    x86_64-*) arch=x86_64 ;;
  esac
  case "$target" in
    *-apple-darwin) platform=macos sdk=macosx minos="$MACOSX_DEPLOYMENT_TARGET" ;;
    *-apple-ios-sim) platform=ios-simulator sdk=iphonesimulator minos="$IPHONEOS_DEPLOYMENT_TARGET" ;;
    *-apple-ios) platform=ios sdk=iphoneos minos="$IPHONEOS_DEPLOYMENT_TARGET" ;;
  esac
  local out="$WORK/prelinked/$target"
  mkdir -p "$out"
  xcrun ld -r -arch "$arch" -platform_version "$platform" "$minos" "$(xcrun --sdk "$sdk" --show-sdk-version)" \
    -all_load -exported_symbols_list "$WORK/exports.txt" \
    "$ROOT/Core/target/$target/release/libwebtiger_core.a" -o "$out/webtiger_core.o"
  xcrun libtool -static -o "$out/libwebtiger_core.a" "$out/webtiger_core.o"
  local stray
  stray="$(nm -gU "$out/libwebtiger_core.a" 2>/dev/null | awk 'NF == 3 { print $3 }' | grep -v '^_wt_' | head -3 || true)"
  if [[ -n "$stray" ]]; then
    echo "$target's library exports more than its C ABI: $stray" >&2
    exit 1
  fi
}
for target in "${TARGETS[@]}"; do
  prelink "$target"
done
library() { echo "$WORK/prelinked/$1/libwebtiger_core.a"; }
cp "$ROOT/Core/crates/wt-ffi/include/webtiger_core.h" "$WORK/headers/"
cat > "$WORK/headers/module.modulemap" <<'MODULEMAP'
module WebTigerCore {
    header "webtiger_core.h"
    export *
}
MODULEMAP
lipo -create "$(library aarch64-apple-darwin)" "$(library x86_64-apple-darwin)" -output "$WORK/macos/libwebtiger_core.a"
slices=(-library "$WORK/macos/libwebtiger_core.a" -headers "$WORK/headers")
if [[ "$MODE" == "all" ]]; then
  slices+=(-library "$(library aarch64-apple-ios)" -headers "$WORK/headers")
  slices+=(-library "$(library aarch64-apple-ios-sim)" -headers "$WORK/headers")
fi
rm -rf "$FRAMEWORK"
mkdir -p "$(dirname "$FRAMEWORK")"
xcodebuild -create-xcframework "${slices[@]}" -output "$FRAMEWORK"
forget_manifests
echo "Static library sizes (before the app's linker strips them):"
ls -l "$WORK/macos/libwebtiger_core.a"
du -sh "$FRAMEWORK"
echo "Built $FRAMEWORK."

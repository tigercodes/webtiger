#!/usr/bin/env bash
# Builds the Mac app as it is released, for the scripts that measure it (060, 063), and
# says where it is.
#
#   Tools/perf/mac-build.sh
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
export PATH="$ROOT/Tools/bin:$PATH"
FRAMEWORK="$ROOT/Packages/Frameworks/WebTigerCore.xcframework"

xcodegen generate >/dev/null
# The core's library, made again when the core is newer than it: with the slices it has, so
# that what the suites need afterwards is still there.
if [[ ! -d "$FRAMEWORK" ]] \
  || [[ -n "$(find Core/crates Core/Cargo.toml Core/Cargo.lock -newer "$FRAMEWORK/Info.plist" -type f -print -quit)" ]]; then
  if [[ -d "$FRAMEWORK/ios-arm64" ]]; then
    Tools/apple/xcframework.sh >/dev/null 2>&1
  else
    Tools/apple/xcframework.sh --mac >/dev/null 2>&1
  fi
fi
xcodebuild -project WebTiger.xcodeproj -scheme WebTiger-macOS -configuration Release \
  -derivedDataPath "$ROOT/.build/DerivedData" -destination 'platform=macOS,arch=arm64' build >/dev/null
echo "$ROOT/.build/DerivedData/Build/Products/Release/WebTiger.app"

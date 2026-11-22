#!/usr/bin/env bash
# One entry point for every check. This is what Claude Code and CI run.
#
#   Tools/test.sh lint     generated code is fresh; rustfmt, clippy; SwiftFormat, SwiftLint
#   Tools/test.sh core     cargo test for the Rust core
#   Tools/test.sh unit     the Swift package's tests on macOS, against the real core and WKWebView
#   Tools/test.sh ios      the Swift package's tests on the iOS simulator
#   Tools/test.sh build    build the apps (macOS and the iOS simulator)
#   Tools/test.sh ui       the apps' UI tests in fake mode (macOS and the iOS simulator);
#                          `ui-mac` and `ui-ios` are its two halves
#   Tools/test.sh android  the Android shell: its JVM tests against the real core, and the debug APK
#   Tools/test.sh android-device  the Android shell on a device: the adapter suite against the
#                          real WebView, isolation, and the app's window
#   Tools/test.sh all      lint, core, unit, build, ui-mac, ios, ui-ios, and android where its
#                          tools are
#
# Exits non-zero on the first failing step. Set IOS_SIM to pick a simulator.
# The Windows shell's checks are Tools/test.ps1's, on the Windows PC. `android` needs a JDK,
# the Android SDK and the NDK: `all` runs it where they are found and says so where they are
# not. `android-device` is not part of `all`: it needs an attached device or an emulator's
# system image (Tools/android/emulator.sh; docs/features/003-android-shell.md §8).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export PATH="$ROOT/Tools/bin:$PATH"
IOS_SIM="${IOS_SIM:-iPhone 17}"
DD="$ROOT/.build/DerivedData"
FRAMEWORK="$ROOT/Packages/Frameworks/WebTigerCore.xcframework"
# No sysdiagnose-style collection after a failure, and a test that hangs fails after ten
# minutes rather than holding the gate.
DIAG=(-collect-test-diagnostics never -test-timeouts-enabled YES)
XC=(xcodebuild -project WebTiger.xcodeproj -derivedDataPath "$DD" -configuration Debug)

step() { printf '\n\033[1;34m== %s\033[0m\n' "$*"; }

ensure_tools() {
  Tools/bootstrap.sh --check >/dev/null 2>&1 || Tools/bootstrap.sh
  xcodegen generate >/dev/null
}

# The Rust core as an xcframework. Rebuilt when any of its sources is newer than it.
ensure_core() {
  local need="${1:-all}"
  if [[ -d "$FRAMEWORK" ]] && [[ "$need" == "mac" || -d "$FRAMEWORK/ios-arm64" ]] \
    && [[ -z "$(find Core/crates Core/Cargo.toml Core/Cargo.lock -newer "$FRAMEWORK/Info.plist" -type f -print -quit)" ]]; then
    return
  fi
  step "Rust core for Apple"
  if [[ "$need" == "mac" && ! -d "$FRAMEWORK/ios-arm64" ]]; then
    Tools/apple/xcframework.sh --mac
  else
    Tools/apple/xcframework.sh
  fi
}

run_lint() {
  step "lint: generated code"
  Tools/proto/generate.sh --check
  python3 -I Tools/design/tokens.py --check
  Tools/strings/generate.sh --check
  python3 -I Tools/testsite/site.py --check
  python3 -I Tools/testsite/site.py self-test
  step "lint: Rust"
  (cd Core && cargo fmt --all --check && cargo clippy --workspace --all-targets --locked -- -D warnings)
  (cd Core/tools/proto-gen && cargo fmt --check)
  # The strings tool (040) has a workspace of its own, as the proto tool has.
  (cd Core/tools/strings-gen && cargo fmt --check \
    && CARGO_TARGET_DIR="$ROOT/Core/target/strings-gen" cargo clippy --all-targets --locked -- -D warnings)
  if command -v cargo-deny >/dev/null 2>&1; then
    (cd Core && cargo deny check)
  else
    echo "cargo-deny is not installed: the dependency policy (Core/deny.toml) was not checked"
  fi
  if command -v buf >/dev/null 2>&1; then
    (cd Core/proto && buf lint && buf format --diff --exit-code)
  else
    echo "buf is not installed: the contract's lint rules (Core/proto/buf.yaml) were not checked"
  fi
  step "lint: Swift"
  swiftformat Packages Apps --lint
  swiftlint lint --quiet --strict
}

run_core() {
  step "core (cargo test)"
  (cd Core && cargo test --workspace --locked)
  step "core: the strings tool (cargo test)"
  (cd Core/tools/strings-gen && CARGO_TARGET_DIR="$ROOT/Core/target/strings-gen" cargo test --locked)
}

run_unit() {
  ensure_core mac
  step "unit (macOS)"
  (cd Packages && swift test 2>&1 | xcbeautify --quiet --renderer terminal)
}

run_ios() {
  ensure_tools
  ensure_core
  # xcodebuild hands a test's process only the variables it is told to: recording the
  # snapshots (004 §4) is asked of the simulator's tests the same way as of the Mac's.
  if [[ -n "${WEBTIGER_RECORD_SNAPSHOTS:-}" ]]; then
    export TEST_RUNNER_WEBTIGER_RECORD_SNAPSHOTS="$WEBTIGER_RECORD_SNAPSHOTS"
  fi
  step "package tests on $IOS_SIM"
  (cd Packages && xcodebuild -scheme WebTiger-Package -destination "platform=iOS Simulator,name=$IOS_SIM" \
    -derivedDataPath "$DD-ios" -skipPackagePluginValidation "${DIAG[@]}" test 2>&1 | xcbeautify --quiet)
}

run_build() {
  ensure_tools
  ensure_core
  step "build WebTiger-macOS"
  "${XC[@]}" -scheme WebTiger-macOS -destination 'platform=macOS,arch=arm64' build 2>&1 | xcbeautify --quiet
  step "build WebTiger-iOS"
  "${XC[@]}" -scheme WebTiger-iOS -destination "platform=iOS Simulator,name=$IOS_SIM" CODE_SIGNING_ALLOWED=NO build 2>&1 | xcbeautify --quiet
}

run_ui_mac() {
  ensure_tools
  ensure_core
  step "app UI tests (fake mode), macOS"
  # XCUITest answers a dialog of the system's that is in its way by pressing its Cancel
  # button, whoever the dialog was for; with two in its way it goes round between them until
  # each test's ten minutes are up. So the Mac's suite is not started while one is on the
  # screen: somebody answers it first.
  local dialogs
  dialogs=$(swift Tools/apple/system-dialogs.swift 2>/dev/null || echo 0)
  if [[ "$dialogs" != 0 ]]; then
    echo "ui: $dialogs dialog(s) of the system's are on this Mac's screen; answer them, then run this again" >&2
    exit 1
  fi
  "${XC[@]}" -scheme WebTiger-macOS -destination 'platform=macOS,arch=arm64' "${DIAG[@]}" test 2>&1 | xcbeautify --quiet
}

run_ui_ios() {
  ensure_tools
  ensure_core
  step "app UI tests (fake mode), $IOS_SIM"
  "${XC[@]}" -scheme WebTiger-iOS -destination "platform=iOS Simulator,name=$IOS_SIM" CODE_SIGNING_ALLOWED=NO "${DIAG[@]}" test 2>&1 | xcbeautify --quiet
}

run_ui() {
  run_ui_mac
  run_ui_ios
}

# A JDK and the Android SDK for Gradle, where the environment does not name them already.
# Nothing is installed: what is missing is said, and the step fails.
ensure_android() {
  # macOS has a `java` on the PATH that only says there is none: it is asked, not looked for.
  if [[ -z "${JAVA_HOME:-}" ]] && ! java -version >/dev/null 2>&1; then
    local jdk
    # Homebrew's JDK 21, or the one Android Studio carries.
    for jdk in /opt/homebrew/opt/openjdk@21 /usr/local/opt/openjdk@21 \
      "/Applications/Android Studio.app/Contents/jbr/Contents/Home"; do
      if [[ -x "$jdk/bin/java" ]]; then export JAVA_HOME="$jdk"; break; fi
    done
    [[ -n "${JAVA_HOME:-}" ]] || { echo "android: no JDK found; set JAVA_HOME to a JDK 21 or later" >&2; exit 1; }
  fi
  if [[ -z "${ANDROID_HOME:-}" && ! -f Android/local.properties ]]; then
    [[ -d "$HOME/Library/Android/sdk" ]] || { echo "android: no Android SDK found; set ANDROID_HOME" >&2; exit 1; }
    export ANDROID_HOME="$HOME/Library/Android/sdk"
  fi
}

# Gradle, quietly: its own progress is dropped, what cargo and the tests say is kept.
android_gradle() {
  (cd Android && ./gradlew --console=plain --quiet "$@")
}

run_android() {
  ensure_android
  step "android: the JNI glue (clippy), and the core's library for this machine"
  (cd Core && cargo clippy -p wt-ffi --features jni --all-targets --locked --target-dir target/jvm -- -D warnings \
    && cargo build -p wt-ffi --features jni --locked --target-dir target/jvm)
  step "android: unit tests on the JVM (the real core through JNI, the adapter's rules, the window)"
  android_gradle :core-engine:testDebugUnitTest :app:testDebugUnitTest
  step "android: debug APK (arm64-v8a, x86_64)"
  android_gradle :app:assembleDebug
  ls -l Android/app/build/outputs/apk/debug/app-debug.apk
}

# On a device: whatever is attached, or the emulator Tools/android/emulator.sh boots. The
# emulator is left running for the next run; `Tools/android/emulator.sh stop` ends it.
run_android_device() {
  ensure_android
  step "android: a device"
  ANDROID_HOME="${ANDROID_HOME:-$HOME/Library/Android/sdk}" Tools/android/emulator.sh start
  step "android: the adapter suite, isolation and the window, on the device"
  # Without mDNS, as the adb server was started (Tools/android/emulator.sh says why).
  ADB_MDNS=0 android_gradle :app:connectedDebugAndroidTest
  # Gradle has been seen to end well when the tests' process crashed part of the way through:
  # the run is then written down as one failure ("Test run failed to complete") and the tests
  # after the crash are simply missing. What the results say is what counts.
  python3 -I - <<'PY'
import glob
import os
import sys
import xml.etree.ElementTree as ET

paths = glob.glob("Android/app/build/outputs/androidTest-results/connected/debug/*.xml")
bad = not paths
if not paths:
    print("android-device: no results were written", file=sys.stderr)
for path in paths:
    suite = ET.parse(path).getroot()
    device = os.path.basename(path).removeprefix("TEST-").removesuffix(".xml")
    failures = int(suite.get("failures", "0")) + int(suite.get("errors", "0"))
    print(f"{device}: {suite.get('tests')} tests, {failures} failures, {suite.get('skipped', '0')} skipped")
    for case in suite.iter("testcase"):
        for problem in list(case.iter("failure")) + list(case.iter("error")):
            first = (problem.text or "").strip().splitlines()[:1]
            print(f"  failed: {case.get('classname', '').rsplit('.', 1)[-1]}.{case.get('name')}: {' '.join(first)}", file=sys.stderr)
    bad = bad or failures > 0
sys.exit(1 if bad else 0)
PY
}

# The Android shell's part of `all`: run where its tools are, and said to be left out where
# they are not, with what is missing.
run_android_if_possible() {
  local missing
  if missing="$( (ensure_android) 2>&1 )"; then
    run_android
  else
    step "android: left out of this run"
    echo "$missing"
  fi
}

case "${1:-all}" in
  lint) ensure_tools; run_lint ;;
  core) run_core ;;
  unit) run_unit ;;
  ios) run_ios ;;
  build) run_build ;;
  ui) run_ui ;;
  ui-mac) run_ui_mac ;;
  ui-ios) run_ui_ios ;;
  android) run_android ;;
  android-device) run_android_device ;;
  # The Mac's UI suite goes before anything starts the iPhone simulator: a process of the
  # simulator's own crashes now and then, the Mac puts "… quit unexpectedly" on the screen,
  # and the Mac's suite does not start while a dialog of the system's is there.
  all) ensure_tools; run_lint; run_core; run_unit; run_build; run_ui_mac; run_ios; run_ui_ios; run_android_if_possible ;;
  *) echo "usage: Tools/test.sh [lint|core|unit|ios|build|ui|ui-mac|ui-ios|android|android-device|all]" >&2; exit 2 ;;
esac
printf '\n\033[1;32mOK: %s\033[0m\n' "${1:-all}"

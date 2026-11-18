#!/usr/bin/env bash
# An Android device for the tests that need one (docs/features/003-android-shell.md §8).
#
#   Tools/android/emulator.sh start   # a device is attached and booted when this returns
#   Tools/android/emulator.sh stop    # stops the emulator this script started
#   Tools/android/emulator.sh status  # says what is attached
#
# `start` uses whatever device is already attached. With none, it boots an emulator without a
# window: the AVD named by WEBTIGER_AVD (default `wt-phone`), which it makes from the newest
# system image installed when there is no such AVD. It downloads nothing: with no system image
# it says which SDK package to install, and fails.
#
# The adb server is started here with ADB_MDNS=0. adb 37's mDNS discovery aborts the server on
# this macOS when the local network carries a service name it cannot parse, and every device
# then reads as offline. A device over USB or the emulator needs no discovery.
set -euo pipefail

SDK="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
ADB="$SDK/platform-tools/adb"
EMULATOR="$SDK/emulator/emulator"
AVD="${WEBTIGER_AVD:-wt-phone}"
STATE="${TMPDIR:-/tmp}/webtiger-emulator"
export ADB_MDNS=0

[[ -x "$ADB" ]] || { echo "emulator: no adb at $ADB; set ANDROID_HOME" >&2; exit 1; }

# The adb server: started here, with ADB_MDNS=0 in its environment, unless one runs already.
ensure_adb() {
  "$ADB" start-server >/dev/null 2>&1 || { echo "emulator: the adb server would not start" >&2; exit 1; }
}

attached() {
  "$ADB" devices | awk 'NR > 1 && $2 == "device" { print $1 }' | head -1
}

booted() {
  [[ "$("$ADB" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" == "1" ]]
}

# A JDK for avdmanager, which is a Java program.
find_java() {
  [[ -n "${JAVA_HOME:-}" ]] && return
  local jdk
  for jdk in /opt/homebrew/opt/openjdk@21 /usr/local/opt/openjdk@21 \
    "/Applications/Android Studio.app/Contents/jbr/Contents/Home"; do
    if [[ -x "$jdk/bin/java" ]]; then export JAVA_HOME="$jdk"; return; fi
  done
}

create_avd() {
  local image
  image="$(find "$SDK/system-images" -mindepth 3 -maxdepth 3 -type d 2>/dev/null | sort -V | tail -1)"
  if [[ -z "$image" ]]; then
    echo "emulator: no system image is installed. Install one, for example:" >&2
    echo "  sdkmanager 'system-images;android-36;google_apis;arm64-v8a'" >&2
    exit 1
  fi
  local package="system-images;${image#"$SDK/system-images/"}"
  package="${package//\//;}"
  find_java
  echo "emulator: making the AVD $AVD from $package"
  echo no | "$SDK/cmdline-tools/latest/bin/avdmanager" create avd -n "$AVD" -k "$package" -d pixel_8 >/dev/null
}

start() {
  ensure_adb
  if [[ -z "$(attached)" ]]; then
    [[ -x "$EMULATOR" ]] || { echo "emulator: no device is attached and the SDK has no emulator" >&2; exit 1; }
    "$EMULATOR" -list-avds | grep -qx "$AVD" || create_avd
    mkdir -p "$STATE"
    echo "emulator: booting $AVD"
    # No window, no sound, no snapshot: the same cold device every time. The software renderer
    # draws the same on a Mac with a screen and on a runner without one.
    (nohup "$EMULATOR" -avd "$AVD" -no-window -no-audio -no-boot-anim -no-snapshot \
      -gpu swiftshader_indirect >"$STATE/emulator.log" 2>&1 &)
    echo "$AVD" >"$STATE/started"
  fi
  local waited=0
  until [[ -n "$(attached)" ]] && booted; do
    sleep 2
    waited=$((waited + 2))
    if ((waited > 300)); then
      echo "emulator: no device booted within five minutes (see $STATE/emulator.log)" >&2
      exit 1
    fi
  done
  # Animations off, as the platform's own UI tests ask: a test waits for nothing to settle.
  local setting
  for setting in window_animation_scale transition_animation_scale animator_duration_scale; do
    "$ADB" shell settings put global "$setting" 0 >/dev/null
  done
  echo "emulator: $(attached) is ready (Android $("$ADB" shell getprop ro.build.version.release | tr -d '\r'))"
}

stop() {
  if [[ -f "$STATE/started" ]]; then
    "$ADB" emu kill >/dev/null 2>&1 || true
    rm -f "$STATE/started"
    echo "emulator: stopped"
  else
    echo "emulator: this script started none"
  fi
}

case "${1:-status}" in
  start) start ;;
  stop) stop ;;
  status)
    ensure_adb
    "$ADB" devices
    ;;
  *)
    echo "usage: Tools/android/emulator.sh [start|stop|status]" >&2
    exit 2
    ;;
esac

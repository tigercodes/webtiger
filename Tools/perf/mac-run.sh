#!/usr/bin/env bash
# The run (000 §7, 063): the Mac app with a list of sites open, measured.
#
#   Tools/perf/mac-run.sh [--web] <list> [runs] [seconds between sites] [seconds idle]
#
# Builds the Mac app as it is released and has Tools/perf/mac-run.swift start it in fake
# mode (its store in memory: nothing of anybody's is read, written or left), where the app
# opens the list's addresses by itself, one after another, each in a tab. It says what the
# app's own process and the engine's processes hold, and the energy they have used, when the
# last has been opened and again after the idle time; for each run, and the middle one.
#
# A list is a file with an address to a line. An address that is not on this machine is
# loaded only when --web is said: the run then sends this machine's requests to other
# people's servers, and that is for whoever runs it to decide. Tools/perf/lists/test-site.txt
# is a list of the test site's pages, which this serves on the machine: it shows that the
# script works, and nothing about the web.
#
# Not a part of `Tools/test.sh all`: it puts the app's window on the screen, takes as long
# as its list, and its numbers are a machine's.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
WEB=()
if [[ "${1:-}" == "--web" ]]; then
  WEB=(--web)
  shift
fi
LIST="${1:?usage: Tools/perf/mac-run.sh [--web] <list> [runs] [seconds between sites] [seconds idle]}"
RUNS="${2:-10}"
EVERY="${3:-20}"
IDLE="${4:-120}"
PORT=8765

APP="$(Tools/perf/mac-build.sh)"

# The test site, on this machine, when the list names it; ended by its own number after.
if grep -q "^http://127.0.0.1:$PORT/" "$LIST"; then
  python3 Tools/testsite/site.py serve "$PORT" >/dev/null 2>&1 &
  SITE=$!
  trap 'kill "$SITE" 2>/dev/null || true' EXIT
  for _ in $(seq 1 50); do
    curl --silent --output /dev/null "http://127.0.0.1:$PORT/" && break
    sleep 0.1
  done
fi

swift Tools/perf/mac-run.swift ${WEB[@]+"${WEB[@]}"} "$APP" "$LIST" "$RUNS" "$EVERY" "$IDLE"

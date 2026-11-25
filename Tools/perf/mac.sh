#!/usr/bin/env bash
# The plan's budgets that are measured in an app, on a Mac (000 §7, 060): the app's own
# memory on the standard session, and its start.
#
#   Tools/perf/mac.sh [runs]
#
# Builds the Mac app as it is released, serves the test site on this machine, and has
# Tools/perf/mac-app.swift start the app on the standard session, in fake mode, with its one
# live page from that site. It fails when a budget is not met. Not a part of
# `Tools/test.sh all`: it puts the app's window on the screen, and its numbers are this
# machine's.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
export PATH="$ROOT/Tools/bin:$PATH"
RUNS="${1:-5}"
PORT=8765

APP="$(Tools/perf/mac-build.sh)"

# The site, on this machine; ended by its own number when the measuring is.
python3 Tools/testsite/site.py serve "$PORT" >/dev/null 2>&1 &
SITE=$!
trap 'kill "$SITE" 2>/dev/null || true' EXIT
for _ in $(seq 1 50); do
  curl --silent --output /dev/null "http://127.0.0.1:$PORT/" && break
  sleep 0.1
done

swift Tools/perf/mac-app.swift "$APP" "http://127.0.0.1:$PORT/" "$RUNS"

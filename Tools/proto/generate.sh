#!/usr/bin/env bash
# Every language's code from Core/proto/webtiger/core/v1, written into the tree,
# where it is checked in, so no build runs a generator:
#   Rust   Core/crates/wt-proto/src/gen/     prost (Core/tools/proto-gen)
#   Swift  Packages/Sources/WTContract/      swift-protobuf, for the Apple shell
# Android generates its Kotlin at build time with Wire from the same files.
# The Windows shell's C (nanopb) joins this script with the Windows shell.
# With --check nothing in the tree changes: the code is generated into a scratch
# folder and the script fails when it differs from what is checked in
# (Tools/test.sh lint runs it).
#
# Every tool is pinned and verified, and cached in Core/target/proto-tools:
#   protoc 36.2          GitHub's macOS release zip, SHA-256 checked
#   prost-build 0.14.4   Core/tools/proto-gen, with its own Cargo.lock
#   protoc-gen-swift     built from swift-protobuf at the version and revision
#                        Packages/Package.resolved names, so the generator and the
#                        runtime the apps link are one release
#   Tools/proto/generate.sh [--check]
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TOOLS="$ROOT/Core/target/proto-tools"
check=0
case "${1:-}" in
  --check) check=1 ;;
  "") ;;
  *) echo "usage: $0 [--check]" >&2; exit 2 ;;
esac

PROTOC_VERSION=36.2
PROTOC_SHA256=83d0dc80672f53486f54299bca6177159f9cad1fd54d4d3143eeea53c4423e1b
mkdir -p "$TOOLS"

# Downloads $1 to $2 once, and checks it against the SHA-256 $3.
fetch() {
  local url="$1" file="$2" sha="$3" actual
  if [[ ! -f "$file" ]]; then
    curl -fsSL -o "$file.part" "$url"
    mv "$file.part" "$file"
  fi
  actual="$(shasum -a 256 "$file" | cut -d' ' -f1)"
  if [[ "$actual" != "$sha" ]]; then
    rm -f "$file"
    echo "$(basename "$file") has SHA-256 $actual, not $sha: removed" >&2
    exit 1
  fi
}

protoc_dir="$TOOLS/protoc-$PROTOC_VERSION-osx"
protoc="$protoc_dir/bin/protoc"
if [[ ! -x "$protoc" ]]; then
  zip="$TOOLS/protoc-$PROTOC_VERSION-osx-universal_binary.zip"
  fetch "https://github.com/protocolbuffers/protobuf/releases/download/v$PROTOC_VERSION/protoc-$PROTOC_VERSION-osx-universal_binary.zip" \
    "$zip" "$PROTOC_SHA256"
  rm -rf "$protoc_dir"
  mkdir -p "$protoc_dir"
  unzip -q "$zip" -d "$protoc_dir"
fi

# The swift-protobuf the package resolved: its version and revision.
resolved="$ROOT/Packages/Package.resolved"
if [[ ! -f "$resolved" ]]; then
  echo "Packages/Package.resolved is missing: run 'swift package resolve' in Packages first" >&2
  exit 1
fi
read -r version revision < <(python3 -I - "$resolved" <<'PY'
import json, sys
pins = json.load(open(sys.argv[1]))["pins"]
pin = next(p for p in pins if p["identity"] == "swift-protobuf")
print(pin["state"]["version"], pin["state"]["revision"])
PY
)
swift_source="$TOOLS/swift-protobuf-$version"
swift_plugin="$swift_source/.build/release/protoc-gen-swift"
if [[ ! -x "$swift_plugin" ]]; then
  rm -rf "$swift_source"
  git -c advice.detachedHead=false clone -q --depth 1 --branch "$version" https://github.com/apple/swift-protobuf "$swift_source"
  head="$(git -C "$swift_source" rev-parse HEAD)"
  if [[ "$head" != "$revision" ]]; then
    rm -rf "$swift_source"
    echo "swift-protobuf $version is at $head, not the $revision Package.resolved names: removed" >&2
    exit 1
  fi
  (cd "$swift_source" && swift build -c release --product protoc-gen-swift >/dev/null)
fi

rust_tree="$ROOT/Core/crates/wt-proto/src/gen"
swift_tree="$ROOT/Packages/Sources/WTContract"
if (( check )); then
  scratch="$(mktemp -d "${TMPDIR:-/tmp}/webtiger-proto.XXXXXX")"
  trap 'rm -rf "$scratch"' EXIT
  rust_out="$scratch/rust" swift_out="$scratch/swift"
  mkdir -p "$rust_out" "$swift_out"
else
  rust_out="$rust_tree" swift_out="$swift_tree"
  rm -rf "$rust_out"
  rm -f "$swift_out"/*.pb.swift
  mkdir -p "$rust_out" "$swift_out"
fi

# The contract's files in name order.
contract=()
while IFS= read -r file; do contract+=("webtiger/core/v1/$file"); done \
  < <(cd "$ROOT/Core/proto/webtiger/core/v1" && ls -- *.proto | LC_ALL=C sort)

# Rust (prost), run from Core/ so rustup picks Core/rust-toolchain.toml.
(cd "$ROOT/Core" && PROTOC="$protoc" CARGO_TARGET_DIR="$ROOT/Core/target/proto-gen" \
  cargo run --quiet --locked --manifest-path tools/proto-gen/Cargo.toml -- "$rust_out")

cd "$ROOT/Core/proto"
# Swift: only the contract's files are generated. wt/options.proto's extension
# of FieldOptions needs descriptor.proto's types, which swift-protobuf's runtime
# doesn't have, and Swift reads no option.
"$protoc" -I . -I "$protoc_dir/include" --plugin=protoc-gen-swift="$swift_plugin" --swift_out="$swift_out" \
  --swift_opt=Visibility=Public,FileNaming=DropPath,UseAccessLevelOnImports=true "${contract[@]}"

if (( check )); then
  stale=()
  diff -rq "$rust_out" "$rust_tree" >/dev/null 2>&1 || stale+=("Core/crates/wt-proto/src/gen")
  swift_listing() { (cd "$1" && ls -- *.pb.swift 2>/dev/null); }
  swift_stale=0
  [[ "$(swift_listing "$swift_out")" == "$(swift_listing "$swift_tree")" ]] || swift_stale=1
  for file in "$swift_out"/*.pb.swift; do
    cmp -s "$file" "$swift_tree/$(basename "$file")" || swift_stale=1
  done
  (( swift_stale )) && stale+=("Packages/Sources/WTContract")
  if (( ${#stale[@]} )); then
    printf 'Generated protobuf code is stale; run Tools/proto/generate.sh:\n' >&2
    printf '  %s\n' "${stale[@]}" >&2
    exit 1
  fi
  echo "Generated protobuf code is up to date (swift-protobuf $version)."
else
  echo "Generated Rust (prost) and Swift (swift-protobuf $version) from Core/proto with protoc $PROTOC_VERSION."
fi

#!/usr/bin/env bash
# Installs the pinned build tools WebTiger needs on a Mac into Tools/bin without
# Homebrew: the Apple shell's (XcodeGen, SwiftLint, SwiftFormat, xcbeautify) and
# the core's (buf for the contract's lint rules, cargo-deny for the dependency
# policy). Safe to re-run: existing, matching binaries are kept.
#
#   Tools/bootstrap.sh            # install everything
#   Tools/bootstrap.sh --check    # only report what is missing
#
# Every archive is pinned by SHA-256. Bump the version and the hash together, to
# a release at least two weeks old.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="$ROOT/Tools/bin"
DL="$ROOT/Tools/.downloads"
mkdir -p "$BIN" "$DL"

# name | version | url | sha256 | path of the executable inside the archive
TOOLS=(
  "xcodegen|2.46.0|https://github.com/yonaskolb/XcodeGen/releases/download/2.46.0/xcodegen.zip|4d9e34b62172d645eed6457cac13fc222569974098ef4ee9c3368bedf0196806|xcodegen/bin/xcodegen"
  "swiftlint|0.65.1|https://github.com/realm/SwiftLint/releases/download/0.65.1/portable_swiftlint.zip|c1e429b0599cf1b516f369a2d9ec04eaf0e436f3c12b637df8851fa52ff694d0|swiftlint"
  "swiftformat|0.63.1|https://github.com/nicklockwood/SwiftFormat/releases/download/0.63.1/swiftformat.zip|385ef1a263ba28685157b98c5536b9c9105e124518f28b7ef8a2bee4b167eaeb|swiftformat"
  "xcbeautify|3.2.1|https://github.com/cpisciotta/xcbeautify/releases/download/3.2.1/xcbeautify-3.2.1-arm64-apple-macosx.zip|ed32972e41bc3aa19588f8b0f4a0209ef70c663ad60bbf4ad84a2de73355d637|xcbeautify"
  "buf|1.75.0|https://github.com/bufbuild/buf/releases/download/v1.75.0/buf-Darwin-arm64.tar.gz|5478531c4645c5b80866e87d76d73f0e8bf5a35644b12c92cbdcc408e7c8b9d3|buf/bin/buf"
  "cargo-deny|0.20.2|https://github.com/EmbarkStudios/cargo-deny/releases/download/0.20.2/cargo-deny-0.20.2-aarch64-apple-darwin.tar.gz|fe67d82a10d8597a3549364cb733a3f9cc1bfff9031b7ae46384a9f2a72090c3|cargo-deny-0.20.2-aarch64-apple-darwin/cargo-deny"
)

check_only=false
[[ "${1:-}" == "--check" ]] && check_only=true
missing=0

for entry in "${TOOLS[@]}"; do
  IFS='|' read -r name version url sha inner <<<"$entry"
  stamp="$BIN/.$name.version"
  if [[ -x "$BIN/$name" && -f "$stamp" && "$(cat "$stamp")" == "$version" ]]; then
    echo "ok       $name $version"
    continue
  fi
  missing=$((missing + 1))
  if $check_only; then
    echo "missing  $name $version"
    continue
  fi
  echo "install  $name $version"
  case "$url" in
    *.tar.gz) kind=tar.gz ;;
    *) kind=zip ;;
  esac
  archive="$DL/$name-$version.$kind"
  if [[ ! -f "$archive" ]]; then
    curl -sSL --fail -o "$archive" "$url"
  fi
  actual="$(shasum -a 256 "$archive" | awk '{print $1}')"
  if [[ "$actual" != "$sha" ]]; then
    echo "SHA-256 mismatch for $name: expected $sha got $actual" >&2
    rm -f "$archive"
    exit 1
  fi
  extract="$DL/$name-$version"
  rm -rf "$extract"
  mkdir -p "$extract"
  if [[ "$kind" == "tar.gz" ]]; then
    tar -xzf "$archive" -C "$extract"
  else
    unzip -q -o "$archive" -d "$extract"
  fi
  if [[ ! -f "$extract/$inner" ]]; then
    echo "archive for $name does not contain $inner" >&2
    exit 1
  fi
  # XcodeGen needs its share/ directory next to bin/; keep the tree and wrap it.
  if [[ "$name" == "xcodegen" ]]; then
    rm -rf "$BIN/.xcodegen-tree"
    cp -R "$extract/xcodegen" "$BIN/.xcodegen-tree"
    # A wrapper, not a symlink: XcodeGen finds share/SettingPresets relative to its real path.
    printf '#!/usr/bin/env bash\nexec "%s/.xcodegen-tree/bin/xcodegen" "$@"\n' "$BIN" >"$BIN/xcodegen"
  else
    cp "$extract/$inner" "$BIN/$name"
  fi
  chmod +x "$BIN/$name"
  xattr -d com.apple.quarantine "$BIN/$name" 2>/dev/null || true
  echo "$version" >"$stamp"
done

if $check_only && [[ $missing -gt 0 ]]; then
  echo "run Tools/bootstrap.sh to install $missing tool(s)" >&2
  exit 1
fi
echo "tools in $BIN"

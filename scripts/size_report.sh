#!/usr/bin/env bash
set -euo pipefail

# Simple size report for APK/App Bundle and native (.so) payloads.
# Usage: ./scripts/size_report.sh [apk_or_aab_path]

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_DIR="$ROOT/app/app"
APK_DIR="$APP_DIR/build/outputs/apk"
AAB_DIR="$APP_DIR/build/outputs/bundle"

pick_latest() {
  local dir="$1" pattern="$2"
  find "$dir" -type f -name "$pattern" 2>/dev/null | sort | tail -n 1
}

TARGET="${1:-}"
if [[ -z "$TARGET" ]]; then
  TARGET="$(pick_latest "$APK_DIR" "*.apk")"
  [[ -z "$TARGET" ]] && TARGET="$(pick_latest "$AAB_DIR" "*.aab")"
fi

if [[ -z "$TARGET" || ! -f "$TARGET" ]]; then
  echo "No APK/AAB found. Build first, or pass a path: ./scripts/size_report.sh app/app/build/outputs/apk/release/app-release.apk" >&2
  exit 1
fi

echo "== Size report for: $TARGET =="
ls -lh "$TARGET"

echo
echo "-- APK/AAB contents (dex/lib/res/other, bytes) --"
unzip -l "$TARGET" | awk '
  /classes[0-9]*\.dex$/ { dex += $1 }
  /^ *[0-9]+ .*lib\/.*\.so$/ { so += $1 }
  /^ *[0-9]+ .*res\// { res += $1 }
  { total += $1 }
  END {
    printf "dex,%d\nlib,.so,%d\nres,%d\nother,%d\n", dex, so, res, (total - dex - so - res)
  }' | column -t -s,

echo
echo "-- Native libs by ABI --"
unzip -l "$TARGET" | awk '/lib\/[^/]+\/.*\.so$/ { size=$1; split($4, parts, "/"); abi=parts[2]; so=parts[3]; data[abi]+=size; printf "%10s %10d %s\n", abi, size, so } END { for (k in data) printf "%10s %10d <total>\n", k, data[k] }' | sort

echo
echo "-- Checked-in JNI libs (uncompressed on disk) --"
find "$APP_DIR/src/main/jniLibs" -type f -name "*.so" -print0 2>/dev/null \
  | xargs -0 ls -lh | sed 's/^/jniLibs: /'

echo
echo "-- Rust target artifacts (if present) --"
if [[ -d "$ROOT/rust/target" ]]; then
  du -sh "$ROOT/rust/target"/* 2>/dev/null | sed 's/^/rust target: /'
else
  echo "rust target: (not built yet)"
fi

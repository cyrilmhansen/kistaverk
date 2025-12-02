#!/usr/bin/env bash
set -euo pipefail

# Generate a deps.json summary for the Android app About screen.
# Runs from the rust/ directory; writes to ../app/app/src/main/assets/deps.json.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_ASSETS="$ROOT_DIR/../app/app/src/main/assets"
OUT_FILE="$APP_ASSETS/deps.json"

mkdir -p "$APP_ASSETS"

write_json() {
  cat > "$OUT_FILE" <<EOF
{
  "packages": [
$1
  ]
}
EOF
  echo "Wrote deps metadata to $OUT_FILE"
}

if cargo metadata --format-version 1 --locked -q >/dev/null 2>&1; then
  metadata="$(cargo metadata --format-version 1 --locked)"
elif cargo metadata --format-version 1 --locked --offline -q >/dev/null 2>&1; then
  metadata="$(cargo metadata --format-version 1 --locked --offline)"
fi

if [[ -n "$metadata" ]]; then
  if command -v jq >/dev/null 2>&1; then
    echo "$metadata" | jq '
      .packages
      | map(if .name == "kistaverk_core" and (.license == null or .license == "") then .license = "LGPL-2.0" else . end)
      | {packages: [.[] | {name, version, license, repository, homepage}]}
    ' > "$OUT_FILE"
    echo "Wrote deps metadata to $OUT_FILE (cargo metadata + jq)"
    exit 0
  else
    # basic parse without jq: take first package as fallback
    pkg_name="$(echo "$metadata" | sed -n 's/.*"name":[[:space:]]*"\([^"]*\)".*/\1/p' | head -n1)"
    pkg_ver="$(echo "$metadata" | sed -n 's/.*"version":[[:space:]]*"\([^"]*\)".*/\1/p' | head -n1)"
    write_json "    { \"name\": \"${pkg_name:-kistaverk_core}\", \"version\": \"${pkg_ver:-0.0.0}\", \"license\": \"LGPL-2.0\", \"repository\": \"\", \"homepage\": \"\" }"
    exit 0
  fi
fi

echo "cargo metadata unavailable; falling back to Cargo.lock parse"
lock_file="$ROOT_DIR/Cargo.lock"
if [[ ! -f "$lock_file" ]]; then
  echo "Cargo.lock missing; writing minimal deps.json"
  write_json "    { \"name\": \"kistaverk_core\", \"version\": \"0.0.0\", \"license\": \"\", \"repository\": \"\", \"homepage\": \"\" }"
  exit 0
fi

packages=$(awk '
  /^\[\[package\]\]/ {name=""; version=""; next}
  /^name = / {name=$3; gsub(/"/, "", name)}
  /^version = / {version=$3; gsub(/"/, "", version)}
  /^source = / { if (name != "" && version != "") { print name " " version; name=""; version="" } }
  END { if (name != "" && version != "") print name " " version }
' "$lock_file" | sort -u)

entries=""
first=true
while read -r line; do
  [[ -z "$line" ]] && continue
  name=$(echo "$line" | awk '{print $1}')
  ver=$(echo "$line" | awk '{print $2}')
  prefix=""
  if [ "$first" = false ]; then
    prefix=",\n"
  else
    first=false
  fi
  license=""; if [ "$name" = "kistaverk_core" ]; then license="MIT OR Apache-2.0"; fi
  entries+="${prefix}    { \"name\": \"${name}\", \"version\": \"${ver}\", \"license\": \"${license}\", \"repository\": \"\", \"homepage\": \"\" }"
done <<< "$packages"

if [[ -z "$entries" ]]; then
  entries="    { \"name\": \"kistaverk_core\", \"version\": \"0.0.0\", \"license\": \"\", \"repository\": \"\", \"homepage\": \"\" }"
fi

write_json "$entries"

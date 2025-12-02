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

metadata=""
if cargo metadata --format-version 1 --locked -q >/dev/null 2>&1; then
  metadata="$(cargo metadata --format-version 1 --locked)"
elif cargo metadata --format-version 1 --locked --offline -q >/dev/null 2>&1; then
  metadata="$(cargo metadata --format-version 1 --locked --offline)"
fi

if [[ -n "$metadata" ]]; then
  if command -v jq >/dev/null 2>&1; then
    echo "$metadata" | jq '
      .packages
      | map(if .name == "kistaverk_core" and (.license == null or .license == "") then .license = "AGPL-3.0-or-later" else . end)
      | {packages: [.[] | {name, version, license, repository, homepage}]}
    ' > "$OUT_FILE"
    echo "Wrote deps metadata to $OUT_FILE (cargo metadata + jq)"
    exit 0
  else
    # basic parse without jq: take first package as fallback
    pkg_name="$(echo "$metadata" | sed -n 's/.*"name":[[:space:]]*"\([^"]*\)".*/\1/p' | head -n1)"
    pkg_ver="$(echo "$metadata" | sed -n 's/.*"version":[[:space:]]*"\([^"]*\)".*/\1/p' | head -n1)"
    write_json "    { \"name\": \"${pkg_name:-kistaverk_core}\", \"version\": \"${pkg_ver:-0.0.0}\", \"license\": \"AGPL-3.0-or-later\", \"repository\": \"\", \"homepage\": \"\" }"
    exit 0
  fi
fi

echo "cargo metadata unavailable; falling back to Cargo.lock parse"
lock_file="$ROOT_DIR/Cargo.lock"
if [[ ! -f "$lock_file" ]]; then
  echo "Cargo.lock missing; writing minimal deps.json"
  write_json "    { \"name\": \"kistaverk_core\", \"version\": \"0.0.0\", \"license\": \"AGPL-3.0-or-later\", \"repository\": \"\", \"homepage\": \"\" }"
  exit 0
fi

python3 - <<'PY' "$lock_file" "$OUT_FILE"
import json, sys, tomllib
lock_path, out_path = sys.argv[1:]
data = tomllib.load(open(lock_path, "rb"))
packages = []
for pkg in data.get("package", []):
    name = pkg.get("name")
    ver = pkg.get("version")
    if not name:
        continue
    license = "AGPL-3.0-or-later" if name == "kistaverk_core" else pkg.get("license", "") or ""
    packages.append({
        "name": name,
        "version": ver or "",
        "license": license,
        "repository": "",
        "homepage": "",
    })
packages.sort(key=lambda p: (p["name"], p["version"]))
with open(out_path, "w", encoding="utf-8") as f:
    json.dump({"packages": packages}, f, ensure_ascii=False, indent=2)
print(f"Wrote deps metadata to {out_path} (Cargo.lock parse)")
PY

#!/usr/bin/env bash
set -euo pipefail

# Generate dependency metadata from cargo metadata into app assets.
# Output: app/app/src/main/assets/deps.json

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ASSETS_DIR="${ROOT_DIR}/app/app/src/main/assets"
OUT_FILE="${ASSETS_DIR}/deps.json"

mkdir -p "${ASSETS_DIR}"

cargo metadata --format-version 1 --no-deps --locked \
  | jq '{packages: [.packages[] | {name, version, license: (.license // ""), repository: (.repository // ""), homepage: (.homepage // "")}]}' \
  > "${OUT_FILE}"

echo "Wrote ${OUT_FILE}"

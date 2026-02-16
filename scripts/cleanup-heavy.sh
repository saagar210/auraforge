#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

# Heavy build outputs that are safe to regenerate.
paths=(
  "dist"
  "src-tauri/target"
  "node_modules/.vite"
  ".vite"
)

for path in "${paths[@]}"; do
  if [ -e "$path" ]; then
    rm -rf "$path"
    echo "[clean:heavy] removed $path"
  else
    echo "[clean:heavy] skipped $path (not present)"
  fi
done

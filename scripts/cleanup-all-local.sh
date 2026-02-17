#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

# Local reproducible artifacts/caches.
paths=(
  "node_modules"
  "dist"
  "src-tauri/target"
  ".vite"
)

safe_remove() {
  local path="$1"
  if [ ! -e "$path" ]; then
    echo "[clean:all-local] skipped $path (not present)"
    return 0
  fi

  if [ "$path" = "node_modules" ]; then
    # npm can keep transient file handles; rename first, then delete.
    local tombstone="node_modules.__cleanup.$$"
    mv "$path" "$tombstone"
    rm -rf "$tombstone"
    echo "[clean:all-local] removed $path"
    return 0
  fi

  rm -rf "$path"
  echo "[clean:all-local] removed $path"
}

for path in "${paths[@]}"; do
  safe_remove "$path"
done

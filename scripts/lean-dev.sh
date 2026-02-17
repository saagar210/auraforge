#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

LEAN_TMP_DIR="$(mktemp -d -t auraforge-lean-XXXXXX)"
LEAN_CARGO_TARGET_DIR="$LEAN_TMP_DIR/cargo-target"
LEAN_VITE_CACHE_DIR="$LEAN_TMP_DIR/vite-cache"
mkdir -p "$LEAN_CARGO_TARGET_DIR" "$LEAN_VITE_CACHE_DIR"

cleanup() {
  if [ -n "${LEAN_TMP_DIR:-}" ] && [ -d "$LEAN_TMP_DIR" ]; then
    rm -rf "$LEAN_TMP_DIR"
    echo "[lean-dev] cleaned temporary caches: $LEAN_TMP_DIR"
  fi
}
trap cleanup EXIT INT TERM

export CARGO_TARGET_DIR="$LEAN_CARGO_TARGET_DIR"
export VITE_CACHE_DIR="$LEAN_VITE_CACHE_DIR"

echo "[lean-dev] CARGO_TARGET_DIR=$CARGO_TARGET_DIR"
echo "[lean-dev] VITE_CACHE_DIR=$VITE_CACHE_DIR"

if [ -n "${LEAN_DEV_MAX_SECONDS:-}" ]; then
  echo "[lean-dev] auto-stop after ${LEAN_DEV_MAX_SECONDS}s (LEAN_DEV_MAX_SECONDS)"
  npm run tauri -- dev &
  tauri_pid=$!
  (
    sleep "$LEAN_DEV_MAX_SECONDS"
    kill -INT "$tauri_pid" 2>/dev/null || true
  ) &
  timer_pid=$!
  wait "$tauri_pid" || true
  kill "$timer_pid" 2>/dev/null || true
else
  exec npm run tauri -- dev
fi

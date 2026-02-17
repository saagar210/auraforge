#!/usr/bin/env bash
set -euo pipefail

echo "AuraForge local setup (non-destructive)."
command -v node >/dev/null 2>&1 && node -v || echo "node: missing"
command -v npm >/dev/null 2>&1 && npm -v || echo "npm: missing"
command -v cargo >/dev/null 2>&1 && cargo --version || echo "cargo: missing"

echo
echo "Install deps (package-lock present):"
echo "  npm install"
echo "Lean dev mode (README.md):"
echo "  npm run dev:lean"

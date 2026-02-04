# AuraForge Test Report

**Date:** 2026-02-03
**Branch:** audit/fix-hardening

## Environment

- **Node:** v20.19.6
- **npm:** 11.4.2
- **rustc:** 1.89.0 (29483883e 2025-08-04)
- **cargo:** 1.89.0 (c24e10642 2025-06-23)

## Commands & Results

### Frontend

- `npm ci` — **PASS**
- `npx tsc -b` — **PASS**
- `npm run lint` — **PASS** (typecheck-based)
- `npm test` — **PASS** (no frontend tests configured)
- `npm run build` — **PASS** (Vite build succeeded; chunk size warning emitted)

### Tauri / Rust (src-tauri)

- `cargo fmt --check` — **PASS**
- `cargo clippy --all-targets --all-features -- -D warnings` — **FAIL** (glib-2.0 not available in environment; `glib-sys` build script failed)
- `cargo test` — **FAIL** (blocked by missing glib-2.0)
- `cargo build` — **FAIL** (blocked by missing glib-2.0)

## Notes

- Rust checks are blocked in this environment due to missing system dependency `glib-2.0` (via `glib-sys`). Installing the library and setting `PKG_CONFIG_PATH` should unblock clippy/test/build.
- Frontend lint and test scripts now exist; lint currently runs TypeScript typechecking and tests report that no frontend tests are configured.

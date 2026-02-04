# AuraForge Audit Report

## Findings

| ID | Area | Severity | Evidence | Fix Plan |
| --- | --- | --- | --- | --- |
| AF-001 | Data model consistency | Medium | Backend `Message.metadata` is persisted as text; frontend expects structured metadata which can cause runtime mismatches if consumers treat it as an object.【F:src-tauri/src/db/mod.rs†L182-L244】【F:src/types.ts†L15-L31】 | Parse metadata into JSON values on the backend before emitting to the UI (implemented).【F:src-tauri/src/db/mod.rs†L8-L240】【F:src-tauri/src/types.rs†L1-L24】 |
| AF-002 | Privacy / search config | Medium | `web_search` previously forced DuckDuckGo even when search was disabled or provider was `none`, violating user intent.【F:src-tauri/src/commands/mod.rs†L788-L804】 | Respect `search.enabled` and `provider` in `web_search` (implemented).【F:src-tauri/src/commands/mod.rs†L788-L804】 |
| AF-003 | Correctness / disk space | Low | `check_disk_space` treated unparsable `df` output as 0 KB, which could incorrectly block exports on systems with different `df` formats.【F:src-tauri/src/commands/mod.rs†L315-L345】 | Treat parse failures as “unknown” and assume sufficient space with a warning (implemented).【F:src-tauri/src/commands/mod.rs†L315-L345】 |
| AF-004 | Release readiness / CI | Medium | CI lacked `libglib2.0-dev`, causing `glib-sys` failures in Rust checks. | Install `libglib2.0-dev` in CI to satisfy `glib-sys` (implemented).【F:.github/workflows/ci.yml†L26-L34】 |
| AF-005 | DX / quality gates | Low | `package.json` lacked `lint` and `test` scripts, limiting standardized local/CI checks.【F:package.json†L1-L16】 | Add scripts for lint/typecheck and test stubs (implemented).【F:package.json†L1-L16】 |
| AF-006 | Security posture | Medium | CSP allows `unsafe-eval` and `unsafe-inline`, which reduces defense-in-depth in production contexts.【F:src-tauri/tauri.conf.json†L15-L37】 | Audit which directives are needed for Vite/Tauri production and tighten CSP where possible (deferred).【F:src-tauri/tauri.conf.json†L15-L37】 |

## Quick Wins (<=10)

1. Normalize backend metadata into JSON values before emitting to the UI (done).【F:src-tauri/src/db/mod.rs†L8-L240】【F:src-tauri/src/types.rs†L1-L24】
2. Respect user search settings by returning empty results when disabled (done).【F:src-tauri/src/commands/mod.rs†L788-L804】
3. Harden disk-space parsing fallback to avoid false “out of space” results (done).【F:src-tauri/src/commands/mod.rs†L315-L345】
4. Ensure CI installs `libglib2.0-dev` for Rust checks (done).【F:.github/workflows/ci.yml†L26-L34】
5. Add lint/test scripts for consistent local checks (done).【F:package.json†L1-L16】

## High Risk (Top 3) + Mitigations

1. **Search configuration bypass** could trigger unexpected external calls. **Mitigation:** respect `search.enabled`/`provider` in `web_search` (implemented).【F:src-tauri/src/commands/mod.rs†L788-L804】
2. **Backend/frontend metadata mismatch** can cause runtime errors as metadata usage grows. **Mitigation:** parse metadata into structured JSON on the backend (implemented).【F:src-tauri/src/db/mod.rs†L8-L240】【F:src-tauri/src/types.rs†L1-L24】
3. **CI dependency gap** can block Rust checks. **Mitigation:** install `libglib2.0-dev` in CI (implemented).【F:.github/workflows/ci.yml†L26-L34】

## Deferred (Not fixed yet)

- Tighten CSP after confirming Vite/Tauri runtime requirements (remove `unsafe-eval` if possible).【F:src-tauri/tauri.conf.json†L15-L37】

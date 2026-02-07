# AuraForge Audit Report (2026-02-07)

## Scope
Correctness-first runtime audit across React/Zustand frontend and Tauri/Rust backend, with emphasis on desktop safety (filesystem, async concurrency, cancellation, and partial-failure handling).

## Phase 0: Repo Orientation (Stop/Go)
- **Top-level structure**: `src/` (React app), `src-tauri/` (Rust backend + Tauri config), `distribution/`, `README.md`, `CONTRIBUTING.md`.
- **Frontend entry + state**: `src/main.tsx` -> `src/App.tsx` -> centralized state/actions in `src/stores/chatStore.ts` (Zustand).
- **Rust command boundary**: `src-tauri/src/commands/mod.rs` exposed via `tauri::generate_handler!` in `src-tauri/src/lib.rs`.
- **Config files**: `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`, `vite.config.ts`, `tsconfig.json`, `package.json`, `src-tauri/Cargo.toml`.
- **Frontend -> Rust invocation**: `invoke(...)` calls in `chatStore`/components; event bridge via `listen(...)` for `stream:*`, `generate:*`, `model:pull_progress`.
- **Filesystem IO locations**:
  - Config read/write: `src-tauri/src/config.rs`
  - DB init and recovery: `src-tauri/src/lib.rs`, `src-tauri/src/db/mod.rs`
  - Export writes: `save_to_folder` in `src-tauri/src/commands/mod.rs`
- **Long-running/streaming ops**:
  - LLM response streaming + cancel flag: `src-tauri/src/llm/mod.rs`
  - Model pull streaming + progress events: `src-tauri/src/llm/mod.rs`
  - Multi-doc generation loop: `src-tauri/src/docgen/mod.rs`
- **Stop/Go decision**: **GO** (execution flow understood end-to-end)

## Phase 1: Baseline Verification

### Frontend
- `npm install` -> pass
- `npx tsc --noEmit` -> pass
- `npm run lint --if-present` -> no lint script present (no-op)
- `npm run build` -> pass (Vite chunk size warning only)

### Rust / Tauri
- `cargo fmt --check` -> initially failed (format drift)
- `cargo clippy --all-targets --all-features -- -D warnings` -> pass
- `cargo test` -> pass
- `cargo build` -> pass

Baseline failure inventory before fixes:
1. Rust formatting drift (`cargo fmt --check`).

## Phase 2: Findings (Prioritized)

| Area | Severity | Evidence | Fix Strategy | Status |
|---|---|---|---|---|
| Correctness & State Safety | High | `src/stores/chatStore.ts` async `loadDocuments`/`checkStale` updated state after await without re-checking active session | Session-scope post-await mutations; pass explicit session IDs for stale checks | Fixed |
| Correctness & State Safety | High | `src/stores/chatStore.ts` cancel fallback used untracked `setTimeout`, which could reset a newer stream in same session | Introduce tracked cancel safety timer + clear on stream done/error/cleanup | Fixed |
| Rust-Side Safety | High | `src-tauri/src/llm/mod.rs` `stream_chat` returned success without explicit `done` marker when stream ended unexpectedly | Require explicit completion (`done=true`) including trailing-buffer parse; return interruption error otherwise | Fixed |
| Rust-Side Safety | High | `src-tauri/src/llm/mod.rs` `pull_model` returned `Ok(())` if stream ended before `success` status | Require explicit `success` status, emit error progress, return interruption error when missing | Fixed |
| Filesystem & Persistence | High | `src-tauri/src/commands/mod.rs` export wrote directly into final dir; failure mid-loop left partial output | Stage writes in unique temp dir and rename only after all writes succeed; clean staging on error | Fixed |
| Filesystem & Persistence | Medium | `src-tauri/src/db/mod.rs` message ordering + retry-delete used second-resolution timestamps | Order by `rowid` for deterministic insertion order and correct "last assistant" delete semantics | Fixed |
| Build/Release Hygiene | Medium | `cargo fmt --check` initially failed | Format all Rust files with `cargo fmt` | Fixed |
| Filesystem & Persistence | Medium | `src-tauri/src/config.rs` uses temp-write + rename pattern that may not be atomic/replace-safe on all Windows cases | Add platform-aware replace logic and tests for repeated writes | Deferred |
| Security Posture | Medium | CSP currently permits `'unsafe-inline'` and `'unsafe-eval'` in `src-tauri/tauri.conf.json` | Tighten CSP to minimum required directives, validate runtime behavior | Deferred |
| Verification Depth | Medium | No frontend/IPC integration tests for multi-session race/cancel paths | Add targeted tests for store async races and command/event interaction | Deferred |

## Top 3 “Could Bite Later” Risks
1. **Windows config persistence edge cases**: config replacement semantics can differ from Unix, potentially causing save failures or non-atomic updates.
2. **Permissive CSP**: broad script/style allowances increase blast radius if untrusted content handling changes.
3. **Limited integration coverage**: critical async/event interleavings are mostly validated through manual/runtime checks rather than automated integration tests.

## Explicitly Deferred Items
1. **Windows-safe config replacement hardening**
   - Deferred because it needs platform-specific handling and test validation on Windows to avoid regressions.
2. **CSP tightening pass**
   - Deferred because tightening requires careful runtime validation against current markdown/render/plugin behavior.
3. **Frontend/Tauri integration test suite**
   - Deferred due scope; requires introducing and wiring test harnesses for event-driven desktop flows.

## Phase 3: Fix Loop Summary (Completed)
Implemented smallest safe fixes first (highest severity first), each in an isolated commit:

1. `f46c7cd` - `fix(ui): prevent stale async state races in chat store`
   - Session-guarded document/staleness updates.
   - Tracked cancel safety timer to avoid false stream resets.

2. `be288c5` - `fix(llm): fail interrupted streams instead of returning partial success`
   - Streaming now requires explicit completion for both chat and model pull.
   - Interrupted/partial streams now surface proper errors.

3. `8bc0e75` - `fix(export): avoid partial plan folders on write failures`
   - Export now stages files in temp dir and promotes atomically after all writes succeed.

4. `0f240e1` - `fix(db): order message reads and retries by insertion`
   - Stable ordering by insertion (`rowid`) and retry deletion correctness.
   - Added regression tests for identical timestamps.

## Phase 4: Hardening Pass
- No architecture rewrite.
- Focused hardening delivered in existing modules only:
  - Async/session race protection
  - Stream completion enforcement
  - Atomic export staging
  - Deterministic DB ordering
- Rust formatting normalized (`cargo fmt`) to keep release checks green.

## Phase 5: Final Verification
- `npm install` -> pass
- `npx tsc --noEmit` -> pass
- `npm run lint --if-present` -> no-op (script absent)
- `npm run build` -> pass
- `cargo fmt --check` -> pass
- `cargo clippy --all-targets --all-features -- -D warnings` -> pass
- `cargo test` -> pass (`42 passed`)
- `cargo build` -> pass

## Remaining Known Risks
- Windows-specific config save replacement semantics not yet hardened.
- CSP still broad relative to least-privilege target.
- No dedicated integration tests for cross-session event races.

## Implementation Addendum (Model-Agnostic Execution Pack)

### New Findings Addressed
| Area | Severity | Evidence | Fix Strategy | Status |
|---|---|---|---|---|
| Correctness & State Safety | High | Forge flow could fail late with opaque backend error when readiness must-haves were missing | Add frontend readiness precheck + explicit user confirmation path that retries with `force=true` | Fixed |
| Output Consistency | High | Output pack was Claude-skewed and lacked explicit handoff metadata for other coding agents | Add target-aware `MODEL_HANDOFF.md`, persist generation metadata, and include export `manifest.json` | Fixed |
| UX/Data Integrity | Medium | Document tab rendering depended on a fixed filename list and would hide new docs | Replace rigid mapping with ranked sort + fallback for unknown docs | Fixed |
| Configuration Robustness | Medium | `output.default_target` and optional `llm.api_key` were not editable in settings | Add full settings support and validation for default target + API key field | Fixed |

### Fixed Issues Summary
- Added explicit forge targets (`claude`, `codex`, `cursor`, `gemini`, `generic`) end-to-end across config, API types, Rust commands, and frontend state.
- Added readiness analysis command and confirmation UX to prevent accidental generation of incomplete plans.
- Added `MODEL_HANDOFF.md` generation with target-specific execution rules and readiness context.
- Persisted generation metadata in SQLite and emitted `manifest.json` during folder export for reproducible handoffs.
- Updated preview/help/docs to include model handoff and dynamic document ordering.

### Remaining Risks (Post-Addendum)
- LLM provider abstraction is still largely Ollama-centric; provider-specific request/auth flows for non-Ollama backends are not yet implemented.
- No automated frontend tests currently validate readiness confirmation and forced-generation UX.
- CSP tightening and Windows-specific config replacement hardening remain deferred from the earlier audit pass.

## Suggested Next Iteration Priorities
1. Implement and validate Windows-safe config replacement path (`save_config`) with platform-targeted tests.
2. Tighten CSP directives and verify markdown/render workflows still function.
3. Add minimal integration coverage for `chatStore` async race/cancel scenarios and stream event handling.

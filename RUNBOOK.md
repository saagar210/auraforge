# AuraForge Runbook

Updated: 2026-02-07  
Scope: macOS + Linux runtime and release operations

## 1) Operational Goals

1. Keep AuraForge usable even when optional dependencies fail (search providers, model pull, flaky local runtimes).
2. Prevent silent data loss (atomic config writes, transactional DB updates, staged folder export).
3. Maintain a local-first workflow with no required paid API keys.

## 2) Required Runtime Dependencies

## 2.1 Core (required)

- Node.js 18+
- Rust 1.75+
- Ollama or another local OpenAI-compatible runtime endpoint

## 2.2 Optional (non-blocking)

- Tavily API key (search quality enhancement only)
- Self-hosted SearXNG endpoint

If optional search providers fail, AuraForge falls back to DuckDuckGo and should remain usable.

## 3) Canonical Verification Commands

Run from repo root unless noted:

```bash
npx tsc --noEmit
npm run test
npm run build
cd src-tauri
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build
cd ..
npm run tauri build
```

Expected: all commands pass and `npm run tauri build` emits macOS/Linux bundles.

## 4) Runtime Smoke Checklist (Packaged App)

1. Launch app and confirm sessions list loads.
2. Create new session and send a prompt.
3. Confirm response streams and can be canceled safely.
4. Forge planning documents.
5. Save to folder and confirm docs plus `manifest.json` exist.
6. Restart app and confirm settings persist.
7. Disable primary search provider and confirm fallback still allows chat.

## 5) Failure Playbooks

## 5.1 Local model runtime unavailable

Symptoms:
- setup check fails
- stream never starts
- provider health command fails

Actions:
1. Verify local runtime process is up (`ollama serve` or equivalent local endpoint).
2. Verify configured base URL in settings.
3. Re-run model list and connection test in app settings.
4. If issue persists, capture endpoint URL, provider mode, and command error payload.

## 5.2 Streaming appears stuck

Symptoms:
- spinner persists without tokens
- user canceled but UI remains busy

Actions:
1. Trigger cancel and wait for timeout cleanup.
2. Retry response for the same session.
3. Switch sessions and return; verify stale stream did not overwrite current session.
4. If reproducible, collect logs around `stream:*` events and session ID transitions.

## 5.3 Export fails or partial output concerns

Symptoms:
- save-to-folder returns permission/path error
- expected files missing

Actions:
1. Verify destination exists and is writable.
2. Verify project folder name does not collide with existing folder.
3. Retry export to a fresh writable destination.
4. Confirm folder contains full document set and `manifest.json`.

Note: export writes stage to a temp directory and promote atomically, so partial final folders should not occur on normal failure paths.

## 5.4 Search provider failures

Symptoms:
- search warnings during technical prompts
- missing external context in responses

Actions:
1. Confirm optional provider credentials and endpoint settings.
2. Validate fallback path by disabling optional provider and retrying.
3. Confirm chat still proceeds with fallback/without search.

## 5.5 Config or DB corruption recovery

Symptoms:
- startup warns about corrupt config/db
- settings fail to load

Actions:
1. Inspect `~/.auraforge/` for backup files (`*.bak`).
2. Restart app; it should regenerate defaults and attempt recovery.
3. If DB remains broken, backup corrupted file and restore from known good copy.
4. Re-run smoke checklist.

## 6) Release Flow

1. Follow `RELEASE_CHECKLIST.md` in order.
2. Do not ship if any required gate is red.
3. Update `AUDIT_REPORT.md` and `IMPLEMENTATION_MAP.md` with final status.

## 7) Known Deferred Items

1. Windows-specific release hardening (out of active scope).
2. External PM integrations (GitHub Issues / Linear / Jira / Notion sync).

## 8) Incident Capture Template

Record the following for every production issue:

- Timestamp and environment (macOS/Linux version, app version)
- Active provider mode and endpoint
- Session ID (if available)
- Repro steps
- Observed vs expected behavior
- Error payload shown to user
- Immediate mitigation used


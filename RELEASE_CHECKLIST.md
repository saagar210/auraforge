# AuraForge Release Checklist

Use this checklist for every release candidate on macOS/Linux.

## 1) Branch + Version Hygiene

- Confirm working tree is clean:
  - `git status --short --branch`
- Confirm release branch target and base are correct.
- Confirm `src-tauri/tauri.conf.json` has expected:
  - `version`
  - `identifier`

## 2) Frontend Gate

- `npx tsc --noEmit`
- `npm run test`
- `npm run build`

Expected:
- Typecheck passes with no errors.
- Store/integration tests pass.
- Production build succeeds.

## 3) Rust/Tauri Gate

- `cd src-tauri`
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `cargo test commands::tests::prepare_export_documents_backfills_required_reports`
- `cargo test importer::tests::summarize_codebase_emits_grounded_sections_with_citations`
- `AURAFORGE_INGEST_SMOKE_REPO=/path/to/other/repo cargo test importer::tests::smoke_import_real_repo_from_env -- --ignored --nocapture` (manual release smoke)
- `cargo build`

Expected:
- No formatting drift.
- No clippy warnings.
- All tests pass.
- Debug build succeeds.

## 4) Packaging Gate

From repo root:

- `npm run tauri build`

Expected macOS outputs:
- `src-tauri/target/release/bundle/macos/AuraForge.app`
- `src-tauri/target/release/bundle/dmg/AuraForge_<version>_aarch64.dmg`

Linux CI expected outputs:
- `.deb`
- `.AppImage`

## 5) Runtime Smoke (manual)

Validate in packaged app:

1. App starts cleanly and shows sessions.
2. New session -> send message -> streaming response works.
3. Cancel response works and does not leave stuck spinner.
4. Forge documents succeeds.
5. Save to folder succeeds and writes:
   - `docs/`, `handoff/`, `context/`, and `reports/` folders
   - `manifest.json`
   - `reports/LINT_REPORT.md`
   - `reports/ARTIFACT_CHANGELOG.md`
   - `reports/ARTIFACT_DIFF.json`
6. Settings persist across restart.
7. Search failure paths do not block responses.

## 6) Security/Policy Spot Check

- Confirm CSP and capability policy changes were intentional:
  - `src-tauri/tauri.conf.json`
  - `src-tauri/capabilities/default.json`
- Confirm no new secrets in repo:
  - `rg -n "api[_-]?key|token|secret" src src-tauri`

## 7) Release Notes + Audit Sync

- Update `AUDIT_REPORT.md` with:
  - fixes landed
  - verification commands and outcomes
  - remaining risks
- Update `IMPLEMENTATION_MAP.md` status blocks.
- Add release evidence to `docs/release-notes-YYYY-MM-DD.md`.
- Confirm `RUNBOOK.md` still matches operational behavior and troubleshooting flows.

## 8) Final CI Gate

- Confirm GitHub Actions `linux-ci` is green:
  - `checks`
  - `bundle-linux`

Do not ship if any required gate is red.

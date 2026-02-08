# AuraForge Release Notes (2026-02-08)

## Scope

This release closes the remaining follow-ups from the next-phase hardening plan:
- manifest schema compatibility assertions
- CI/runtime optimization for invariant checks
- external repo-ingest smoke validation with grounded evidence

## Highlights

1. Export manifest compatibility guardrails:
- Added explicit compatibility range checks for `manifest.json` schema versions.
- Added regression tests to enforce support for current schema and expected backward compatibility.

2. CI efficiency + invariant enforcement:
- Removed redundant targeted Rust test reruns in CI.
- Added fast invariant and docs-drift reference checks:
  - required invariant tests still present in source
  - required artifact report docs still documented
  - roadmap doc presence check

3. Repo-ingest smoke validation:
- Added manual, env-driven smoke test:
  - `importer::tests::smoke_import_real_repo_from_env` (ignored by default)
- Executed smoke run against:
  - `/Users/d/Projects/MemoryKernel`

## Smoke Run Evidence

Command:
```bash
cd src-tauri
AURAFORGE_INGEST_SMOKE_REPO=/Users/d/Projects/MemoryKernel \
  cargo test importer::tests::smoke_import_real_repo_from_env -- --ignored --nocapture
```

Observed metrics:
- `SMOKE_FILES_SCANNED=178`
- `SMOKE_FILES_INCLUDED=178`
- `SMOKE_BYTES_READ=1084918`
- `SMOKE_STACKS=Rust`
- `SMOKE_KEY_FILES=22`
- `SMOKE_CITATIONS=10`
- `SMOKE_CITATION_SAMPLE=Cargo.toml:1-6, README.md:1-6, components/outcome-memory/Cargo.toml:1-6`

Stop/Go decision:
- **Go**: ingestion remained grounded with citation output and no hallucination-only sections.

## Verification Summary

- `npx tsc --noEmit` passed
- `npm run test -- --run` passed
- `npm run build` passed
- `cargo fmt --check` passed
- `cargo clippy --all-targets --all-features -- -D warnings` passed
- `cargo test` passed
- `cargo build` passed
- `npm run tauri build` passed

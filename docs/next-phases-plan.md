# AuraForge Next Phases Implementation Plan

Updated: 2026-02-08
Execution status: Completed

## Completion Summary

- N1 Reliability Gates and Artifact Integrity: Completed
- N2 Frontend Bundle Risk Reduction: Completed
- N3 Repo Ingest Confidence and Evidence Quality: Completed
- N4 Release Readiness and Operationalization: Completed

Manual repo-ingest smoke was executed against a second local production-like repository (`/Users/d/Projects/MemoryKernel`) and recorded in release notes.

This plan extends the completed Foundation/Iter1/Iter2/Polish roadmap with the next execution wave focused on hardening, performance, and operational repeatability.

## Phase N1: Reliability Gates and Artifact Integrity

### Deliverables
- Enforce export-pack invariants so required reports always exist:
  - `reports/LINT_REPORT.md`
  - `reports/ARTIFACT_CHANGELOG.md`
  - `reports/ARTIFACT_DIFF.json`
- Backfill missing reports for legacy sessions with deterministic placeholder artifacts.
- Add explicit CI checks for artifact integrity tests.

### File-level touchpoints
- `src-tauri/src/commands/mod.rs`
- `.github/workflows/linux-ci.yml`

### Stop/Go checkpoint
- **Go** only if:
  - export tests prove required reports are always present
  - CI includes dedicated artifact-invariant checks
- **Stop** if any export regression changes folder contract (`docs/`, `handoff/`, `context/`, `reports/`)

### Verification plan
- `cargo test commands::tests::prepare_export_documents_backfills_required_reports`
- `cargo test`
- full CI run of `linux-ci`

### Risk fixes addressed
- Missing report artifacts in legacy runs.
- Silent regressions in export layout.

### Follow-ups
- Add manifest-level compatibility checks for future schema versions.

## Phase N2: Frontend Bundle Risk Reduction

### Deliverables
- Lazy-load heavyweight UI surfaces:
  - document preview
  - settings panel
  - help panel
- Lazy-load syntax highlighting payload only when code fences are rendered.
- Add chunking strategy to reduce startup payload concentration.

### File-level touchpoints
- `src/App.tsx`
- `src/components/DocumentPreview.tsx`
- `src/components/SyntaxCodeBlock.tsx`
- `vite.config.ts`

### Stop/Go checkpoint
- **Go** only if:
  - `npm run build` passes
  - no runtime regressions in document preview, settings, or help flows
- **Stop** if lazy boundaries break keyboard shortcuts, modal behavior, or preview rendering

### Verification plan
- `npx tsc --noEmit`
- `npm run test -- --run`
- `npm run build`
- manual app smoke via `npm run tauri build`

### Risk fixes addressed
- Large eager bundle concentration and startup fragility.

### Follow-ups
- Evaluate route-level split if future features push chunk sizes upward.

## Phase N3: Repo Ingest Confidence and Evidence Quality

### Deliverables
- Add importer tests that verify grounded sections and citation presence when evidence exists.
- Add importer tests that verify explicit `[TBD]` markers for sparse evidence.

### File-level touchpoints
- `src-tauri/src/importer/mod.rs`
- `.github/workflows/linux-ci.yml`

### Stop/Go checkpoint
- **Go** only if:
  - tests verify grounded output sections
  - tests verify `[TBD]` behavior under sparse evidence
- **Stop** if ingest can produce unsupported claims without citation evidence

### Verification plan
- `cargo test importer::tests::summarize_codebase_emits_grounded_sections_with_citations`
- `cargo test importer::tests::summarize_codebase_marks_tbd_when_evidence_is_sparse`
- `cargo test`

### Risk fixes addressed
- Hallucination risk in ingest summaries.
- Inconsistent handling of unknowns.

### Follow-ups
- Run a real-world ingest smoke test against an external local repository before release sign-off.

## Phase N4: Release Readiness and Operationalization

### Deliverables
- Keep runbook/checklist aligned with artifact invariants and ingest verification.
- Track residual risks as explicit follow-ups with measurable acceptance criteria.

### File-level touchpoints
- `RUNBOOK.md`
- `RELEASE_CHECKLIST.md`
- `README.md`

### Stop/Go checkpoint
- **Go** only if docs and CI commands match the actual implementation.
- **Stop** if documented commands diverge from workflow or produce non-green outcomes.

### Verification plan
- Execute every command listed in `RUNBOOK.md` and `RELEASE_CHECKLIST.md`.
- Confirm bundle output artifacts exist and are consumable.

### Risk fixes addressed
- Documentation drift versus shipped behavior.

### Follow-ups
- Add periodic doc drift checks in CI (docs lint + command consistency scan).

## Global measurable criteria

- **Output quality:** lint and diff artifacts always present in export packs.
- **Reproducibility:** deterministic placeholders for legacy sessions + stable export path mapping.
- **Verification:** all gates are executable via local and CI command sets.
- **Grounded ingest:** citations present when evidence exists; `[TBD]` markers when evidence is sparse.

## Follow-up Closure (2026-02-08)

1. Manifest schema compatibility checks were added and verified via tests.
2. CI runtime was optimized by removing redundant targeted test reruns while keeping invariant checks explicit.
3. Documentation drift checks were added to CI (`README.md`, `RUNBOOK.md`, `RELEASE_CHECKLIST.md`, and roadmap doc presence).
4. External repo-ingest smoke run completed and release notes updated with observed metrics and citation samples.

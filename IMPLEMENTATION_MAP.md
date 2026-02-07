# AuraForge Implementation Map (Roadmap Remainder)

Updated: 2026-02-07
Owner: Engineering

## 1) Scope and constraints

This map covers all unchecked roadmap items in `README.md`:

1. Confidence scoring
2. Planning coverage UI
3. Audit report fixes (`AppError`, transaction safety, CSP hardening)
4. Windows and Linux builds
5. Additional local model runtimes (LM Studio/Ollama-compatible endpoints)
6. Project templates
7. Import existing codebases for refactoring plans
8. Export to GitHub Issues / Linear integration
9. Conversation branching

Non-negotiable constraints:

- Keep model inference local-first by default.
- Keep existing local folder export flow as a stable finish line.
- Prefer incremental delivery with small, testable slices.
- No architecture rewrite unless needed for correctness/security.

## 2) Current baseline

- Frontend: React 19 + TypeScript + Zustand.
- Backend: Tauri 2 + Rust + SQLite.
- Core forge pipeline includes:
  - Readiness analysis
  - Target-aware `MODEL_HANDOFF.md`
  - Export `manifest.json`
- Stability hardening already completed:
  - Session race fixes
  - Streaming interruption handling
  - Atomic export staging
  - Deterministic DB ordering

## 3) Delivery sequence (recommended)

Execution order is designed to reduce risk:

1. Confidence scoring
2. Planning coverage UI
3. Audit report fixes (`AppError`, transaction safety, CSP)
4. Additional local model runtimes
5. Project templates
6. Import existing codebases
7. Conversation branching
8. Windows and Linux builds
9. GitHub/Linear export integration

Reasoning:

- Items 1-3 improve correctness and transparency now.
- Items 4-7 expand core product capability.
- Item 8 is packaging/distribution-heavy and easier after runtime features stabilize.
- Item 9 is optional external integration and can be gated by user demand.

## 4) Detailed implementation map by roadmap item

## 4.1 Confidence scoring

Goal:

- Produce a post-generation confidence score that reflects document completeness and coherence.

Design:

- Expand existing readiness into a second pass that evaluates generated documents themselves.
- Use deterministic heuristics first (no extra model call required).

Implementation steps:

1. Backend scoring engine
- Add `src-tauri/src/docgen/confidence.rs`.
- Inputs: generated docs + readiness report + session metadata.
- Checks:
  - Required files present
  - Required headings present per file
  - `[TBD]` density per file
  - Cross-reference validity (`PROMPTS` references `SPEC` sections, etc.)
  - Command coverage for implementation phases
- Output:
  - `confidence_score` (0-100)
  - `confidence_factors` array with weighted contributions
  - `blocking_gaps` array

2. Persistence and commands
- Add confidence fields into `generation_metadata` JSON payload.
- Add command: `get_generation_confidence(session_id)`.

3. UI
- Show confidence badge in preview header and in `MODEL_HANDOFF.md`.
- Provide human-readable reasons for low confidence.

4. Tests
- Unit tests for each scoring factor in Rust.
- Fixture-based tests with synthetic docs.

Acceptance criteria:

- Confidence score is returned for every generation.
- Score decreases predictably when sections are missing.
- No additional network calls required.

Verification:

- `cargo test` (new confidence tests)
- `npm run build`

Risks:

- Overly strict heuristics causing false low scores.
- Mitigation: calibrated thresholds + explainability in score breakdown.

## 4.2 Planning coverage UI

Goal:

- Show real-time planning topic coverage while chatting.

Design:

- Reuse backend readiness categories; expose per-topic status to frontend.

Implementation steps:

1. Backend
- Add command: `get_planning_coverage(session_id)`.
- Return structured topic map:
  - `must_have`: each topic as `missing|partial|covered`
  - `should_have`: same
  - evidence snippets (message ids)

2. Frontend
- Add coverage panel in sidebar.
- Update status on message send completion and session switch.
- Include CTA buttons:
  - “Ask about missing topic”
  - “Forge anyway”

3. UX safeguards
- Keep panel read-only by default.
- Do not block chat actions; advisory only.

4. Tests
- Store tests for refresh timing and session isolation.
- Snapshot tests for panel states.

Acceptance criteria:

- Coverage updates within one chat turn.
- No state leakage between sessions.
- Must-have gaps visible before forge.

Verification:

- `npm run build`
- Frontend store tests for coverage refresh.

Risks:

- UI clutter.
- Mitigation: collapsible panel and concise labels.

## 4.3 Audit report fixes

This roadmap item includes three subprojects.

### 4.3.1 Structured `AppError` hardening

Goal:

- Ensure all backend errors crossing the Tauri boundary are typed, actionable, and consistent.

Implementation steps:

1. Enumerate all `AppError::Unknown` uses; replace with specific variants.
2. Ensure every command maps failures to user-safe messages and machine-safe codes.
3. Add context fields (`path`, `operation`, `session_id`) where relevant.
4. Standardize frontend normalization path for consistent toast/error rendering.

Acceptance criteria:

- No `Unknown` in normal runtime paths.
- Every error has stable `code`, `message`, `recoverable`, optional `action`.

Verification:

- Rust unit tests for error conversion.
- Smoke tests for representative error cases.

### 4.3.2 Transaction safety pass

Goal:

- Ensure multi-step DB/file operations are atomic and recoverable.

Implementation steps:

1. Review DB mutations in commands and DB module.
2. Wrap multi-write workflows in explicit transactions where missing.
3. Add rollback-safe behavior for mixed DB + filesystem paths.
4. Extend tests for concurrent operations and failure injection.

Acceptance criteria:

- No partial DB state after injected failures.
- Export and generation metadata stay consistent.

Verification:

- `cargo test` with added failure-path tests.

### 4.3.3 CSP hardening

Goal:

- Minimize CSP allowances while preserving app functionality.

Implementation steps:

1. Audit current `tauri.conf.json` CSP directives.
2. Remove unnecessary `'unsafe-*'` gradually.
3. Validate markdown rendering, code highlighting, plugins, and links after each change.
4. Add CSP regression checklist in docs.

Acceptance criteria:

- CSP is stricter than current baseline.
- No broken UI rendering/features.

Verification:

- `npm run build`
- Manual runtime checks in dev and packaged app.

Risks:

- CSP tightening can break rendering unexpectedly.
- Mitigation: tighten in small increments with immediate validation.

## 4.4 Windows and Linux builds

Goal:

- Ship installers/artifacts for macOS, Windows, and Linux.

Implementation steps:

1. Platform readiness audit
- Replace macOS assumptions in shell usage, paths, permissions.
- Gate platform-specific code via cfg flags.

2. CI matrix
- Add GitHub Actions matrix for:
  - macOS
  - ubuntu
  - windows
- Run:
  - frontend build
  - `cargo fmt --check`
  - `cargo clippy -- -D warnings`
  - `cargo test`
  - `tauri build` artifact creation

3. Packaging docs
- Add per-platform install and troubleshooting.

Acceptance criteria:

- Successful CI builds and release artifacts on all target OSes.
- Startup and core flows verified on each OS.

Verification:

- CI matrix green.
- Manual smoke test checklist per platform.

Risks:

- Filesystem and path edge cases differ by OS.
- Mitigation: centralized path utilities + platform test coverage.

## 4.5 Additional local model runtimes

Goal:

- Support local runtimes beyond default Ollama while keeping local-first posture.

Design:

- Introduce runtime adapter interface instead of hardcoding Ollama assumptions.

Implementation steps:

1. Backend abstraction
- Add `llm::provider` trait with capabilities:
  - health check
  - list models
  - stream chat
  - pull model (optional capability)

2. Providers
- Keep existing Ollama adapter.
- Add OpenAI-compatible local endpoint adapter (for LM Studio/Ollama-compatible servers).

3. Config
- Keep default provider as Ollama.
- Add explicit provider mode without enabling remote paid APIs by default.

4. Frontend
- Provider selection in advanced settings with clear local-only labels.

Acceptance criteria:

- Existing Ollama flow unchanged.
- Alternate local-compatible endpoint works for chat + generation.

Verification:

- Provider contract tests.
- End-to-end smoke on both adapters.

Risks:

- Streaming protocol differences across runtimes.
- Mitigation: adapter-specific parser modules + fallback handling.

## 4.6 Project templates for common app types

Goal:

- Speed planning by offering structured starting templates.

Implementation steps:

1. Template catalog
- Add local JSON/YAML templates:
  - SaaS web app
  - Tauri desktop app
  - CLI tool
  - API service

2. Session bootstrap
- Allow “Start from template” when creating session.
- Inject template context as initial system/user messages.

3. Template versioning
- Add version field + migration path for template schema.

Acceptance criteria:

- User can pick template and immediately get better-scaffolded planning dialogue.
- Templates are local files editable by maintainers.

Verification:

- Unit tests for template loading/validation.
- UI flow smoke tests.

Risks:

- Template drift over time.
- Mitigation: template ownership + quarterly review checklist.

## 4.7 Import existing codebases for refactoring plans

Goal:

- Allow users to point AuraForge at a repo and generate refactor plans grounded in real code.

Implementation steps:

1. Ingestion pipeline
- Add safe local repo scanner:
  - file tree summary
  - dependency files
  - selected source snippets
- Respect ignore patterns and size limits.

2. Privacy/safety
- Local-only processing.
- Explicit include/exclude controls.

3. Prompt integration
- Add “refactor mode” conversation seed with repository context.

4. Export
- Include imported-context summary in generated docs and manifest.

Acceptance criteria:

- User can import a local path and generate a refactor-focused plan.
- Large repos handled without UI freeze.

Verification:

- Performance tests on medium/large repo samples.
- Error-path tests for permission-denied and malformed files.

Risks:

- Context overload and latency.
- Mitigation: chunking + summarization caps + UI progress indicators.

## 4.8 Export to GitHub Issues / Linear integration

Goal:

- Optional push of generated phase tasks into external trackers.

Implementation steps:

1. Integration boundaries
- Keep local export as default and primary flow.
- Add explicit opt-in integration page.

2. Data mapping
- Map `PROMPTS.md` phases to issue tickets with labels/milestones.
- Preserve local source of truth (`manifest.json` + docs).

3. Credentials
- Secure token storage via OS keychain facilities.

4. Retry and idempotency
- Prevent duplicate issue creation on retries.

Acceptance criteria:

- User can export phases as issues with deterministic mapping.
- Integration failure never blocks local export.

Verification:

- Integration tests with mock APIs.
- Manual test against sandbox repos/workspaces.

Risks:

- Scope creep from third-party API differences.
- Mitigation: narrow v1 scope to create/update only.

## 4.9 Conversation branching

Goal:

- Let users explore alternate planning decisions without losing the main thread.

Design:

- Tree model: session has branches, each branch has message lineage.

Implementation steps:

1. DB schema
- Add `branches` table and branch-aware message linkage.

2. Backend commands
- Create branch, switch branch, list branches, merge summary.

3. Frontend UX
- Branch picker in sidebar.
- “Fork from message” action.

4. Doc generation
- Forge per selected branch only.
- Include branch id/name in manifest and handoff doc.

Acceptance criteria:

- Branching does not corrupt main timeline.
- Generation/export works independently per branch.

Verification:

- DB tests for lineage integrity.
- Store tests for branch switching and session isolation.

Risks:

- Complexity in message/event filtering.
- Mitigation: branch id propagated through every command/event payload.

## 5) Cross-cutting engineering standards for all items

Required for every roadmap implementation:

1. Code quality gates
- `npm run build`
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `cargo build`

2. Change discipline
- One logical change per commit.
- Conventional commits only.
- No broad refactors without explicit requirement.

3. Documentation updates
- Update `README.md` roadmap item status when complete.
- Update `AUDIT_REPORT.md` if risk posture changes.

4. Export stability contract
- Local folder export remains available and functional for all features.
- `manifest.json` remains backward-compatible and versioned when schema changes.

## 6) Definition of done (global)

A roadmap item is complete only when all are true:

1. Functional acceptance criteria met.
2. Automated checks pass.
3. Error paths handled and tested.
4. Documentation updated.
5. No regression in core planning -> forge -> save-to-folder flow.

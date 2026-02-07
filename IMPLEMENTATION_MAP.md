# AuraForge Implementation Plan (Roadmap Remainder)

Updated: 2026-02-07
Owner: Engineering
Status: Active roadmap complete (`confidence scoring`, `planning coverage UI`, `audit report fixes`, `additional local runtimes`, `project templates`, `codebase import`, `conversation branching`, `linux builds`, `release runbook/checklist hardening`). Deferred-only items remain below.

## 1) Fixed Product Constraints

These constraints are mandatory for all remaining work:

1. **Local-first model execution**
- Do not require paid remote APIs for core usage.
- Keep Ollama as default.
- New providers must support local/self-hosted endpoints.

2. **Web search remains enabled**
- Keep Tavily + existing search providers as part of grounding.
- Search is advisory input, not a hard dependency for forge completion.

3. **Local folder export remains the finish line**
- `Save to folder` remains the primary outcome for completed planning sessions.
- New features must not block or complicate this path.

4. **No external PM integrations in this phase**
- Notion/Linear/Jira/GitHub issue sync is explicitly deferred.
- Keep integration surfaces out of active implementation scope.

5. **Platform focus**
- Active runtime target: macOS + Linux.
- Windows work is documented as deferred unless priorities change.

## 2) Scope (What Is Left)

## 2.1 Active roadmap items

Active roadmap items are complete.

## 2.2 Explicitly deferred

1. Export to GitHub Issues / Linear integration
- Deferred to avoid integration complexity and credential surfaces.
- Re-evaluate only after core local planning loop is fully hardened.

2. Windows build support
- Deferred per current product direction.
- Keep code changes Windows-safe where easy, but do not spend delivery time on Windows packaging/debugging.

## 2.3 Decision log (2026-02-07)

1. Keep GitHub/Linear/Notion/Jira export integrations deferred for now.
- Rationale: maintain local-first reliability and avoid credential/security complexity during core hardening.

2. Keep Windows packaging deferred for this phase.
- Rationale: active runtime target is macOS + Linux, and Linux CI coverage is now in place.

## 3) Engineering Guardrails

These guardrails apply to every item below:

1. One logical change per commit.
2. No large dependency additions without written justification in PR body.
3. Preserve command and data backward compatibility where possible.
4. All cross-boundary errors must become user-safe `AppError` variants.
5. DB writes that represent one user action must be transactional.
6. Export writes remain atomic (stage + rename).

## 4) Verification Contract

Run the smallest meaningful checks per step, then full suite at milestone boundaries.

## 4.1 Narrow checks (per change)

- Frontend/type changes: `npx tsc --noEmit` and targeted `npm run build` when UI affected.
- Rust backend changes: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, targeted `cargo test <module_or_name>`.

## 4.2 Milestone checks (after each roadmap item)

- `npx tsc --noEmit`
- `npm run build`
- `cd src-tauri && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test && cargo build`

## 4.3 Release gate (after all active items)

- Full milestone checks
- `npm run tauri build` on macOS
- Linux bundle build in CI (or local Linux runner)

## 5) Execution Order (No Week Buckets)

Ordered to reduce risk and avoid backtracking:

1. Planning coverage UI
2. Audit report fixes (`AppError`, transaction safety, CSP)
3. Additional local model runtimes
4. Project templates
5. Import existing codebases
6. Conversation branching
7. Linux build/release hardening

## 6) Detailed Implementation Plan by Item

## 6.1 Planning Coverage UI

### Objective
Provide real-time visibility into conversation coverage (must-have vs should-have topics) before forging, without blocking normal chat flow.

### Why now
Confidence scoring is complete; coverage is the complementary pre-forge signal.

### Backend tasks

1. Add coverage command and result model.
- Files:
  - `src-tauri/src/types.rs`
  - `src-tauri/src/commands/mod.rs`
  - `src-tauri/src/lib.rs`
- Add:
  - `CoverageTopicStatus { topic, tier, status, evidence_message_ids }`
  - `CoverageReport { must_have, should_have, summary }`
  - `get_planning_coverage(session_id)` command.

2. Reuse readiness analysis internals.
- Files:
  - `src-tauri/src/docgen/quality.rs`
- Refactor readiness extraction so coverage and readiness share one source of truth.

3. Persist optional snapshot for export context.
- Files:
  - `src-tauri/src/db/mod.rs`
  - `src-tauri/src/types.rs`
- Add nullable `coverage_json` field in `generation_metadata` (migrated safely like `confidence_json`).

### Frontend tasks

1. Add coverage state and fetch action.
- File:
  - `src/stores/chatStore.ts`
- Add:
  - `planningCoverage: CoverageReport | null`
  - `getPlanningCoverage()` action.

2. Add collapsible sidebar section.
- Files:
  - `src/components/Sidebar.tsx`
  - `src/App.tsx`
  - `src/types.ts`
- Show status chips (`Missing`, `Partial`, `Covered`) grouped by tier.
- Add one-click prompt inject actions for missing must-have topics.

3. Keep UX non-blocking.
- Forge remains allowed.
- If must-haves are missing, show advisory copy only.

### Tests

- Rust:
  - Unit tests for topic status mapping in `quality.rs`.
- Frontend:
  - Store-level tests for refresh on session switch and message completion.

### Acceptance criteria

1. Coverage updates after each assistant response and on session switch.
2. No leakage between sessions.
3. Missing must-have topics are visible before forge.
4. No change to final export behavior.

### Rollback strategy

Feature-flag coverage panel rendering behind store availability; if regressions appear, disable UI while retaining backend command.

## 6.2 Audit Report Fixes

## 6.2.1 Structured `AppError`

### Objective
Remove generic runtime errors and standardize frontend-safe error payloads.

### Backend tasks

1. Expand `AppError` variants and metadata.
- File:
  - `src-tauri/src/error.rs`
- Add structured variants for:
  - invalid input
  - not found
  - permission denied
  - dependency unavailable
  - operation interrupted
  - serialization/parse failures

2. Replace broad `Unknown` usages.
- Files:
  - `src-tauri/src/commands/mod.rs`
  - `src-tauri/src/llm/mod.rs`
  - `src-tauri/src/config.rs`
  - `src-tauri/src/db/mod.rs`
- Ensure conversion keeps stable fields:
  - `code`
  - `message`
  - `recoverable`
  - `action_hint` (optional)

### Frontend tasks

1. Normalize display and fallback logic.
- Files:
  - `src/utils/errorMessages.ts`
  - `src/stores/chatStore.ts`
  - `src/components/Toast.tsx`

### Tests

- Rust unit tests for error conversions and JSON-safe serialization.
- Frontend tests for known error codes -> user message mapping.

### Acceptance criteria

1. No runtime `Unknown` errors in normal code paths.
2. User-facing errors are actionable and non-technical.
3. Logs retain technical context.

## 6.2.2 Transaction safety

### Objective
Guarantee action-level atomicity for multi-step DB operations.

### Tasks

1. Enumerate write workflows and wrap transaction boundaries.
- File:
  - `src-tauri/src/db/mod.rs`
- Workflows:
  - save message + metadata
  - forge output document replacement
  - generation metadata upsert + associated records
  - branch operations (future item dependency)

2. Ensure command-level consistency.
- File:
  - `src-tauri/src/commands/mod.rs`
- Avoid partial state when downstream step fails.

3. Add failure-injection tests.
- Use temporary DB and simulated mid-transaction errors.

### Acceptance criteria

1. No partial DB state from injected failures.
2. `save_to_folder` and metadata remain coherent with document state.

## 6.2.3 CSP hardening

### Objective
Reduce CSP permissions while maintaining current UI behavior.

### Tasks

1. Tighten CSP incrementally.
- File:
  - `src-tauri/tauri.conf.json`
- Targets:
  - remove/limit `'unsafe-eval'`
  - reduce `'unsafe-inline'` usage where feasible
  - constrain `connect-src` to required local/search endpoints

2. Validate renderer/runtime compatibility.
- Files impacted by behavior:
  - `src/components/DocumentPreview.tsx`
  - markdown/highlight stack

3. Add CSP troubleshooting section.
- Files:
  - `README.md`
  - `AUDIT_REPORT.md`

### Acceptance criteria

1. CSP is strictly tighter than current baseline.
2. No regression in markdown rendering, syntax highlighting, link opening, or search.

## 6.3 Additional Local Model Runtimes

### Objective
Support more local/self-hosted endpoints without introducing paid-cloud dependency requirements.

### Target support

1. Ollama (existing baseline)
2. OpenAI-compatible local endpoint profile (for LM Studio and compatible runtimes)

### Backend tasks

1. Add provider abstraction.
- Files:
  - `src-tauri/src/llm/mod.rs`
  - create `src-tauri/src/llm/providers/` modules
- Provider contract:
  - health check
  - list models
  - chat streaming
  - optional model pull

2. Extend config schema.
- Files:
  - `src-tauri/src/config.rs`
  - `src-tauri/src/types.rs`
- Add provider config blocks (base URL, headers, model naming rules).
- Keep defaults local and free.

3. Update command plumbing.
- File:
  - `src-tauri/src/commands/mod.rs`
- Route chat/generation through selected provider.

### Frontend tasks

1. Provider selector and endpoint settings.
- Files:
  - `src/components/SettingsPanel.tsx`
  - `src/stores/chatStore.ts`
  - `src/types.ts`
- Include safe validation and clear “local only” copy.

### Tests

- Provider contract tests with mocked streams.
- Config validation tests for each provider mode.

### Acceptance criteria

1. Existing Ollama workflow unchanged.
2. LM Studio-compatible endpoint can run chat + forge pipeline.
3. No mandatory API key requirement for core flows.

## 6.4 Project Templates

### Objective
Offer predefined planning starters to reduce ambiguity and improve plan quality.

### Tasks

1. Add template schema and loader.
- Files:
  - create `src-tauri/src/templates/mod.rs`
  - create `src-tauri/templates/*.json`
- Template fields:
  - `id`, `name`, `description`, `target_stack`, `seed_questions`, `seed_constraints`, `version`

2. Add commands.
- Files:
  - `src-tauri/src/commands/mod.rs`
  - `src-tauri/src/lib.rs`
- Commands:
  - `list_templates`
  - `start_session_from_template`

3. Frontend template picker.
- Files:
  - `src/components/OnboardingWizard.tsx`
  - `src/stores/chatStore.ts`

4. Template evolution contract.
- Add schema versioning/migration with strict validation.

### Tests

- Template parsing/validation tests.
- Session bootstrap tests for initial prompt injection.

### Acceptance criteria

1. User can choose template at session start.
2. Conversation starts with deterministic, template-guided context.
3. Local export output remains compatible.

## 6.5 Import Existing Codebases

### Objective
Allow local repo ingestion to generate refactor-oriented planning documents grounded in existing code.

### Tasks

1. Build safe ingestion pipeline.
- Files:
  - create `src-tauri/src/import/mod.rs`
  - extend `src-tauri/src/commands/mod.rs`
- Inputs:
  - selected root path
  - include/exclude globs
  - size/token budgets
- Outputs:
  - structured codebase summary persisted per session

2. Privacy and stability constraints.
- Enforce local-only processing.
- Hard caps for file size, file count, and aggregate bytes.
- Return actionable errors for permission denied and binary files.

3. Prompt integration.
- Include import summary in context for planning/refactor mode.
- Add explicit marker in `MODEL_HANDOFF.md` and `manifest.json`.

4. UI flow.
- Files:
  - `src/components/OnboardingWizard.tsx`
  - `src/components/Sidebar.tsx`
  - `src/stores/chatStore.ts`
- Add import chooser, scan progress, and cancellation.

### Tests

- Import filter tests (`.gitignore`, globs, large file skips).
- Performance test on medium-size fixture repo.
- UI state tests for cancellation and failure recovery.

### Acceptance criteria

1. User can import a local codebase without freezing UI.
2. Forge output references actual imported context.
3. Export remains local and deterministic.

## 6.6 Conversation Branching

### Objective
Support alternate planning paths without losing main thread history.

### Data model

1. Session contains many branches.
2. Branch contains message lineage.
3. Forge runs against selected branch only.

### Tasks

1. DB schema migration.
- File:
  - `src-tauri/src/db/mod.rs`
- Add:
  - `branches` table
  - `messages.branch_id`
  - branch metadata (`name`, `created_from_message_id`, timestamps)

2. Branch commands.
- Files:
  - `src-tauri/src/commands/mod.rs`
  - `src-tauri/src/lib.rs`
- Commands:
  - `create_branch`
  - `list_branches`
  - `switch_branch`
  - `rename_branch`

3. Frontend branch UX.
- Files:
  - `src/components/Sidebar.tsx`
  - `src/stores/chatStore.ts`
  - `src/types.ts`
- Add:
  - branch selector
  - “fork from message” action in `ChatMessage`

4. Generation/export isolation.
- Ensure documents/metadata are branch-scoped.
- Include `branch_id` in manifest metadata.

### Tests

- DB lineage integrity tests.
- Store tests for rapid branch switching.
- Generation tests confirming branch isolation.

### Acceptance criteria

1. Branches cannot corrupt each other’s timelines.
2. User can fork, switch, and forge branch-specific outputs reliably.

## 6.7 Linux Build and Distribution Hardening

### Objective
Make Linux a first-class supported desktop target.

### Tasks

1. CI Linux pipeline.
- Add/expand workflow under `.github/workflows/`:
  - frontend build
  - Rust checks
  - Tauri Linux bundle artifact

2. Runtime smoke checklist.
- Validate:
  - startup
  - chat stream
  - forge
  - save-to-folder
  - settings persistence

3. Packaging docs.
- Update `README.md` install section with Linux artifacts and troubleshooting.

### Acceptance criteria

1. Linux CI build is green for release branch.
2. Linux smoke checklist passes.

### Note on Windows
Windows remains deferred; keep changes portability-safe but do not block roadmap completion on Windows packaging.

## 7) Commit Plan (One Logical Change Per Commit)

Recommended commit sequence:

1. `feat(coverage): add planning coverage backend and sidebar UI`
2. `fix(error): replace unknown runtime failures with typed AppError variants`
3. `fix(db): enforce transaction boundaries for multi-step writes`
4. `chore(security): tighten tauri csp directives`
5. `feat(llm): add local openai-compatible provider adapter`
6. `feat(templates): add template-based session bootstrap`
7. `feat(import): add local codebase ingestion for refactor mode`
8. `feat(branching): add branch-aware conversation flows`
9. `build(linux): add linux packaging checks and docs`

## 8) Definition of Done

A roadmap item is complete only when all conditions are true:

1. Functional behavior matches acceptance criteria.
2. Narrow checks pass for each commit.
3. Milestone checks pass after item completion.
4. Error paths are handled and tested.
5. `README.md` roadmap status is updated.
6. `Save to folder` path remains stable and verified.

## 9) Immediate Next Action

Run release gate checks from `RELEASE_CHECKLIST.md` for each release candidate, then execute deferred items only when product scope changes.

## 10) Next Roadmap Block (Post-Merge)

### Block A: Model-Agnostic Export Interop Hardening

Objective:
- Ensure exported planning packs can be consumed and validated reliably by any coding model/tooling pipeline.

Scope:
1. Expand `manifest.json` to include deterministic file metadata (`bytes`, `lines`, `sha256`) per document.
2. Make manifest file ordering deterministic across exports.
3. Add regression tests for ordering and checksum generation.
4. Update docs so pack consumers understand validation fields.

Status:
- Completed 2026-02-07.

### Block H: Test Architecture Expansion (Phase 6)

Objective:
- Ensure new frontend store tests are enforced in CI rather than local-only.

Scope:
1. Extend Linux CI frontend checks to run `npm run test` in addition to typecheck/build.
2. Keep Rust fmt/clippy/test/build gates unchanged.

Status:
- Completed 2026-02-07.

### Block I: Importer Performance and Scalability (Phase 7)

Objective:
- Reduce memory and I/O overhead during repository import scans while preserving output behavior.

Scope:
1. Replace full-file reads with capped prefix reads (bounded by import byte budget).
2. Track imported bytes using actual read size for better budget accounting.
3. Add regression tests for prefix-read behavior on large and small files.

Status:
- Completed 2026-02-07.

### Block F: Search Reliability and Fallback Hardening (Phase 4)

Objective:
- Keep web search non-blocking and resilient under provider failures while reducing duplicate query churn.

Scope:
1. Add TTL-based in-memory cache for repeated `(provider, query)` lookups.
2. Keep Tavily fallback to DuckDuckGo and add SearXNG fallback to DuckDuckGo on failure.
3. Normalize provider/query cache keys for stable reuse.
4. Add regression test for cache-key normalization behavior.

Status:
- Completed 2026-02-07.

### Block G: UI State Gating for Long-Running Operations (Phase 5)

Objective:
- Prevent overlapping operations that can produce stuck or contradictory UI states.

Scope:
1. Block `sendMessage` while document generation is active, with explicit toast feedback.
2. Block `generateDocuments` while a stream response is active, with explicit toast feedback.
3. Block retry flow while generation is active.
4. Add store-level race tests for these gating rules.

Status:
- Completed 2026-02-07.

### Block E: Local Provider Contract Hardening (Phase 3)

Objective:
- Normalize local provider behavior and reduce provider-branching drift across runtime paths.

Scope:
1. Introduce explicit provider kind resolution (`ollama`, `openai_compatible` aliases).
2. Route model listing, connection checks, model checks, streaming, and generation through normalized provider kind handling.
3. Keep pull-model enforcement explicit for Ollama-only behavior.
4. Add regression tests for provider parsing and unsupported-provider validation.

Status:
- Completed 2026-02-07.

### Block D: Filesystem and Persistence Robustness (Phase 2)

Objective:
- Prevent path traversal/partial-write edge cases and improve config durability guarantees.

Scope:
1. Add destination preflight checks for export folder existence/type/readonly state.
2. Validate export filenames to reject nested/absolute/blank paths before writing files.
3. Continue atomic export behavior with sanitized write inputs and deterministic manifest output.
4. Harden config persistence using atomic write helper with parent-directory fsync and temp cleanup on rename failure.
5. Add regression tests for export filename validation and atomic config replacement.

Status:
- Completed 2026-02-07.

### Block C: Security and Runtime Hardening (Phase 1)

Objective:
- Reduce runtime attack surface and remove packaging warnings without changing user workflow.

Scope:
1. Tighten Tauri CSP by removing unneeded sources (`https:` image source wildcard, websocket connect sources not used by app flow).
2. Restrict default capability permissions by removing broad plugin defaults and keeping explicit allow-list entries.
3. Fix bundle identifier format to avoid `.app` suffix warning during macOS packaging.
4. Re-run full build/test/release gate verification.

Status:
- Completed 2026-02-07.

### Block B: Async Race Integration Coverage (Frontend/Tauri)

Objective:
- Add targeted integration-style tests around frontend store async race paths that previously caused stale-state and stuck-stream regressions.

Scope:
1. Add frontend test harness for store-level async/event tests.
2. Cover `loadDocuments` session-switch race (must not apply stale results).
3. Cover `cancelResponse` safety timeout behavior across session switches.
4. Cover `stream:done` timeout clearing to prevent false stream resets.

Status:
- Completed 2026-02-07.

### Block J: Release Gate and Packaging Operations (Phase 8)

Objective:
- Make release readiness repeatable with explicit verification and smoke criteria for macOS/Linux.

Scope:
1. Add a release checklist with branch hygiene, frontend/rust verification gates, packaging gate, smoke checks, and policy checks.
2. Include security/config spot-check commands in the same checklist.
3. Require audit/map sync before release signoff.

Status:
- Completed 2026-02-07 (`RELEASE_CHECKLIST.md`).

### Block K: Runtime Runbook and Documentation Closure (Phase 9)

Objective:
- Provide operational troubleshooting guidance and align top-level docs with the local-first/model-agnostic posture.

Scope:
1. Add runtime runbook covering dependency checks, smoke flows, and failure playbooks.
2. Update README to link runbook/checklist and remove stale quality signaling.
3. Sync implementation map and audit report with final roadmap closure.

Status:
- Completed 2026-02-07 (`RUNBOOK.md`, README/AUDIT updates).

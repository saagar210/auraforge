# AuraForge Code Audit Report

## 1. Executive Summary
- **Overall code quality rating**: 6/10
- **Critical issues found**: 0
- **Moderate issues found**: 13
- **Minor issues found**: 8
- **Top 3 strengths**:
  1. Clear separation between Rust backend and React frontend, with a single IPC layer.
  2. Consistent visual system implementation matching the Forge aesthetic and design tokens.
  3. Solid baseline SQLite schema and unit tests for core DB operations.
- **Top 3 areas needing improvement**:
  1. Spec compliance gaps (commands, onboarding flow, search behavior, error model).
  2. Robustness/data integrity (panic-prone unwraps, missing transactions, non-atomic config writes).
  3. Security posture (CSP disabled, broad capabilities, plaintext API keys).

## 2. Critical Issues (Must Fix)
None found.

## 3. Moderate Issues (Should Fix)

1) **Panic-prone `unwrap/expect` in production paths**
- **File**: `src-tauri/src/config.rs`
- **Line(s)**: 35-37
- **Issue**: `dirs::home_dir().expect(...)` will panic if the home directory is unavailable.
- **Impact**: App can crash on startup in edge environments or sandboxed contexts.
- **Fix**: Return a structured error and surface it in onboarding/health status instead of panicking.

- **File**: `src-tauri/src/lib.rs`
- **Line(s)**: 21, 30, 174
- **Issue**: `expect(...)` on config load, DB creation, and app run will panic on failure.
- **Impact**: Startup crashes rather than user-facing errors or recovery.
- **Fix**: Replace `expect` with error propagation; show setup/error UI for recoverable issues.

- **File**: `src-tauri/src/db/mod.rs`
- **Line(s)**: 29, 70, 77, 90, 110, 120, 139, 170, 200, 220, 236, 263, 282, 294, 303, 314, 327
- **Issue**: `Mutex::lock().unwrap()` will panic on poisoned mutexes.
- **Impact**: Any panic in DB operations can cascade into persistent crashes.
- **Fix**: Handle poisoned locks (`lock().map_err(...)`) and surface errors.

- **File**: `src-tauri/src/docgen/mod.rs`
- **Line(s)**: 27
- **Issue**: `state.config.lock().unwrap()` can panic.
- **Impact**: Document generation can crash on poisoned config mutex.
- **Fix**: Return a recoverable error instead of panicking.

- **File**: `src-tauri/src/commands/mod.rs`
- **Line(s)**: 48, 90, 98, 112, 158, 168, 319
- **Issue**: `state.config.lock().unwrap()` can panic in multiple commands.
- **Impact**: IPC commands can crash the app instead of returning errors.
- **Fix**: Propagate lock errors and translate to user-facing messages.

2) **Config validation and atomicity are missing**
- **File**: `src-tauri/src/config.rs`
- **Line(s)**: 48-86
- **Issue**: Config writes are non-atomic and the loader only recreates defaults on parse failure; no field validation.
- **Impact**: Partial writes can corrupt config; invalid fields silently reset to defaults, losing user settings.
- **Fix**: Write to a temp file and `rename` atomically; implement explicit validation (provider values, URLs) and return detailed errors.

- **File**: `src-tauri/src/commands/mod.rs`
- **Line(s)**: 52-85
- **Issue**: `config_valid` is always `true` and health check never validates config.
- **Impact**: UI cannot reflect misconfiguration (e.g., invalid URLs, missing API keys).
- **Fix**: Validate config during health checks and populate structured errors.

3) **Structured error model (AppError) is missing**
- **File**: `src-tauri/src/commands/mod.rs`
- **Line(s)**: 46-127, 213-567
- **Issue**: Commands return `Result<_, String>` instead of structured error types described in SPEC.
- **Impact**: Frontend can’t distinguish error classes or offer targeted recovery actions.
- **Fix**: Implement `AppError` + error codes and return structured responses from all commands.

4) **Document generation can cause data loss and inconsistent state**
- **File**: `src-tauri/src/docgen/mod.rs`
- **Line(s)**: 29-109
- **Issue**: Existing documents are deleted before new generation begins; generation isn’t transactional.
- **Impact**: If generation fails mid-run, the user loses previously cached documents.
- **Fix**: Generate into a temp table or in-memory buffer, then replace in a transaction upon success.

- **File**: `src-tauri/src/db/mod.rs`
- **Line(s)**: 163-181, 230-287
- **Issue**: Multi-step DB writes aren’t wrapped in transactions.
- **Impact**: Partial writes can leave `messages` and `sessions.updated_at` out of sync or docs partially written.
- **Fix**: Use `Connection::transaction()` for multi-statement operations.

5) **Search provider handling violates spec and user intent**
- **File**: `src-tauri/src/search/mod.rs`
- **Line(s)**: 34-57
- **Issue**: Unknown providers (including `none` and `searxng`) fall back to DuckDuckGo.
- **Impact**: Users opting out of search still trigger external requests; SearXNG is not supported despite UI/config.
- **Fix**: Respect `provider: none` and implement SearXNG client or disable the option in UI.

- **File**: `src/components/SettingsPanel.tsx`
- **Line(s)**: 287-314
- **Issue**: UI exposes SearXNG even though no backend implementation exists.
- **Impact**: Users can choose a provider that never works.
- **Fix**: Add SearXNG backend or remove the option until implemented.

6) **Search trigger patterns are incomplete vs spec**
- **File**: `src-tauri/src/search/trigger.rs`
- **Line(s)**: 46-63
- **Issue**: Missing wildcard patterns like “is * maintained”, “in 2026”, “does * work with”.
- **Impact**: Proactive search fails to trigger for common queries, reducing response quality.
- **Fix**: Align trigger patterns with SPEC and add tests for the missing cases.

7) **No cancellation path for streaming responses**
- **File**: `src-tauri/src/llm/mod.rs`
- **Line(s)**: 273-363
- **Issue**: Streaming loop has no cancellation signal; only model pull is cancellable.
- **Impact**: Users cannot stop long or incorrect responses; UI may remain stuck.
- **Fix**: Add a cancellation flag per session and expose `cancel_response` IPC command.

8) **Chat messages are not rendered as Markdown**
- **File**: `src/components/ChatMessage.tsx`
- **Line(s)**: 36-38
- **Issue**: Message content renders as plain text, ignoring markdown syntax.
- **Impact**: Spec-required formatting (code blocks, links, emphasis) is missing in chat.
- **Fix**: Use `react-markdown` with safe defaults for assistant messages.

9) **Onboarding flow diverges from spec**
- **File**: `src/App.tsx`
- **Line(s)**: 74-78
- **Issue**: Onboarding is only shown before `wizardCompleted`; later health failures do not reopen it.
- **Impact**: Users can be stranded without guidance if Ollama stops working after initial setup.
- **Fix**: Re-show onboarding when health check fails, regardless of completion state.

- **File**: `src/components/OnboardingWizard.tsx`
- **Line(s)**: 67-106
- **Issue**: Health polling only during install-ollama step; no Tavily/web-search step.
- **Impact**: Spec-required real-time checks and search configuration are missing.
- **Fix**: Poll health throughout onboarding and add the optional web search configuration step.

10) **Streaming events are not session-scoped**
- **File**: `src/stores/chatStore.ts`
- **Line(s)**: 595-606
- **Issue**: `stream:chunk` updates global streaming content without checking session.
- **Impact**: Switching sessions mid-stream can leak content into another session.
- **Fix**: Include `session_id` in stream events and gate updates on the active session.

11) **Export UX mismatch: default path ignored + folder overwrite**
- **File**: `src/App.tsx`
- **Line(s)**: 109-114
- **Issue**: File picker uses a hardcoded default path instead of config value.
- **Impact**: User preferences for save location are ignored.
- **Fix**: Load `output.default_save_path` from config and use it.

- **File**: `src-tauri/src/commands/mod.rs`
- **Line(s)**: 503-515
- **Issue**: `create_dir_all` silently reuses existing folder.
- **Impact**: Existing exported plans can be overwritten without warning.
- **Fix**: Detect existing folder and prompt to rename/overwrite per spec.

12) **Menu items are not wired to actions**
- **File**: `src-tauri/src/lib.rs`
- **Line(s)**: 56-117
- **Issue**: Menu defines `save_to_folder`, `rename_session`, `delete_session` actions.
- **Impact**: Clicking these menu items does nothing.
- **Fix**: Handle these actions in the frontend menu listener.

- **File**: `src/stores/chatStore.ts`
- **Line(s)**: 650-666
- **Issue**: Menu handler only handles new_session, toggle_sidebar, toggle_preview, help_panel.
- **Impact**: Missing menu behavior for defined actions.
- **Fix**: Add cases for save/rename/delete.

13) **Document links may open inside the app instead of external browser**
- **File**: `src/components/DocumentPreview.tsx`
- **Line(s)**: 248-255
- **Issue**: Links use `<a target="_blank">` rather than Tauri shell open.
- **Impact**: Links may open in a new WebView (security/UX risk) instead of external browser.
- **Fix**: Intercept clicks and call `@tauri-apps/plugin-shell` `open`.

## 4. Minor Issues (Nice to Fix)

1) **Message metadata type mismatch**
- **File**: `src-tauri/src/types.rs`
- **Line(s)**: 14-20
- **Issue**: Backend sends `metadata` as `Option<String>` (JSON string).
- **Impact**: Frontend expects structured `MessageMetadata`, risking runtime type mismatch.
- **Fix**: Serialize metadata as structured JSON and update TypeScript typing accordingly.

- **File**: `src/types.ts`
- **Line(s)**: 16-22
- **Issue**: `Message.metadata` is typed as `MessageMetadata | null`.
- **Impact**: Any direct usage will break if backend returns a string.
- **Fix**: Align TS type to match backend or update backend to send structured objects.

2) **Stale document check compares timestamps as strings**
- **File**: `src-tauri/src/commands/mod.rs`
- **Line(s)**: 474-476
- **Issue**: `mt > dt` compares timestamp strings, not parsed times.
- **Impact**: Potential incorrect staleness detection if formats change.
- **Fix**: Compare as `chrono::DateTime` or use SQL to compare.

3) **Disk space check parsing is brittle**
- **File**: `src-tauri/src/commands/mod.rs`
- **Line(s)**: 189-200
- **Issue**: Assumes `df` output format and uses `unwrap_or(0)`.
- **Impact**: Can return false low disk space without surfacing errors.
- **Fix**: Handle parse errors explicitly and return a clear error to the UI.

4) **Accessibility gaps in modals and tooltips**
- **File**: `src/components/SettingsPanel.tsx`
- **Line(s)**: 105-120
- **Issue**: No focus trap; keyboard focus can escape modal.
- **Impact**: Reduced accessibility for keyboard users.
- **Fix**: Add focus trap and initial focus management.

- **File**: `src/components/HelpPanel.tsx`
- **Line(s)**: 11-20
- **Issue**: Missing `aria-modal` and focus management.
- **Impact**: Screen readers may not treat it as a modal.
- **Fix**: Add `aria-modal="true"` and focus trap.

- **File**: `src/components/OnboardingWizard.tsx`
- **Line(s)**: 135-145
- **Issue**: No focus trap in onboarding modal.
- **Impact**: Focus can move behind the modal.
- **Fix**: Implement focus trapping.

- **File**: `src/components/InfoTooltip.tsx`
- **Line(s)**: 9-12
- **Issue**: Tooltip is hover-only and not keyboard accessible.
- **Impact**: Users navigating with keyboard/screen readers may miss content.
- **Fix**: Add focus/aria-triggered tooltip behavior.

5) **Hardcoded colors bypass design tokens**
- **File**: `src/components/ChatMessage.tsx`
- **Line(s)**: 20-25
- **Issue**: Inline shadows use hardcoded RGBA values.
- **Impact**: Makes theme changes harder and inconsistent with design system.
- **Fix**: Move to CSS variables or Tailwind tokens.

- **File**: `src/components/ThinkingIndicator.tsx`
- **Line(s)**: 11-15
- **Issue**: Flame gradient uses hardcoded hex colors.
- **Impact**: Diverges from centralized theme tokens.
- **Fix**: Use CSS variables for gradient colors.

- **File**: `src/components/ThermalBackground.tsx`
- **Line(s)**: 11-32
- **Issue**: Inline gradients use hardcoded RGBA values.
- **Impact**: Inconsistent theming and harder maintenance.
- **Fix**: Replace with tokenized CSS variables.

6) **Unused dependencies**
- **File**: `package.json`
- **Line(s)**: 15
- **Issue**: `@tauri-apps/plugin-fs` is declared but unused in frontend.
- **Impact**: Larger install footprint and potential security surface.
- **Fix**: Remove or use if needed.

- **File**: `src-tauri/Cargo.toml`
- **Line(s)**: 19, 44, 50
- **Issue**: `tauri-plugin-fs`, `anyhow`, and `tokio-stream` appear unused.
- **Impact**: Extra dependencies increase build size and attack surface.
- **Fix**: Remove or justify use.

7) **Repository hygiene: committed artifacts and missing assets**
- **File**: `.gitignore`
- **Line(s)**: 1-20
- **Issue**: Build artifacts (`dist/`, `src-tauri/gen/`, `.DS_Store`, `tsconfig.tsbuildinfo`) are present despite being ignored; DMG is tracked.
- **Impact**: Repo bloat and noisy diffs.
- **Fix**: Remove committed artifacts and enforce ignore rules.

- **File**: `README.md`
- **Line(s)**: 1-6, 70-76
- **Issue**: References to `screenshots/hero.png` and other images, but `screenshots/` only contains `.gitkeep`.
- **Impact**: Broken images in documentation.
- **Fix**: Add the missing images or remove references.

8) **Cmd+? handler may not fire reliably**
- **File**: `src/App.tsx`
- **Line(s)**: 175-178
- **Issue**: The key handler checks for `e.key === "/"` which may not match `?` on some layouts.
- **Impact**: Help shortcut may be inconsistent.
- **Fix**: Check for both `/` and `?`, or rely solely on the menu shortcut.

## 5. Spec Compliance Matrix

| Feature | Spec Section | Status | Notes |
|---|---|---|---|
| TypeScript interfaces | Tauri Commands → TypeScript Types | ⚠️ | `Message.metadata` shape mismatch; missing structured health errors. |
| Rust command signatures | Tauri Commands | ⚠️ | Missing `cancel_response` and `web_search`; `update_config` signature differs. |
| Database schema | Database Schema | ✅ | Tables + indexes match spec; migrations missing. |
| Config file structure | Configuration | ⚠️ | Shape matches, but validation + atomic writes missing. |
| Streaming architecture | Tauri Event System | ⚠️ | `stream:search` event not used; `stream:done` unused on frontend. |
| Error handling matrix | Error Handling | ❌ | No `AppError` model; errors are strings. |
| Document generation prompts | Document Generation Prompts | ✅ | Prompts closely match spec. |
| Web search trigger patterns | Search Triggers | ⚠️ | Missing wildcard patterns and “in 2026” freshness terms. |
| State machines | State Machines | ⚠️ | Cancel response path not implemented; stale docs logic partial. |
| Keyboard shortcuts | Keyboard Shortcuts | ⚠️ | Cmd+? handling inconsistent; menu actions incomplete. |
| First-run experience | Onboarding | ⚠️ | Lacks web search configuration step + continuous polling. |
| Success criteria | Success Criteria | ⚠️ | No automated tests or instrumentation to verify. |

## 6. Security Findings
- **CSP disabled**: `src-tauri/tauri.conf.json:25-26` sets `csp: null`, removing browser-level protections.
- **Broad capabilities**: `src-tauri/capabilities/default.json:6-13` grants `fs:default` and `shell:default` without narrowing scope.
- **Plaintext API keys**: `src-tauri/src/config.rs:16-22, 81-85` stores Tavily API key in plaintext YAML.
- **External link handling**: `src/components/DocumentPreview.tsx:248-255` opens links via `<a>` instead of shell open, risking in-app navigation.

## 7. Performance Observations
- **Full-history reload after each send**: `src/stores/chatStore.ts:186-188, 382-387` re-fetches all messages; will scale poorly with long sessions.
- **Blocking DB/FS in async commands**: `src-tauri/src/commands/mod.rs:276-518` uses sync IO inside async commands; could block the runtime under load.
- **No pagination for messages/docs**: `src-tauri/src/db/mod.rs:199-217, 259-278` always loads full sets.
- **Syntax highlighting cost**: `src/components/DocumentPreview.tsx:136-158` uses Prism on large docs; consider virtualization for big outputs.

## 8. Test Coverage Assessment
- **Existing tests**:
  - `src-tauri/src/db/mod.rs` (unit tests for DB operations)
  - `src-tauri/src/search/trigger.rs` (unit tests for trigger logic)
- **Missing**:
  - No frontend component tests (chat, onboarding, settings, preview)
  - No IPC integration tests
  - No end-to-end tests
- **Recommendation**: Add Vitest/RTL for UI, and at least one Tauri integration test for send_message + generate_documents.

## 9. Architecture Notes
- **File inventory**:
  - Frontend: `src/` (React components, Zustand store, Tailwind styles)
  - Backend: `src-tauri/src/` (commands, db, llm, search, docgen)
  - Config/spec: `SPEC.md`, `DESIGN.md`
  - Build artifacts in repo: `dist/`, `distribution/`, `src-tauri/gen/`, `.DS_Store`, `tsconfig.tsbuildinfo`
- **Structure**: Clean separation between UI and backend, but `chatStore` acts as a “god store” (sessions, messages, config, onboarding, export) and would benefit from module split.
- **Orphan/misplaced**: Generated assets and OS files are tracked despite `.gitignore` entries.

## 10. Recommendations (Prioritized)
1. Implement structured error handling (`AppError`), replace panic-prone unwraps, and wire health/config validation.
2. Fix document generation data integrity by using transactions and avoiding delete-before-generate.
3. Implement cancelable streaming (`cancel_response`) and session-scoped stream events.
4. Align search providers and triggers with spec; add SearXNG or remove the option.
5. Bring onboarding flow back in line with spec (continuous health polling, web search step, re-open on failure).
6. Render markdown in chat messages and ensure external links open via shell.
7. Tighten security: enable CSP and restrict Tauri capabilities.
8. Add baseline frontend + IPC integration tests.

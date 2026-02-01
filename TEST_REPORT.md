# AuraForge Test Report

**Date:** 2026-01-31
**Commit baseline:** b5d1f6b (pre-test)
**Platform:** macOS (Darwin 25.2.0, Apple Silicon)

---

## Bugs Found & Fixed

### BUG-1: `AppConfig::default()` search provider mismatch (CRITICAL)

**File:** `src-tauri/src/types.rs:120`
**Severity:** Critical
**Impact:** When config parsing fails, the fallback `AppConfig::default()` used `provider: "tavily"` with an empty API key. This would also fail validation, meaning any config corruption resulted in an unrecoverable state where the app was permanently stuck at onboarding.

**Root cause:** The `DEFAULT_CONFIG_YAML` in `config.rs` was previously fixed to use `duckduckgo`, but the Rust `Default` impl in `types.rs` was not updated to match.

**Fix:** Changed `Default::default()` search provider from `"tavily"` to `"duckduckgo"`.

### BUG-2: `check_model` prefix match too permissive (HIGH)

**File:** `src-tauri/src/llm/mod.rs:281-285`
**Severity:** High
**Impact:** Health check reported "model available" when the configured model tag didn't match any installed model. For example, config says `qwen3-coder:30b-a3b-instruct-q4_K_M` but only `qwen3-coder:latest` is installed. The prefix match `"qwen3-coder:"` matched both, so health check passed. But Ollama's chat API requires the exact model name, so all chat requests failed with 404.

**Root cause:** The prefix match logic checked if any installed model shared the same base name (before the colon), regardless of the tag portion.

**Fix:** Prefix match now only applies when the configured model name has no colon (i.e., user specified just the base name like `"qwen3-coder"` without a tag). When a specific tag is configured (contains `:`), only exact match is accepted.

### BUG-3: SettingsPanel silent save failure (MEDIUM)

**File:** `src/components/SettingsPanel.tsx:72-103`
**Severity:** Medium
**Impact:** When saving settings with invalid values (e.g., Tavily provider with empty API key), the backend rejected the config but the Settings dialog closed silently. The user believed the save succeeded.

**Root cause:** `handleSave` called `updateConfig()` but didn't check the return value. The dialog always closed after the call regardless of success/failure.

**Fix:** Check return value from `updateConfig()`. If null (save failed), display an error message in the dialog footer instead of closing.

### BUG-4: "Re-run Setup" button non-functional (MEDIUM)

**File:** `src/App.tsx:95-100`, `src/stores/chatStore.ts`, `src/components/SettingsPanel.tsx:105-109`
**Severity:** Medium
**Impact:** The "Re-run Setup" button in Settings reset wizard state flags but the onboarding wizard never appeared. Users couldn't re-run the setup process.

**Root cause:** `showOnboarding` was purely health-check driven (`!ollama_connected || !model_available || !database_ok || !config_valid`). It didn't consider wizard completion state at all. Setting `wizardCompleted: false` had no effect on whether the wizard displayed.

**Fix:**
- Added `preferencesLoaded` boolean to the store to prevent flash of wizard before preferences load
- Changed `showOnboarding` condition to also check `!wizardCompleted`, gated by `preferencesLoaded`
- Now the wizard shows both when health fails AND when the user explicitly re-runs setup

---

## Test Results by Suite

### Suite 1: Application Lifecycle

| Test | Status | Notes |
|------|--------|-------|
| Fresh start creates `~/.auraforge/` | PASS | Config + DB created with correct defaults |
| Config defaults are valid (duckduckgo) | PASS | Fixed by BUG-1 |
| Database schema (5 tables, WAL, indexes) | PASS | sessions, messages, documents, preferences, schema_migrations |
| Foreign keys enabled | PASS | Enabled per-connection in Rust (not visible from external sqlite3) |
| Onboarding wizard shows on first launch | PASS | Now correctly shows when wizard_completed is not set (BUG-4 fix) |
| Graceful shutdown | PASS | No crash, DB WAL checkpoint on close |

### Suite 2: Session Management

| Test | Status | Notes |
|------|--------|-------|
| Create session | PASS | Default name "New Project", UUID-based ID |
| Auto-name on first message | PASS | Truncates to 60 chars with "..." |
| Rename session | PASS | Updates name and updated_at |
| Delete session with cascade | PASS | Messages and documents deleted via FK cascade |
| Session ordering by updated_at | PASS | Most recently updated first |
| Multi-session isolation | PASS | Messages scoped to session_id |

### Suite 3: Chat Functionality

| Test | Status | Notes |
|------|--------|-------|
| Send message flow | PASS | User msg saved, streamed response, assistant msg saved |
| Session-scoped streaming events | PASS | Events filtered by session_id |
| Cancel response | PASS | AtomicBool flag checked per chunk |
| Retry last message | PASS | `retry: true` reuses last user msg |
| Error display with friendly messages | PASS | `friendlyError()` maps error codes to user language |
| Stream buffer with RAF batching | PASS | Prevents excessive re-renders |

### Suite 4: Web Search

| Test | Status | Notes |
|------|--------|-------|
| Trigger detection (36 test cases) | PASS | All 36 unit tests pass |
| DuckDuckGo HTML scraping | PASS | Parses results, extracts URLs from DDG redirects |
| Tavily fallback to DuckDuckGo | PASS | On API key/rate limit errors |
| SearXNG provider | PASS | URL-based, validated in config |
| Search context injection | PASS | Results injected as system message |
| Proactive search toggle | PASS | Respects `config.search.proactive` |

### Suite 5: Document Generation

| Test | Status | Notes |
|------|--------|-------|
| 5-document pipeline (SPEC, CLAUDE, PROMPTS, README, CONVERSATION) | PASS | Sequential generation with progress events |
| CONVERSATION.md from data (no LLM) | PASS | Generated directly from session/messages |
| Heading validation with retry | PASS | Retries once if output doesn't start with `#` |
| Atomic document replacement | PASS | Transaction: delete old + insert new |
| Staleness detection | PASS | Compares latest message time vs document time |
| Forge button gating (3+ exchanges) | PASS | Requires `userMessageCount >= 3 && assistantMessageCount >= 3` |

### Suite 6: Save & Export

| Test | Status | Notes |
|------|--------|-------|
| Save to folder | PASS | Creates `{sanitized-name}-plan/` subdirectory |
| Folder name sanitization | PASS | Alphanumeric + hyphens, lowercase, max 60 chars |
| Folder exists check | PASS | Returns `FolderExists` error, doesn't overwrite |
| Permission denied handling | PASS | Specific error message |
| Disk space check | PASS | `df -k /` parsed, threshold 20 GB |
| Dialog integration | PASS | Uses `@tauri-apps/plugin-dialog` for folder picker |

### Suite 7: UI/UX Polish

| Test | Status | Notes |
|------|--------|-------|
| Message windowing (120 initial, 80 step) | PASS | Virtual scrolling with older message loading |
| Auto-scroll to bottom | PASS | Respects `isAtBottomRef` for user scroll position |
| Scroll-to-top loads older messages | PASS | Triggers at 48px threshold |
| Document preview tab system | PASS | Ordered by TAB_ORDER, copy/save buttons |
| Markdown rendering with syntax highlighting | PASS | react-markdown + remark-gfm + prism |
| Link safety check (https/mailto only) | PASS | Non-safe URLs prevented |
| Toast notifications with action | PASS | "Open Folder" action on save success |
| Sidebar collapse | PASS | Fully hidden when collapsed |

### Suite 8: Keyboard Shortcuts

| Test | Status | Notes |
|------|--------|-------|
| ⌘+Enter send message | PASS | `e.metaKey && e.key === "Enter"` |
| ⌘+N new project | PASS | |
| ⌘+G generate documents | PASS | Only when `canForge` is true |
| ⌘+S save to folder | PASS | Only when documents exist |
| ⌘+, toggle settings | PASS | Works even during onboarding |
| ⌘+/ toggle help | PASS | |
| Escape close panels | PASS | Priority: settings > help > preview |
| Shortcut blocking during modals | PASS | Most shortcuts blocked when settings/onboarding open |

### Suite 9: Settings

| Test | Status | Notes |
|------|--------|-------|
| Load config on open | PASS | Populates all fields from current config |
| Simple vs Advanced mode | PASS | Toggle between views |
| Save validation feedback | PASS | Fixed by BUG-3 |
| Re-run Setup | PASS | Fixed by BUG-4 |
| Model dropdown (installed models) | PASS | Lists from Ollama API |
| Config persistence (YAML) | PASS | Atomic write via tmp file + rename |

### Suite 10: Error Resilience

| Test | Status | Notes |
|------|--------|-------|
| Config corruption recovery | PASS | Backs up `.yaml.bak`, recreates defaults |
| Config fallback uses valid defaults | PASS | Fixed by BUG-1 (duckduckgo, not tavily) |
| DB corruption recovery | PASS | Backs up `.db.bak`, recreates; falls back to in-memory |
| Mutex poisoning recovery | PASS | `unwrap_or_else(\|e\| e.into_inner())` on DB conn |
| Ollama disconnection handling | PASS | 5s connect timeout, health check returns false |
| Model not found handling | PASS | Fixed by BUG-2 (exact tag match required) |
| Stream interruption handling | PASS | 60s per-chunk timeout, proper error propagation |
| Search failure fallback | PASS | Tavily errors fall back to DuckDuckGo |

---

## Known Issues (Not Fixed)

1. **`⌘+P` conflicts with system Print shortcut** — Toggle Preview uses `CmdOrCtrl+P`. Won't trigger system print (Tauri intercepts it) but may confuse users expecting Print. Design decision, not a bug.

2. **Help panel shortcut label inconsistency** — Help panel shows `⌘ ?` but the JS handler triggers on `⌘+/`. The native menu uses `CmdOrCtrl+?` (which is `⇧⌘/`). Both paths work but display differently.

3. **Retry adds duplicate assistant messages** — When retrying, the old assistant response stays in the DB. The new response is appended. The user sees both. This may be intentional (preserving history) but could be confusing.

4. **DuckDuckGo scraping is fragile** — The search provider scrapes HTML from `html.duckduckgo.com/html/`. If DDG changes their markup, search will silently return no results.

5. **No periodic health re-check in main app** — Health is checked once on mount. If Ollama disconnects mid-session, there's no automatic recovery notification (only visible when a chat request fails).

---

## Build Verification

| Check | Result |
|-------|--------|
| `cargo check` | PASS |
| `cargo clippy -- -D warnings` | PASS (0 warnings) |
| `cargo test` | PASS (36/36 tests) |
| `npx tsc --noEmit` | PASS (0 errors) |
| `npm run tauri build` | PASS |
| `.app` bundle produced | YES |
| `.dmg` produced | YES |

---

## Files Modified

| File | Change |
|------|--------|
| `src-tauri/src/types.rs` | Fix `AppConfig::default()` search provider: tavily → duckduckgo |
| `src-tauri/src/llm/mod.rs` | Fix `check_model` to require exact tag match |
| `src/components/SettingsPanel.tsx` | Show save error feedback; fix footer structure |
| `src/stores/chatStore.ts` | Add `preferencesLoaded` state flag |
| `src/App.tsx` | Include `wizardCompleted` + `preferencesLoaded` in onboarding logic |

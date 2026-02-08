<h1 align="center">AuraForge</h1>

<p align="center">
  <strong>Plan before you build. Build with confidence.</strong>
</p>

<p align="center">
  A desktop app that transforms project ideas into structured, implementation-ready planning documents through conversational AI — powered by local LLM runtimes (Ollama or OpenAI-compatible local endpoints), so your data stays on your machine.
</p>

<p align="center">
  <a href="https://github.com/saagar210/auraforge"><img src="https://img.shields.io/badge/Tauri-2.0-blue?logo=tauri&logoColor=white" alt="Tauri 2.0" /></a>
  <a href="https://github.com/saagar210/auraforge"><img src="https://img.shields.io/badge/React-19-61DAFB?logo=react&logoColor=white" alt="React 19" /></a>
  <a href="https://github.com/saagar210/auraforge"><img src="https://img.shields.io/badge/Rust-2021_Edition-DEA584?logo=rust&logoColor=white" alt="Rust" /></a>
  <a href="https://github.com/saagar210/auraforge"><img src="https://img.shields.io/badge/TypeScript-Strict-3178C6?logo=typescript&logoColor=white" alt="TypeScript Strict" /></a>
  <a href="https://github.com/saagar210/auraforge"><img src="https://img.shields.io/badge/Tailwind_CSS-4.0-38BDF8?logo=tailwindcss&logoColor=white" alt="Tailwind CSS 4" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-green" alt="MIT License" /></a>
</p>

---

## Table of Contents

- [The Problem](#the-problem)
- [What AuraForge Does](#what-auraforge-does)
- [Feature Highlights](#feature-highlights)
- [Architecture](#architecture)
- [Tech Stack](#tech-stack)
- [Getting Started](#getting-started)
- [Operations](#operations)
- [Generated Output](#generated-output)
- [Quality Metrics](#quality-metrics)
- [Keyboard Shortcuts](#keyboard-shortcuts)
- [Project Status](#project-status)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgments](#acknowledgments)

---

## The Problem

You have a project idea. You open your coding assistant and start typing. Twenty minutes later, you're refactoring your third attempt at an auth system because you didn't think through the data model first.

**Planning is the bottleneck of AI-assisted development.** ChatGPT gives you walls of generic advice. Docs assume you already know what you're building. And no tool takes you from _"I want to build X"_ to _"here's your spec, config, and step-by-step prompts — start coding."_

The gap between idea and execution-ready plan is where most projects stall.

## What AuraForge Does

AuraForge is a planning partner that thinks with you, not for you. You describe what you want to build. It asks hard questions, challenges weak assumptions, and grounds decisions in current best practices via live web search. When the plan is solid, it generates a complete execution document pack you can drop into your project and start building immediately.

**The key insight:** the output isn't advice — it's artifacts. A spec your AI coding tool can follow. Prompts broken into phases. A conversation log so you remember _why_ you made each decision.

### The Workflow

| Step | What Happens |
|------|-------------|
| **1. Describe your idea** | Start a new session and explain what you want to build in plain language. |
| **2. Refine through dialogue** | AuraForge asks clarifying questions — scope, tech choices, trade-offs. Web search provides current context automatically when it detects technical topics. |
| **3. Converge on decisions** | The conversation narrows from broad idea to concrete plan. Tech stack, architecture, phases — all decided collaboratively. |
| **4. Forge the plan** | After 3+ exchanges, click **Forge the Plan** to generate planning docs plus a model handoff from your conversation. |
| **5. Save and build** | Export to a folder. Open `START_HERE.md` and `MODEL_HANDOFF.md` — they walk you through setup and first prompts for your coding assistant. |

### The Output Documents

| Document | Purpose |
|----------|---------|
| **START_HERE.md** | Quick-start bridge — prerequisites, step-by-step setup, and first prompt flow. Non-technical friendly. |
| **SPEC.md** | Technical specification with interface contracts in real code (Rust structs, TS interfaces — not pseudocode). Undiscussed topics marked `[TBD]`. |
| **CLAUDE.md** | Project context file optimized for Claude Code. Keep alongside `MODEL_HANDOFF.md` for model-specific execution rules. |
| **PROMPTS.md** | Phased implementation with complexity indicators, per-phase prerequisites, verification checklists, and cross-references to SPEC.md + CLAUDE.md. |
| **README.md** | Planning folder orientation — key decisions, known gaps, document guide. |
| **MODEL_HANDOFF.md** | Target-aware handoff notes for Codex, Claude, Cursor, Gemini, or generic coding agents. |
| **CONVERSATION.md** | Full planning transcript from session data (no LLM needed). Revisit to understand _why_ decisions were made. |

Documents are generated sequentially with cross-referencing — each document receives all previously generated documents as context, ensuring consistency across the entire output.

---

## Feature Highlights

### Conversational Planning with Streaming
Natural dialogue powered by local LLM inference. Responses stream token-by-token via NDJSON parsing with `requestAnimationFrame`-batched rendering to prevent excessive re-renders. Cancel mid-stream with an `AtomicBool` flag checked per chunk. Retry any response — the old assistant message is deleted from the database before streaming the replacement. AuraForge keeps conversation quality high by limiting itself to two questions per turn, finishing one topic before moving on, and pushing back on vague answers before generation.

### Local Runtime Flexibility (No Required Paid APIs)
AuraForge supports `ollama` and `openai_compatible` local provider modes (including LM Studio-style OpenAI-compatible endpoints). Ollama remains the default path, while provider normalization keeps model listing, health checks, and generation behavior consistent across supported local runtimes.

### Grounded in Reality via Web Search
Search providers with automatic failover: **DuckDuckGo** (free, HTML scraping with multi-selector fallback), **Tavily** (API-based, optional), and **SearXNG** (self-hosted). Search triggers automatically for technical prompts. A normalized provider/query cache reduces duplicate lookups and fallback logic keeps planning usable when providers fail.

### Multi-Document Generation with Cross-Referencing
Sequential generation in dependency order: `SPEC` -> `CLAUDE` -> `PROMPTS` -> `README` -> `START_HERE` -> `MODEL_HANDOFF`. Each document receives prior docs as context for consistency. Undiscussed topics are marked `[TBD — not discussed during planning]` instead of being fabricated. Documents are replaced atomically in one transaction, and staleness detection compares latest message timestamps against generation metadata.

### Planning Readiness Tracking
Before generation, AuraForge computes planning coverage across must-have and should-have topics, then records quality/confidence metadata. If must-haves are missing, the UI requires explicit confirmation before forcing generation with `[TBD]` markers.

### Template-Based Starts and Codebase Import
You can start from built-in planning templates (SaaS web app, API service, CLI tool, Tauri desktop app) and import an existing local codebase for grounded refactor planning. Import runs locally with bounded reads and byte caps to keep the app responsive.

### Conversation Branching
Create branches from any message to explore alternate planning decisions without losing the main thread. Branch lineage is persisted, and generation/export can run from the selected branch context.

### Model-Agnostic Export Packs
`save_to_folder` remains the finish line. Exports include deterministic `manifest.json` metadata (`bytes`, `lines`, `sha256`) for each generated file so any coding model workflow can validate pack integrity.

### Local-First and Private
All data stays on your machine. Conversations live in a local SQLite database with WAL mode. Config is stored as YAML in `~/.auraforge/`. Network calls are limited to your configured local model runtime and optional web search providers. No telemetry, no cloud sync, and no required paid model APIs.

### Resilient Data Layer
SQLite with WAL mode, foreign key cascades, and automatic schema migrations. Config writes are atomic (write to temp file, `fsync`, rename). If the config file corrupts, AuraForge backs it up and recreates valid defaults. If the database corrupts, it backs it up and falls back to an in-memory database so the app stays functional. Mutex poisoning is recovered via `unwrap_or_else(|e| e.into_inner())`.

### Session Management
Multiple concurrent planning sessions with UUID-based IDs. Sessions auto-name from the first message (truncated to 60 characters). Delete cascades to messages and documents via foreign keys. Sessions ordered by `updated_at` with the most recent first. Message windowing shows the latest 120 messages initially, loading 80 more on scroll-to-top.

### The Forge Aesthetic
A dark, atmospheric UI built around the metaphor of crafting. Ember particles drift across a thermal gradient background. Eleven CSS keyframe animations (ember float, thermal drift, molten flow, forge glow, pulse, shimmer) create an atmosphere of focused creation. Custom design tokens define the color system: void backgrounds, molten accents, ember highlights. Typography uses Cinzel for headings, Inter for body, JetBrains Mono for code. Respects `prefers-reduced-motion` — all animations are disabled when the OS preference is set. Background animations automatically pause when the app is not visible to save CPU. React error boundaries catch render crashes with a recovery UI instead of a white screen.

---

## Architecture

```
┌────────────────────────────────────────────────────────────┐
│                   AuraForge Desktop App                     │
├───────────────────────┬────────────────────────────────────┤
│    React Frontend     │          Rust Backend              │
│                       │                                    │
│                       │                                    │
│  ┌────────────────┐   │   ┌──────────────────────┐         │
│  │  Zustand Store  │◄─IPC─►│   Tauri Commands      │         │
│  │  (single store) │   │   └──┬──────┬──────┬────┘         │
│  └───────┬────────┘   │      │      │      │              │
│          │            │      ▼      ▼      ▼              │
│  ┌───────▼────────┐   │  ┌──────┐┌─────┐┌───────┐         │
│  │  Components     │   │  │SQLite││ LLM ││Search │         │
│  │  - Sidebar      │   │  │ WAL  ││     ││       │         │
│  │  - Chat         │   │  └──────┘└──┬──┘└──┬────┘         │
│  │  - DocPreview   │   │             │      │              │
│  │  - Onboarding   │   │             ▼      ▼              │
│  │  - Settings     │   │        ┌───────┐┌──────────────┐  │
│  │  - ForgeButton  │   │        │Local  ││DuckDuckGo    │  │
│  └────────────────┘   │        │LLM(s) ││/ SearXNG     │  │
│                       │        └───────┘└──────────────┘  │
└───────────────────────┴────────────────────────────────────┘
```

**Data flow:** User message -> saved to SQLite -> trigger detection scans for search-worthy content -> (optional) web search results injected as system context -> message + context sent to selected local runtime -> NDJSON stream parsed chunk-by-chunk -> chunks batched via `requestAnimationFrame` -> assistant response saved to SQLite -> frontend re-renders.

**IPC:** Tauri's type-safe command system bridges frontend and Rust. Streaming responses use event emitters filtered by `session_id` to prevent cross-session contamination. Session-switch guards and operation gating prevent stale async writes and overlapping long-running actions.

**Security:** Content Security Policy restricts connections to `localhost` and configured search endpoints. Tauri capabilities are minimal — only `core`, `dialog`, and `opener` plugins are enabled. Link rendering in chat is restricted to `https:` and `mailto:` schemes. SQL identifiers are validated against injection. URL schemes are restricted to `http`/`https`. Config files are written with `0600` permissions on Unix. Input lengths are capped (100 KB messages, 200-char session names). The importer skips symlinks to prevent traversal attacks.

---

## Tech Stack

| Layer | Technology | Role |
|-------|-----------|------|
| **Desktop Runtime** | [Tauri 2.0](https://tauri.app) | ~6.7 MB installer (vs ~150 MB for Electron), native OS integration, Rust-powered IPC |
| **Frontend** | [React 19](https://react.dev) | Component-driven UI with streaming state management |
| **State** | [Zustand 5](https://zustand.docs.pmnd.rs) | Single-store architecture, no provider wrapping |
| **Backend** | [Rust 2021](https://www.rust-lang.org) | Memory-safe backend with async Tokio runtime |
| **LLM** | [Ollama](https://ollama.com) + OpenAI-compatible local runtimes | Local inference with provider abstraction and streaming |
| **Database** | [SQLite](https://sqlite.org) via rusqlite | WAL mode, foreign key cascades, schema migrations |
| **Search** | DuckDuckGo + [Tavily](https://tavily.com) + [SearXNG](https://docs.searxng.org) | Three providers with automatic failover |
| **Styling** | [Tailwind CSS 4](https://tailwindcss.com) | Utility-first with `@theme` design tokens |
| **Markdown** | react-markdown + remark-gfm | GFM rendering with PrismLight syntax highlighting (13 languages) |
| **Build** | [Vite 6](https://vite.dev) | Frontend bundling, HMR in development |

---

## Getting Started

### Option A: Install the App

1. Install and start a local runtime:
   - [Ollama](https://ollama.com/download) (default), or
   - an OpenAI-compatible local endpoint (for example LM Studio)
2. Download the latest release artifact for your platform from [**Releases**](https://github.com/saagar210/auraforge/releases/latest) (`.dmg` on macOS, `.deb`/`.AppImage` on Linux)
3. Install and open AuraForge
4. Follow the setup wizard — it handles provider configuration and model selection

See [`distribution/INSTALL.md`](distribution/INSTALL.md) for detailed instructions and [`distribution/QUICK_REFERENCE.md`](distribution/QUICK_REFERENCE.md) for a workflow cheat sheet.

### Option B: Build from Source

**Prerequisites:**
- **macOS or Linux**
- **One local model runtime**:
  - [Ollama](https://ollama.com), or
  - OpenAI-compatible local endpoint
- **Node.js** 18+ and **Rust** 1.75+
- _Optional:_ [Tavily API key](https://tavily.com) for higher-quality web search
- _Optional:_ self-hosted [SearXNG](https://docs.searxng.org) endpoint for custom search routing

```bash
# Example using Ollama (default path)
brew install ollama
ollama pull qwen3-coder

# Clone and run
git clone https://github.com/saagar210/auraforge.git
cd auraforge
npm install
npm run tauri dev
```

### First Run

AuraForge checks local runtime connectivity and model availability on startup. If anything is missing, a setup wizard walks you through setup. Once configured, describe what you want to build and AuraForge takes it from there.

Health is re-checked every 60 seconds. If your local model runtime disconnects mid-session, a toast notification alerts you immediately.

### Production Build

```bash
npm run tauri build
```

Produces a `.app` bundle and `.dmg` installer in `src-tauri/target/release/bundle/`.
On Linux, the same command produces `.deb` and `.AppImage` bundles in `src-tauri/target/release/bundle/`.

---

## Operations

- Runtime operations and troubleshooting: [`RUNBOOK.md`](RUNBOOK.md)
- Release gate checklist: [`RELEASE_CHECKLIST.md`](RELEASE_CHECKLIST.md)
- Correctness/security audit history: [`AUDIT_REPORT.md`](AUDIT_REPORT.md)

---

## Generated Output

When you click **Forge the Plan** (available after 3+ exchanges), AuraForge creates:

```
my-project-plan/
├── START_HERE.md       # Quick-start guide — read this first
├── README.md           # Planning folder orientation
├── SPEC.md             # Technical specification
├── CLAUDE.md           # Claude-oriented context file
├── PROMPTS.md          # Phased implementation prompts
├── MODEL_HANDOFF.md    # Target-specific execution handoff
├── CONVERSATION.md     # Full planning transcript
└── manifest.json       # Export metadata + file checksums
```

Save to any folder via `Cmd+S` or the Save button. Folder names are sanitized to lowercase alphanumeric + hyphens (max 60 characters). AuraForge checks for existing folders (won't overwrite), verifies disk space (20 GB threshold), and handles permission errors with specific messages. `manifest.json` includes deterministic file metadata (`filename`, `bytes`, `lines`, `sha256`) to make handoff packs verifiable across coding models.

---

## Quality Metrics

| Metric | Result |
|--------|--------|
| `npm run test` | **23/23 passing** |
| `cargo test` | **74/74 passing** |
| `cargo clippy -- -D warnings` | **0 warnings** |
| `npx tsc --noEmit` | **0 errors** |
| `npm run tauri build` | **Produces macOS + Linux bundles** |
| Source lines | **~9,200** across Rust, TypeScript, and CSS |
| Rust test coverage focus | Config validation, DB operations, search triggers/cache, DuckDuckGo HTML parsing, provider parsing, importer bounds, export manifest integrity, docgen quality scoring |

### What's Tested

- **Search trigger detection:** 18 unit tests covering technology keywords, question patterns, comparison queries, version lookups, and negative cases
- **DuckDuckGo parser:** HTML parsing with `.result` selectors, `uddg` redirect extraction, fallback link extraction, empty-result handling
- **Database operations:** Session CRUD, message cascade deletion, document atomic replacement, assistant message deletion on retry, SQL identifier validation
- **Config security:** URL scheme validation (http/https only), file permission enforcement (0600), atomic writes
- **Provider behavior:** local provider alias parsing + unsupported provider validation
- **Import/export safety:** importer prefix-read bounds, symlink traversal prevention, manifest deterministic ordering/checksum metadata
- **Docgen quality:** plan readiness scoring, coverage analysis for must-have/should-have topics, empty and rich conversation scoring
- **Frontend components:** ErrorBoundary crash recovery, ConfirmModal interactions, Toast rendering and dismissal, ForgeButton states, EmptyState variants
- **Frontend async races:** session switch + cancel/timeout race handling in `chatStore`

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+Enter` | Send message |
| `Cmd+N` | New project |
| `Cmd+G` | Forge the Plan (when available) |
| `Cmd+S` | Save to folder (when documents exist) |
| `Cmd+,` | Toggle settings |
| `Shift+Cmd+P` | Toggle document preview |
| `Cmd+/` | Toggle help panel |
| `Cmd+\` | Toggle sidebar |
| `Escape` | Close panels (priority: settings > help > preview) |

---

## Project Status

### Completed

**v0.1.0** — Core planning engine, conversational AI, five-document generation, web search, forge aesthetic, .dmg distribution.

**v0.2.0** — Document quality overhaul: advisory readiness system, deeper conversations (topic-at-a-time, pushback on vague answers), reality-grounded generation (real code not pseudocode, tech-stack-consistent commands), cross-referenced documents, hallucination guardrails with `[TBD]` markers, START_HERE.md as 6th document.

- [x] Confidence scoring — post-generation assessment of document completeness
- [x] Planning coverage UI — sidebar indicator tracking topic coverage during conversation
- [x] Audit report fixes — structured error handling (`AppError`), transaction safety, CSP hardening (see AUDIT_REPORT.md)
- [x] Release operations runbook + checklist for macOS/Linux
- [x] Linux builds (Windows deferred)
- [x] Additional local model runtimes (LM Studio/OpenAI-compatible local endpoints)
- [x] Project templates for common app types
- [x] Import existing codebases for refactoring plans
- [x] Conversation branching — explore alternate decisions without losing the main thread
- [x] Security hardening — SQL identifier validation, URL scheme restrictions, config file permissions, input length limits, symlink traversal prevention
- [x] React quality pass — error boundaries, confirm modals (no more `window.confirm`), `useShallow` selectors to prevent unnecessary re-renders, animation pausing on background tabs
- [x] Bundle optimization — PrismLight with 13 registered languages instead of full Prism bundle
- [x] Test coverage expansion — 74 Rust tests, 23 frontend tests across 6 test files

### Deferred by Scope

- [ ] Export to GitHub Issues / Linear integration
- [ ] Windows packaging support (explicitly deferred)

---

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, code style, and PR guidelines.

```bash
# Run all checks before submitting
npm run test
npx tsc --noEmit
cd src-tauri && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test && cd ..
```

If you find a bug or have a feature idea, [open an issue](https://github.com/saagar210/auraforge/issues).

---

## License

MIT License. See [LICENSE](LICENSE) for details.

---

## Acknowledgments

- [Tauri](https://tauri.app) — proving desktop apps don't need Electron's footprint
- [Ollama](https://ollama.com) — making local LLM inference accessible to everyone
- [Tavily](https://tavily.com) — web search API for grounded AI responses
- [SearXNG](https://docs.searxng.org) — privacy-respecting metasearch for self-hosters

<h1 align="center">AuraForge</h1>

<p align="center">
  <strong>Plan before you build. Build with confidence.</strong>
</p>

<p align="center">
  A desktop app that transforms project ideas into structured, implementation-ready planning documents through conversational AI — powered by local LLMs via Ollama, so your data never leaves your machine.
</p>

<p align="center">
  <a href="https://github.com/saagar210/auraforge"><img src="https://img.shields.io/badge/Tauri-2.0-blue?logo=tauri&logoColor=white" alt="Tauri 2.0" /></a>
  <a href="https://github.com/saagar210/auraforge"><img src="https://img.shields.io/badge/React-19-61DAFB?logo=react&logoColor=white" alt="React 19" /></a>
  <a href="https://github.com/saagar210/auraforge"><img src="https://img.shields.io/badge/Rust-2021_Edition-DEA584?logo=rust&logoColor=white" alt="Rust" /></a>
  <a href="https://github.com/saagar210/auraforge"><img src="https://img.shields.io/badge/TypeScript-Strict-3178C6?logo=typescript&logoColor=white" alt="TypeScript Strict" /></a>
  <a href="https://github.com/saagar210/auraforge"><img src="https://img.shields.io/badge/Tailwind_CSS-4.0-38BDF8?logo=tailwindcss&logoColor=white" alt="Tailwind CSS 4" /></a>
  <a href="https://github.com/saagar210/auraforge"><img src="https://img.shields.io/badge/Tests-41_passing-brightgreen" alt="41 Tests Passing" /></a>
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
- [Generated Output](#generated-output)
- [Quality Metrics](#quality-metrics)
- [Keyboard Shortcuts](#keyboard-shortcuts)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgments](#acknowledgments)

---

## The Problem

You have a project idea. You open Claude Code and start typing. Twenty minutes later, you're refactoring your third attempt at an auth system because you didn't think through the data model first.

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
| **5. Save and build** | Export to a folder. Open `START_HERE.md` — it walks you through setup, copying `CLAUDE.md`, and pasting your first prompt into Claude Code. |

### The Output Documents

| Document | Purpose |
|----------|---------|
| **START_HERE.md** | Quick-start bridge — prerequisites, step-by-step setup, your first Claude Code prompt. Non-technical friendly. |
| **SPEC.md** | Technical specification with interface contracts in real code (Rust structs, TS interfaces — not pseudocode). Undiscussed topics marked `[TBD]`. |
| **CLAUDE.md** | Project context for Claude Code — tech stack, commands, conventions, and anti-patterns. Drop in your repo root. |
| **PROMPTS.md** | Phased implementation with complexity indicators, per-phase prerequisites, verification checklists, and cross-references to SPEC.md + CLAUDE.md. |
| **README.md** | Planning folder orientation — key decisions, known gaps, document guide. |
| **MODEL_HANDOFF.md** | Target-aware handoff notes for Codex, Claude, Cursor, Gemini, or generic coding agents. |
| **CONVERSATION.md** | Full planning transcript from session data (no LLM needed). Revisit to understand _why_ decisions were made. |

Documents are generated sequentially with cross-referencing — each document receives all previously generated documents as context, ensuring consistency across the entire output.

---

## Feature Highlights

### Conversational Planning with Streaming
Natural dialogue powered by Ollama's local LLM inference. Responses stream token-by-token via NDJSON parsing with `requestAnimationFrame`-batched rendering to prevent excessive re-renders. Cancel mid-stream with an `AtomicBool` flag checked per chunk. Retry any response — the old assistant message is deleted from the database before streaming the replacement. v0.2.0 conversations go deeper — AuraForge limits itself to two questions per turn, finishes one topic before moving to the next, and pushes back on vague answers before allowing generation.

### Grounded in Reality via Web Search
Search providers with automatic failover: **DuckDuckGo** (free, HTML scraping with multi-selector fallback), **Tavily** (API-based, higher quality), and **SearXNG** (self-hosted). Search triggers automatically when the conversation involves technical topics — detected by matching against 46 technology keywords and 25 trigger patterns. Results are injected as system context so the LLM can reference current versions, best practices, and real-world trade-offs.

### Multi-Document Generation with Cross-Referencing
Sequential generation in dependency order: SPEC → CLAUDE → PROMPTS → README → START_HERE. Each document receives all previously generated documents as context, enabling cross-referencing (e.g., PROMPTS.md references exact conventions from CLAUDE.md, START_HERE.md generates setup steps matching the actual tech stack). Documents use `[TBD — not discussed during planning]` markers for undiscussed topics instead of inventing content. Output validation retries once if a document doesn't start with a proper heading (`#`). Documents are stored atomically — old versions are deleted and new ones inserted in a single database transaction. Staleness detection compares the latest message timestamp against document generation time.

### Planning Readiness Tracking
When you trigger document generation, AuraForge assesses conversation coverage across key planning topics: problem statement, user flow, tech stack, data model, and scope boundaries. If gaps exist, it reports them and asks for explicit confirmation before forcing generation with `[TBD]` markers. This prevents accidental exports of incomplete plans.

### Local-First and Private
All data stays on your machine. Conversations live in a local SQLite database with WAL mode. Config is stored as YAML in `~/.auraforge/`. The only network calls are to your local Ollama instance and (optionally) web search providers. No telemetry, no cloud sync, and no paid model APIs required.

### Resilient Data Layer
SQLite with WAL mode, foreign key cascades, and automatic schema migrations. Config writes are atomic (write to temp file, `fsync`, rename). If the config file corrupts, AuraForge backs it up and recreates valid defaults. If the database corrupts, it backs it up and falls back to an in-memory database so the app stays functional. Mutex poisoning is recovered via `unwrap_or_else(|e| e.into_inner())`.

### Session Management
Multiple concurrent planning sessions with UUID-based IDs. Sessions auto-name from the first message (truncated to 60 characters). Delete cascades to messages and documents via foreign keys. Sessions ordered by `updated_at` with the most recent first. Message windowing shows the latest 120 messages initially, loading 80 more on scroll-to-top.

### The Forge Aesthetic
A dark, atmospheric UI built around the metaphor of crafting. Ember particles drift across a thermal gradient background. Eleven CSS keyframe animations (ember float, thermal drift, molten flow, forge glow, pulse, shimmer) create an atmosphere of focused creation. Custom design tokens define the color system: void backgrounds, molten accents, ember highlights. Typography uses Cinzel for headings, Inter for body, JetBrains Mono for code. Respects `prefers-reduced-motion` — all animations are disabled when the OS preference is set.

---

## Architecture

```
┌────────────────────────────────────────────────────────────┐
│                   AuraForge Desktop App                     │
├───────────────────────┬────────────────────────────────────┤
│    React Frontend     │          Rust Backend              │
│    (21 components)    │          (8 modules)               │
│                       │                                    │
│  ┌────────────────┐   │   ┌──────────────────────┐         │
│  │  Zustand Store  │◄─IPC─►│   23 Tauri Commands   │         │
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
│  │  - ForgeButton  │   │        │Ollama ││DuckDuckGo    │  │
│  └────────────────┘   │        │(local)││/ SearXNG     │  │
│                       │        └───────┘└──────────────┘  │
└───────────────────────┴────────────────────────────────────┘
```

**Data flow:** User message → saved to SQLite → trigger detection scans for search-worthy content → (optional) web search results injected as system context → message + context sent to Ollama → NDJSON stream parsed chunk-by-chunk → chunks batched via `requestAnimationFrame` → assistant response saved to SQLite → frontend re-renders.

**IPC:** All 23 commands use Tauri's type-safe command system. Streaming responses use Tauri event emitters filtered by `session_id` to prevent cross-session contamination. A session-switch guard (`AtomicBool` per request) prevents stale responses from overwriting current state.

**Security:** Content Security Policy restricts connections to `localhost` and configured search endpoints. Tauri capabilities are minimal — only `core`, `dialog`, and `opener` plugins are enabled. Link rendering in chat is restricted to `https:` and `mailto:` schemes.

---

## Tech Stack

| Layer | Technology | Role |
|-------|-----------|------|
| **Desktop Runtime** | [Tauri 2.0](https://tauri.app) | ~6.7 MB installer (vs ~150 MB for Electron), native OS integration, Rust-powered IPC |
| **Frontend** | [React 19](https://react.dev) | Component-driven UI with streaming state management |
| **State** | [Zustand 5](https://zustand.docs.pmnd.rs) | Single-store architecture, no provider wrapping |
| **Backend** | [Rust 2021](https://www.rust-lang.org) | Memory-safe backend with async Tokio runtime |
| **LLM** | [Ollama](https://ollama.com) | Local inference, NDJSON streaming, any GGUF model |
| **Database** | [SQLite](https://sqlite.org) via rusqlite | WAL mode, foreign key cascades, schema migrations |
| **Search** | DuckDuckGo + [Tavily](https://tavily.com) + [SearXNG](https://docs.searxng.org) | Three providers with automatic failover |
| **Styling** | [Tailwind CSS 4](https://tailwindcss.com) | Utility-first with `@theme` design tokens |
| **Markdown** | react-markdown + remark-gfm | GFM rendering with Prism syntax highlighting |
| **Build** | [Vite 6](https://vite.dev) | Frontend bundling, HMR in development |

---

## Getting Started

### Option A: Install the App

1. Install [Ollama](https://ollama.com/download) and open it
2. Download the latest `.dmg` from [**Releases**](https://github.com/saagar210/auraforge/releases/latest)
3. Drag AuraForge to Applications and open it
4. Follow the setup wizard — it handles model download and configuration

See [`distribution/INSTALL.md`](distribution/INSTALL.md) for detailed instructions and [`distribution/QUICK_REFERENCE.md`](distribution/QUICK_REFERENCE.md) for a workflow cheat sheet.

### Option B: Build from Source

**Prerequisites:**
- **macOS** with Apple Silicon (Intel works too, just slower inference)
- **Ollama** installed and running ([ollama.com](https://ollama.com))
- **Node.js** 18+ and **Rust** 1.75+
- _Optional:_ [Tavily API key](https://tavily.com) for higher-quality web search
- _Optional:_ self-hosted [SearXNG](https://docs.searxng.org) endpoint for custom search routing

```bash
# Install Ollama and pull the default model
brew install ollama
ollama pull qwen3-coder:30b-a3b-instruct-q4_K_M

# Clone and run
git clone https://github.com/saagar210/auraforge.git
cd auraforge
npm install
npm run tauri dev
```

### First Run

AuraForge checks for Ollama connectivity and model availability on startup. If anything is missing, a setup wizard walks you through installation and model download. Once set up, describe what you want to build and AuraForge takes it from there.

Health is re-checked every 60 seconds. If Ollama disconnects mid-session, a toast notification alerts you immediately.

### Production Build

```bash
npm run tauri build
```

Produces a `.app` bundle and `.dmg` installer in `src-tauri/target/release/bundle/`.

---

## Generated Output

When you click **Forge the Plan** (available after 3+ exchanges), AuraForge creates:

```
my-project-plan/
├── START_HERE.md       # Quick-start guide — read this first
├── README.md           # Planning folder orientation
├── SPEC.md             # Technical specification
├── CLAUDE.md           # Project context for Claude Code
├── PROMPTS.md          # Phased implementation prompts
├── MODEL_HANDOFF.md    # Target-specific model handoff
└── CONVERSATION.md     # Full planning transcript
```

Save to any folder via `Cmd+S` or the Save button. Folder names are sanitized to lowercase alphanumeric + hyphens (max 60 characters). AuraForge checks for existing folders (won't overwrite), verifies disk space (20 GB threshold), and handles permission errors with specific messages.

---

## Quality Metrics

| Metric | Result |
|--------|--------|
| `cargo test` | **45/45 passing** |
| `cargo clippy -- -D warnings` | **0 warnings** |
| `npx tsc --noEmit` | **0 errors** |
| `npm run tauri build` | **Produces .app + .dmg** |
| Source lines | **~8,825** across Rust, TypeScript, and CSS |
| Rust test coverage | Config parsing, DB operations, search trigger detection (18 cases), URL extraction, model matching, message deletion |

### What's Tested

- **Search trigger detection:** 18 unit tests covering technology keywords, question patterns, comparison queries, version lookups, and negative cases
- **Database operations:** Session CRUD, message cascade deletion, document atomic replacement, assistant message deletion on retry
- **Config handling:** YAML round-trip, default generation, corruption recovery
- **URL extraction:** DuckDuckGo redirect URL parsing (`uddg=` parameter extraction)

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

## Roadmap

### Shipped

**v0.1.0** — Core planning engine, conversational AI, five-document generation, web search, forge aesthetic, .dmg distribution.

**v0.2.0** — Document quality overhaul: advisory readiness system, deeper conversations (topic-at-a-time, pushback on vague answers), reality-grounded generation (real code not pseudocode, tech-stack-consistent commands), cross-referenced documents, hallucination guardrails with `[TBD]` markers, START_HERE.md as 6th document.

### Next

- [x] Confidence scoring — post-generation assessment of document completeness
- [x] Planning coverage UI — sidebar indicator tracking topic coverage during conversation
- [ ] Audit report fixes — structured error handling (`AppError`), transaction safety, CSP hardening (see AUDIT_REPORT.md)

### Future

- [ ] Linux builds (Windows deferred)
- [ ] Additional local model runtimes (LM Studio/Ollama-compatible endpoints)
- [ ] Project templates for common app types
- [ ] Import existing codebases for refactoring plans
- [ ] Export to GitHub Issues / Linear integration
- [ ] Conversation branching — explore alternate decisions without losing the main thread

---

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, code style, and PR guidelines.

```bash
# Run all checks before submitting
cd src-tauri && cargo fmt && cargo clippy -- -D warnings && cargo test && cd ..
npx tsc --noEmit
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

<h1 align="center">AuraForge</h1>

<p align="center">
  <strong>Plan before you build. Build with confidence.</strong>
</p>

<p align="center">
  A desktop app that transforms project ideas into structured, implementation-ready planning documents through conversational AI — local-first with Ollama by default, with optional OpenAI and Anthropic provider support.
</p>

<p align="center">
  <a href="https://github.com/saagar210/auraforge"><img src="https://img.shields.io/badge/Tauri-2.0-blue?logo=tauri&logoColor=white" alt="Tauri 2.0" /></a>
  <a href="https://github.com/saagar210/auraforge"><img src="https://img.shields.io/badge/React-19-61DAFB?logo=react&logoColor=white" alt="React 19" /></a>
  <a href="https://github.com/saagar210/auraforge"><img src="https://img.shields.io/badge/Rust-2021_Edition-DEA584?logo=rust&logoColor=white" alt="Rust" /></a>
  <a href="https://github.com/saagar210/auraforge"><img src="https://img.shields.io/badge/TypeScript-Strict-3178C6?logo=typescript&logoColor=white" alt="TypeScript Strict" /></a>
  <a href="https://github.com/saagar210/auraforge"><img src="https://img.shields.io/badge/Tailwind_CSS-4.0-38BDF8?logo=tailwindcss&logoColor=white" alt="Tailwind CSS 4" /></a>
  <a href="https://github.com/saagar210/auraforge"><img src="https://img.shields.io/badge/Tests-Rust_%2B_Vitest-brightgreen" alt="Rust and Vitest tests" /></a>
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

AuraForge is a planning partner that thinks with you, not for you. You describe what you want to build. It asks hard questions, challenges weak assumptions, and grounds decisions in current best practices via live web search. When the plan is solid, it generates six production-ready documents that you drop into your project and start building immediately.

**The key insight:** the output isn't advice — it's artifacts. A spec your AI coding tool can follow. Prompts broken into phases. A conversation log so you remember _why_ you made each decision.

### The Workflow

| Step | What Happens |
|------|-------------|
| **1. Describe your idea** | Start a new session and explain what you want to build in plain language. |
| **2. Refine through dialogue** | AuraForge asks clarifying questions — scope, tech choices, trade-offs. Web search provides current context automatically when it detects technical topics. |
| **3. Converge on decisions** | The conversation narrows from broad idea to concrete plan. Tech stack, architecture, phases — all decided collaboratively. |
| **4. Forge the plan** | After 3+ exchanges, click **Forge the Plan** to generate all six documents from your conversation. |
| **5. Save and build** | Export to a folder. Open `START_HERE.md` — it walks you through setup, copying `CLAUDE.md`, and pasting your first prompt into Claude Code. |

### The Six Documents

| Document | Purpose |
|----------|---------|
| **START_HERE.md** | Quick-start bridge — prerequisites, step-by-step setup, your first Claude Code prompt. Non-technical friendly. |
| **SPEC.md** | Technical specification with interface contracts in real code (Rust structs, TS interfaces — not pseudocode). Undiscussed topics marked `[TBD]`. |
| **CLAUDE.md** | Project context for Claude Code — tech stack, commands, conventions, and anti-patterns. Drop in your repo root. |
| **PROMPTS.md** | Phased implementation with complexity indicators, per-phase prerequisites, verification checklists, and cross-references to SPEC.md + CLAUDE.md. |
| **README.md** | Planning folder orientation — key decisions, known gaps, document guide. |
| **CONVERSATION.md** | Full planning transcript from session data (no LLM needed). Revisit to understand _why_ decisions were made. |

Documents are generated sequentially with cross-referencing — each document receives all previously generated documents as context, ensuring consistency across the entire output.

---

## Feature Highlights

### Conversational Planning with Multi-Provider LLM Support
Natural dialogue powered by a provider-routed backend: **Ollama** (default local mode), plus **OpenAI** and **Anthropic** adapters when API keys are configured. Ollama responses stream token-by-token via NDJSON parsing with `requestAnimationFrame`-batched rendering to prevent excessive re-renders. Cancel mid-stream with an `AtomicBool` flag checked per chunk. Retry any response — the old assistant message is deleted from the database before streaming the replacement.

### Grounded in Reality via Web Search + Citations
Three search providers with automatic failover: **DuckDuckGo** (free, HTML scraping with multi-selector fallback), **Tavily** (API-based, higher quality), and **SearXNG** (self-hosted). Search triggers automatically when the conversation involves technical topics. Results are injected as system context so the LLM can reference current versions, best practices, and real-world trade-offs. Assistant messages now surface source links and freshness timestamps.

### Six-Document Generation with Version History and Diff
Sequential generation in dependency order: SPEC → CLAUDE → PROMPTS → README → START_HERE. Each document receives all previously generated documents as context, enabling cross-referencing. Documents use `[TBD — not discussed during planning]` markers for undiscussed topics instead of inventing content. Output validation retries once if a document doesn't start with a proper heading (`#`). AuraForge now stores document versions, supports single-document regeneration, and shows previous-vs-current diff views in the document preview.

### Planning Readiness Scorecard
AuraForge assesses conversation coverage across key planning topics: problem statement, user flow, tech stack, data model, and scope boundaries, plus secondary checks like error handling and testing strategy. The Forge UI now shows a readiness scorecard before generation, including unresolved `[TBD]` count and a recommendation.

### Planning Tools Workbench
A new in-app **Planning Tools** panel provides:
- Project templates (SaaS, API, CLI, AI Agent)
- Repository context import (detected languages + key files)
- Conversation branch creation for alternate decision paths
- Issue export preview generation from planning docs

### Local-First and Private
All data stays on your machine by default in local Ollama mode. Conversations live in a local SQLite database with WAL mode. Config is stored as YAML in `~/.auraforge/`. Network calls are only made to configured providers you opt into (Ollama/OpenAI/Anthropic + optional web search). No telemetry, no cloud sync, and no API keys required for local-first setup.

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
│  (chat + planning UI) │      (commands + services)         │
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
│  │  - PlanningOps  │   │        │Ollama ││DuckDuckGo    │  │
│  └────────────────┘   │        │OpenAI ││Tavily / SearX│  │
│                       │        │Anthropic││NG            │  │
│                       │        └───────┘└──────────────┘  │
└───────────────────────┴────────────────────────────────────┘
```

**Data flow:** User message → saved to SQLite → trigger detection scans for search-worthy content → (optional) web search results injected as system context → provider-routed LLM request (Ollama/OpenAI/Anthropic) → streamed or chunked UI updates → assistant response saved to SQLite with search metadata → frontend re-renders.

**IPC:** Commands use Tauri's type-safe command system. Streaming responses use Tauri event emitters filtered by `session_id` to prevent cross-session contamination. A session-switch guard (`AtomicBool` per request) prevents stale responses from overwriting current state.

**Security:** Content Security Policy restricts connections to trusted endpoints (`localhost`, configured search providers, and HTTPS APIs). Tauri capabilities are minimal — only `core`, `dialog`, and `opener` plugins are enabled. Link rendering in chat is restricted to `https:` and `mailto:` schemes.

---

## Tech Stack

| Layer | Technology | Role |
|-------|-----------|------|
| **Desktop Runtime** | [Tauri 2.0](https://tauri.app) | ~6.7 MB installer (vs ~150 MB for Electron), native OS integration, Rust-powered IPC |
| **Frontend** | [React 19](https://react.dev) | Component-driven UI with streaming state management |
| **State** | [Zustand 5](https://zustand.docs.pmnd.rs) | Single-store architecture, no provider wrapping |
| **Backend** | [Rust 2021](https://www.rust-lang.org) | Memory-safe backend with async Tokio runtime |
| **LLM** | Ollama + OpenAI + Anthropic | Provider-routed chat/doc generation (Ollama local-first; cloud providers optional via API keys) |
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
- **Ollama** installed and running for local-first mode ([ollama.com](https://ollama.com))
- **Node.js** 18+ and **Rust** 1.75+
- _Optional:_ [Tavily API key](https://tavily.com) for higher-quality web search
- _Optional:_ `OPENAI_API_KEY` and/or `ANTHROPIC_API_KEY` for cloud providers

```bash
# Install Ollama and pull the default model
brew install ollama
ollama pull qwen3-coder:30b-a3b-q4_K_M

# Clone and run
git clone https://github.com/saagar210/auraforge.git
cd auraforge
npm install
npm run tauri dev
```

### First Run

AuraForge defaults to Ollama and checks local connectivity/model availability on startup. If anything is missing, a setup wizard walks you through installation and model download. Once set up, describe what you want to build and AuraForge takes it from there.

If you prefer cloud providers, set `OPENAI_API_KEY` or `ANTHROPIC_API_KEY`, then switch provider in Settings.

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
└── CONVERSATION.md     # Full planning transcript
```

Save to any folder via `Cmd+S` or the Save button. Folder names are sanitized to lowercase alphanumeric + hyphens (max 60 characters). AuraForge checks for existing folders (won't overwrite), verifies disk space (20 GB threshold), and handles permission errors with specific messages.

---

## Quality Gates

Core validation commands:

- `npm run typecheck`
- `npm test` (Vitest + Testing Library frontend tests)
- `cd src-tauri && cargo fmt --check && cargo clippy -- -D warnings && cargo test`
- `npm run tauri build`

### What's Covered

- **Backend tests:** config parsing/recovery, DB session + message + document flows, search trigger detection, URL extraction.
- **Frontend tests (scaffolded):** component-level checks for planning readiness UI and document diff logic.
- **Runtime safety checks:** type-safe IPC commands, staleness detection, document version history, and folder export guardrails.

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

**v0.3.0 (current)** — Planning tools panel (templates, repo import context, branch creation, issue export preview), multi-provider LLM routing (Ollama/OpenAI/Anthropic), source citations + freshness in chat, document version history with single-document regenerate and diff view, frontend Vitest test harness scaffolding.

### Next

- [ ] Full provider streaming support for OpenAI/Anthropic paths
- [ ] One-click issue export publishing (GitHub/Linear APIs, not just preview)
- [ ] Branch compare/merge workflow in UI
- [ ] CSP hardening and additional security tightening (see AUDIT_REPORT.md)

### Future

- [ ] Windows and Linux builds
- [ ] Team collaboration and shared planning sessions
- [ ] Template marketplace / custom template packs
- [ ] Automated quality scoring tuned by project type

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

- [Anthropic](https://anthropic.com) — Claude and Claude Code, the tools this project is built to complement
- [Tauri](https://tauri.app) — proving desktop apps don't need Electron's footprint
- [Ollama](https://ollama.com) — making local LLM inference accessible to everyone
- [Tavily](https://tavily.com) — web search API for grounded AI responses
- [SearXNG](https://docs.searxng.org) — privacy-respecting metasearch for self-hosters

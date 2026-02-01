<p align="center">
  <img src="screenshots/hero.png" alt="AuraForge" width="720" />
</p>

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
  <a href="https://github.com/saagar210/auraforge"><img src="https://img.shields.io/badge/Tests-39_passing-brightgreen" alt="39 Tests Passing" /></a>
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

AuraForge is a planning partner that thinks with you, not for you. You describe what you want to build. It asks hard questions, challenges weak assumptions, and grounds decisions in current best practices via live web search. When the plan is solid, it generates five production-ready documents that you drop into your project and start building immediately.

**The key insight:** the output isn't advice — it's artifacts. A spec your AI coding tool can follow. Prompts broken into phases. A conversation log so you remember _why_ you made each decision.

### The Workflow

| Step | What Happens |
|------|-------------|
| **1. Describe your idea** | Start a new session and explain what you want to build in plain language. |
| **2. Refine through dialogue** | AuraForge asks clarifying questions — scope, tech choices, trade-offs. Web search provides current context automatically when it detects technical topics. |
| **3. Converge on decisions** | The conversation narrows from broad idea to concrete plan. Tech stack, architecture, phases — all decided collaboratively. |
| **4. Forge the plan** | After 3+ exchanges, click **Forge the Plan** to generate all five documents from your conversation. |
| **5. Save and build** | Export to a folder. Drop `CLAUDE.md` in your project root. Follow `PROMPTS.md` phase by phase. |

### The Five Documents

| Document | Purpose |
|----------|---------|
| **SPEC.md** | Complete technical specification — data models, API surface, architecture decisions, edge cases. |
| **CLAUDE.md** | Project context file for Claude Code. Drop it in your repo root and Claude understands your entire project. |
| **PROMPTS.md** | Phased implementation prompts, scoped and self-contained. Copy, paste, build. |
| **README.md** | Orientation guide — what's in the folder, key decisions, where to start. |
| **CONVERSATION.md** | Full planning transcript generated directly from session data (no LLM needed). Revisit to understand _why_ decisions were made. |

Every document is generated from your specific conversation, referencing your chosen tech stack, architecture, and requirements — not templates or boilerplate.

---

## Feature Highlights

### Conversational Planning with Streaming
Natural dialogue powered by Ollama's local LLM inference. Responses stream token-by-token via NDJSON parsing with `requestAnimationFrame`-batched rendering to prevent excessive re-renders. Cancel mid-stream with an `AtomicBool` flag checked per chunk. Retry any response — the old assistant message is deleted from the database before streaming the replacement.

### Grounded in Reality via Web Search
Three search providers with automatic failover: **DuckDuckGo** (free, HTML scraping with multi-selector fallback), **Tavily** (API-based, higher quality), and **SearXNG** (self-hosted). Search triggers automatically when the conversation involves technical topics — detected by matching against 46 technology keywords and 25 trigger patterns. Results are injected as system context so the LLM can reference current versions, best practices, and real-world trade-offs.

### Five-Document Generation Pipeline
Sequential generation with per-document progress events. Each document uses a specialized prompt template. Output validation retries once if a document doesn't start with a proper heading (`#`). Documents are stored atomically — old versions are deleted and new ones inserted in a single database transaction. Staleness detection compares the latest message timestamp against document generation time.

### Local-First and Private
All data stays on your machine. Conversations live in a local SQLite database with WAL mode. Config is stored as YAML in `~/.auraforge/`. The only network calls are to your local Ollama instance and (optionally) web search providers. No telemetry, no cloud sync, no API keys required to get started.

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
│  └────────────────┘   │        │(local)││Tavily / Searx│  │
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
| **Desktop Runtime** | [Tauri 2.0](https://tauri.app) | ~5 MB binary, native OS integration, Rust-powered IPC |
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
2. Download `AuraForge_0.1.0_aarch64.dmg` from the [`distribution/`](distribution/) folder
3. Drag AuraForge to Applications and open it
4. Follow the setup wizard — it handles model download and configuration

See [`distribution/INSTALL.md`](distribution/INSTALL.md) for detailed instructions and [`distribution/QUICK_REFERENCE.md`](distribution/QUICK_REFERENCE.md) for a workflow cheat sheet.

### Option B: Build from Source

**Prerequisites:**
- **macOS** with Apple Silicon (Intel works too, just slower inference)
- **Ollama** installed and running ([ollama.com](https://ollama.com))
- **Node.js** 18+ and **Rust** 1.75+
- _Optional:_ [Tavily API key](https://tavily.com) for higher-quality web search

```bash
# Install Ollama and pull the default model
brew install ollama
ollama pull qwen3:30b

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
├── README.md           # Orientation — what's here, where to start
├── SPEC.md             # Technical specification
├── CLAUDE.md           # Project context for Claude Code
├── PROMPTS.md          # Phased implementation prompts
└── CONVERSATION.md     # Full planning transcript
```

Save to any folder via `Cmd+S` or the Save button. Folder names are sanitized to lowercase alphanumeric + hyphens (max 60 characters). AuraForge checks for existing folders (won't overwrite), verifies disk space (20 GB threshold), and handles permission errors with specific messages.

---

## Quality Metrics

| Metric | Result |
|--------|--------|
| `cargo test` | **39/39 passing** |
| `cargo clippy -- -D warnings` | **0 warnings** |
| `npx tsc --noEmit` | **0 errors** |
| `npm run tauri build` | **Produces .app + .dmg** |
| Source lines | **~8,000** across Rust, TypeScript, and CSS |
| Rust test coverage | Config parsing, DB operations, search trigger detection (36 cases), URL extraction, model matching, message deletion |

### What's Tested

- **Search trigger detection:** 36 unit tests covering technology keywords, question patterns, comparison queries, version lookups, and negative cases
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

- [ ] Windows and Linux builds
- [ ] Multiple LLM providers (Anthropic API, OpenAI) alongside Ollama
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

- [Anthropic](https://anthropic.com) — Claude and Claude Code, the tools this project is built to complement
- [Tauri](https://tauri.app) — proving desktop apps don't need Electron's footprint
- [Ollama](https://ollama.com) — making local LLM inference accessible to everyone
- [Tavily](https://tavily.com) — web search API for grounded AI responses
- [SearXNG](https://docs.searxng.org) — privacy-respecting metasearch for self-hosters

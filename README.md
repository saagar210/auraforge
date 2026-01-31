<p align="center">
  <img src="screenshots/hero.png" alt="AuraForge" width="720" />
</p>

<h1 align="center">AuraForge</h1>

<p align="center">
  <strong>Plan before you build. Build with confidence.</strong>
</p>

<p align="center">
  A desktop app that transforms project ideas into structured, implementation-ready plans through conversational AI — powered by local LLMs, so your data never leaves your machine.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Tauri-2.0-blue?logo=tauri&logoColor=white" alt="Tauri 2.0" />
  <img src="https://img.shields.io/badge/React-19-61DAFB?logo=react&logoColor=white" alt="React 19" />
  <img src="https://img.shields.io/badge/Rust-1.75+-DEA584?logo=rust&logoColor=white" alt="Rust" />
  <img src="https://img.shields.io/badge/TypeScript-Strict-3178C6?logo=typescript&logoColor=white" alt="TypeScript" />
  <img src="https://img.shields.io/badge/Tailwind-4.0-38BDF8?logo=tailwindcss&logoColor=white" alt="Tailwind CSS 4" />
  <img src="https://img.shields.io/badge/License-MIT-green" alt="MIT License" />
</p>

---

## Table of Contents

- [The Problem](#the-problem)
- [The Solution](#the-solution)
- [Features](#features)
- [How It Works](#how-it-works)
- [Tech Stack](#tech-stack)
- [Architecture](#architecture)
- [Getting Started](#getting-started)
- [Generated Output](#generated-output)
- [Design Philosophy](#design-philosophy)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgments](#acknowledgments)

---

## The Problem

You have a project idea. You open Claude Code and start typing. Twenty minutes later, you're refactoring your third attempt at an auth system because you didn't think through the data model first.

**Planning is the bottleneck of AI-assisted development.** ChatGPT gives you walls of generic advice. Docs assume you already know what you're building. And no tool takes you from _"I want to build X"_ to _"here's your spec, config, and step-by-step prompts — start coding."_

The gap between idea and execution-ready plan is where most projects stall.

## The Solution

AuraForge is a planning partner that thinks with you, not for you. You describe what you want to build. It asks hard questions, challenges weak assumptions, and grounds decisions in current best practices via live web search. When the plan is solid, it generates five production-ready documents — including a `CLAUDE.md` and phased prompts — that you drop into your project and start building immediately.

**The key insight:** the output isn't advice. It's artifacts. A spec your AI coding tool can follow. Prompts broken into phases. A conversation log so you remember _why_ you made each decision.

---

## Features

**Conversational Planning**
Natural dialogue that challenges and refines your ideas. AuraForge pushes back on weak assumptions, suggests alternatives, and tracks what's been decided vs. what's still open.

**Grounded in Reality**
Integrated web search (Tavily + DuckDuckGo fallback) injects current best practices, version info, and real-world trade-offs directly into the conversation. No hallucinated package names.

**Five Output Documents**
One click generates a complete planning package: `README.md`, `SPEC.md`, `CLAUDE.md`, `PROMPTS.md`, and `CONVERSATION.md` — each cross-referenced and immediately usable.

**Local-First & Private**
Powered by Ollama. Your conversations, plans, and data stay on your machine. No API keys required to get started (optional for web search).

**Instant to Claude Code**
Generated `CLAUDE.md` drops into your project root. `PROMPTS.md` gives you phased implementation steps. Copy, paste, build.

**The Forge Aesthetic**
A dark, atmospheric UI built around the metaphor of crafting — ember particles, thermal gradients, and purposeful animations that make planning feel like creation.

---

## How It Works

<p align="center">
  <img src="screenshots/workflow.png" alt="AuraForge workflow" width="680" />
</p>

| Step | What Happens |
|------|-------------|
| **1. Describe your idea** | Start a new session and explain what you want to build in plain language. |
| **2. Refine through dialogue** | AuraForge asks clarifying questions — scope, tech choices, trade-offs. Web search provides current context. |
| **3. Converge on decisions** | The conversation narrows from broad idea to concrete plan. Tech stack, architecture, phases — all decided collaboratively. |
| **4. Forge the plan** | Click "Forge the Plan" to generate all five documents from your conversation. |
| **5. Save and build** | Export to a folder. Drop `CLAUDE.md` in your project. Follow `PROMPTS.md` phase by phase. |

---

## Tech Stack

| Layer | Technology | Why |
|-------|-----------|-----|
| **Desktop Runtime** | Tauri 2.0 | ~5MB binary vs Electron's 150MB+. Native OS integration, Rust-powered backend. |
| **Frontend** | React 19 | Component-driven UI with streaming state management. |
| **State** | Zustand | Minimal boilerplate, excellent TypeScript support, no provider wrapping. |
| **Backend** | Rust | Memory safety, performance, and Tauri's native IPC. |
| **LLM** | Ollama | Local inference, no API costs, full privacy. Supports any GGUF model. |
| **Database** | SQLite (rusqlite) | Embedded, zero-config, handles sessions/messages/documents. |
| **Search** | Tavily + DuckDuckGo | Tavily for quality, DDG as free fallback. Automatic provider failover. |
| **Styling** | Tailwind CSS 4 | Utility-first with `@theme` design tokens for the forge aesthetic. |
| **Markdown** | react-markdown + remark-gfm | Full GFM rendering with syntax-highlighted code blocks. |

---

## Architecture

```
┌────────────────────────────────────────────────────────┐
│                  AuraForge Desktop App                  │
├──────────────────────┬─────────────────────────────────┤
│    React Frontend    │         Rust Backend            │
│                      │                                 │
│  ┌──────────────┐    │    ┌─────────────────────┐      │
│  │   Zustand     │◄──IPC──►│    Tauri Commands    │      │
│  │   Store       │    │    └──┬──────┬──────┬───┘      │
│  └──────┬───────┘    │       │      │      │           │
│         │            │       ▼      ▼      ▼           │
│  ┌──────▼───────┐    │  ┌──────┐ ┌────┐ ┌──────┐      │
│  │  Components   │    │  │SQLite│ │LLM │ │Search│      │
│  │  - Sidebar    │    │  │  DB  │ │    │ │      │      │
│  │  - Chat       │    │  └──────┘ └──┬─┘ └──┬───┘      │
│  │  - DocPreview │    │              │      │           │
│  │  - Settings   │    │              ▼      ▼           │
│  └──────────────┘    │          ┌──────┐ ┌──────────┐  │
│                      │          │Ollama│ │Tavily/DDG│  │
│                      │          │(local)│ │ (web)   │  │
│                      │          └──────┘ └──────────┘  │
└──────────────────────┴─────────────────────────────────┘
```

**Data Flow:** User message → SQLite → Trigger detection → (optional) Web search → Ollama streaming → SQLite → Frontend update

All IPC uses Tauri's type-safe command system. Streaming responses are delivered via event emitters. Session-switch guards prevent stale state from async race conditions.

---

## Getting Started

### Prerequisites

- **macOS** with Apple Silicon (Intel works too, just slower inference)
- **Ollama** installed and running ([ollama.com](https://ollama.com))
- **Node.js** 18+ and **Rust** 1.75+
- _Optional:_ [Tavily API key](https://tavily.com) for higher-quality web search

### Install Ollama and pull a model

```bash
# Install Ollama (if not already)
brew install ollama

# Pull a recommended model
ollama pull qwen3:30b
```

### Clone and run

```bash
git clone https://github.com/saagar210/auraforge.git
cd auraforge
npm install
npm run tauri dev
```

### First run

AuraForge checks for Ollama connectivity and model availability on startup. If anything is missing, the onboarding modal walks you through setup. Once connected, start a new session and describe what you want to build.

---

## Generated Output

When you click **Forge the Plan**, AuraForge generates a folder like this:

```
my-project-plan/
├── README.md           # Quick-start guide — what's in the folder, how to use it
├── SPEC.md             # Complete technical specification
├── CLAUDE.md           # Drop into your project root for Claude Code context
├── PROMPTS.md          # Phased implementation prompts, copy-paste ready
└── CONVERSATION.md     # Full planning transcript for reference
```

### What each document does

| Document | Purpose |
|----------|---------|
| **README.md** | Orientation. Explains the folder, lists key decisions, tells you where to start. |
| **SPEC.md** | Full specification — data models, API surface, architecture, edge cases. |
| **CLAUDE.md** | Project-level context for Claude Code. Drop it in your repo root and Claude understands your project. |
| **PROMPTS.md** | Step-by-step implementation broken into phases. Each prompt is scoped and self-contained. |
| **CONVERSATION.md** | The raw planning dialogue. Revisit to understand _why_ decisions were made. |

These aren't templates or boilerplate. Every document is generated from your specific conversation, referencing your chosen tech stack, architecture, and requirements.

---

## Design Philosophy

AuraForge's interface is built around **"The Forge"** — a visual metaphor for crafting something powerful from raw material.

<p align="center">
  <img src="screenshots/ui-detail.png" alt="AuraForge UI" width="680" />
</p>

The dark canvas isn't cold — warm ember tones and subtle particle effects create an atmosphere of focused creation. Key moments like document generation get visual ceremony (thermal gradients, progress overlays), while routine interactions stay minimal.

Design principles:
- **Atmosphere over decoration** — every effect serves the metaphor
- **Warmth in darkness** — warm undertones throughout, not sterile dark mode
- **Readability is sacred** — atmospheric styling never compromises legibility
- **Ceremony for significance** — forging a plan deserves more visual weight than sending a message

---

## Roadmap

- [ ] Windows and Linux builds
- [ ] Multiple LLM providers (Anthropic API, OpenAI) alongside Ollama
- [ ] Project templates for common app types
- [ ] Import existing codebases for refactoring plans
- [ ] Export to GitHub Issues / Linear integration

---

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

If you find a bug or have a feature idea, [open an issue](https://github.com/saagar210/auraforge/issues).

---

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

---

## Acknowledgments

- [Anthropic](https://anthropic.com) — Claude and Claude Code, the tools this project is built to complement
- [Tauri](https://tauri.app) — for proving desktop apps don't need Electron's footprint
- [Ollama](https://ollama.com) — making local LLM inference accessible
- [Tavily](https://tavily.com) — web search API for grounded AI responses

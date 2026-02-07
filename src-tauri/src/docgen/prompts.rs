pub const DOCGEN_SYSTEM_PROMPT: &str = r##"You are a document generator for AuraForge. You transform planning conversations into specific, actionable documentation for AI coding tools.

## CRITICAL RULES

### What You Must Do
1. Use ONLY information explicitly discussed or decided in the conversation
2. Use exact names, versions, and details from the conversation
3. Mark anything undiscussed as "[TBD — not discussed during planning]"
4. Use today's date: {current_date}
5. Write in the actual programming language of the project (Rust structs for Rust projects, TypeScript interfaces for TS projects — never pseudocode JSON)
6. Cross-reference previously generated documents when provided

### What You Must NEVER Do
1. Invent features, requirements, or technologies not discussed
2. Generate performance metrics unless specific numbers were stated ("sub-100ms" requires the user to have said "100ms")
3. List file paths or directory structures that weren't agreed upon
4. Generate verification commands that don't match the tech stack (no `curl localhost` for desktop apps, no `npm test` for Rust-only projects)
5. Fabricate example data, API keys, or configuration values
6. Include a section with made-up content — use [TBD] instead

### Negative Examples (DO NOT do these)
- User discussed a Tauri desktop app → DON'T generate `curl http://localhost:3000/api/health`
- User didn't discuss database schema → DON'T invent tables and columns
- User said "handle errors" without specifics → DON'T write "Returns 404 for not found, 500 for server errors"
- User mentioned "testing" without details → DON'T generate specific test cases with expected values
- User mentioned a term you're unsure about → DON'T silently interpret it; mark as "[TBD — unclear from conversation: '{term}']"

### Positive Examples (DO these)
- User said "use SQLite with rusqlite" → Write Rust structs with rusqlite types
- User said "I want streaks for habits" → Write specific acceptance criteria about streak calculation
- User didn't mention error handling → Add an Error Handling section with "[TBD — not discussed during planning. Recommend discussing before Phase 3.]"
- User discussed 3 features → Only include those 3 features, no "nice to haves" unless they said them

### Tech Stack Consistency Check
Before finalizing any document, verify:
- All mentioned technologies match the agreed stack
- All commands use the correct package manager / build tool
- All verification steps are executable on the target platform
- No web-server patterns appear for desktop/CLI apps
- No desktop patterns appear for web apps

When extracting information, categorize as:
- **Decided**: User explicitly chose this → Include with confidence
- **Implied**: Reasonable inference from context → Include but note assumption
- **Unknown**: Not discussed → Mark [TBD] with recommendation to discuss"##;

pub const SPEC_PROMPT: &str = r##"Generate SPEC.md based on the planning conversation.

## Structure

### 1. Overview
- Project name, one-line description
- Who it's for (specific user types from conversation)
- Why it exists (problem it solves)

### 2. Goals
- Only goals explicitly stated or clearly implied
- Be specific: "Track daily habits with streak counting" not "Provide habit tracking functionality"

### 3. Non-Goals (Explicitly Out of Scope)
- Things ruled out or deferred during conversation
- If scope boundaries weren't discussed: "[TBD — scope boundaries not discussed. Recommend defining before implementation.]"

### 4. User Stories
- Convert discussed features into: "As a [user type], I want to [specific action] so that [concrete benefit]"
- Only for features actually discussed

### 5. Technical Architecture

#### Tech Stack
| Layer | Technology | Version | Rationale |
- Only include technologies explicitly chosen
- Include versions if mentioned; otherwise "[latest stable]"

#### System Design
- ASCII diagram ONLY if architecture was discussed in enough detail
- If not: "[TBD — system architecture not discussed in detail]"

#### Interface Contract
THIS IS CRITICAL. Based on the tech stack, generate the contract between components:
- **Tauri apps**: List every IPC command with Rust function signature (input types → return type)
- **Web apps (REST)**: List every endpoint (method, path, request body, response body)
- **Web apps (GraphQL)**: List queries and mutations with types
- **CLI tools**: List every command with flags and output format
- **Libraries**: List public API surface

If the conversation covered enough detail, write real signatures:
```rust
// Example for Tauri:
#[tauri::command]
async fn create_habit(name: String, frequency: HabitFrequency) -> Result<Habit, AppError>
```

If not enough detail: "[TBD — command interface not discussed. Define during Phase 1.]"

#### Data Models
- Write in the project's ACTUAL language:
  - Rust project → Rust structs with derive macros
  - TypeScript project → TypeScript interfaces
  - Python project → Pydantic models or dataclasses
- NEVER use pseudocode JSON for data models
- Include relationships between entities
- If data model wasn't discussed: "[TBD — data model not discussed. Critical gap — recommend defining before implementation.]"

### 6. Features
For each feature discussed:

**Feature Name**
- Description: What it does (specific user interaction)
- Acceptance Criteria: How we know it's done (testable statements)
- Edge Cases: What happens when things go wrong (only if discussed)

### 7. Error Handling
- Only include strategies actually discussed
- If not discussed: "[TBD — error handling not discussed. Recommend defining before Phase 3.]"

### 8. Security Considerations
- Only if discussed
- If not: "[TBD — security not discussed]"

### 9. Open Questions
- Any unresolved items from the conversation
- Any [TBD] items collected from above sections, consolidated here with recommendations

<previously_generated_documents>
{previously_generated_docs}
</previously_generated_documents>

<conversation>
{conversation_history}
</conversation>

Generate SPEC.md now:"##;

pub const CLAUDE_PROMPT: &str = r##"Generate CLAUDE.md — the file that Claude Code reads every interaction to understand the project.

This is the MOST IMPORTANT document for execution quality. Every detail here prevents a wrong guess by Claude Code.

## Structure

### Project Name
One-line description.

### Tech Stack
Exact technologies with versions:
```
- Frontend: React 19 + TypeScript 5.x
- Styling: Tailwind CSS 4.x
- State: Zustand 5.x
- Backend: Rust (latest stable via rustup)
- Framework: Tauri 2.0
- Database: SQLite via rusqlite 0.32
- LLM: Ollama (localhost:11434)
```
Only include technologies explicitly decided. No "maybe later" items.

### Project Structure
```
project-root/
├── src/                    # React frontend
│   ├── components/         # Reusable UI components
│   ├── views/              # Page-level components
│   ├── stores/             # Zustand stores
│   ├── hooks/              # Custom React hooks
│   └── App.tsx             # Entry point
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── lib.rs          # Tauri command registration
│   │   ├── main.rs         # App entry
│   │   ├── commands/       # IPC command handlers
│   │   ├── db/             # Database operations
│   │   └── models/         # Data structures
│   └── Cargo.toml
└── package.json
```
Generate this ONLY from the tech stack. Different stacks get different structures:
- Tauri → src/ + src-tauri/
- Next.js → app/ or pages/
- CLI → src/ with main entry point
- Do NOT include directories that don't make sense for the stack

### Commands
Exact commands for this specific tech stack:
```bash
# Install
npm install                      # Frontend dependencies
cd src-tauri && cargo build      # Rust dependencies

# Development
npm run tauri dev                # Launch app in dev mode

# Build
npm run tauri build              # Production build

# Test
cargo test                       # Rust tests
npm run test                     # Frontend tests (if configured)

# Lint
cargo clippy                     # Rust linting
npm run lint                     # Frontend linting (if configured)
```
CRITICAL: Commands must match the tech stack. Tauri apps use `npm run tauri dev`, not `npm run dev`. Rust projects use `cargo test`, not `npm test`.

### Code Conventions
- File naming: [convention from conversation, or default for stack]
- Component pattern: [discussed pattern, or stack default]
- Error handling: [discussed approach, or "[TBD — use Result<T, AppError> pattern until defined]"]
- Import ordering: [if discussed]

### Anti-Patterns — DO NOT
- [Stack-specific anti-patterns]
Examples for Tauri:
- Do NOT use HTTP servers for IPC — use Tauri commands
- Do NOT use `unwrap()` in production Rust code — use `?` with proper error types
- Do NOT create separate REST API — frontend communicates via Tauri invoke()
Examples for React:
- Do NOT use Redux — use Zustand (decided in planning)
- Do NOT use class components — use functional components with hooks
Generate these based on the tech stack and any "don't do X" statements from conversation.

### Important Notes
- [Project-specific gotchas from conversation]
- [Any constraints or requirements mentioned]
- [Dependencies or prerequisites (e.g., "Ollama must be running")]

<previously_generated_documents>
{previously_generated_docs}
</previously_generated_documents>

<conversation>
{conversation_history}
</conversation>

Generate CLAUDE.md now:"##;

pub const PROMPTS_PROMPT: &str = r##"Generate PROMPTS.md — the step-by-step implementation guide for Claude Code.

Each phase is a self-contained unit of work. A user should be able to copy ONE phase into Claude Code and get working software without reading any other phase.

CRITICAL: Reference the CLAUDE.md and SPEC.md that were generated earlier (provided in <previously_generated_documents>). Prompts must use the exact tech stack, commands, and conventions from those documents.

## Phase Structure

Every phase follows this exact format:

---

## Phase N: [Descriptive Name]
**Complexity:** [Straightforward scaffolding | Moderate | Most complex phase — pay close attention]

### Context
What previous phases built. What exists in the codebase right now.
"After Phase 2, you have: working chat UI, session persistence, message display with markdown rendering."

### Objective
One sentence. What this phase accomplishes.
"Add real-time web search integration that triggers during tech-related conversation topics."

### Prerequisites
What must exist and be working before starting:
- "Phase 2 complete — chat UI sends and receives messages"
- "`npm run tauri dev` launches without errors"
- "Ollama running with [model] pulled"

### Implementation Details

**Files to create/modify:**
List every file with its purpose:
- `src-tauri/src/search/mod.rs` — Search client trait and Tavily implementation
- `src-tauri/src/search/tavily.rs` — Tavily API client
- `src/components/SearchIndicator.tsx` — "Searching..." UI indicator

**Dependencies to add:**
```toml
# In Cargo.toml
reqwest = { version = "0.12", features = ["json"] }
```
```bash
npm install [package]@[version]
```

**Key implementation notes:**
- [Specific pattern to use]: "Implement search as an async trait so we can swap providers"
- [Specific crate/API details]: "Tavily API endpoint is POST https://api.tavily.com/search with JSON body {api_key, query, max_results}"
- [Edge case to handle]: "Search timeout should be 5 seconds — don't block the conversation if search is slow"

### Prompt for Claude Code
```
[EXACT prompt to paste — self-contained, references CLAUDE.md and SPEC.md]

Read CLAUDE.md for project conventions and SPEC.md for requirements.

[Specific, detailed instructions with file paths, package names, and expected behavior]

[Explicit "do not" instructions for common mistakes]
```

### Verification Checklist
All items must be binary yes/no — a non-technical user can verify each one.

- [ ] `cargo build` completes without errors
- [ ] `npm run tauri dev` launches the app
- [ ] Type a message mentioning "React vs Vue" — search indicator appears
- [ ] Search results influence the AI response
- [ ] App still works without internet (search gracefully fails)
- [ ] Previous features still work: send message, view history, switch sessions

### Watch Out For
Common mistakes and framework gotchas for THIS specific phase:
- "Tauri commands must be registered in `lib.rs` — if you add a new command and it's not callable from the frontend, check registration"
- "reqwest needs tokio runtime — Tauri 2.0 already provides one, don't add another"
- "Tavily free tier: 1000 requests/month. Add rate limiting or the user will burn through it in testing"

---

## Phase Ordering Rules

1. **Phase 1 is always Project Setup**: Initialize project, install dependencies, verify hello world. Include CLAUDE.md placement.

2. **Order by dependency**: Database before features that store data. Backend before frontend that calls it.

3. **Each phase = verifiable milestone**: After each phase, the app should compile, run, and do something testable.

4. **Scale detail to conversation depth**: If the user discussed a feature in detail, the phase prompt should be detailed. If they said "add search" with no specifics, the prompt should note: "Search implementation details were not discussed in planning — make reasonable choices and document them in CLAUDE.md."

5. **Final phase is always Testing & Polish**: This phase is special (see below).

## Final Phase: Testing & Polish

The last phase always follows this structure:

### Phase N: Testing, Polish & Release Readiness

#### Prompt for Claude Code
```
Final phase. Read CLAUDE.md for conventions and SPEC.md for all requirements.

1. **Compilation & Lint**
   - Run [build command] and fix all warnings
   - Run [lint command] and fix all issues

2. **Run Existing Tests**
   - Run [test command]
   - Fix any failures

3. **Feature Verification** (test each feature from SPEC.md)
   [Auto-generated list from features discussed in conversation]
   - [ ] [Feature 1]: [Specific test steps]
   - [ ] [Feature 2]: [Specific test steps]

4. **Error Handling Tests**
   [Auto-generated from error scenarios discussed]
   - [ ] [Error scenario 1]: [Expected behavior]
   - [ ] [Error scenario 2]: [Expected behavior]

5. **Edge Cases**
   - [ ] App launches on first run (no existing data)
   - [ ] App handles [tech-specific edge case]
   - [ ] [Platform-specific checks]

6. **Production Build**
   - Run [production build command]
   - Test the built artifact (install the .dmg/.exe/.AppImage, or deploy to staging)
   - Verify it works the same as dev mode

7. **Bug Fix Protocol**
   For any bugs found:
   - Fix the bug
   - Verify the fix
   - Re-run [test command] to check for regressions
   - Continue to next test

8. **Cleanup**
   - Remove console.log / println! debugging statements
   - Run formatter: [format command]
   - Run linter one final time
   - Update CLAUDE.md if any conventions changed during implementation

9. **Generate Test Report**
   Create TEST_REPORT.md in the project root with:
   - Date and app version
   - Each test case: pass/fail/notes
   - Any known issues
   - Performance observations

10. **Commit & Push**
    - `git add -A`
    - `git commit -m "v0.1.0: [Project Name] - initial release"`
    - `git push origin main`
```

<previously_generated_documents>
{previously_generated_docs}
</previously_generated_documents>

<conversation>
{conversation_history}
</conversation>

Generate PROMPTS.md now:"##;

pub const README_PROMPT: &str = r##"Generate README.md — a planning-stage orientation document.

IMPORTANT: This is NOT the project's source code README. That gets written during implementation. This README explains the PLANNING FOLDER and how to use its contents.

## Structure

# [Project Name] — Planning Documents

Generated by AuraForge on {current_date}

## What's In This Folder

This folder contains everything you need to build [one-sentence description] using your coding model of choice. The planning is done — the documents guide execution.

## Documents

| File | What It Is | When to Use It |
|------|-----------|----------------|
| START_HERE.md | Quick-start guide with your first prompt | **Read this first** |
| SPEC.md | Complete specification — what you're building and why | Reference during implementation for requirements |
| CLAUDE.md | Project configuration for Claude Code | Copy into your project root before starting |
| MODEL_HANDOFF.md | Target-aware handoff notes for your coding model | Read before starting execution |
| PROMPTS.md | Step-by-step implementation phases | Follow one phase at a time in Claude Code |
| CONVERSATION.md | Full planning transcript | Revisit to understand why decisions were made |
| README.md | This file | You're reading it |

## Project Summary

[2-3 sentences: what it is, who it's for, core capability]

## Key Decisions Made

[Bulleted list of major choices from the conversation — tech stack, architecture, scope boundaries. Only decisions that were explicitly made.]

## Known Gaps

[List any [TBD] items from SPEC.md — things that weren't discussed and will need decisions during implementation]

---

Do NOT include:
- Source code directory listings (the project doesn't exist yet)
- Build/install commands (those go in CLAUDE.md)
- Feature descriptions (those are in SPEC.md)
- Fictional file names or paths

<previously_generated_documents>
{previously_generated_docs}
</previously_generated_documents>

<conversation>
{conversation_history}
</conversation>

Generate README.md now:"##;

pub const START_HERE_PROMPT: &str = r##"Generate START_HERE.md — the bridge document for users who are new to AI coding tools or non-technical.

This document turns "I have a planning folder" into "I'm building software." It must be usable by someone who has never opened a terminal.

## Structure

# Start Here — Building [Project Name]

## What You Have

AuraForge has planned your project: **[Project Name]** — [one-line description].

This folder contains your complete plan. You don't need to read everything — just follow the steps below.

## Prerequisites

Before starting, you need:

[Generate based on tech stack from conversation]

For a Tauri app:
- [ ] **Node.js** (version 18+) — [Download here](https://nodejs.org)
- [ ] **Rust** — Install with: open Terminal, paste `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- [ ] **Claude Code** — [Install instructions](https://docs.anthropic.com/en/docs/claude-code)
- [ ] [Any project-specific prereqs, e.g., "Ollama" for LLM apps]

For a web app:
- [ ] **Node.js** (version 18+) — [Download here](https://nodejs.org)
- [ ] **Claude Code** — [Install instructions](https://docs.anthropic.com/en/docs/claude-code)

[Adapt to actual tech stack]

## Step-by-Step Setup

### 1. Create Your Project Folder
Open Terminal (Mac: Cmd+Space, type "Terminal") and run:
```bash
mkdir ~/Projects/[project-name]
cd ~/Projects/[project-name]
```

### 2. Copy Core Planning Files
Copy `CLAUDE.md` and `MODEL_HANDOFF.md` from this planning folder into your new project folder.

### 3. Open Your Coding Tool
```bash
[tool-command]
```
If unsure, check `MODEL_HANDOFF.md` for target-specific guidance.

### 4. Paste Your First Prompt
Copy and paste this into your coding tool:

```
I'm building [Project Name] — [one-line description].

Read CLAUDE.md in this directory for project conventions.
Read MODEL_HANDOFF.md for model-specific execution rules.
Then read the Phase 1 section from the PROMPTS.md file at [planning folder path or: "I'll paste it below"].

Start with Phase 1. Follow the implementation details exactly. When done, run through the verification checklist and tell me the results.
```

### 5. Work Through Each Phase

After Phase 1 is complete and verified, paste this for each subsequent phase:

```
Phase [N-1] is complete and verified. Proceed to Phase [N] from PROMPTS.md.

Read the full Phase [N] section before starting.
After completing Phase [N], verify that all previous phases still work correctly.
Tell me the verification checklist results.
```

## If Something Goes Wrong

**Claude Code says "I don't have enough context":**
```
Read CLAUDE.md and SPEC.md for full project details. The planning documents contain all requirements and conventions.
```

**A phase fails to compile or has errors:**
```
Fix all compilation errors and warnings before proceeding. Run [build command] and resolve every issue.
```

**You're not sure what happened or where things stand:**
```
Summarize what you've built so far, what's working, and what's left to do based on PROMPTS.md.
```

**You want to change something from the plan:**
```
I want to change [X] from the original plan to [Y]. Update CLAUDE.md if needed, then implement the change. Make sure nothing else breaks.
```

## File Reference

| File | What | When |
|------|------|------|
| **START_HERE.md** | This file — your guide | Now (you're reading it) |
| **CLAUDE.md** | Project config for Claude Code | Copy to project folder before starting |
| **MODEL_HANDOFF.md** | Model-specific execution notes | Read before running phases |
| **SPEC.md** | Full specification | When Claude Code needs requirements detail |
| **PROMPTS.md** | Phase-by-phase implementation | Feed one phase at a time to Claude Code |
| **CONVERSATION.md** | Planning transcript | When you want to know WHY a decision was made |
| **README.md** | Planning folder overview | Quick reference for what's in this folder |

---

Generate START_HERE.md now. Adapt all examples to the actual tech stack from the conversation. Do not include generic instructions — every command, path, and prerequisite must match the project.

<previously_generated_documents>
{previously_generated_docs}
</previously_generated_documents>

<conversation>
{conversation_history}
</conversation>"##;

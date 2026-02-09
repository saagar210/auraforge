# Contributing to AuraForge

Thanks for your interest in contributing to AuraForge.

## Development Setup

### Prerequisites

- macOS (primary development platform)
- Node.js 18+
- Rust 1.75+
- Ollama installed and running

### Getting started

```bash
git clone https://github.com/saagar210/auraforge.git
cd auraforge
npm ci
npm run tauri dev
```

### Project structure

```
auraforge/
├── src/                    # React frontend
│   ├── components/         # UI components
│   ├── stores/             # Zustand state management
│   └── types.ts            # Shared TypeScript types
├── src-tauri/              # Rust backend
│   └── src/
│       ├── commands/       # Tauri command handlers
│       ├── db/             # SQLite database layer
│       ├── docgen/         # Document generation
│       ├── llm/            # Ollama client
│       └── search/         # Web search (Tavily + DuckDuckGo)
├── DESIGN.md               # Design system documentation
└── SPEC.md                 # Product specification
```

## Making Changes

1. Create a branch from `main`
2. Make your changes
3. Run checks before committing:

```bash
# Rust
cd src-tauri
cargo fmt
cargo clippy
cargo test
cd ..

# TypeScript
npx tsc --noEmit
```

4. Commit with a descriptive message following conventional commits:
   - `feat:` for new features
   - `fix:` for bug fixes
   - `docs:` for documentation
   - `chore:` for maintenance

5. Open a pull request against `main`

## Code Style

- **Rust:** Follow `cargo fmt` and `cargo clippy` defaults
- **TypeScript:** Strict mode, explicit types at function boundaries
- **CSS:** Tailwind utility classes, use design tokens from `@theme`
- **Components:** Functional React components with hooks

## Reporting Issues

When filing a bug report, include:

- Steps to reproduce
- Expected vs actual behavior
- Ollama model and version
- OS and version

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

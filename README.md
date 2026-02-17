# AuraForge

## Development Modes

### Normal dev
Use normal mode for fastest repeat startup because caches are kept in the repo.

```bash
npm run dev:tauri
```

### Lean dev (low disk)
Lean mode runs the same app startup command, but moves heavy transient build caches to a temporary directory and removes them automatically when the process exits.

```bash
npm run dev:lean
```

## Cleanup Commands

### Remove heavy build artifacts only
Keeps dependencies (`node_modules`) but removes large generated outputs.

```bash
npm run clean:heavy
```

Removes:
- `dist`
- `src-tauri/target`
- `node_modules/.vite`
- `.vite`

### Full local cleanup (all reproducible caches)
Use when you need maximum disk recovery. Next startup/install will be slower.

```bash
npm run clean:all-local
```

Removes:
- `node_modules`
- `dist`
- `src-tauri/target`
- `.vite`

## Tradeoffs

- Normal dev: faster restart speed, higher local disk usage over time.
- Lean dev: lower disk usage, but slower startup/rebuild because cache is not reused.
- `clean:heavy`: good daily cleanup balance.
- `clean:all-local`: biggest disk reclaim, slowest next run.

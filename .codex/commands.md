# AuraForge .codex command map

| Action | Command | Source |
| --- | --- | --- |
| setup deps | `npm install` | `package-lock.json` convention, `package.json` |
| lint fallback | `npm run build` | `package.json` (no dedicated lint script) |
| test | _none configured (blocks by default as NOT_RUN; bypass only with `CODEX_ALLOW_NOT_RUN_GATES=1` + explicit risk acceptance)_ | `package.json`, `.github/workflows` |
| build | `npm run build` | `package.json` |
| lean dev | `npm run dev:lean` | `README.md`, `package.json` |

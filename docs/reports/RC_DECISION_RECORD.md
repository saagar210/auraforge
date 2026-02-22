# RC Decision Record

## Decision Metadata
- Decision status: `No-go`
- Decision date: `2026-02-22`
- Decision timestamp (UTC): `2026-02-22T12:01:17Z`
- Decision owners: `AuraForge PM` (primary), `AuraForge Eng` (backup)
- Candidate label: `RC1-phase4-week4-close`
- Target channel: `qa`
- Follow-on track: `Week 5 Track B (remediation sprint)`

## Required Evidence Fields
- Latest signed workflow run URL: `https://github.com/saagar210/auraforge/actions/runs/22276721898`
- Latest signed workflow run ID: `22276721898`
- Latest signed workflow result: `failure` (missing required `APPLE_*` secrets)
- Prior signed workflow run URL: `https://github.com/saagar210/auraforge/actions/runs/22276565984`
- Signed artifact ID/name: `N/A (signed run terminated before artifact build)`
- Signed artifact app path: `N/A (signed run terminated before artifact build)`
- Signed artifact dmg path: `N/A (signed run terminated before artifact build)`
- Signed artifact app SHA256: `N/A (signed run terminated before artifact build)`
- Signed artifact dmg SHA256: `N/A (signed run terminated before artifact build)`
- Signing identity: `N/A (signed run terminated before signing)`
- Notarization status: `N/A (signed run terminated before notarization)`
- Unsigned control run URL: `https://github.com/saagar210/auraforge/actions/runs/22276565971`
- Unsigned artifact ID/name: `5606803423 / auraforge-5-unsigned-qa`
- Unsigned artifact digest: `sha256:169ebd16b7dcca951c1776600859f0018b76ea68a78e4c6ec5286dfe0de89e21`

## Gate Results (Pass/Fail Contract)
| Gate | Source | Status | Evidence |
| --- | --- | --- | --- |
| Engineering baseline (`run_verify_commands.sh`) | `.codex/verify.commands` | Pass | Local run passed on 2026-02-22 |
| Web tests (`npm run test:web`) | `package.json` | Pass | Included in verify run on 2026-02-22 |
| Smoke tests (`npm run test:smoke`) | `package.json` | Pass | Deterministic local smoke lane passed on 2026-02-22 |
| Rust tests (`cargo test --manifest-path src-tauri/Cargo.toml`) | `package.json` | Pass | Included in verify run on 2026-02-22 |
| Security (`npm audit --json`) | `docs/release/RC_CHECKLIST.md` | Pass | 0 vulnerabilities on 2026-02-22 |
| Signed CI release (`release-rc` with `require_signed=true`) | `.github/workflows/release-rc.yml` | Fail | Latest run `22276721898` failed at signing prerequisite enforcement |
| Signed artifact verification (`codesign`/`spctl`/`stapler`) | `.github/workflows/release-rc.yml`, `scripts/release/verify-macos-artifact.sh` | Not run | Signed build blocked before verification steps |
| Critical-path signed artifact smoke | `docs/release/SIGNED_SMOKE_CHECKLIST.md` | Not run | No signed artifact available |
| Unsigned control release (`release-rc` with `require_signed=false`) | `.github/workflows/release-rc.yml` | Pass | Run `22276565971` produced unsigned QA artifact |

## Recommendation
- Current recommendation: `No-go`
- Rationale:
1. Signed CI gate is functioning correctly but blocked by missing required `APPLE_*` secrets.
2. Signed artifact verification and signed smoke are both blocked by the missing-secrets prerequisite.
3. Engineering and security baselines are stable, so remaining work is isolated to signing/notarization closure.

## Open Blockers
1. Configure required `APPLE_*` secrets in repository settings.
2. Re-run signed release workflow (`channel=qa`, `require_signed=true`) and collect signing + notarization evidence.
3. Execute signed critical-path smoke checklist on the signed artifact.
4. Re-run `npm run phase4:gates`; update this record to `Go` only if all required signed gates pass.

## Execution Notes (2026-02-22)
1. Command: `gh pr view 12 -R saagar210/auraforge --json state,mergedAt,mergeCommit`
   - Result: PR merged at `2026-02-22T11:41:01Z`; merge commit `09b4add63251e18d35d4c08cb8d21881339192cb`.
2. Command: `gh workflow list -R saagar210/auraforge --json name,path,state`
   - Result: `release-rc` workflow is active on default branch.
3. Command: `gh secret list -R saagar210/auraforge --json name,updatedAt`
   - Result: `[]` (required `APPLE_*` secrets missing).
4. Command: `gh workflow run release-rc.yml -R saagar210/auraforge --ref main -f channel=qa -f require_signed=true`
   - Result: latest run `22276721898` failed early with missing-secrets enforcement.
5. Command: `gh workflow run release-rc.yml -R saagar210/auraforge --ref main -f channel=qa -f require_signed=false`
   - Result: run `22276565971` succeeded; unsigned artifact uploaded.
6. Command: `bash .codex/scripts/run_verify_commands.sh`, `npm audit --json`, `npm run phase4:gates`
   - Result: engineering and security lanes pass; release prerequisite lane fails due missing secrets.

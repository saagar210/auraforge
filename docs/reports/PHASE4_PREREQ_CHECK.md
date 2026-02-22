# Phase Four Prerequisite Check

## Snapshot
- Timestamp (UTC): `2026-02-22T12:01:17Z`
- Status: `Blocked`
- Week 4 closure outcome: `No-go`
- Week 5 track: `Track B (remediation sprint)`

## Required Prerequisites
| Prerequisite | Status | Evidence | Owner |
| --- | --- | --- | --- |
| `release-rc` workflow available on default branch (`main`) | Pass | `gh workflow list -R saagar210/auraforge --json name,path,state` includes `.github/workflows/release-rc.yml` as `active` | AuraForge Eng |
| PR `#12` merged to `main` | Pass | `gh pr view 12 -R saagar210/auraforge --json state,mergedAt,mergeCommit` shows `state=MERGED`, `mergedAt=2026-02-22T11:41:01Z` | AuraForge PM + reviewer |
| Required `APPLE_*` repo secrets configured | Fail | `gh secret list -R saagar210/auraforge --json name,updatedAt` returned `[]` | AuraForge PM |
| Signed release dispatch works (`require_signed=true`) | Fail | Latest run `22276721898` failed in `Enforce signed mode when required` due missing `APPLE_*` secrets | AuraForge PM + Eng |
| Unsigned control release path works (`require_signed=false`) | Pass | Run `22276565971` succeeded; artifact `auraforge-5-unsigned-qa` (ID `5606803423`) uploaded | AuraForge Eng |
| Local baseline gates passing | Pass | `bash .codex/scripts/run_verify_commands.sh` passed | AuraForge Eng |
| Security baseline clean | Pass | `npm audit --json` shows 0 vulnerabilities | AuraForge Eng |

## Blocking Actions (Ordered)
1. Provision all required `APPLE_*` secrets listed in `docs/release/SECRETS_INVENTORY.md`.
2. Re-run `npm run phase4:prereqs` and require pass.
3. Re-run `release-rc` with `require_signed=true` and capture signature/notarization evidence.
4. Execute `docs/release/SIGNED_SMOKE_CHECKLIST.md` against the signed artifact.
5. Re-run `npm run phase4:gates`; close Week 5 Track B only when signed gates pass.

## Command Evidence
| Command | Result | Notes |
| --- | --- | --- |
| `gh pr view 12 -R saagar210/auraforge --json state,mergedAt,mergeCommit,baseRefName,headRefName,url` | Pass | PR merged to `main`; merge commit `09b4add63251e18d35d4c08cb8d21881339192cb` |
| `gh workflow list -R saagar210/auraforge --json name,path,state` | Pass | `release-rc` now listed and active |
| `gh secret list -R saagar210/auraforge --json name,updatedAt` | Fail (prereq) | No required APPLE secrets present |
| `npm run phase4:prereqs` | Fail | Only failing lane is missing required `APPLE_*` secrets |
| `gh run view 22276721898 -R saagar210/auraforge --log-failed` | Fail (expected) | Signed-required run fails early with explicit missing-secrets message |
| `gh run view 22276565971 -R saagar210/auraforge --json status,conclusion,url` | Pass | Unsigned control run completed successfully |
| `gh api repos/saagar210/auraforge/actions/runs/22276565971/artifacts` | Pass | Artifact ID `5606803423`, digest `sha256:169ebd16...` |
| `bash .codex/scripts/run_verify_commands.sh` | Pass | Engineering baseline green |
| `npm audit --json` | Pass | Security baseline green |
| `npm run phase4:gates` | Fail | Engineering + security pass; release prerequisite lane fails |

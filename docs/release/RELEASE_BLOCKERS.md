# Release Blockers

## Open Blockers

| Blocker | Impact | Owner | Mitigation | Target Resolution | Status |
| --- | --- | --- | --- | --- | --- |
| Required `APPLE_*` secrets missing (`APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`) | Signed/notarized release path is blocked | AuraForge PM | Configure all secret values per `docs/release/SECRETS_INVENTORY.md` | 2026-02-23 | Open |
| Signed release lane fails when `require_signed=true` | Cannot produce signed RC artifact or pass release gate | AuraForge Eng + PM | Re-run `release-rc` after secrets provisioning; require pass on signing/notarization checks | 2026-02-23 | Open |
| Signed critical-path smoke not executed on signed artifact | Go decision cannot be made | AuraForge Eng | Run `docs/release/SIGNED_SMOKE_CHECKLIST.md` on signed artifact and attach evidence | 2026-02-23 | Open |

## Resolved Today (2026-02-22)

| Item | Evidence | Status |
| --- | --- | --- |
| PR `#12` merged to `main` | `gh pr view 12 -R saagar210/auraforge --json state,mergedAt,mergeCommit` | Closed |
| `release-rc` published on default branch | `gh workflow list -R saagar210/auraforge --json name,path,state` shows `.github/workflows/release-rc.yml` active | Closed |
| Unsigned control release path validated | Run `22276565971` succeeded; artifact `auraforge-5-unsigned-qa` ID `5606803423` | Closed |

## Week 4 Outcome
- Week 4 closed as: `No-go`
- Follow-on path: `Week 5 Track B (remediation sprint)`

## Exit Criteria for Blocker Closure
- [x] PR `#12` merged to `main`.
- [x] `release-rc` appears in default-branch workflow list.
- [ ] All six required `APPLE_*` secrets are present.
- [ ] Signed CI RC artifact produced and verifiably signed/notarized.
- [ ] Signed smoke checklist passes critical path end-to-end.

## Note
- `main` branch protection remains disabled by repository-owner direction for this closure sequence.

# SpecLint + PromptLint Rules

## Severity model

- `critical`: blocks generation when `output.lint_mode=fail_on_critical`
- `warning`: non-blocking, must be reviewed
- `info`: advisory only

## Implemented rules

- `tbd_leftover`
  - Detects unresolved `[TBD ...]` markers.
  - Critical for core execution docs.
- `missing_acceptance_criteria`
  - SPEC has feature sections but no acceptance criteria.
- `inconsistent_project_naming`
  - Top-level heading naming mismatch across docs.
- `vague_requirements`
  - Detects vague language (`robust`, `scalable`, etc.).
- `missing_verification_steps`
  - Missing required verification checklist structure.

## Outputs

- `LINT_REPORT.md`
- `ARTIFACT_DIFF.json` includes lint summary linkage via generation run metadata.

## Override behavior

If critical findings exist:
- default: fail and require `force=true`
- optional: set `output.lint_mode=warn`

# Artifact Diffing and Changelog

AuraForge compares current generated artifacts to the previous run for the same session.

## Outputs

- `ARTIFACT_CHANGELOG.md` (human-readable)
- `ARTIFACT_DIFF.json` (machine-readable)

## Diff statuses

- `added`
- `removed`
- `changed`
- `unchanged`

## Generation run metadata

Each generation persists:
- `run_id`
- `target`
- `provider`
- `model`
- input fingerprint
- lint/diff summaries

## Verification

- Diff output ordering is deterministic.
- Hashes and manifest metadata are deterministic for unchanged artifacts.

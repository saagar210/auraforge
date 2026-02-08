# Repo Ingest Mode

Repo ingest is local-first and evidence-grounded.

## Outputs

- Architecture summary
- Risks / gaps checklist
- Phased implementation plan
- Verification plan

## Grounding contract

Claims must cite local repository evidence.

Citation shape:
- `path`
- `line_start` (optional when unknown)
- `line_end` (optional when unknown)
- `snippet`

When evidence is missing, outputs must use `[TBD]` and explain what is missing.

## Safety constraints

- Symlink traversal blocked
- Read depth and byte budgets capped
- Binary files excluded

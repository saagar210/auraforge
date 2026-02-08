# Export Preset Definitions

AuraForge export presets are target-aware and deterministic.

## Supported presets

- `codex`
- `claude_code`
- `cursor`
- `generic_agent`

## Folder layout (all presets)

- `docs/` core planning docs
- `handoff/` model handoff + execution checklist
- `context/` conversation transcript
- `reports/` lint and artifact diff reports
- `manifest.json` root metadata and hashes

## Manifest schema compatibility

- Current schema: `v3`
- Backward compatibility floor: `v2`
- AuraForge validates schema compatibility in tests to prevent accidental breaking changes for downstream handoff consumers.

## Required handoff content

`handoff/MODEL_HANDOFF.md` and `handoff/EXECUTION_CHECKLIST.md` must include verification steps and lint gates.

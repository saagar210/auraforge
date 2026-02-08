# Template Authoring Guide

AuraForge templates define deterministic planning starters.

## Template schema

Required keys:
- `id`
- `name`
- `description`
- `target_stack`
- `version`
- `seed_prompt`

Optional keys:
- `recommended_target`
- `required_sections`
- `verification_focus`

## Authoring rules

- Keep `seed_prompt` concrete and scoped to one planning mode.
- Include explicit verification language.
- Avoid ambiguous language. If unknown, mark `[TBD]`.
- Keep template IDs stable; treat ID changes as breaking.

## Validation checklist

- Template parses via `list_templates`.
- Session bootstrap works via `create_session_from_template`.
- Generated docs include verification instructions.
- Lint report has zero critical findings for template fixture runs.

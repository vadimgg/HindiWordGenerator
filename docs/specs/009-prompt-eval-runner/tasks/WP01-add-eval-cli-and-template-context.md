---
id: WP01
title: Add eval CLI and template context
agent_type: rust-engineer
status: planned
dependencies: []
acceptance_refs: [AC01, AC03, AC04, AC05]
extra_skills: []
read_scope:
  - src/cli.rs
  - src/source_ids.rs
  - src/sentence_plan.rs
write_scope:
  - Cargo.toml
  - src/cli.rs
  - src/eval.rs
  - src/main.rs
protected_scope: []
validation:
  - cargo test cli
  - cargo test eval
manual_validation_reason: null
created_at: 2026-05-15T09:38:00.000000+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP01 - Add eval CLI and template context

## Goal

Add the `hindi eval` command shape and pure template-context machinery. This
work package does not need to contact Ollama yet; it should prove YAML input,
field selection, `--max-items`, and Handlebars rendering work.

## Done When

- CLI parses `hindi eval --input <path> --prompt <path> [--fields <list>] [--max-items <n>]`.
- Help text documents the command and does not mention `--model`.
- Eval context exposes `input_yaml`, `items_yaml`, structured `items`,
  `input_path`, and `prompt_path`.
- Missing selected fields fail clearly.
- `cargo test cli` and `cargo test eval` pass.

## Must Not

- Do not call Ollama in WP01.
- Do not write `output/`.
- Do not implement nested selector syntax.

## Handoff Notes

Use Handlebars for runtime prompt templates. The prompt examples should prefer
`{{#each items}}` so one-item and many-item prompts use the same structure.

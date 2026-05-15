---
id: WP03
title: Seed sentence eval prompts and smoke test
agent_type: rust-engineer
status: planned
dependencies: [WP01, WP02]
acceptance_refs: [AC08]
extra_skills: []
read_scope:
  - docs/ROMANISATION.md
  - generation_prompt_sentences_enrichment.txt
write_scope:
  - prompts/sentences/
protected_scope: []
validation:
  - make check
manual_validation_reason: "Live eval smoke requires exactly one Ollama model running and should be recorded in the final handoff."
created_at: 2026-05-15T09:38:02.000000+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP03 - Seed sentence eval prompts and smoke test

## Goal

Add reusable sentence prompt templates for the sub-tasks we want to compare and
run one live eval smoke when a single Ollama model is available.

## Done When

- Prompt templates exist for source QA, English translation, literal
  translation, register, word breakdown, word breakdown from existing
  translation, and full enrichment.
- Templates use `{{#each items}}` and include Hindi with romanisation directly
  underneath wherever Hindi appears.
- `make check` passes.
- One live `hindi eval` smoke result is recorded in the final handoff, or the
  reason it was skipped is recorded.

## Must Not

- Do not tune prompts into a large benchmark suite in this WP.
- Do not add evaluator-agent scoring.
- Do not commit generated `eval/` run folders.

## Handoff Notes

Keep prompt templates practical and easy to edit. The goal is a useful prompt
workbench, not perfect prompt quality on the first pass.

---
id: WP01
title: Normalize model tokens
agent_type: rust-engineer
status: planned
dependencies: []
acceptance_refs: [AC01, AC02, AC04]
extra_skills: []
read_scope:
  - src/sentence_enrichment.rs
  - src/sentence_validate.rs
write_scope:
  - src/sentence_enrichment.rs
protected_scope: []
validation:
  - cargo test sentence_enrichment
  - cargo test sentence_validate
manual_validation_reason: null
created_at: 2026-05-15T08:56:00.610976+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP01 - Normalize model tokens

## Goal

Remove non-word entries from model-provided `tokens[]` during enrichment merge,
before strict validation runs. The validator remains unchanged and must still
reject punctuation/space tokens if they reach candidate validation.

## Done When

- Model enrichment output with punctuation tokens is normalized to word-only
  `tokens[]`.
- Existing trusted source merge behavior is preserved.
- Unit tests cover punctuation removal.
- `cargo test sentence_enrichment` and `cargo test sentence_validate` pass.

## Must Not

- Do not loosen `sentence_validate`.
- Do not invent missing `words[]` entries or rewrite word IDs.
- Do not modify accepted output fixtures.

## Handoff Notes

The live `translategemma:12b` run failed because the model emitted `?` and `.`
as token entries. Removing only non-word token entries should allow the existing
romanisation reconstruction validator to work from true words.

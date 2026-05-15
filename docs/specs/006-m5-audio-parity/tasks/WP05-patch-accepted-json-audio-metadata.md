---
id: WP05
title: Patch accepted JSON audio metadata
agent_type: rust-engineer
status: planned
dependencies: [WP04]
acceptance_refs: [AC03, AC07, AC08, AC09, AC10]
extra_skills: []
read_scope:
  - src/sentence_audio.rs
  - src/sentence_schema.rs
  - src/accepted_writer.rs
write_scope:
  - src/sentence_audio.rs
  - src/accepted_writer.rs
protected_scope:
  - input/**
  - audio/**
validation:
  - cargo test
manual_validation_reason: null
created_at: 2026-05-15T07:42:25.774719+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP05 - Patch accepted JSON audio metadata

## Goal

Patch accepted sentence batch JSON after required audio files exist, adding only
missing `audio` fields and replacing the JSON file atomically.

## Done When

- Patch logic adds missing `audio` fields using the scanner plan.
- Existing `audio` fields remain unchanged.
- All learner content fields remain unchanged at the data level.
- JSON writes go through temp path and rename.
- Failed patch leaves the original JSON file intact.
- Tests compare before/after data to prove metadata-only changes.
- `cargo test` passes.

## Must Not

- Do not regenerate, reorder, or reformat by hand with string manipulation.
- Do not change Hindi, romanisation, English, literal, register, tokens, words,
  `source_ref`, or tags.
- Do not backfill source lineage.

## Handoff Notes

Use structured JSON APIs. Reformatting accepted JSON via `serde_json` is
acceptable only if tests prove semantic learner fields are unchanged.

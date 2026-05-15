---
id: WP02
title: Add generation progress output
agent_type: rust-engineer
status: planned
dependencies: [WP01]
acceptance_refs: [AC03]
extra_skills: []
read_scope:
  - src/sentence_generate.rs
write_scope:
  - src/sentence_generate.rs
protected_scope: []
validation:
  - cargo test sentence_generate
manual_validation_reason: null
created_at: 2026-05-15T08:56:09.500803+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP02 - Add generation progress output

## Goal

Make `hindi sentences generate` visibly alive during slow local model calls by
printing concise phase lines and elapsed timings.

## Done When

- Generation output prints progress around model readiness, prompt send/model
  response, validation, write/report, and final status.
- Output remains readable and does not print per-token noise.
- `cargo test sentence_generate` passes.

## Must Not

- Do not add model lifecycle management.
- Do not change CLI arguments.
- Do not write accepted output before validation succeeds.

## Handoff Notes

The prior live run waited more than three minutes with no output before
validation failed. One line per major phase is enough.

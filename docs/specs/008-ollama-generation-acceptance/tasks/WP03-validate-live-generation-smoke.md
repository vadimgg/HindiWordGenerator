---
id: WP03
title: Validate live generation smoke
agent_type: project-manager
status: planned
dependencies: [WP01, WP02]
acceptance_refs: [AC04, AC05, AC06]
extra_skills: []
read_scope:
  - docs/backlog/backlog.jsonl
write_scope:
  - docs/backlog/backlog.jsonl
protected_scope: []
validation:
  - make check
  - cargo run -- doctor
  - cargo run -- source ids check
  - cargo run -- sentences plan --max-batches 1
  - cargo run -- sentences generate --max-batches 1
manual_validation_reason: null
created_at: 2026-05-15T08:56:17.015574+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP03 - Validate live generation smoke

## Goal

Run the full automated check suite and one live Ollama generation smoke test,
then update BL001/BL002 based on the observed result.

## Done When

- `make check` passes.
- Doctor, source ID check, and sentence plan commands pass.
- A live `sentences generate --max-batches 1` run is attempted.
- If validation passes, accepted output is inspected briefly.
- If validation fails, no accepted output is written and the remaining issue is
  captured in backlog.

## Must Not

- Do not force-write accepted output.
- Do not delete diagnostic run reports during the test.
- Do not mark BL001 or BL002 done unless the test result supports it.

## Handoff Notes

The smoke run depends on whichever Ollama model is locally reachable through the
configured `ollama:translategemma:12b` default.

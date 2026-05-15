---
id: WP02
title: Wire staged generation and run reports
agent_type: rust-engineer
status: done
dependencies: ["WP01"]
acceptance_refs: ["AC01", "AC02", "AC03", "AC08", "AC09", "AC10", "AC11", "AC12"]
extra_skills: []
read_scope:
  - "src/sentence_generate.rs"
  - "src/sentence_enrichment.rs"
  - "src/sentence_stages.rs"
  - "src/run_report.rs"
  - "src/ollama.rs"
  - "src/sentence_plan.rs"
  - "src/sentence_validate.rs"
  - "src/accepted_writer.rs"
  - "docs/specs/010-staged-sentence-generation/**"
write_scope:
  - "src/sentence_generate.rs"
  - "src/run_report.rs"
  - "src/sentence_enrichment.rs"
  - "src/sentence_stages.rs"
protected_scope:
  - "input/**"
  - "output/**"
  - "audio/**"
  - ".agents/rendered/**"
validation:
  - "cargo fmt --check"
  - "cargo test sentence_generate"
  - "cargo test run_report"
manual_validation_reason: null
created_at: 2026-05-15T14:43:59.039229+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP02 - Wire staged generation and run reports

## Goal

Replace the current single model call inside `hindi sentences generate` with
the staged generation internals from WP01. This work package owns orchestration,
model readiness, stage sequencing, validation-before-write behavior, and
stage-aware run reports.

## Done When

- `hindi sentences generate --max-batches <n>` keeps the same public command
  shape and high-level output.
- Planner/source/output errors still stop before model calls.
- The configured `sentence_generation` model is checked once and used for all
  stages.
- Generation calls register, literal, and word-breakdown-from-translation stages
  in order for each planned batch.
- Stage/merge/validation failure writes no accepted output.
- Accepted output still goes through the existing atomic writer and collision
  refusal.
- Sentence run reports include per-stage prompt ID, prompt fingerprint, model,
  duration, success, and error fields.
- Fake model tests cover successful generation and stage failure no-write
  behavior.
- Validation commands pass: `cargo fmt --check`, plus the relevant generator and
  run report tests.

## Must Not

- Do not add CLI-managed model switching.
- Do not add source QA or review/accept behavior.
- Do not continue using the old full-enrichment prompt as default generation.
- Do not write partial accepted batches.
- Do not mutate existing accepted output.
- Do not edit `input/`, `output/`, or `audio/`.

## Handoff Notes

The command may still write failed diagnostic run reports under
`runs/sentences/`, but accepted output remains the point of no return. Keep
progress messages compact; stage detail belongs in progress output and run
reports, not raw model response dumps.

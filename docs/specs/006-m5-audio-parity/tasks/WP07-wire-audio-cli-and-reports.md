---
id: WP07
title: Wire audio CLI and reports
agent_type: rust-engineer
status: planned
dependencies: [WP05]
acceptance_refs: [AC01, AC02, AC09]
extra_skills: []
read_scope:
  - docs/specs/006-m5-audio-parity/cli.md
  - src/cli.rs
  - src/main.rs
  - src/sentence_audio.rs
write_scope:
  - src/cli.rs
  - src/main.rs
  - src/sentence_audio.rs
protected_scope:
  - input/**
validation:
  - cargo test
  - cargo clippy --all-targets --all-features -- -D warnings
  - cargo run -- sentences --help
manual_validation_reason: null
created_at: 2026-05-15T07:42:25.841323+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP07 - Wire audio CLI and reports

## Goal

Expose the full audio backfill flow through `hindi sentences audio`, including
help text, success/no-op summaries, and clear failure output.

## Done When

- CLI accepts `hindi sentences audio`.
- `hindi sentences --help` lists `audio`.
- Success output matches the shape in `cli.md`.
- No-op output is clear when all sentence audio is complete.
- Empty-output and backend-failure messages include recovery guidance.
- `cargo test`, clippy, and `cargo run -- sentences --help` pass.

## Must Not

- Do not add `--force`, `--repair`, or word audio flags.
- Do not make the command interactive.
- Do not run real audio generation against protected project output unless the
  user explicitly asks during implementation.

## Handoff Notes

The command may write to `output/` and `audio/` during real use. Automated tests
must stay inside temp fixtures.

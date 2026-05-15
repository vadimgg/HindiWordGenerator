---
id: WP05
title: Add end to end smoke checks
agent_type: rust-engineer
status: planned
dependencies: [WP02, WP04]
acceptance_refs: [AC01, AC02, AC03, AC04, AC05, AC06, AC07, AC08, AC09]
extra_skills: []
read_scope:
  - docs/specs/007-m6-viewer-export-parity/**
  - docs/DESIGN.md
  - docs/ROADMAP.md
  - src/**
  - viewer/**
write_scope:
  - src/**
  - docs/specs/007-m6-viewer-export-parity/review.md
protected_scope:
  - input/**
  - output/**
  - audio/**
validation:
  - cargo test
  - cargo clippy --all-targets --all-features -- -D warnings
manual_validation_reason: null
created_at: 2026-05-15T07:58:21.676340+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP05 - Add end to end smoke checks

## Goal

Add the controlled smoke checks needed to prove viewer/export parity without broad real-data mutation.

## Done When

- Scope from `spec.md` and `plan.md` is implemented or reviewed for this work package.
- Relevant command/help behavior is covered when this package touches CLI.
- Tests or documented review notes prove the package outcome.
- Validation listed in frontmatter passes before marking done.

## Must Not

- Do not modify `input/`, `output/`, or `audio/` in automated tests.
- Do not add live AnkiConnect export to Rust.
- Do not redesign the Astro viewer.
- Stop and ask before running broad real-data generation/audio smoke.

## Handoff Notes

Use `docs/specs/007-m6-viewer-export-parity/spec.md`, `architecture.md`, and `cli.md` as the active contract. Older planning docs are reference only.

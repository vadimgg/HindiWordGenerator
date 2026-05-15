---
id: WP01
title: Implement sentence audio scanner
agent_type: rust-engineer
status: planned
dependencies: [WP02]
acceptance_refs: [AC02, AC03, AC04, AC05]
extra_skills: []
read_scope:
  - src/sentence_schema.rs
  - src/project.rs
  - output/sentences/**
write_scope:
  - src/sentence_audio.rs
  - src/main.rs
  - src/cli.rs
protected_scope:
  - input/**
  - output/**
  - audio/**
validation:
  - cargo test
manual_validation_reason: null
created_at: 2026-05-15T07:42:18.670360+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP01 - Implement sentence audio scanner

## Goal

Add the pure planning/scanning layer for accepted sentence audio. It should read
accepted batch JSON, classify missing vs existing audio, and compute planned
relative MP3 paths without writing files.

## Done When

- Scanner loads `output/sentences/*.json` in deterministic order.
- Scanner reports scanned batches/cards, missing audio, existing audio, and
  planned audio paths.
- Planned paths use `audio/sentences/<batch-stem>/<nn>_<slug>.mp3`.
- Existing `audio` fields are skipped by default.
- Unit tests cover empty output, existing audio, missing audio, and slug/path
  generation.
- `cargo test` passes.

## Must Not

- Do not synthesize MP3 files yet.
- Do not patch accepted JSON yet.
- Do not mutate real `output/` or `audio/` during tests.

## Handoff Notes

This package can use temp-project fixtures. It should make later writing logic
boring: the plan should already know what needs synthesis and what needs JSON
metadata.

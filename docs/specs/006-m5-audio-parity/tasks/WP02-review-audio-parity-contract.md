---
id: WP02
title: Review audio parity contract
agent_type: rust-engineer
status: planned
dependencies: []
acceptance_refs: [AC01, AC02, AC03, AC04, AC05, AC06, AC07, AC08, AC09, AC10, AC11]
extra_skills: []
read_scope:
  - docs/DESIGN.md
  - docs/ROADMAP.md
  - docs/specs/006-m5-audio-parity/**
  - archive/python/runtime/audio_generator.py
  - viewer/src/utils/audioHelpers.ts
  - viewer/src/utils/audioAssets.js
write_scope:
  - docs/specs/006-m5-audio-parity/review.md
protected_scope:
  - input/**
  - output/**
  - audio/**
validation:
  - cargo test
manual_validation_reason: null
created_at: 2026-05-15T07:42:18.701459+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP02 - Review audio parity contract

## Goal

Confirm the M5 contract against active docs, archived Python audio behavior, and
viewer audio expectations before code edits. Record only implementation-relevant
risks or corrections in `review.md`.

## Done When

- Active audio rules in `docs/DESIGN.md` and `docs/ROADMAP.md` have been read.
- Archived Python `audio_generator.py` path and behavior have been compared to
  this spec.
- Viewer explicit-audio-path expectations have been checked.
- Any mismatch is documented in `docs/specs/006-m5-audio-parity/review.md`.
- `cargo test` still passes before implementation begins.

## Must Not

- Do not edit source code in this work package.
- Do not run real audio generation.
- Do not modify `input/`, `output/`, or `audio/`.

## Handoff Notes

This is the review-first package. The rest of implementation should preserve
the core rule: audio may only add missing `audio` metadata and MP3 files.

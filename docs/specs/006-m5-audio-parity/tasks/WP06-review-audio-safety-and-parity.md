---
id: WP06
title: Review audio safety and parity
agent_type: rust-reviewer
status: done
dependencies: [WP07]
acceptance_refs: [AC01, AC02, AC03, AC04, AC05, AC06, AC07, AC08, AC09, AC10, AC11]
extra_skills: []
read_scope:
  - docs/specs/006-m5-audio-parity/**
  - src/**
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
  - cargo clippy --all-targets --all-features -- -D warnings
  - cargo run -- sentences --help
manual_validation_reason: null
created_at: 2026-05-15T07:42:25.806921+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP06 - Review audio safety and parity

## Goal

Review the completed M5 implementation for data safety, command clarity, and
viewer/export audio path compatibility before opening the PR.

## Done When

- Review confirms audio patching is metadata-only.
- Review confirms MP3 and JSON writes are atomic.
- Review confirms no real protected data was modified by tests.
- Review confirms command output matches `cli.md` or documents intentional
  differences.
- Any follow-up risks are recorded in `review.md`.
- `cargo test`, clippy, and help smoke pass.

## Must Not

- Do not make broad refactors during review.
- Do not introduce word audio.
- Do not run real audio generation unless the user explicitly asks.

## Handoff Notes

This is the final safety gate before the M5 PR.

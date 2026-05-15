---
id: WP04
title: Write audio files safely
agent_type: rust-engineer
status: planned
dependencies: [WP03]
acceptance_refs: [AC04, AC05, AC06, AC09, AC10]
extra_skills: []
read_scope:
  - src/accepted_writer.rs
  - src/sentence_audio.rs
  - src/tts.rs
write_scope:
  - src/sentence_audio.rs
  - src/tts.rs
protected_scope:
  - input/**
  - output/**
validation:
  - cargo test
manual_validation_reason: null
created_at: 2026-05-15T07:42:18.794591+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP04 - Write audio files safely

## Goal

Use the scanner plan and TTS boundary to create missing MP3 files through temp
paths and rename, while reusing existing MP3s when they are already present.

## Done When

- Missing MP3s are written to temp paths and renamed into final paths.
- Failed synthesis removes temp files when possible and leaves no final MP3.
- Existing final MP3s are skipped and not regenerated.
- Parent directories under `audio/sentences/` are created as needed.
- Tests cover success, existing-file skip, and synthesis failure.
- `cargo test` passes.

## Must Not

- Do not patch accepted JSON in this package.
- Do not overwrite existing MP3 files.
- Do not write outside `audio/sentences/`.

## Handoff Notes

After this package, rerunning audio after a JSON patch failure should be safe
because existing MP3s will be reused.

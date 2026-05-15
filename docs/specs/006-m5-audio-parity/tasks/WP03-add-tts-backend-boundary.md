---
id: WP03
title: Add TTS backend boundary
agent_type: rust-engineer
status: planned
dependencies: [WP01]
acceptance_refs: [AC09, AC10, AC11]
extra_skills: []
read_scope:
  - archive/python/runtime/audio_generator.py
  - docs/specs/006-m5-audio-parity/architecture.md
write_scope:
  - src/tts.rs
  - src/sentence_audio.rs
protected_scope:
  - input/**
  - output/**
  - audio/**
validation:
  - cargo test
manual_validation_reason: null
created_at: 2026-05-15T07:42:18.733693+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP03 - Add TTS backend boundary

## Goal

Introduce a replaceable Hindi TTS boundary so sentence audio can be tested
without network access and the first real backend can be swapped later.

## Done When

- A small TTS trait or equivalent boundary exists.
- Fake backend tests can synthesize deterministic MP3 bytes into temp paths.
- Backend errors preserve enough context to name the affected card/batch.
- The real backend boundary is present but does not make automated tests depend
  on Google/gTTS or network access.
- `cargo test` passes.

## Must Not

- Do not hardwire backend calls directly into CLI parsing.
- Do not require network access in automated tests.
- Do not introduce model or Ollama behavior into audio.

## Handoff Notes

The archived Python implementation used `gTTS(text, lang="hi")`. M5 may reuse
that idea behind the boundary, but scanner and JSON patching should not care
which backend is used.

---
id: WP04
title: Wire sentence generation pipeline
agent_type: rust-engineer
status: planned
dependencies: ["WP03"]
acceptance_refs: ["AC01", "AC02", "AC03", "AC04", "AC15", "AC16", "AC17", "AC21", "AC22"]
extra_skills: []
read_scope: ["docs/specs/005-m4-direct-local-sentence-generation/**", "docs/DESIGN.md", "docs/ROADMAP.md", "src/**", "Cargo.toml", "Cargo.lock", "input/sentences/*.yaml", "output/sentences/*.json"]
write_scope: ["src/**", "Cargo.toml", "Cargo.lock", "docs/specs/005-m4-direct-local-sentence-generation/**"]
protected_scope: ["input/**", "output/**", "audio/**"]
validation: ["cargo fmt", "cargo test", "cargo clippy --all-targets -- -D warnings", "cargo run -- sentences plan --max-batches 1", "git diff --name-only -- input audio", "git diff --check"]
manual_validation_reason: null
created_at: 2026-05-15T04:18:27.151882+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP04 - Wire Sentence Generation Pipeline

## Goal

Expose `hindi sentences generate --max-batches <n>` and wire the orchestration
through planner, model boundary, prompt/merge, M3 validator, and M3 writer.

## Done When

- CLI exposes `sentences generate --max-batches <n>`.
- Generation reuses planner source rows and targets.
- Planner errors stop before model calls.
- Fake-client happy path validates and writes temp accepted output.
- Fake-client invalid path writes no accepted output.
- Existing plan command stays read-only and compatible.
- Validation commands in frontmatter pass.

## Must Not

- Add source QA.
- Add review/accept workflow.
- Shell out to Ollama.
- Modify real `input/` or `audio/`.

## Handoff Notes

Tests should use fake model clients and temp directories. Real output writes
should only happen during intentional manual smoke.

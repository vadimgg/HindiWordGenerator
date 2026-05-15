---
id: WP02
title: Implement Ollama client and model readiness
agent_type: rust-engineer
status: done
dependencies: ["WP01"]
acceptance_refs: ["AC05", "AC06", "AC07", "AC08", "AC09"]
extra_skills: []
read_scope: ["docs/specs/005-m4-direct-local-sentence-generation/**", "docs/DESIGN.md", "src/**", "Cargo.toml", "Cargo.lock"]
write_scope: ["src/**", "Cargo.toml", "Cargo.lock", "docs/specs/005-m4-direct-local-sentence-generation/**"]
protected_scope: ["input/**", "output/**", "audio/**", "runs/**"]
validation: ["cargo fmt", "cargo test", "cargo clippy --all-targets -- -D warnings", "git diff --name-only -- input output audio runs", "git diff --check"]
manual_validation_reason: null
created_at: 2026-05-15T04:18:27.100529+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP02 - Implement Ollama Client And Model Readiness

## Goal

Implement configuration/model parsing and the local Ollama HTTP boundary. This
work owns readiness behavior and recovery messages, but not generation
orchestration.

## Done When

- Missing `hindi.toml` defaults to `ollama:translategemma:12b`.
- Explicit `[models].sentence_generation` is parsed.
- `ollama:<model>` parses into provider/model; unsupported providers fail.
- Ollama readiness can be tested with fake clients without local Ollama.
- Recovery output contains `ollama run <model>`.
- Validation commands in frontmatter pass.

## Must Not

- Shell out to `ollama`.
- Start, stop, unload, or switch models.
- Write accepted output.
- Modify protected paths.

## Handoff Notes

Use local HTTP abstractions and fake clients in tests. Real Ollama smoke is for
later integration, not required for this work package.

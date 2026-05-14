---
id: WP02
title: Add M1 behavior tests
agent_type: rust-engineer
status: done
dependencies: ["WP01"]
acceptance_refs: ["AC03", "AC04", "AC05", "AC06", "AC07", "AC09", "AC10", "AC11"]
extra_skills: []
read_scope: ["docs/specs/001-m1-rust-cli-skeleton/**", "Cargo.toml", "Cargo.lock", "src/**"]
write_scope: ["src/**", "tests/**"]
protected_scope: ["input/**", "output/**", "audio/**", "archive/**", "viewer/**"]
validation: ["cargo fmt", "cargo test", "cargo run -- doctor", "git diff --check"]
manual_validation_reason: null
created_at: 2026-05-14T17:34:33.682806+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP02 - Add M1 behavior tests

## Goal

Add focused test coverage for the M1 CLI skeleton, especially root discovery,
required vs optional checks, command exposure, and the Ollama reachability seam.

## Done When

- Tests cover successful project-root discovery from a temp project child path.
- Tests cover root-discovery failure outside a project.
- Tests cover missing optional `hindi.toml` without failure.
- Tests cover missing required folders or prompts with failure.
- Tests cover Ollama reachable/unreachable report states without requiring a
  running service.
- Tests or CLI assertions confirm `doctor` exists and `sentences plan` does not.
- Validation commands listed in frontmatter pass.

## Must Not

- Do not require a live Ollama service for unit tests.
- Do not write project data while testing missing-path behavior.
- Do not broaden M1 into YAML parsing, generation, audio, viewer, or export.

## Handoff Notes

Depends on WP01's crate/module shape. If WP01 already added some tests, extend
them rather than duplicating test scaffolding.

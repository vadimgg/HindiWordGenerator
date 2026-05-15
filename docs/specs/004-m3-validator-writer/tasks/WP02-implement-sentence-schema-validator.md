---
id: WP02
title: Implement sentence schema and validator
agent_type: rust-engineer
status: done
dependencies: ["WP01"]
acceptance_refs: ["AC01", "AC02", "AC03", "AC04", "AC05", "AC06", "AC07", "AC08", "AC09", "AC10", "AC11", "AC12", "AC13", "AC14"]
extra_skills: []
read_scope: ["docs/specs/004-m3-validator-writer/**", "docs/DESIGN.md", "docs/ROMANISATION.md", "src/**", "Cargo.toml", "Cargo.lock", "input/sentences/*.yaml"]
write_scope: ["src/**", "Cargo.toml", "Cargo.lock", "docs/specs/004-m3-validator-writer/**"]
protected_scope: ["input/**", "output/**", "audio/**", "runs/**"]
validation: ["cargo fmt", "cargo test", "cargo clippy --all-targets -- -D warnings", "git diff --name-only -- input output audio runs", "git diff --check"]
manual_validation_reason: null
created_at: 2026-05-15T03:01:05.669584+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP02 - Implement Sentence Schema And Validator

## Goal

Implement typed sentence-batch schema and pure validation rules. The validator
should accept candidate data plus expected source rows and return typed
validation results without writing files or printing output.

## Done When

- Candidate sentence batch JSON parses into typed structs.
- Required fields, register enum, token/word shape, `word_id` links, source
  coverage, source fingerprints, and romanisation reconstruction are validated.
- Validation reports useful batch/sentence errors.
- Unit tests cover happy path and failure cases named in `testing.md`.
- Validation commands in frontmatter pass.

## Must Not

- Implement file writing.
- Touch viewer code.
- Add `hindi sentences generate`.
- Modify protected paths.
- Accept legacy `word_index` in new Rust candidate validation.

## Handoff Notes

The viewer handles `word_index` fallback later. The Rust validator should be
strict for new output and require `word_id`.

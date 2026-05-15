---
id: WP05
title: Review validation writer safety
agent_type: rust-reviewer
status: done
dependencies: ["WP04"]
acceptance_refs: ["AC01", "AC02", "AC03", "AC04", "AC05", "AC06", "AC07", "AC08", "AC09", "AC10", "AC11", "AC12", "AC13", "AC14", "AC15", "AC16", "AC17", "AC18", "AC19", "AC20"]
extra_skills: []
read_scope: ["docs/specs/004-m3-validator-writer/**", "docs/DESIGN.md", "docs/ROADMAP.md", "README.md", "src/**", "Cargo.toml", "Cargo.lock", "viewer/**", "input/sentences/*.yaml", "output/sentences/*.json"]
write_scope: ["docs/specs/004-m3-validator-writer/**"]
protected_scope: ["input/**", "output/**", "audio/**", "runs/**"]
validation: ["cargo fmt", "cargo test", "cargo clippy --all-targets -- -D warnings", "cargo run -- sentences plan --max-batches 1", "git diff --name-only -- input output audio runs", "git diff --check"]
manual_validation_reason: null
created_at: 2026-05-15T03:01:05.772592+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP05 - Review Validation Writer Safety

## Goal

Review the implemented M3 validator, writer, viewer compatibility, docs, and
protected-path behavior before the branch is handed back for PR/manual merge.

## Done When

- Acceptance criteria are checked against code and command output.
- Protected path diff prints nothing.
- `review.md` captures validation results, changed files, and follow-ups.
- Validation commands in frontmatter pass.

## Must Not

- Implement M4 generation or Ollama model calls.
- Close or merge the spec without explicit user approval.
- Modify protected paths.

## Handoff Notes

The most important review questions are: no partial accepted writes, no
`word_index` acceptance in the Rust validator, and no real `output/` mutation
from M3 commands/tests.

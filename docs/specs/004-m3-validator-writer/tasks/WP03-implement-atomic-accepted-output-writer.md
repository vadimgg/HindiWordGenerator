---
id: WP03
title: Implement atomic accepted output writer
agent_type: rust-engineer
status: planned
dependencies: ["WP02"]
acceptance_refs: ["AC15", "AC16", "AC17", "AC18"]
extra_skills: []
read_scope: ["docs/specs/004-m3-validator-writer/**", "docs/DESIGN.md", "src/**", "Cargo.toml", "Cargo.lock"]
write_scope: ["src/**", "Cargo.toml", "Cargo.lock", "docs/specs/004-m3-validator-writer/**"]
protected_scope: ["input/**", "output/**", "audio/**", "runs/**"]
validation: ["cargo fmt", "cargo test", "cargo clippy --all-targets -- -D warnings", "cargo run -- sentences plan --max-batches 1", "git diff --name-only -- input output audio runs", "git diff --check"]
manual_validation_reason: null
created_at: 2026-05-15T03:01:05.707264+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP03 - Implement Atomic Accepted Output Writer

## Goal

Implement the reusable accepted-output writer that accepts already-validated
sentence batches, refuses collisions, serializes before writing, writes through
a temp path in the target directory, and renames only at the point of acceptance.

## Done When

- Writer refuses to overwrite an existing target file.
- Writer serializes before creating the accepted target.
- Writer uses a temp file in the target directory and renames to accept.
- Tests prove collision/failure cases leave no accepted target mutation.
- Existing `sentences plan` remains read-only and still works.
- Validation commands in frontmatter pass.

## Must Not

- Add model calls.
- Add a production command that writes real `output/`.
- Overwrite accepted files.
- Modify protected paths outside temp test fixtures.

## Handoff Notes

Use temp directories in tests. M4 will be responsible for calling the writer on
real planned output paths.

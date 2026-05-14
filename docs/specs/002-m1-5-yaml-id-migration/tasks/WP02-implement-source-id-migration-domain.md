---
id: WP02
title: Implement source ID migration domain
agent_type: rust-engineer
status: planned
dependencies: ["WP01"]
acceptance_refs: ["AC01", "AC02", "AC03", "AC04", "AC05", "AC06", "AC07", "AC08", "AC09", "AC12"]
extra_skills: []
read_scope: ["docs/specs/002-m1-5-yaml-id-migration/**", "docs/DESIGN.md", "src/**", "Cargo.toml", "Cargo.lock", "input/sentences/*.yaml", "input/words/*.yaml"]
write_scope: ["src/**", "Cargo.toml", "Cargo.lock", "docs/specs/002-m1-5-yaml-id-migration/**"]
protected_scope: ["input/**", "output/**", "audio/**", "runs/**", "archive/python/legacy-input/**"]
validation: ["cargo fmt", "cargo test", "git diff --check"]
manual_validation_reason: null
created_at: 2026-05-14T18:13:33.505901+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP02 - Implement source ID migration domain

## Goal

Implement the pure Rust source-ID domain: discover/parse active YAML source
files, validate existing IDs, allocate missing IDs in memory, and expose typed
reports for check/dry-run/migration flows.

## Done When

- Source ID validation detects missing, malformed, and duplicate IDs.
- Allocation preserves existing IDs and fills missing IDs in file order.
- Unit tests cover allocation, duplicate detection, malformed IDs, and
  idempotency.
- No active YAML files are edited by this work package.
- `cargo fmt`, `cargo test`, and `git diff --check` pass.

## Must Not

- Wire the user-facing CLI command beyond what is needed for tests.
- Write active source YAML files.
- Edit `input/`, `output/`, `audio/`, `runs/`, or archived legacy input.
- Implement M2 planner logic.

## Handoff Notes

Keep the domain reusable by M2: validator output should be useful to the future
planner, but this task should not implement planner behavior.

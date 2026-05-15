---
id: WP04
title: Review planner read-only safety
agent_type: rust-reviewer
status: done
dependencies: ["WP03"]
acceptance_refs: ["AC01", "AC02", "AC03", "AC04", "AC05", "AC06", "AC07", "AC08", "AC09", "AC10", "AC11", "AC12", "AC13", "AC14", "AC15"]
extra_skills: []
read_scope: ["docs/specs/003-m2-sentence-planner/**", "docs/DESIGN.md", "docs/ROADMAP.md", "README.md", "src/**", "Cargo.toml", "Cargo.lock", "input/sentences/*.yaml", "output/sentences/*.json"]
write_scope: ["docs/specs/003-m2-sentence-planner/**"]
protected_scope: ["input/**", "output/**", "audio/**", "runs/**"]
validation: ["cargo fmt", "cargo test", "cargo run -- sentences plan --max-batches 1", "cargo run -- source ids check", "git diff --name-only -- input output audio runs", "git diff --check"]
manual_validation_reason: null
created_at: 2026-05-15T02:44:03.515273+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP04 - Review Planner Read-Only Safety

## Goal

Review the implemented planner against the spec and protected-path contract.
Confirm it derives useful state without mutating source, accepted output, audio,
or run folders.

## Done When

- Acceptance criteria are checked against code and command output.
- Protected path diff prints nothing.
- `review.md` captures validation and any follow-ups.
- Validation commands in frontmatter pass.

## Must Not

- Implement new planner behavior beyond small review fixes.
- Close or merge the spec without explicit user approval.
- Modify protected paths.

## Handoff Notes

The most important review question is whether lineage-less output is visible
and not counted as done.

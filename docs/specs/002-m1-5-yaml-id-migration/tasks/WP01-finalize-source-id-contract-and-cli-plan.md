---
id: WP01
title: Finalize source ID contract and CLI plan
agent_type: rust-reviewer
status: planned
dependencies: []
acceptance_refs: ["AC01", "AC02", "AC03", "AC04", "AC05", "AC06", "AC07", "AC08", "AC09", "AC10", "AC11", "AC12", "AC13"]
extra_skills: []
read_scope: ["docs/specs/002-m1-5-yaml-id-migration/**", "docs/DESIGN.md", "docs/ROADMAP.md", "README.md", "src/**", "input/sentences/*.yaml", "input/words/*.yaml"]
write_scope: ["docs/specs/002-m1-5-yaml-id-migration/**"]
protected_scope: ["output/**", "audio/**", "runs/**", "archive/python/legacy-input/**"]
validation: ["brief spec status", "git diff --check"]
manual_validation_reason: null
created_at: 2026-05-14T18:13:30.936348+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP01 - Finalize source ID contract and CLI plan

## Goal

Review the spec before implementation and make sure the CLI shape, source ID
rules, protected paths, and validation plan are precise enough for a Rust
engineer to implement without guessing.

## Done When

- `spec.md`, `architecture.md`, `cli.md`, `testing.md`, and `plan.md` agree on
  the command names and write scope.
- Work packages have concrete read/write/protected scopes.
- Any reviewer pushback is either applied to the spec or explicitly recorded in
  handoff notes.
- `brief spec status` works and reports this spec.
- `git diff --check` passes.

## Must Not

- Implement Rust code.
- Edit source YAML.
- Change `output/`, `audio/`, `runs/`, or archived legacy input.
- Close the spec.

## Handoff Notes

The active docs already define file-scoped IDs. This task is a planning gate,
not a second design pass for source identity.

---
id: WP01
title: Review planner contract and output shape
agent_type: rust-reviewer
status: planned
dependencies: []
acceptance_refs: ["AC01", "AC02", "AC03", "AC04", "AC05", "AC06", "AC07", "AC08", "AC09", "AC10", "AC11", "AC12", "AC13", "AC14", "AC15"]
extra_skills: []
read_scope: ["docs/specs/003-m2-sentence-planner/**", "docs/DESIGN.md", "docs/ROADMAP.md", "README.md", "src/**", "input/sentences/*.yaml", "output/sentences/*.json"]
write_scope: ["docs/specs/003-m2-sentence-planner/**"]
protected_scope: ["input/**", "output/**", "audio/**", "runs/**"]
validation: ["brief spec status", "git diff --check"]
manual_validation_reason: null
created_at: 2026-05-15T02:43:56.672956+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP01 - Review planner contract and output shape

## Goal

Review the M2 planner contract before implementation. Confirm read-only
behavior, output sections, lineage classification, source fingerprinting, and
batch filename rules are specific enough for implementation.

## Done When

- Spec docs agree on command name, flags, output shape, and read-only scope.
- Any ambiguity about missing lineage, source changed, or batch filenames is
  resolved in the spec.
- Work-package scopes are concrete and protected paths are explicit.
- `brief spec status` and `git diff --check` pass.

## Must Not

- Implement Rust code.
- Edit source YAML or accepted output.
- Close or merge the spec.

## Handoff Notes

This is a planning gate. Keep feedback scoped to whether M2 is implementable
and safe.

---
id: WP02
title: Implement sentence planner domain
agent_type: rust-engineer
status: done
dependencies: ["WP01"]
acceptance_refs: ["AC02", "AC03", "AC04", "AC05", "AC06", "AC07", "AC08", "AC09", "AC10", "AC11", "AC12", "AC14", "AC15"]
extra_skills: []
read_scope: ["docs/specs/003-m2-sentence-planner/**", "docs/DESIGN.md", "docs/ROADMAP.md", "src/**", "Cargo.toml", "Cargo.lock", "input/sentences/*.yaml", "output/sentences/*.json"]
write_scope: ["src/**", "Cargo.toml", "Cargo.lock", "docs/specs/003-m2-sentence-planner/**"]
protected_scope: ["input/**", "output/**", "audio/**", "runs/**"]
validation: ["cargo fmt", "cargo test", "git diff --name-only -- input output audio runs", "git diff --check"]
manual_validation_reason: null
created_at: 2026-05-15T02:44:03.480754+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP02 - Implement sentence planner domain

## Goal

Build the read-only planning domain: load sentence source rows, compute source
fingerprints, read accepted sentence output, classify lineage state, select next
batch targets, and return a typed report for CLI rendering.

## Done When

- Unit tests cover source fingerprinting, missing lineage, done, source changed,
  pending/deferred derivation, and next batch filename selection.
- Domain code writes no files.
- Existing M1.5 source ID validation remains intact.
- `cargo fmt`, `cargo test`, protected path diff, and `git diff --check` pass.

## Must Not

- Wire a user-facing command beyond what tests require.
- Call Ollama or implement generation.
- Modify `input/`, `output/`, `audio/`, or `runs/`.
- Implement full M3 schema validation.

## Handoff Notes

Accepted output parsing can be narrow for M2: extract sentence cards and
optional `source_ref`; full learner-card validation belongs to M3.

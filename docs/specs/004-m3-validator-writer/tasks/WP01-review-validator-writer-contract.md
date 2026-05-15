---
id: WP01
title: Review validator and writer contract
agent_type: plan-reviewer
status: planned
dependencies: []
acceptance_refs: ["AC01", "AC02", "AC03", "AC04", "AC05", "AC06", "AC07", "AC08", "AC09", "AC10", "AC11", "AC12", "AC13", "AC14", "AC15", "AC16", "AC17", "AC18", "AC19", "AC20"]
extra_skills: []
read_scope: ["docs/specs/004-m3-validator-writer/**", "docs/DESIGN.md", "docs/ROADMAP.md", "README.md", "src/**", "viewer/**", "Cargo.toml"]
write_scope: ["docs/specs/004-m3-validator-writer/**"]
protected_scope: ["input/**", "output/**", "audio/**", "runs/**"]
validation: ["brief spec ready", "git diff --check"]
manual_validation_reason: null
created_at: 2026-05-15T03:00:58.982471+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP01 - Review Validator And Writer Contract

## Goal

Review the spec, architecture, testing plan, and current code shape before
implementation. The output is a ready implementation packet with no unresolved
schema/write-order decisions.

## Done When

- Spec acceptance covers schema, validation, writer, viewer compatibility, and
  protected-path behavior.
- Work packages have bounded scopes, dependencies, and validation commands.
- M3 remains infrastructure-only; no `sentences generate` command is introduced.
- Validation commands in frontmatter pass.

## Must Not

- Edit production Rust or viewer code.
- Add dependencies.
- Mark the spec complete.
- Modify protected paths.

## Handoff Notes

User explicitly wants brief-managed specs before implementation. Keep this
packet small and practical; avoid re-expanding docs into old planning sprawl.

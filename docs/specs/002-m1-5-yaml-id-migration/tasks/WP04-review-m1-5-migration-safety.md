---
id: WP04
title: Review M1.5 migration safety
agent_type: rust-reviewer
status: planned
dependencies: ["WP03"]
acceptance_refs: ["AC01", "AC02", "AC03", "AC04", "AC05", "AC06", "AC07", "AC08", "AC09", "AC10", "AC11", "AC12", "AC13"]
extra_skills: []
read_scope: ["docs/specs/002-m1-5-yaml-id-migration/**", "docs/DESIGN.md", "docs/ROADMAP.md", "README.md", "src/**", "Cargo.toml", "Cargo.lock", "input/sentences/*.yaml", "input/words/*.yaml"]
write_scope: ["docs/specs/002-m1-5-yaml-id-migration/**"]
protected_scope: ["output/**", "audio/**", "runs/**", "archive/python/legacy-input/**"]
validation: ["cargo fmt", "cargo test", "cargo run -- source ids check", "python3 archive/python/scripts/check-agent-workflows.py", "uv run python archive/python/scripts/check-python-contracts.py", "git diff --check"]
manual_validation_reason: null
created_at: 2026-05-14T18:13:38.628329+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP04 - Review M1.5 migration safety

## Goal

Review the implementation and source YAML migration for safety before closeout.
Confirm IDs are stable, protected paths did not change, and M2 can rely on the
new source identity contract.

## Done When

- All acceptance criteria are reviewed against code and diffs.
- Validation commands in frontmatter pass.
- `git diff --name-only -- output audio runs archive/python/legacy-input`
  prints nothing.
- Review notes are added to `review.md` or this task if follow-ups remain.

## Must Not

- Implement new behavior beyond small review fixes.
- Close the spec or merge without explicit user approval.
- Modify protected paths.

## Handoff Notes

Pay special attention to the YAML diff. This review should treat source text
changes as suspicious unless they are clearly unrelated user edits.

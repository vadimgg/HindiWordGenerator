---
id: WP03
title: Wire CLI migrate active YAML and update docs
agent_type: rust-engineer
status: planned
dependencies: ["WP02"]
acceptance_refs: ["AC01", "AC02", "AC03", "AC04", "AC05", "AC06", "AC07", "AC08", "AC09", "AC10", "AC11", "AC12", "AC13"]
extra_skills: []
read_scope: ["docs/specs/002-m1-5-yaml-id-migration/**", "docs/DESIGN.md", "docs/ROADMAP.md", "README.md", "src/**", "Cargo.toml", "Cargo.lock", "input/sentences/*.yaml", "input/words/*.yaml"]
write_scope: ["src/**", "Cargo.toml", "Cargo.lock", "input/sentences/*.yaml", "input/words/*.yaml", "docs/ROADMAP.md", "docs/specs/002-m1-5-yaml-id-migration/**"]
protected_scope: ["output/**", "audio/**", "runs/**", "archive/python/legacy-input/**"]
validation: ["cargo fmt", "cargo test", "cargo run -- source ids check", "cargo run -- source ids migrate --check", "cargo run -- source ids migrate", "cargo run -- source ids check", "python3 archive/python/scripts/check-agent-workflows.py", "uv run python archive/python/scripts/check-python-contracts.py", "git diff --check"]
manual_validation_reason: null
created_at: 2026-05-14T18:13:36.045617+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP03 - Wire CLI migrate active YAML and update docs

## Goal

Expose the source ID commands, run the migration on active YAML files, and mark
the M1.5 roadmap row complete once the repository source data has stable IDs.

## Done When

- `cargo run -- source ids check` reports current source ID state.
- `cargo run -- source ids migrate --check` previews planned source YAML edits
  without writing.
- `cargo run -- source ids migrate` adds IDs to active YAML files only.
- Every active sentence and word YAML item has a stable quoted ID.
- `docs/ROADMAP.md` marks YAML item IDs migrated as done.
- Full validation command list in frontmatter passes.

## Must Not

- Modify `output/`, `audio/`, `runs/`, or archived legacy input.
- Backfill `source_ref` into generated output.
- Implement `hindi sentences plan`.
- Regenerate existing IDs if the migration is rerun.

## Handoff Notes

This is the only task in the spec expected to edit active source YAML. Review
the resulting diff carefully; IDs should appear before `hindi` and existing
source text should stay unchanged.

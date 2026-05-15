---
id: WP03
title: Wire planner CLI and docs
agent_type: rust-engineer
status: planned
dependencies: ["WP02"]
acceptance_refs: ["AC01", "AC09", "AC10", "AC11", "AC12", "AC13", "AC14"]
extra_skills: []
read_scope: ["docs/specs/003-m2-sentence-planner/**", "docs/DESIGN.md", "docs/ROADMAP.md", "README.md", "src/**", "Cargo.toml", "Cargo.lock", "input/sentences/*.yaml", "output/sentences/*.json"]
write_scope: ["src/**", "Cargo.toml", "Cargo.lock", "docs/ROADMAP.md", "docs/specs/003-m2-sentence-planner/**"]
protected_scope: ["input/**", "output/**", "audio/**", "runs/**"]
validation: ["cargo fmt", "cargo test", "cargo run -- sentences plan --max-batches 1", "cargo run -- source ids check", "python3 archive/python/scripts/check-agent-workflows.py", "uv run python archive/python/scripts/check-python-contracts.py", "git diff --name-only -- input output audio runs", "git diff --check"]
manual_validation_reason: null
created_at: 2026-05-15T02:44:03.498177+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP03 - Wire planner CLI and docs

## Goal

Expose `hindi sentences plan --max-batches <n>`, update help/doctor text, and
mark the roadmap planner status once the command is working.

## Done When

- `cargo run -- sentences plan --max-batches 1` prints the planner report.
- Invalid or missing `--max-batches` returns a clear usage error.
- Help text no longer says the planner is unavailable.
- Roadmap marks sentence planner done after implementation.
- Full validation command list in frontmatter passes.

## Must Not

- Write `input/`, `output/`, `audio/`, or `runs/`.
- Implement sentence generation.
- Backfill `source_ref`.

## Handoff Notes

Keep the output calm and aligned with `doctor` / `source ids`; no interactive
prompts.

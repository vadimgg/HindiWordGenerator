---
id: WP05
title: Add run reports and generation output UX
agent_type: cli-ux-reviewer
status: planned
dependencies: ["WP04"]
acceptance_refs: ["AC18", "AC19", "AC20"]
extra_skills: []
read_scope: ["docs/specs/005-m4-direct-local-sentence-generation/**", "docs/ROADMAP.md", "src/**"]
write_scope: ["src/**", "docs/ROADMAP.md", "docs/specs/005-m4-direct-local-sentence-generation/**"]
protected_scope: ["input/**", "output/**", "audio/**"]
validation: ["cargo fmt", "cargo test", "cargo clippy --all-targets -- -D warnings", "git diff --name-only -- input audio", "git diff --check"]
manual_validation_reason: null
created_at: 2026-05-15T04:18:34.794357+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP05 - Add Run Reports And Generation Output UX

## Goal

Add diagnostic run reports for accepted and failed generation attempts, then
make the command output clear about model, target, accepted/skipped writes, run
report path, and next step.

## Done When

- Run reports are written under `runs/sentences/`.
- Reports include command, status, sources, targets, prompt metadata, model,
  timings, validation, accepted/skipped writes.
- Failure output includes run report path when a report exists.
- Success output matches `cli.md` shape closely enough for users.
- Validation commands in frontmatter pass.

## Must Not

- Treat run reports as planner input.
- Add interactive prompts.
- Modify real `input/` or `audio/`.
- Add model switching commands.

## Handoff Notes

Run reports are diagnostic and safe to delete intentionally. Accepted output
remains the source of truth.

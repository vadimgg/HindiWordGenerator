---
id: WP06
title: Review M4 generation safety
agent_type: rust-reviewer
status: planned
dependencies: ["WP05"]
acceptance_refs: ["AC01", "AC02", "AC03", "AC04", "AC05", "AC06", "AC07", "AC08", "AC09", "AC10", "AC11", "AC12", "AC13", "AC14", "AC15", "AC16", "AC17", "AC18", "AC19", "AC20", "AC21", "AC22"]
extra_skills: []
read_scope: ["docs/specs/005-m4-direct-local-sentence-generation/**", "docs/DESIGN.md", "docs/ROADMAP.md", "README.md", "src/**", "Cargo.toml", "Cargo.lock", "generation_prompt_sentences_enrichment.txt", "input/sentences/*.yaml", "output/sentences/*.json"]
write_scope: ["docs/specs/005-m4-direct-local-sentence-generation/**"]
protected_scope: ["input/**", "audio/**"]
validation: ["cargo fmt", "cargo test", "cargo clippy --all-targets -- -D warnings", "cargo run -- sentences plan --max-batches 1", "git diff --name-only -- input audio", "git diff --check"]
manual_validation_reason: null
created_at: 2026-05-15T04:18:34.779568+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP06 - Review M4 Generation Safety

## Goal

Review the implemented M4 command against model, validation, accepted-write, run
report, CLI output, and protected-path contracts before handoff.

## Done When

- Acceptance criteria are checked against code and command output.
- No code shells out to `ollama run`, `ollama stop`, or model-switch commands.
- Protected path diff for `input` and `audio` prints nothing.
- `review.md` captures validation, changed files, and follow-ups.
- Validation commands in frontmatter pass.

## Must Not

- Implement source QA or multi-model orchestration.
- Close or merge the spec without explicit user approval.
- Modify protected paths.

## Handoff Notes

Real Ollama smoke is useful if the user has started `translategemma:12b`, but
unit/fake-client validation should not depend on it.

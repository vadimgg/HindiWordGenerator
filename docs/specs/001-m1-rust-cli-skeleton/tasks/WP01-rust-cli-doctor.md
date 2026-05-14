---
id: WP01
title: Create Rust CLI skeleton and doctor command
agent_type: rust-engineer
status: done
dependencies: []
acceptance_refs: ["AC01", "AC02", "AC03", "AC04", "AC05", "AC06", "AC07", "AC08", "AC09", "AC10", "AC11"]
extra_skills: []
read_scope: ["docs/specs/001-m1-rust-cli-skeleton/**", "docs/DESIGN.md", "docs/ROADMAP.md", "README.md", "generation_prompt_sentences_enrichment.txt", "generation_prompt_sentences.txt"]
write_scope: ["Cargo.toml", "Cargo.lock", "src/**", "tests/**"]
protected_scope: ["input/**", "output/**", "audio/**", "archive/**", "viewer/**"]
validation: ["cargo fmt", "cargo test", "cargo run -- doctor", "git diff --check"]
manual_validation_reason: null
created_at: 2026-05-14T17:34:33.521254+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP01 - Create Rust CLI skeleton and doctor command

## Goal

Create the initial Rust binary crate and implement `hindi doctor` as a
read-only project inspection command.

## Done When

- `Cargo.toml` and `src/` exist for one binary crate.
- `cargo run -- doctor` prints sections for Project, Data, Prompts, Ollama, and
  Next.
- Project-root discovery works from the repository root and is structured so it
  can work from child directories.
- Required data folders and prompt files are reported as `ok` or `missing`.
- Missing `hindi.toml` is reported but not fatal.
- Ollama reachability checks `/api/version` or an equivalent cheap service
  endpoint and does not load a model.
- `hindi sentences plan` is not exposed.
- Validation commands listed in frontmatter pass, except Ollama reachability may
  report unavailable if the local service is genuinely down.

## Must Not

- Do not write to `input/`, `output/`, `audio/`, `runs/`, or `exports/`.
- Do not create missing project data folders.
- Do not implement `hindi sentences plan`.
- Do not call `/api/generate`, `ollama run`, or any model-loading path.
- Do not modify archived Python or viewer code.

## Handoff Notes

Use [../spec.md](../spec.md), [../architecture.md](../architecture.md), and
[../cli.md](../cli.md) as the source of truth. Keep the first implementation
small; M1 is a diagnostic shell, not generation.

---
id: WP01
title: Review M4 generation contract
agent_type: plan-reviewer
status: planned
dependencies: []
acceptance_refs: ["AC01", "AC02", "AC03", "AC04", "AC05", "AC06", "AC07", "AC08", "AC09", "AC10", "AC11", "AC12", "AC13", "AC14", "AC15", "AC16", "AC17", "AC18", "AC19", "AC20", "AC21", "AC22"]
extra_skills: []
read_scope: ["docs/specs/005-m4-direct-local-sentence-generation/**", "docs/DESIGN.md", "docs/ROADMAP.md", "src/**", "Cargo.toml", "generation_prompt_sentences_enrichment.txt"]
write_scope: ["docs/specs/005-m4-direct-local-sentence-generation/**"]
protected_scope: ["input/**", "output/**", "audio/**", "runs/**"]
validation: ["brief spec ready", "git diff --check"]
manual_validation_reason: null
created_at: 2026-05-15T04:18:27.085184+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP01 - Review M4 Generation Contract

## Goal

Review the M4 spec and current M2/M3 implementation boundaries before source
edits. The result should be a ready implementation packet with clear model,
prompt, validation, write, and run-report ownership.

## Done When

- Spec keeps M4 to one model role, `sentence_generation`.
- Spec clearly says the CLI does not spawn/stop/switch Ollama.
- Work packages have bounded scopes, dependencies, protected paths, and
  validation commands.
- Validation commands in frontmatter pass.

## Must Not

- Edit production Rust code.
- Add model calls.
- Modify protected paths.
- Mark the spec complete.

## Handoff Notes

The user explicitly asked whether Ollama would be spawned/switched. The answer
for M4 is no: user starts Ollama separately; CLI checks readiness and calls the
local HTTP API.

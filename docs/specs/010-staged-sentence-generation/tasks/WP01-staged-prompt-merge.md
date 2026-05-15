---
id: WP01
title: Add staged prompt parsing and merge internals
agent_type: rust-engineer
status: done
dependencies: []
acceptance_refs: ["AC04", "AC05", "AC06", "AC07", "AC12"]
extra_skills: []
read_scope:
  - "src/sentence_generate.rs"
  - "src/sentence_enrichment.rs"
  - "src/eval.rs"
  - "src/eval_prompts/**"
  - "src/sentence_plan.rs"
  - "src/sentence_validate.rs"
  - "docs/specs/010-staged-sentence-generation/**"
write_scope:
  - "src/sentence_enrichment.rs"
  - "src/sentence_stages.rs"
  - "src/eval_prompts/**"
  - "src/sentence_generate.rs"
protected_scope:
  - "input/**"
  - "output/**"
  - "audio/**"
  - ".agents/rendered/**"
validation:
  - "cargo fmt --check"
  - "cargo test sentence_enrichment"
  - "cargo test sentence_stages"
manual_validation_reason: null
created_at: 2026-05-15T14:43:58.997251+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP01 - Add staged prompt parsing and merge internals

## Goal

Add the internal staged prompt and merge layer that generation can call later.
This work package owns prompt metadata, stage response parsing, exact source ID
coverage checks, and candidate construction from trusted planner rows plus
register/literal/word-breakdown outputs.

## Done When

- Required generation stage IDs exist: `sentence/register`, `sentence/literal`,
  and `sentence/word-breakdown-from-translation`.
- Each stage has prompt text, version or stable identifier, and a fingerprint
  available to run reports.
- Stage parsers accept structured model output and reject duplicate source IDs.
- The staged merger rejects missing, duplicate, and extra stage IDs.
- The staged merger copies title, subtitle, Hindi, romanisation, English, tags,
  target path, and `source_ref` from planner/YAML data only.
- The staged merger produces candidate `literal`, `register`, `tokens`,
  `words`, and optional `anki_tags`.
- Tests prove the successful merge path and missing/duplicate/extra ID failure
  paths.
- Validation commands pass: `cargo fmt --check`, plus the relevant staged parser
  and merger tests.

## Must Not

- Do not call Ollama from parser or merger code.
- Do not write accepted output or run reports in this work package.
- Do not add user-facing CLI flags or commands.
- Do not use the full-enrichment prompt as a fallback.
- Do not trust model-provided Hindi, romanisation, English, lineage, or
  filenames.
- Do not edit `input/`, `output/`, or `audio/`.

## Handoff Notes

The existing `merge_enrichment` function already has the right trust boundary:
Rust owns source fields and model output only supplies enrichment. Preserve that
shape while splitting the input into staged records. Prefer sharing prompt text
with the eval prompt registry if it can be done cleanly; otherwise make the
generation prompt fingerprints explicit.

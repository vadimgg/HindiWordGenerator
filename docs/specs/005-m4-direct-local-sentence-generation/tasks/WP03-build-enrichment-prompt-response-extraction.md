---
id: WP03
title: Build enrichment prompt and response extraction
agent_type: hindi-prompt-tuner
status: done
dependencies: ["WP02"]
acceptance_refs: ["AC10", "AC11", "AC12", "AC13", "AC14"]
extra_skills: []
read_scope: ["docs/specs/005-m4-direct-local-sentence-generation/**", "generation_prompt_sentences_enrichment.txt", "src/**", "input/sentences/*.yaml"]
write_scope: ["src/**", "generation_prompt_sentences_enrichment.txt", "docs/specs/005-m4-direct-local-sentence-generation/**"]
protected_scope: ["input/**", "output/**", "audio/**", "runs/**"]
validation: ["cargo fmt", "cargo test", "cargo clippy --all-targets -- -D warnings", "git diff --name-only -- input output audio runs", "git diff --check"]
manual_validation_reason: null
created_at: 2026-05-15T04:18:27.131811+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP03 - Build Enrichment Prompt And Response Extraction

## Goal

Implement prompt payload construction, response JSON extraction, and enrichment
merge rules. The model may produce enrichment only; Rust keeps trusted source
fields.

## Done When

- Prompt payload contains source row ID, Hindi, romanisation, English, and tags.
- Prompt payload excludes title, subtitle, source_ref, target filename, and
  fingerprint.
- Extractor accepts raw JSON and fenced JSON.
- Extractor rejects responses with no JSON object.
- Merge ignores model-returned trusted fields and uses enrichment only.
- Validation commands in frontmatter pass.

## Must Not

- Call Ollama.
- Write accepted output or run reports.
- Modify protected paths.
- Add source QA or multi-prompt orchestration.

## Handoff Notes

The existing enrichment prompt already states the trust boundary. Code should
enforce it even if the model ignores instructions.

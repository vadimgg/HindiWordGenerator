---
id: WP02
title: Run eval through Ollama and write artifacts
agent_type: rust-engineer
status: done
dependencies: [WP01]
acceptance_refs: [AC02, AC06, AC07, AC09]
extra_skills: []
read_scope:
  - src/eval.rs
  - src/ollama.rs
write_scope:
  - .gitignore
  - src/eval.rs
  - src/ollama.rs
protected_scope: []
validation:
  - cargo test eval
  - cargo test ollama
manual_validation_reason: null
created_at: 2026-05-15T09:38:01.000000+00:00
started_at: null
completed_at: null
canceled_at: null
---

# WP02 - Run eval through Ollama and write artifacts

## Goal

Connect `hindi eval run` to the one currently running Ollama model and persist
each run under ignored `eval/<prompt-id>/<run-id>/` without touching accepted
learner output.

## Done When

- Eval detects exactly one running Ollama model and prints it.
- Zero or multiple running models fail with recovery text.
- Eval writes `prompt.txt`, `response.txt`, `meta.json`, and `summary.txt`.
- `.gitignore` ignores `eval/`.
- Tests prove eval writes nothing under `output/`.

## Must Not

- Do not add `--model`.
- Do not start, stop, or switch Ollama models.
- Do not write accepted output.

## Handoff Notes

Use Ollama `/api/ps` for running-model detection rather than shelling out.

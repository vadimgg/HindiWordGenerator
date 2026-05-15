# Review

## Summary

Implemented `hindi eval run` and `hindi eval grade` as an ignored prompt
workbench under `eval/<prompt-id>/<run-id>/`. The runner uses built-in paired
prompt templates, selects fields from YAML source input, requires exactly one
running Ollama model from `/api/ps`, writes prompt/response/meta/summary
artifacts, and records human/agent grading responses as canonical `grade.json`.
Follow-up testing added `hindi eval grade --response <path>` so spawned-agent or
copied evaluator responses can be imported without pretending to be `$EDITOR`.

## Validation

Passed:

- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run -- eval --help`
- `cargo run -- eval run --input input/sentences/complete_hindi_chapter_02_sentences.yaml --prompt-id sentence/register --max-items 1`
  failed safely because no Ollama model was running.
- Live `hindi eval run --input input/sentences/complete_hindi_chapter_02_sentences.yaml --prompt-id sentence/register --max-items 2`
  succeeded against `ollama:translategemma:12b` in 4.7s.
- A spawned evaluator agent graded the generated `grade_prompt.txt`; its YAML
  response was imported with `hindi eval grade --response <path>` and recorded
  as `grade.json` with score `16/20`.
- `make check`

## Changed Files

- `.gitignore`
- `Cargo.toml`
- `Cargo.lock`
- `src/cli.rs`
- `src/main.rs`
- `src/ollama.rs`
- `src/eval.rs`
- `src/eval_prompts/*`

## Follow-Ups

- Run a live eval smoke once exactly one Ollama model is running.
- Review the seeded prompts after a few real eval runs; they are intentionally
  practical starting points, not final benchmark prompts.

<!-- brief:close:start -->
## Summary

All work packages are done or intentionally canceled.

## Tasks

| ID | Status | Notes |
|---|---|---|
| WP01 | done | Add eval CLI and template context |
| WP02 | done | Run eval through Ollama and write artifacts |
| WP03 | done | Seed sentence eval prompts, grading prompts, and smoke test |

## Acceptance

- WP01: AC01, AC03, AC04, AC05
- WP02: AC02, AC06, AC07, AC09
- WP03: AC08, AC10, AC11

## Validation

- `cargo test cli`
- `cargo test eval`
- `cargo test eval`
- `cargo test ollama`
- `make check`

## Scope Check

- WP01 write scope: Cargo.toml, src/cli.rs, src/eval.rs, src/main.rs
- WP02 write scope: .gitignore, src/eval.rs, src/ollama.rs
- WP03 write scope: src/eval_prompts/, src/eval.rs

## PR Readiness

Spec is ready for `brief spec complete`.

<!-- brief:close:end -->

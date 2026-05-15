# Prompt Eval Runner

## Scope

Add `hindi eval run`, `hindi eval grade`, and `hindi eval report` for running
built-in prompt templates against YAML source input, grading eval results, and
scanning the resulting runs without touching accepted learner output. `run`
uses the currently running Ollama model and writes a structured run folder under
ignored `eval/<prompt-id>/<run-id>/`. `grade` renders the matching built-in
grading prompt for a previous eval run, opens a single editor packet containing
the prompt plus a paste area, and stores the structured grader response back
into the run folder. `report` reads those run folders and prints source rows,
model, timing, grade, verdict, and notes in a colored table.

## Problem

We need to compare sentence sub-task prompts before generating more real cards:
source QA, English translation, literal translation, register detection,
word-by-word breakdown, mapping existing translations to words, and full
enrichment. The current generation command is too tied to accepted output, and
the old Python experiment scripts are archived and less structured than the
Rust workflow.

## Goals

- Provide `hindi eval run <prompt-id> <input-yaml>`.
- Provide `hindi eval grade <run-id-or-path> [--response <path>]`.
- Provide `hindi eval report [--no-color] [--verbose] [--output none|failures|all]`.
- Use the single currently running Ollama model from Ollama `/api/ps`; do not
  switch or start models for eval runs.
- Use built-in prompt IDs such as `sentence/register`, with paired input and
  grading templates compiled into the Rust binary.
- Render prompts with Handlebars and support batch-friendly `{{#each items}}`.
- Provide YAML-first template context: `input_yaml`, `items_yaml`, `items`,
  `input_path`, `prompt_id`, and `run_path`.
- Support `--fields` and `--max-items` so prompts can receive only the fields
  they need.
- Default `--fields` to `id,hindi,romanisation,english`.
- Always write run artifacts under ignored `eval/<prompt-id>/<run-id>/`, e.g.
  `eval/sentence/register/2026-05-15_143012_translategemma_12b/`.
- Seed paired sentence prompt templates for the major sub-tasks we want to test.
- Store grader responses in a consistent parsed result file for reporting.

## Non-Goals

- Do not write or modify `output/`.
- Do not manage Ollama model lifecycle or add `--model` in v1.
- Do not implement model-to-model comparison analytics in v1.
- Do not automatically call a remote evaluator model in v1.
- Do not build a full JSONPath/query language for nested field selection.

## Acceptance Criteria

| ID | Criteria |
|---|---|
| AC01 | `hindi eval run <prompt-id> <input-yaml>`, `hindi eval grade <run-id-or-path> [--response <path>]`, and `hindi eval report [--no-color] [--verbose] [--output none\|failures\|all]` are parsed and documented in help output. |
| AC02 | `hindi eval run` requires exactly one running Ollama model from `/api/ps` and prints the selected model. |
| AC03 | Built-in prompt templates render with Handlebars using `{{#each items}}`, `{{items_yaml}}`, `{{input_yaml}}`, `{{input_path}}`, `{{prompt_id}}`, and `{{run_path}}`. |
| AC04 | `--fields id,hindi,romanisation,english` selects top-level item fields; when omitted, fields default to `id,hindi,romanisation,english`; missing fields fail clearly. |
| AC05 | `--max-items <n>` limits the selected item list before rendering. |
| AC06 | Each eval run writes `eval/<prompt-id>/<run-id>/prompt.txt`, `response.txt`, `meta.json`, and `summary.txt`. |
| AC07 | `eval/` is ignored by git. |
| AC08 | Paired sentence input/grading prompt templates exist for source QA, English translation, literal translation, register, word breakdown, word breakdown from existing translation, and full enrichment. |
| AC09 | The command never writes accepted output under `output/`. |
| AC10 | `hindi eval grade` renders the grading prompt for an eval run, opens `grade_packet.md` in `$EDITOR` unless `--response <path>` is provided, accepts/persists pasted or imported grader YAML or JSON, writes `grade_prompt.txt`, `grade_response.txt`, `grade.json`, and updates `summary.txt`. |
| AC11 | `grade.json` uses the shared grading schema: axis scores for accuracy, completeness, format compliance, consistency, confidence; total; verdict; item flags; summary. |
| AC12 | `hindi eval report` scans `eval/`, prints source Hindi with romanisation and English, groups results by test with one model row per run, summarizes score percent, timing, verdict, and notes, hides run folders unless `--verbose` is passed, and can show model response snippets with `--output failures` or `--output all`. |

## Architecture Notes

### Files And Folders Changed

- `Cargo.toml`
- `.gitignore`
- `src/cli.rs`
- `src/main.rs`
- `src/eval.rs`
- `src/eval_prompts/*.yaml.hbs`

### Workflow State Touched

- New ignored run output under `eval/<prompt-id>/<run-id>/`.
- No accepted learner output is touched.

### External Effects And Reuse

- Calls local Ollama HTTP API.
- Uses Ollama `/api/ps` for running-model detection.
- Reuses existing source YAML parsing shape where possible.
- Reuses existing raw HTTP Ollama client patterns where practical.
- Opens `$EDITOR` for grading packet response capture.

## Testing Plan

### Unit Tests

- CLI parse/help coverage for `hindi eval run`, `hindi eval grade`, and
  `hindi eval report`.
- Template context construction with full YAML and selected fields.
- Missing selected field error.
- `--max-items` selection.
- Run ID/model slug safety.
- Grader response parsing for YAML/JSON and shared grade schema validation.

### Integration Tests

- Fake Ollama client test for eval command writing run artifacts under the
  prompt-id hierarchy.
- Verify output is under `eval/` and not `output/`.
- Fake editor/import test for `hindi eval grade`.
- Report rendering test for graded and ungraded eval runs.

### Smoke Tests

- `cargo run -- eval run sentence/register input/sentences/complete_hindi_chapter_02_sentences.yaml --max-items 2`
- `cargo run -- eval grade sentence/register/<run-id>`
- `cargo run -- eval report`
- `make check`

### Drift / Consistency Checks

- `rg "hindi eval" docs README.md src`
- `git status --short` should show no generated eval runs after smoke because
  `eval/` is ignored.

### Not Covered In This Spec

- Model-to-model comparison analytics are deferred until we have several eval
  result files to compare.

## Open Questions

None for v1. `--model`, deeper comparison analytics, and automatic evaluator
API calls are deferred deliberately.

# Prompt Eval Runner

## Scope

Add `hindi eval input` and `hindi eval grade` for running built-in prompt
templates against YAML source input and grading eval results without touching
accepted learner output. `input` uses the currently running Ollama model and
writes a structured run folder under ignored `eval/`. `grade` renders the
matching built-in grading prompt for a previous eval run, opens it in `$EDITOR`,
and lets the user paste a structured grader response back into the run folder.

## Problem

We need to compare sentence sub-task prompts before generating more real cards:
source QA, English translation, literal translation, register detection,
word-by-word breakdown, mapping existing translations to words, and full
enrichment. The current generation command is too tied to accepted output, and
the old Python experiment scripts are archived and less structured than the
Rust workflow.

## Goals

- Provide `hindi eval input --input <yaml> --prompt-id <id>`.
- Provide `hindi eval grade --run <eval-folder>`.
- Use the single currently running Ollama model from `ollama ps`; do not switch
  or start models for eval input.
- Use built-in prompt IDs such as `sentence/register`, with paired input and
  grading templates compiled into the Rust binary.
- Render prompts with Handlebars and support batch-friendly `{{#each items}}`.
- Provide YAML-first template context: `input_yaml`, `items_yaml`, `items`,
  `input_path`, `prompt_id`, and `run_path`.
- Support `--fields` and `--max-items` so prompts can receive only the fields
  they need.
- Always write run artifacts under ignored `eval/<run-id>/`.
- Seed paired sentence prompt templates for the major sub-tasks we want to test.
- Store grader responses in a consistent parsed result file for reporting.

## Non-Goals

- Do not write or modify `output/`.
- Do not manage Ollama model lifecycle or add `--model` in v1.
- Do not implement prompt comparison reports in v1.
- Do not automatically call a remote evaluator model in v1.
- Do not build a full JSONPath/query language for nested field selection.

## Acceptance Criteria

| ID | Criteria |
|---|---|
| AC01 | `hindi eval input --input <path> --prompt-id <id>` and `hindi eval grade --run <eval-folder>` are parsed and documented in help output. |
| AC02 | `hindi eval input` requires exactly one running Ollama model from `ollama ps` and prints the selected model. |
| AC03 | Built-in prompt templates render with Handlebars using `{{#each items}}`, `{{items_yaml}}`, `{{input_yaml}}`, `{{input_path}}`, `{{prompt_id}}`, and `{{run_path}}`. |
| AC04 | `--fields id,hindi,romanisation,english` selects top-level item fields; missing fields fail clearly. |
| AC05 | `--max-items <n>` limits the selected item list before rendering. |
| AC06 | Each input run writes `eval/<run-id>/prompt.txt`, `response.txt`, `result.json`, and `summary.txt`. |
| AC07 | `eval/` is ignored by git. |
| AC08 | Paired sentence input/grading prompt templates exist for source QA, English translation, literal translation, register, word breakdown, word breakdown from existing translation, and full enrichment. |
| AC09 | The command never writes accepted output under `output/`. |
| AC10 | `hindi eval grade` renders the grading prompt for an eval run, opens it in `$EDITOR`, accepts/persists pasted grader YAML or JSON, and updates the run summary. |

## Architecture Notes

### Files And Folders Changed

- `Cargo.toml`
- `.gitignore`
- `src/cli.rs`
- `src/main.rs`
- `src/eval.rs`
- `src/eval_prompts/*.yaml.hbs`

### Workflow State Touched

- New ignored run output under `eval/`.
- No accepted learner output is touched.

### External Effects And Reuse

- Calls local Ollama HTTP API.
- Uses `ollama ps` data semantics, implemented through Rust HTTP calls where
  practical.
- Reuses existing source YAML parsing shape where possible.
- Reuses existing raw HTTP Ollama client patterns where practical.
- Opens `$EDITOR` for grading prompt and response capture.

## Testing Plan

### Unit Tests

- CLI parse/help coverage for `hindi eval input` and `hindi eval grade`.
- Template context construction with full YAML and selected fields.
- Missing selected field error.
- `--max-items` selection.
- Run ID/model slug safety.
- Grader response parsing for YAML/JSON.

### Integration Tests

- Fake Ollama client test for eval command writing run artifacts.
- Verify output is under `eval/` and not `output/`.
- Fake editor/import test for `hindi eval grade`.

### Smoke Tests

- `cargo run -- eval input --input input/sentences/complete_hindi_chapter_02_sentences.yaml --prompt-id sentence/register --max-items 2`
- `cargo run -- eval grade --run eval/<run-id>`
- `make check`

### Drift / Consistency Checks

- `rg "hindi eval" docs README.md src`
- `git status --short` should show no generated eval runs after smoke because
  `eval/` is ignored.

### Not Covered In This Spec

- Human quality scoring and aggregate reports are deferred until we have several
  eval result files to compare.

## Open Questions

None for v1. `--model`, comparison reports, and automatic evaluator API calls
are deferred deliberately.

# Prompt Eval Runner

## Scope

Add a top-level `hindi eval` command for running reusable prompt templates
against YAML source input without touching accepted learner output. The command
uses the currently running Ollama model, renders Handlebars prompt templates
with selected YAML items, writes each run to an ignored `eval/` folder, and
prints a compact result summary.

## Problem

We need to compare sentence sub-task prompts before generating more real cards:
source QA, English translation, literal translation, register detection,
word-by-word breakdown, mapping existing translations to words, and full
enrichment. The current generation command is too tied to accepted output, and
the old Python experiment scripts are archived and less structured than the
Rust workflow.

## Goals

- Provide `hindi eval --input <yaml> --prompt <hbs>`.
- Use the single currently running Ollama model from `ollama ps`; do not switch
  or start models.
- Render prompts with Handlebars and support batch-friendly `{{#each items}}`.
- Provide YAML-first template context: `input_yaml`, `items_yaml`, `items`,
  `input_path`, and `prompt_path`.
- Support `--fields` and `--max-items` so prompts can receive only the fields
  they need.
- Always write run artifacts under ignored `eval/<run-id>/`.
- Seed sentence prompt templates for the major sub-tasks we want to test.

## Non-Goals

- Do not write or modify `output/`.
- Do not manage Ollama model lifecycle or add `--model` in v1.
- Do not implement prompt comparison reports in v1.
- Do not add evaluator-agent scoring in v1.
- Do not build a full JSONPath/query language for nested field selection.

## Acceptance Criteria

| ID | Criteria |
|---|---|
| AC01 | `hindi eval --input <path> --prompt <path>` is parsed and documented in help output. |
| AC02 | The command requires exactly one running Ollama model from `ollama ps` and prints the selected model. |
| AC03 | Prompt templates render with Handlebars using `{{#each items}}`, `{{items_yaml}}`, `{{input_yaml}}`, `{{input_path}}`, and `{{prompt_path}}`. |
| AC04 | `--fields id,hindi,romanisation,english` selects top-level item fields; missing fields fail clearly. |
| AC05 | `--max-items <n>` limits the selected item list before rendering. |
| AC06 | Each run writes `eval/<run-id>/prompt.txt`, `response.txt`, `result.json`, and `summary.txt`. |
| AC07 | `eval/` is ignored by git. |
| AC08 | Sentence prompt templates exist for source QA, English translation, literal translation, register, word breakdown, word breakdown from existing translation, and full enrichment. |
| AC09 | The command never writes accepted output under `output/`. |

## Architecture Notes

### Files And Folders Changed

- `Cargo.toml`
- `.gitignore`
- `src/cli.rs`
- `src/main.rs`
- `src/eval.rs`
- `prompts/sentences/*.yaml.hbs`

### Workflow State Touched

- New ignored run output under `eval/`.
- No accepted learner output is touched.

### External Effects And Reuse

- Calls local Ollama HTTP API.
- Uses `ollama ps` data semantics, implemented through Rust HTTP calls where
  practical.
- Reuses existing source YAML parsing shape where possible.
- Reuses existing raw HTTP Ollama client patterns where practical.

## Testing Plan

### Unit Tests

- CLI parse/help coverage for `hindi eval`.
- Template context construction with full YAML and selected fields.
- Missing selected field error.
- `--max-items` selection.
- Run ID/model slug safety.

### Integration Tests

- Fake Ollama client test for eval command writing run artifacts.
- Verify output is under `eval/` and not `output/`.

### Smoke Tests

- `cargo run -- eval --input input/sentences/complete_hindi_chapter_02_sentences.yaml --prompt prompts/sentences/register.yaml.hbs --max-items 2`
- `make check`

### Drift / Consistency Checks

- `rg "hindi eval" docs README.md src`
- `git status --short` should show no generated eval runs after smoke because
  `eval/` is ignored.

### Not Covered In This Spec

- Human quality scoring and aggregate reports are deferred until we have several
  eval result files to compare.

## Open Questions

None for v1. `--model`, comparison reports, and evaluator-agent scoring are
deferred deliberately.

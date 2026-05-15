# Testing

## Drift This Must Prevent

- Eval writes accepted output.
- Prompt templates assume JSON when source input is YAML-first.
- `--fields` silently drops missing data.
- `--model` appears in eval help before model switching exists.
- Multiple running Ollama models are accepted silently.
- Input prompts and grading prompts drift apart for the same prompt ID.
- Run folders become flat and hard to compare by prompt ID.

## Coverage Summary

| Change | Risk | Test | Purpose |
|---|---|---|---|
| CLI command | Bad args or stale help | `cargo test cli` | Confirms `run`, `grade`, and `report` command shape. |
| Template context | Wrong prompt data | `cargo test eval` | Confirms `items`, `items_yaml`, `input_yaml`. |
| Field selection | Missing/incorrect fields | Unit tests | Confirms errors and defaults. |
| Artifact writes | Eval pollutes source/output | Integration test | Confirms writes stay under `eval/`. |
| Prompt templates | Examples drift | `rg` and smoke run | Confirms templates use `{{#each items}}`. |
| Grading flow | Grader response cannot be recorded | Unit/integration test | Confirms grade packet rendering and response parsing. |
| Report flow | Eval results are hard to scan | Unit/manual test | Confirms source display and result table rendering. |

## Unit Tests

- Parse `hindi eval run`, `hindi eval grade`, and `hindi eval report` commands
  and help.
- Build default selected fields.
- Build custom selected fields.
- Fail on missing selected field.
- Render a template with `{{#each items}}`.
- Render `items_yaml`.
- Resolve prompt IDs to paired input/grading templates.
- Resolve grade run arguments by prepending `eval/` only when the argument does
  not already start with `eval/`.
- Extract grader response from `grade_packet.md` using the exact
  `## Paste Grader Response Below` marker.
- Parse grader response as YAML or JSON into the shared five-axis schema.
- Render eval report rows for graded and ungraded runs.

## Integration Tests

- Fake model eval run writes expected files to an `eval/` folder.
- The same run writes nothing under `output/`.
- Fake grade run writes `grade_prompt.txt`, `grade_packet.md`,
  `grade_response.txt`, and `grade.json`.
- Fake grade run can import a grader response from `--response <path>` without
  opening `$EDITOR`.
- Fake report run reads `meta.json` plus optional `grade.json` without writing
  files.

## Drift Checks

- `rg "eval (run|grade|report) --model" docs src` should not show eval model
  switching.
- `rg "{{#each items}}" src/eval_prompts`.

## Manual Review Checks

- Run one live eval run against `sentence/register` with `--max-items 2`.
- Inspect `summary.txt`, `prompt.txt`, and `response.txt`.
- Run `hindi eval grade <eval-folder>` and paste a small valid grader
  response.
- Run `hindi eval report` and confirm the report includes source Hindi,
  romanisation, English, grouped test/model rows, timing, score percent,
  verdict, summary footer, and notes.
- Run `hindi eval report --verbose` and confirm run folders and raw score
  fractions are shown.
- Run `hindi eval report --output failures` and confirm response snippets are
  shown only for failed runs.

## Not Covered

- Model-to-model comparison analytics are deferred. This spec makes prompt runs
  structured, graded, and scannable first.

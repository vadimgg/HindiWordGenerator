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
| CLI command | Bad args or stale help | `cargo test cli` | Confirms `run` and `grade` command shape. |
| Template context | Wrong prompt data | `cargo test eval` | Confirms `items`, `items_yaml`, `input_yaml`. |
| Field selection | Missing/incorrect fields | Unit tests | Confirms errors and defaults. |
| Artifact writes | Eval pollutes source/output | Integration test | Confirms writes stay under `eval/`. |
| Prompt templates | Examples drift | `rg` and smoke run | Confirms templates use `{{#each items}}`. |
| Grading flow | Grader response cannot be recorded | Unit/integration test | Confirms grade packet rendering and response parsing. |

## Unit Tests

- Parse `hindi eval run` and `hindi eval grade` commands and help.
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

## Integration Tests

- Fake model eval run writes expected files to an `eval/` folder.
- The same run writes nothing under `output/`.
- Fake grade run writes `grade_prompt.txt`, `grade_packet.md`,
  `grade_response.txt`, and `grade.json`.
- Fake grade run can import a grader response from `--response <path>` without
  opening `$EDITOR`.

## Drift Checks

- `rg "eval (run|grade) --model" docs src` should not show eval model
  switching.
- `rg "{{#each items}}" src/eval_prompts`.

## Manual Review Checks

- Run one live eval run against `sentence/register` with `--max-items 2`.
- Inspect `summary.txt`, `prompt.txt`, and `response.txt`.
- Run `hindi eval grade <eval-folder>` and paste a small valid grader
  response.

## Not Covered

- Prompt quality ranking is deferred. This spec makes prompt runs structured and
  reproducible first.
